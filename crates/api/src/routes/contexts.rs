//! User Context Management Routes
//!
//! These routes handle the multi-context user workflow where a single user
//! can have multiple memberships across organizations with different roles.

use axum::{
    extract::{Path, State},
    Json,
};
use db::models::{CreateMembership, MembershipRole, MembershipStatus, TenantDbStatus};
use db::{MembershipRepository, OrganizationRepository, TenantDatabaseRepository, UserRepository};
use serde::{Deserialize, Serialize};
use shared::types::OrganizationId;
use shared::DomainError;
use uuid::Uuid;

use crate::{
    auth::{create_token, AuthUser},
    error::{ApiError, ApiResult},
    state::AppState,
};

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Debug, Serialize)]
pub struct MembershipResponse {
    pub id: String,
    pub organization_id: String,
    pub organization_name: String,
    pub organization_slug: String,
    pub role: String,
    pub title: Option<String>,
    pub joined_at: String,
}

#[derive(Debug, Serialize)]
pub struct UserContextsResponse {
    pub user_id: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub memberships: Vec<MembershipResponse>,
    pub default_membership_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SwitchContextRequest {
    pub membership_id: String,
}

#[derive(Debug, Serialize)]
pub struct SwitchContextResponse {
    pub token: String,
    pub membership: MembershipResponse,
}

#[derive(Debug, Deserialize)]
pub struct SetDefaultContextRequest {
    pub membership_id: Option<String>, // None to clear default
}

#[derive(Debug, Serialize)]
pub struct ContextTokenResponse {
    pub token: String,
    pub user_id: String,
    pub membership_id: Option<String>,
    pub organization_id: Option<String>,
    pub role: Option<String>,
}

// ============================================================================
// Route Handlers
// ============================================================================

/// Get current user's contexts (all memberships)
/// GET /contexts
pub async fn list_contexts(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> ApiResult<Json<UserContextsResponse>> {
    // Get user info
    let user = UserRepository::find_by_id_unchecked(&state.pool, auth_user.user_id)
        .await?
        .ok_or_else(|| ApiError::from(DomainError::UserNotFound(auth_user.user_id.to_string())))?;

    // Get all active memberships with org details
    let memberships =
        MembershipRepository::find_with_org_by_user(&state.pool, auth_user.user_id).await?;

    let membership_responses: Vec<MembershipResponse> = memberships
        .into_iter()
        .map(|m| MembershipResponse {
            id: m.id.to_string(),
            organization_id: m.organization_id.to_string(),
            organization_name: m.organization_name,
            organization_slug: m.organization_slug,
            role: m.role.to_string(),
            title: m.title,
            joined_at: m.joined_at.to_rfc3339(),
        })
        .collect();

    // Get default membership ID from user record
    let default_membership_id: Option<String> =
        sqlx::query_scalar("SELECT default_membership_id::text FROM users WHERE id = $1")
            .bind(auth_user.user_id)
            .fetch_optional(&state.pool)
            .await?
            .flatten();

    Ok(Json(UserContextsResponse {
        user_id: user.id.to_string(),
        email: user.email,
        first_name: user.first_name,
        last_name: user.last_name,
        memberships: membership_responses,
        default_membership_id,
    }))
}

/// Switch to a different context (membership)
/// POST /contexts/switch
///
/// Returns a new JWT token scoped to the selected membership/organization
pub async fn switch_context(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(req): Json<SwitchContextRequest>,
) -> ApiResult<Json<SwitchContextResponse>> {
    let membership_id: Uuid = req.membership_id.parse().map_err(|_| {
        ApiError::from(DomainError::ValidationError(
            "Invalid membership ID".to_string(),
        ))
    })?;

    // Find the membership
    let membership = MembershipRepository::find_by_id(&state.pool, membership_id)
        .await?
        .ok_or_else(|| {
            ApiError::from(DomainError::ValidationError(
                "Membership not found".to_string(),
            ))
        })?;

    // Verify it belongs to the current user
    if membership.user_id != auth_user.user_id {
        return Err(ApiError::from(DomainError::Unauthorized(
            "Cannot switch to another user's membership".to_string(),
        )));
    }

    // Verify it's active
    if membership.status != MembershipStatus::Active {
        return Err(ApiError::from(DomainError::Unauthorized(
            "Membership is not active".to_string(),
        )));
    }

    // Get organization details
    let org = OrganizationRepository::find_by_id(&state.pool, membership.organization_id)
        .await?
        .ok_or_else(|| {
            ApiError::from(DomainError::OrganizationNotFound(
                membership.organization_id.to_string(),
            ))
        })?;

    // Create new token with membership context
    let token = create_token_with_membership(
        auth_user.user_id,
        membership.id,
        membership.organization_id,
        &state.jwt_secret,
    )?;

    Ok(Json(SwitchContextResponse {
        token,
        membership: MembershipResponse {
            id: membership.id.to_string(),
            organization_id: membership.organization_id.to_string(),
            organization_name: org.name,
            organization_slug: org.slug,
            role: membership.role.to_string(),
            title: membership.title,
            joined_at: membership.joined_at.to_rfc3339(),
        },
    }))
}

/// Get a token without organization context (for account-level operations)
/// POST /contexts/clear
///
/// Returns a JWT token without organization scope
pub async fn clear_context(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> ApiResult<Json<ContextTokenResponse>> {
    // Create token without org context
    let token = create_token(auth_user.user_id, None, &state.jwt_secret).map_err(|_| {
        ApiError::from(shared::AppError::Internal(
            "Token creation failed".to_string(),
        ))
    })?;

    Ok(Json(ContextTokenResponse {
        token,
        user_id: auth_user.user_id.to_string(),
        membership_id: None,
        organization_id: None,
        role: None,
    }))
}

/// Set default context for quick login
/// PUT /contexts/default
pub async fn set_default_context(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(req): Json<SetDefaultContextRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    if let Some(membership_id_str) = req.membership_id {
        let membership_id: Uuid = membership_id_str.parse().map_err(|_| {
            ApiError::from(DomainError::ValidationError(
                "Invalid membership ID".to_string(),
            ))
        })?;

        // Verify membership exists and belongs to user
        let membership = MembershipRepository::find_by_id(&state.pool, membership_id)
            .await?
            .ok_or_else(|| {
                ApiError::from(DomainError::ValidationError(
                    "Membership not found".to_string(),
                ))
            })?;

        if membership.user_id != auth_user.user_id {
            return Err(ApiError::from(DomainError::Unauthorized(
                "Cannot set another user's membership as default".to_string(),
            )));
        }

        // Update user's default membership
        sqlx::query("UPDATE users SET default_membership_id = $1 WHERE id = $2")
            .bind(membership_id)
            .bind(auth_user.user_id)
            .execute(&state.pool)
            .await?;
    } else {
        // Clear default
        sqlx::query("UPDATE users SET default_membership_id = NULL WHERE id = $1")
            .bind(auth_user.user_id)
            .execute(&state.pool)
            .await?;
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Default context updated"
    })))
}

