use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{extract::State, Json};
use db::models::{CreateMembership, CreateUser, MembershipRole, MembershipStatus, UserRole};
use db::{MembershipRepository, OrganizationRepository, UserRepository};
use serde::{Deserialize, Serialize};
use shared::types::OrganizationId;
use shared::DomainError;

use crate::{
    auth::{create_token, AuthUser},
    error::{ApiError, ApiResult},
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub org_slug: String,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub membership: Option<MembershipInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memberships: Option<Vec<MembershipInfo>>,
}

#[derive(Debug, Serialize, Clone)]
pub struct MembershipInfo {
    pub id: String,
    pub organization_id: String,
    pub organization_name: String,
    pub organization_slug: String,
    pub role: String,
    pub is_default: bool,
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
    // Look up organization by slug
    let organization = OrganizationRepository::find_by_slug(&state.pool, &req.org_slug)
        .await?
        .ok_or_else(|| ApiError::from(DomainError::OrganizationNotFound(req.org_slug.clone())))?;

    // Check if email already exists in this organization
    if UserRepository::find_by_email(&state.pool, organization.id, &req.email)
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
    let (user_role, membership_role) = match req.role.as_deref() {
        Some("walker") => (UserRole::Walker, MembershipRole::Walker),
        Some("admin") => (UserRole::Admin, MembershipRole::Admin),
        _ => (UserRole::Customer, MembershipRole::Customer),
    };

    // Create user with organization_id
    let user = UserRepository::create(
        &state.pool,
        CreateUser {
            organization_id: organization.id,
            email: req.email,
            password_hash,
            role: user_role,
            first_name: req.first_name,
            last_name: req.last_name,
            phone: req.phone,
            timezone: None,
        },
    )
    .await?;

    // Create membership for the user in this organization
    let membership = MembershipRepository::create(
        &state.pool,
        CreateMembership {
            user_id: user.id,
            organization_id: organization.id,
            role: membership_role,
            status: Some(MembershipStatus::Active),
            title: None,
        },
    )
    .await?;

    // Set this as the user's default membership
    sqlx::query("UPDATE users SET default_membership_id = $1 WHERE id = $2")
        .bind(membership.id)
        .bind(user.id)
        .execute(&state.pool)
        .await?;

    // Create token with org_id
    let token = create_token(user.id, Some(organization.id), &state.jwt_secret).map_err(|_| {
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
            role: membership_role.to_string(),
        },
        membership: Some(MembershipInfo {
            id: membership.id.to_string(),
            organization_id: organization.id.to_string(),
            organization_name: organization.name,
            organization_slug: organization.slug,
            role: membership_role.to_string(),
            is_default: true,
        }),
        memberships: None,
    }))
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub org_slug: String,
    pub email: String,
    pub password: String,
}

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> ApiResult<Json<AuthResponse>> {
    // Look up organization by slug
    let organization = OrganizationRepository::find_by_slug(&state.pool, &req.org_slug)
        .await?
        .ok_or_else(|| ApiError::from(DomainError::OrganizationNotFound(req.org_slug.clone())))?;

    // Find user within organization - validates user belongs to this organization
    let user = UserRepository::find_by_email(&state.pool, organization.id, &req.email)
        .await?
        .ok_or_else(|| ApiError::from(DomainError::InvalidCredentials))?;

    // Verify password
    let parsed_hash = PasswordHash::new(&user.password_hash)
        .map_err(|_| ApiError::from(DomainError::InvalidCredentials))?;

    Argon2::default()
        .verify_password(req.password.as_bytes(), &parsed_hash)
        .map_err(|_| ApiError::from(DomainError::InvalidCredentials))?;

    // Get user's membership in this organization (used for future reference if needed)
    let _membership =
        MembershipRepository::find_by_user_and_org(&state.pool, user.id, organization.id).await?;

    // Get all user's memberships for the response
    let all_memberships = MembershipRepository::find_with_org_by_user(&state.pool, user.id).await?;

    // Get default membership ID
    let default_membership_id: Option<uuid::Uuid> =
        sqlx::query_scalar("SELECT default_membership_id FROM users WHERE id = $1")
            .bind(user.id)
            .fetch_optional(&state.pool)
            .await?
            .flatten();

    let memberships_info: Vec<MembershipInfo> = all_memberships
        .into_iter()
        .map(|m| MembershipInfo {
            id: m.id.to_string(),
            organization_id: m.organization_id.to_string(),
            organization_name: m.organization_name,
            organization_slug: m.organization_slug,
            role: m.role.to_string(),
            is_default: default_membership_id.map(|d| d == m.id).unwrap_or(false),
        })
        .collect();

    // Create token with org_id
    let token = create_token(user.id, Some(organization.id), &state.jwt_secret).map_err(|_| {
        ApiError::from(shared::AppError::Internal(
            "Token creation failed".to_string(),
        ))
    })?;

    // Find current membership info
    let current_membership = memberships_info
        .iter()
        .find(|m| m.organization_id == organization.id.to_string())
        .cloned();

    let role = current_membership
        .as_ref()
        .map(|m| m.role.clone())
        .unwrap_or_else(|| user.role.to_string());

    Ok(Json(AuthResponse {
        token,
        user: UserResponse {
            id: user.id.to_string(),
            email: user.email,
            first_name: user.first_name,
            last_name: user.last_name,
            role,
        },
        membership: current_membership,
        memberships: Some(memberships_info),
    }))
}

