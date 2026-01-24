use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{
    extract::{Path, State},
    Json,
};
use db::models::{AuthProvider, CreateUserIdentity};
use db::{UserIdentityRepository, UserRepository};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    error::{ApiError, ApiResult},
    routes::oauth::{verify_apple_token_public, verify_google_token_public},
    state::AppState,
};

#[derive(Debug, Serialize)]
pub struct IdentityResponse {
    pub id: String,
    pub provider: String,
    pub provider_user_id: String,
    pub provider_email: Option<String>,
    pub created_at: String,
    pub can_unlink: bool,
}

#[derive(Debug, Serialize)]
pub struct ListIdentitiesResponse {
    pub identities: Vec<IdentityResponse>,
}

/// List all linked identities for the current user
pub async fn list_identities(
    State(state): State<AppState>,
    auth: AuthUser,
) -> ApiResult<Json<ListIdentitiesResponse>> {
    let user_id = auth.user_id.into_uuid();

    let identities = UserIdentityRepository::find_by_user(&state.pool, user_id).await?;
    let count = identities.len();

    let response: Vec<IdentityResponse> = identities
        .into_iter()
        .map(|i| IdentityResponse {
            id: i.id.to_string(),
            provider: format!("{:?}", i.provider).to_lowercase(),
            provider_user_id: mask_provider_id(&i.provider, &i.provider_user_id),
            provider_email: i.provider_email,
            created_at: i.created_at.to_rfc3339(),
            can_unlink: count > 1, // Can only unlink if more than one identity
        })
        .collect();

    Ok(Json(ListIdentitiesResponse {
        identities: response,
    }))
}

/// Mask sensitive parts of provider IDs for display
fn mask_provider_id(provider: &AuthProvider, id: &str) -> String {
    match provider {
        AuthProvider::Phone => {
            // Show last 4 digits of phone number
            if id.len() > 4 {
                format!("***{}", &id[id.len() - 4..])
            } else {
                "****".to_string()
            }
        }
        AuthProvider::Wallet => {
            // Show first 6 and last 4 of wallet address
            if id.len() > 10 {
                format!("{}...{}", &id[..6], &id[id.len() - 4..])
            } else {
                id.to_string()
            }
        }
        AuthProvider::Google | AuthProvider::Apple | AuthProvider::Email => {
            // For email-based providers, show partial email
            if let Some(at_pos) = id.find('@') {
                let local = &id[..at_pos];
                let domain = &id[at_pos..];
                if local.len() > 2 {
                    format!("{}***{}", &local[..2], domain)
                } else {
                    format!("***{}", domain)
                }
            } else {
                // Not an email (e.g., numeric ID)
                if id.len() > 8 {
                    format!("{}...", &id[..8])
                } else {
                    id.to_string()
                }
            }
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct UnlinkIdentityPath {
    pub id: Uuid,
}

/// Unlink an identity from the current user's account
pub async fn unlink_identity(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(path): Path<UnlinkIdentityPath>,
) -> ApiResult<Json<serde_json::Value>> {
    let user_id = auth.user_id.into_uuid();

    // Check how many identities the user has
    let count = UserIdentityRepository::count_for_user(&state.pool, user_id).await?;

    if count <= 1 {
        return Err(ApiError::from(shared::DomainError::ValidationError(
            "Cannot unlink your last authentication method. Please link another method first."
                .to_string(),
        )));
    }

    // Verify the identity belongs to this user
    let identities = UserIdentityRepository::find_by_user(&state.pool, user_id).await?;
    let identity = identities.iter().find(|i| i.id == path.id);

    if identity.is_none() {
        return Err(ApiError::from(shared::DomainError::ValidationError(
            "Identity not found or does not belong to you.".to_string(),
        )));
    }

    // Delete the identity
    UserIdentityRepository::delete(&state.pool, path.id).await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Authentication method unlinked successfully."
    })))
}

// ============================================================================
// Link Identity Endpoints
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct LinkGoogleRequest {
    pub id_token: String,
}

#[derive(Debug, Deserialize)]
pub struct LinkAppleRequest {
    pub id_token: String,
}