/// Create a customer membership when booking from a new organization
/// POST /contexts/join-as-customer/:org_slug
pub async fn join_as_customer(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(org_slug): Path<String>,
) -> ApiResult<Json<SwitchContextResponse>> {
    // Find organization
    let org = OrganizationRepository::find_by_slug(&state.pool, &org_slug)
        .await?
        .ok_or_else(|| ApiError::from(DomainError::OrganizationNotFound(org_slug.clone())))?;

    // Check if user already has a membership
    let existing = MembershipRepository::find_by_user_org_role(
        &state.pool,
        auth_user.user_id,
        org.id,
        MembershipRole::Customer,
    )
    .await?;

    let membership = if let Some(existing) = existing {
        existing
    } else {
        // Create new customer membership
        MembershipRepository::create(
            &state.pool,
            CreateMembership {
                user_id: auth_user.user_id,
                organization_id: org.id,
                role: MembershipRole::Customer,
                status: Some(MembershipStatus::Active),
                title: None,
            },
        )
        .await?
    };

    // Create token with new context
    let token = create_token_with_membership(
        auth_user.user_id,
        membership.id,
        membership.organization_id,
        &state.jwt_secret,
    )?;

    Ok(Json(SwitchContextResponse {
        token,
        membership: MembershipResponse {
            id: membership.id.to_string(),
            organization_id: org.id.to_string(),
            organization_name: org.name,
            organization_slug: org.slug,
            role: membership.role.to_string(),
            title: membership.title,
            joined_at: membership.joined_at.to_rfc3339(),
        },
    }))
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Create a JWT token with membership context
fn create_token_with_membership(
    user_id: shared::types::UserId,
    membership_id: Uuid,
    org_id: OrganizationId,
    secret: &str,
) -> Result<String, ApiError> {
    use jsonwebtoken::{encode, EncodingKey, Header};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    struct MembershipClaims {
        sub: String,                   // User ID
        org_id: Option<String>,        // Organization ID
        membership_id: Option<String>, // Membership ID
        platform_admin: Option<bool>,
        exp: usize,
        iat: usize,
    }

    let now = chrono::Utc::now();
    let claims = MembershipClaims {
        sub: user_id.to_string(),
        org_id: Some(org_id.to_string()),
        membership_id: Some(membership_id.to_string()),
        platform_admin: None,
        exp: (now + chrono::Duration::hours(24)).timestamp() as usize,
        iat: now.timestamp() as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|_| {
        ApiError::from(shared::AppError::Internal(
            "Token creation failed".to_string(),
        ))
    })
}

// ============================================================================
// Organization Management (Owner-only)
// ============================================================================

#[derive(Debug, Serialize)]
pub struct DeleteOrganizationResponse {
    pub message: String,
    pub organization_id: String,
    pub status: String,
}

/// Delete (deactivate) the current organization
/// DELETE /contexts/organization
///
/// Only organization owners can delete their organization.
/// This is a soft-delete that marks the org as inactive.
pub async fn delete_organization(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> ApiResult<Json<DeleteOrganizationResponse>> {
    // Require organization context
    let org_id = auth_user.org_id.ok_or_else(|| {
        ApiError::from(DomainError::Unauthorized(
            "Organization context required".to_string(),
        ))
    })?;

    // Get user's membership in this org
    let memberships =
        MembershipRepository::find_with_org_by_user(&state.pool, auth_user.user_id).await?;

    let current_membership = memberships.iter().find(|m| m.organization_id == org_id);

    // Check if user is an owner
    let is_owner = current_membership
        .map(|m| m.role == MembershipRole::Owner)
        .unwrap_or(false);

    if !is_owner {
        return Err(ApiError::from(DomainError::Unauthorized(
            "Only organization owners can delete the organization".to_string(),
        )));
    }

    // Verify org exists
    let org = OrganizationRepository::find_by_id(&state.pool, org_id)
        .await?
        .ok_or_else(|| ApiError::from(DomainError::OrganizationNotFound(org_id.to_string())))?;

    // Soft-delete: Update tenant_database status to 'inactive'
    TenantDatabaseRepository::update_status_by_org_id(&state.pool, org_id, TenantDbStatus::Inactive)
        .await?;

    Ok(Json(DeleteOrganizationResponse {
        message: "Organization deactivated successfully".to_string(),
        organization_id: org_id.to_string(),
        status: "inactive".to_string(),
    }))
}