/// Universal login request (no org_slug required)
#[derive(Debug, Deserialize)]
pub struct UniversalLoginRequest {
    pub email: String,
    pub password: String,
}

/// Universal login - authenticates user globally without org context
/// POST /auth/login/universal
///
/// Returns a context-free token with all user's memberships.
/// User can then switch to a specific context using /contexts/switch
pub async fn universal_login(
    State(state): State<AppState>,
    Json(req): Json<UniversalLoginRequest>,
) -> ApiResult<Json<AuthResponse>> {
    // Find user by email globally (across all organizations)
    let user = UserRepository::find_by_email_globally(&state.pool, &req.email)
        .await?
        .ok_or_else(|| ApiError::from(DomainError::InvalidCredentials))?;

    // Verify password
    let parsed_hash = PasswordHash::new(&user.password_hash)
        .map_err(|_| ApiError::from(DomainError::InvalidCredentials))?;

    Argon2::default()
        .verify_password(req.password.as_bytes(), &parsed_hash)
        .map_err(|_| ApiError::from(DomainError::InvalidCredentials))?;

    // Get all user's memberships
    let all_memberships = MembershipRepository::find_with_org_by_user(&state.pool, user.id).await?;

    // Get default membership ID
    let default_membership_id: Option<uuid::Uuid> =
        sqlx::query_scalar("SELECT default_membership_id FROM users WHERE id = $1")
            .bind(user.id)
            .fetch_optional(&state.pool)
            .await?
            .flatten();

    let memberships_info: Vec<MembershipInfo> = all_memberships
        .into_iter()
        .map(|m| MembershipInfo {
            id: m.id.to_string(),
            organization_id: m.organization_id.to_string(),
            organization_name: m.organization_name,
            organization_slug: m.organization_slug,
            role: m.role.to_string(),
            is_default: default_membership_id.map(|d| d == m.id).unwrap_or(false),
        })
        .collect();

    // If user has a default membership, create token with that context
    // Otherwise, create a context-free token
    let (token, default_membership) = if let Some(default_id) = default_membership_id {
        // Find the default membership's org_id
        if let Some(default_mem) = memberships_info
            .iter()
            .find(|m| m.id == default_id.to_string())
        {
            let org_id: uuid::Uuid = default_mem.organization_id.parse().unwrap();
            let token = create_token(
                user.id,
                Some(OrganizationId::from_uuid(org_id)),
                &state.jwt_secret,
            )
            .map_err(|_| {
                ApiError::from(shared::AppError::Internal(
                    "Token creation failed".to_string(),
                ))
            })?;
            (token, Some(default_mem.clone()))
        } else {
            let token = create_token(user.id, None, &state.jwt_secret).map_err(|_| {
                ApiError::from(shared::AppError::Internal(
                    "Token creation failed".to_string(),
                ))
            })?;
            (token, None)
        }
    } else if let Some(first_membership) = memberships_info.first() {
        // Use first membership as default if no default is set
        let org_id: uuid::Uuid = first_membership.organization_id.parse().unwrap();
        let token = create_token(
            user.id,
            Some(OrganizationId::from_uuid(org_id)),
            &state.jwt_secret,
        )
        .map_err(|_| {
            ApiError::from(shared::AppError::Internal(
                "Token creation failed".to_string(),
            ))
        })?;
        (token, Some(first_membership.clone()))
    } else {
        // No memberships - context-free token
        let token = create_token(user.id, None, &state.jwt_secret).map_err(|_| {
            ApiError::from(shared::AppError::Internal(
                "Token creation failed".to_string(),
            ))
        })?;
        (token, None)
    };

    let role = default_membership
        .as_ref()
        .map(|m| m.role.clone())
        .unwrap_or_else(|| "user".to_string());

    Ok(Json(AuthResponse {
        token,
        user: UserResponse {
            id: user.id.to_string(),
            email: user.email,
            first_name: user.first_name,
            last_name: user.last_name,
            role,
        },
        membership: default_membership,
        memberships: Some(memberships_info),
    }))
}

