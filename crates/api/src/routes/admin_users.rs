use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use axum::{extract::State, Json};
use db::models::{CreateUser, UserRole};
use db::UserRepository;
use serde::{Deserialize, Serialize};
use shared::DomainError;

use crate::{
    auth::{AuthUser, TenantContext},
    error::{ApiError, ApiResult},
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct CreateWalkerRequest {
    pub email: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
    pub phone: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct WalkerResponse {
    pub id: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub phone: Option<String>,
    pub role: String,
    pub created_at: String,
}

/// Admin endpoint to create a new walker
/// Requires authentication and creates the walker in the admin's organization
pub async fn create_walker(
    State(_state): State<AppState>,
    auth_user: AuthUser,
    tenant: TenantContext,
    Json(req): Json<CreateWalkerRequest>,
) -> ApiResult<Json<WalkerResponse>> {
    // Verify the requesting user is an admin
    let admin = UserRepository::find_by_id(&tenant.pool, tenant.org_id, auth_user.user_id)
        .await?
        .ok_or_else(|| ApiError::from(DomainError::UserNotFound(auth_user.user_id.to_string())))?;

    if !admin.is_admin() {
        return Err(ApiError::from(shared::AppError::Forbidden));
    }

    // Check if email already exists in this organization
    if UserRepository::find_by_email(&tenant.pool, tenant.org_id, &req.email)
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

    // Create walker
    let walker = UserRepository::create(
        &tenant.pool,
        CreateUser {
            organization_id: tenant.org_id,
            email: req.email,
            password_hash,
            role: UserRole::Walker,
            first_name: req.first_name,
            last_name: req.last_name,
            phone: req.phone,
            timezone: None,
        },
    )
    .await?;

    Ok(Json(WalkerResponse {
        id: walker.id.to_string(),
        email: walker.email,
        first_name: walker.first_name,
        last_name: walker.last_name,
        phone: walker.phone,
        role: walker.role.to_string(),
        created_at: walker.created_at.to_rfc3339(),
    }))
}