#[derive(Debug, Deserialize)]
pub struct LinkEmailRequest {
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LinkIdentityResponse {
    pub success: bool,
    pub message: String,
    pub identity: Option<IdentityResponse>,
}

/// Link a Google account to the current user
/// POST /users/me/identities/google
pub async fn link_google(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<LinkGoogleRequest>,
) -> ApiResult<Json<LinkIdentityResponse>> {
    let user_id = auth.user_id.into_uuid();

    // Verify the Google token
    let verified = verify_google_token_public(&req.id_token).await?;

    // Check if this Google account is already linked to another user
    if let Some(existing_identity) = UserIdentityRepository::find_by_provider(
        &state.pool,
        AuthProvider::Google,
        &verified.provider_user_id,
    )
    .await?
    {
        if existing_identity.user_id != user_id {
            return Err(ApiError::from(shared::DomainError::ValidationError(
                "This Google account is already linked to another user.".to_string(),
            )));
        }
        // Already linked to this user
        return Ok(Json(LinkIdentityResponse {
            success: true,
            message: "Google account is already linked to your account.".to_string(),
            identity: None,
        }));
    }

    // Check if user already has a Google identity
    let existing_identities = UserIdentityRepository::find_by_user(&state.pool, user_id).await?;
    if existing_identities
        .iter()
        .any(|i| i.provider == AuthProvider::Google)
    {
        return Err(ApiError::from(shared::DomainError::ValidationError(
            "You already have a Google account linked. Unlink it first to link a different one."
                .to_string(),
        )));
    }

    // Create the identity link
    let identity = UserIdentityRepository::create(
        &state.pool,
        CreateUserIdentity {
            user_id,
            provider: AuthProvider::Google,
            provider_user_id: verified.provider_user_id,
            provider_email: Some(verified.email.clone()),
            provider_data: Some(serde_json::json!({
                "name": verified.name,
                "picture": verified.picture,
            })),
        },
    )
    .await?;

    let count = UserIdentityRepository::count_for_user(&state.pool, user_id).await?;

    Ok(Json(LinkIdentityResponse {
        success: true,
        message: "Google account linked successfully.".to_string(),
        identity: Some(IdentityResponse {
            id: identity.id.to_string(),
            provider: "google".to_string(),
            provider_user_id: mask_provider_id(&AuthProvider::Google, &identity.provider_user_id),
            provider_email: Some(verified.email),
            created_at: identity.created_at.to_rfc3339(),
            can_unlink: count > 1,
        }),
    }))
}

/// Link an Apple account to the current user
/// POST /users/me/identities/apple
pub async fn link_apple(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<LinkAppleRequest>,
) -> ApiResult<Json<LinkIdentityResponse>> {
    let user_id = auth.user_id.into_uuid();

    // Verify the Apple token
    let verified = verify_apple_token_public(&req.id_token).await?;

    // Check if this Apple account is already linked to another user
    if let Some(existing_identity) = UserIdentityRepository::find_by_provider(
        &state.pool,
        AuthProvider::Apple,
        &verified.provider_user_id,
    )
    .await?
    {
        if existing_identity.user_id != user_id {
            return Err(ApiError::from(shared::DomainError::ValidationError(
                "This Apple account is already linked to another user.".to_string(),
            )));
        }
        // Already linked to this user
        return Ok(Json(LinkIdentityResponse {
            success: true,
            message: "Apple account is already linked to your account.".to_string(),
            identity: None,
        }));
    }

    // Check if user already has an Apple identity
    let existing_identities = UserIdentityRepository::find_by_user(&state.pool, user_id).await?;
    if existing_identities
        .iter()
        .any(|i| i.provider == AuthProvider::Apple)
    {
        return Err(ApiError::from(shared::DomainError::ValidationError(
            "You already have an Apple account linked. Unlink it first to link a different one."
                .to_string(),
        )));
    }

    // Create the identity link
    let email = verified
        .email
        .clone()
        .unwrap_or_else(|| format!("{}@privaterelay.appleid.com", &verified.provider_user_id));

    let identity = UserIdentityRepository::create(
        &state.pool,
        CreateUserIdentity {
            user_id,
            provider: AuthProvider::Apple,
            provider_user_id: verified.provider_user_id,
            provider_email: Some(email.clone()),
            provider_data: None,
        },
    )
    .await?;

    let count = UserIdentityRepository::count_for_user(&state.pool, user_id).await?;

    Ok(Json(LinkIdentityResponse {
        success: true,
        message: "Apple account linked successfully.".to_string(),
        identity: Some(IdentityResponse {
            id: identity.id.to_string(),
            provider: "apple".to_string(),
            provider_user_id: mask_provider_id(&AuthProvider::Apple, &identity.provider_user_id),
            provider_email: Some(email),
            created_at: identity.created_at.to_rfc3339(),
            can_unlink: count > 1,
        }),
    }))
}

/// Add email/password authentication to the current user (for OAuth-only users)
/// POST /users/me/identities/email
pub async fn link_email(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<LinkEmailRequest>,
) -> ApiResult<Json<LinkIdentityResponse>> {
    let user_id = auth.user_id.into_uuid();

    // Get the user to check if they already have a password
    let user = UserRepository::find_by_id_unchecked(&state.pool, auth.user_id)
        .await?
        .ok_or_else(|| ApiError::from(shared::AppError::Internal("User not found".to_string())))?;

    // Check if user already has an email identity (has password set)
    let existing_identities = UserIdentityRepository::find_by_user(&state.pool, user_id).await?;
    if existing_identities
        .iter()
        .any(|i| i.provider == AuthProvider::Email)
    {
        return Err(ApiError::from(shared::DomainError::ValidationError(
            "You already have email/password authentication set up.".to_string(),
        )));
    }

    // Validate password strength
    if req.password.len() < 8 {
        return Err(ApiError::from(shared::DomainError::ValidationError(
            "Password must be at least 8 characters long.".to_string(),
        )));
    }

    // Hash the new password
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(req.password.as_bytes(), &salt)
        .map_err(|_| {
            ApiError::from(shared::AppError::Internal(
                "Password hashing failed".to_string(),
            ))
        })?
        .to_string();

    // Update user's password hash
    sqlx::query("UPDATE users SET password_hash = $1 WHERE id = $2")
        .bind(&password_hash)
        .bind(user_id)
        .execute(&state.pool)
        .await?;

    // Create email identity
    let identity = UserIdentityRepository::create(
        &state.pool,
        CreateUserIdentity {
            user_id,
            provider: AuthProvider::Email,
            provider_user_id: user.email.clone(),
            provider_email: Some(user.email.clone()),
            provider_data: None,
        },
    )
    .await?;

    let count = UserIdentityRepository::count_for_user(&state.pool, user_id).await?;

    Ok(Json(LinkIdentityResponse {
        success: true,
        message: "Email/password authentication added successfully.".to_string(),
        identity: Some(IdentityResponse {
            id: identity.id.to_string(),
            provider: "email".to_string(),
            provider_user_id: mask_provider_id(&AuthProvider::Email, &user.email),
            provider_email: Some(user.email),
            created_at: identity.created_at.to_rfc3339(),
            can_unlink: count > 1,
        }),
    }))
}

/// Change password for users with email identity
#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

pub async fn change_password(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<ChangePasswordRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let user_id = auth.user_id.into_uuid();

    // Get the user
    let user = UserRepository::find_by_id_unchecked(&state.pool, auth.user_id)
        .await?
        .ok_or_else(|| ApiError::from(shared::AppError::Internal("User not found".to_string())))?;

    // Check if user has email identity
    let existing_identities = UserIdentityRepository::find_by_user(&state.pool, user_id).await?;
    if !existing_identities
        .iter()
        .any(|i| i.provider == AuthProvider::Email)
    {
        return Err(ApiError::from(shared::DomainError::ValidationError(
            "You don't have email/password authentication set up. Use the link email endpoint first.".to_string(),
        )));
    }

    // Verify current password
    let parsed_hash = PasswordHash::new(&user.password_hash)
        .map_err(|_| ApiError::from(shared::DomainError::InvalidCredentials))?;

    Argon2::default()
        .verify_password(req.current_password.as_bytes(), &parsed_hash)
        .map_err(|_| ApiError::from(shared::DomainError::InvalidCredentials))?;

    // Validate new password
    if req.new_password.len() < 8 {
        return Err(ApiError::from(shared::DomainError::ValidationError(
            "New password must be at least 8 characters long.".to_string(),
        )));
    }

    // Hash new password
    let salt = SaltString::generate(&mut OsRng);
    let new_hash = Argon2::default()
        .hash_password(req.new_password.as_bytes(), &salt)
        .map_err(|_| {
            ApiError::from(shared::AppError::Internal(
                "Password hashing failed".to_string(),
            ))
        })?
        .to_string();

    // Update password
    sqlx::query("UPDATE users SET password_hash = $1 WHERE id = $2")
        .bind(&new_hash)
        .bind(user_id)
        .execute(&state.pool)
        .await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Password changed successfully."
    })))
}