#[derive(Debug, Serialize)]
pub struct ValidateResponse {
    pub valid: bool,
    pub user_id: Option<String>,
}

/// Validate the current JWT token
pub async fn validate_token(auth_user: AuthUser) -> ApiResult<Json<ValidateResponse>> {
    Ok(Json(ValidateResponse {
        valid: true,
        user_id: Some(auth_user.user_id.to_string()),
    }))
}

/// Response for token refresh
#[derive(Debug, Serialize)]
pub struct RefreshResponse {
    pub token: String,
    pub expires_in: i64, // seconds until expiry
}

/// Refresh the current JWT token
/// POST /auth/refresh
///
/// Returns a new token with extended expiry (24 hours from now).
/// The current token must be valid but can be close to expiring.
pub async fn refresh_token(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> ApiResult<Json<RefreshResponse>> {
    // Create a new token with the same claims but fresh expiry
    let token =
        create_token(auth_user.user_id, auth_user.org_id, &state.jwt_secret).map_err(|_| {
            ApiError::from(shared::AppError::Internal(
                "Token creation failed".to_string(),
            ))
        })?;

    Ok(Json(RefreshResponse {
        token,
        expires_in: 24 * 60 * 60, // 24 hours in seconds
    }))
}

/// Full session information response
#[derive(Debug, Serialize)]
pub struct SessionResponse {
    pub user: UserResponse,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub membership: Option<MembershipInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memberships: Option<Vec<MembershipInfo>>,
    pub org_id: Option<String>,
}

/// Get full session information from the current token
/// GET /auth/session
///
/// Returns the current user, their active membership (if any),
/// and all their memberships for context switching.
/// This is useful for frontend apps to hydrate their state after
/// reading a token from shared cookies.
pub async fn session_info(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> ApiResult<Json<SessionResponse>> {
    // Fetch user info (unchecked - we already validated the token)
    let user = db::UserRepository::find_by_id_unchecked(&state.pool, auth_user.user_id)
        .await?
        .ok_or_else(|| ApiError::from(DomainError::InvalidCredentials))?;

    // Get all user's memberships
    let all_memberships =
        MembershipRepository::find_with_org_by_user(&state.pool, auth_user.user_id).await?;

    // Get default membership ID
    let default_membership_id: Option<uuid::Uuid> =
        sqlx::query_scalar("SELECT default_membership_id FROM users WHERE id = $1")
            .bind(auth_user.user_id)
            .fetch_optional(&state.pool)
            .await?
            .flatten();

    let memberships_info: Vec<MembershipInfo> = all_memberships
        .into_iter()
        .map(|m| MembershipInfo {
            id: m.id.to_string(),
            organization_id: m.organization_id.to_string(),
            organization_name: m.organization_name,
            organization_slug: m.organization_slug,
            role: m.role.to_string(),
            is_default: default_membership_id.map(|d| d == m.id).unwrap_or(false),
        })
        .collect();

    // Find current membership based on org_id in token
    let current_membership = if let Some(org_id) = &auth_user.org_id {
        memberships_info
            .iter()
            .find(|m| m.organization_id == org_id.to_string())
            .cloned()
    } else {
        // If no org_id in token, use default membership
        memberships_info.iter().find(|m| m.is_default).cloned()
    };

    let role = current_membership
        .as_ref()
        .map(|m| m.role.clone())
        .unwrap_or_else(|| user.role.to_string());

    Ok(Json(SessionResponse {
        user: UserResponse {
            id: user.id.to_string(),
            email: user.email,
            first_name: user.first_name,
            last_name: user.last_name,
            role,
        },
        membership: current_membership,
        memberships: Some(memberships_info),
        org_id: auth_user.org_id.map(|id| id.to_string()),
    }))
}
