use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{extract::State, Json};
use db::models::{CreateUser, UserRole};
use db::UserRepository;
use serde::{Deserialize, Serialize};
use shared::types::OrganizationId;
use shared::DomainError;

use crate::{
    auth::create_token,
    error::{ApiError, ApiResult},
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub organization_id: OrganizationId,
    pub email: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
    pub phone: Option<String>,
    pub role: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserResponse,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub role: String,
}

pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> ApiResult<Json<AuthResponse>> {
    // Check if email already exists in this organization
    if UserRepository::find_by_email(&state.pool, req.organization_id, &req.email)
        .await?
        .is_some()
    {
        return Err(ApiError::from(DomainError::EmailAlreadyExists));
    }

    // Hash password
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(req.password.as_bytes(), &salt)
        .map_err(|_| {
            ApiError::from(shared::AppError::Internal(
                "Password hashing failed".to_string(),
            ))
        })?
        .to_string();

    // Determine role
    let role = match req.role.as_deref() {
        Some("walker") => UserRole::Walker,
        Some("admin") => UserRole::Admin,
        _ => UserRole::Customer,
    };

    // Create user with organization_id
    let user = UserRepository::create(
        &state.pool,
        CreateUser {
            organization_id: req.organization_id,
            email: req.email,
            password_hash,
            role,
            first_name: req.first_name,
            last_name: req.last_name,
            phone: req.phone,
            timezone: None,
        },
    )
    .await?;

    // Create token with org_id
    let token =
        create_token(user.id, Some(user.organization_id), &state.jwt_secret).map_err(|_| {
            ApiError::from(shared::AppError::Internal(
                "Token creation failed".to_string(),
            ))
        })?;

    Ok(Json(AuthResponse {
        token,
        user: UserResponse {
            id: user.id.to_string(),
            email: user.email,
            first_name: user.first_name,
            last_name: user.last_name,
            role: user.role.to_string(),
        },
    }))
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub organization_id: OrganizationId,
    pub email: String,
    pub password: String,
}

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> ApiResult<Json<AuthResponse>> {
    // Find user within organization
    let user = UserRepository::find_by_email(&state.pool, req.organization_id, &req.email)
        .await?
        .ok_or_else(|| ApiError::from(DomainError::InvalidCredentials))?;

    // Verify password
    let parsed_hash = PasswordHash::new(&user.password_hash)
        .map_err(|_| ApiError::from(DomainError::InvalidCredentials))?;

    Argon2::default()
        .verify_password(req.password.as_bytes(), &parsed_hash)
        .map_err(|_| ApiError::from(DomainError::InvalidCredentials))?;

    // Create token with org_id
    let token =
        create_token(user.id, Some(user.organization_id), &state.jwt_secret).map_err(|_| {
            ApiError::from(shared::AppError::Internal(
                "Token creation failed".to_string(),
            ))
        })?;

    Ok(Json(AuthResponse {
        token,
        user: UserResponse {
            id: user.id.to_string(),
            email: user.email,
            first_name: user.first_name,
            last_name: user.last_name,
            role: user.role.to_string(),
        },
    }))
}
