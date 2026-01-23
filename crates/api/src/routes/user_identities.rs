use axum::{
    extract::{Path, State},
    Json,
};
use db::models::AuthProvider;
use db::UserIdentityRepository;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    error::{ApiError, ApiResult},
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

    Ok(Json(ListIdentitiesResponse { identities: response }))
}

/// Mask sensitive parts of provider IDs for display
fn mask_provider_id(provider: &AuthProvider, id: &str) -> String {
    match provider {
        AuthProvider::Phone => {
            // Show last 4 digits of phone number
            if id.len() > 4 {
                format!("***{}", &id[id.len()-4..])
            } else {
                "****".to_string()
            }
        },
        AuthProvider::Wallet => {
            // Show first 6 and last 4 of wallet address
            if id.len() > 10 {
                format!("{}...{}", &id[..6], &id[id.len()-4..])
            } else {
                id.to_string()
            }
        },
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
            "Cannot unlink your last authentication method. Please link another method first.".to_string(),
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
