use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use axum::{
    extract::{Path, State},
    Json,
};
use chrono::{Duration, Utc};
use db::models::{
    CreateInvitation, CreateMembership, CreateOrganization, CreateTenantDatabase, CreateUser,
    InvitationStatus, InvitationType, MembershipRole, MembershipStatus, OrganizationSettings,
    UserRole,
};
use db::{
    InvitationRepository, MembershipRepository, OrganizationRepository, TenantDatabaseRepository,
    UserRepository,
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use shared::types::{OrganizationId, UserId};
use shared::DomainError;
use uuid::Uuid;

use crate::{
    auth::{create_token, AuthUser},
    error::{ApiError, ApiResult},
    routes::auth::{AuthResponse, MembershipInfo, UserResponse},
    state::AppState,
};

const INVITATION_EXPIRY_DAYS: i64 = 7;
const MAX_INVITATIONS_PER_HOUR: i64 = 20;

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct InviteWalkerRequest {
    pub email: Option<String>,
    pub phone: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct InviteClientRequest {
    pub email: Option<String>,
    pub phone: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct InviteResponse {
    pub success: bool,
    pub message: String,
    pub invitation_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct JoinTenantRequest {
    pub invite_token: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateTenantRequest {
    pub business_name: String,
    pub slug: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreateTenantResponse {
    pub success: bool,
    pub organization_id: String,
    pub slug: String,
    pub token: String,
    pub user: UserResponse,
}

#[derive(Debug, Serialize)]
pub struct InvitationInfoResponse {
    pub id: String,
    pub invitation_type: String,
    pub organization_name: String,
    pub inviter_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub status: String,
    pub expires_at: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct ValidateInviteRequest {
    pub token: String,
}

#[derive(Debug, Serialize)]
pub struct ValidateInviteResponse {
    pub valid: bool,
    pub organization_name: Option<String>,
    pub inviter_name: Option<String>,
    pub invitation_type: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AcceptInviteRequest {
    pub invite_token: String,
    pub email: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
    pub phone: Option<String>,
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Generate a secure random token
fn generate_invite_token() -> String {
    let mut rng = rand::thread_rng();
    let bytes: [u8; 32] = rng.gen();
    hex::encode(bytes)
}

/// Hash the token for storage
fn hash_token(token: &str) -> Result<String, ApiError> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(token.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|_| {
            ApiError::from(shared::AppError::Internal(
                "Failed to hash token".to_string(),
            ))
        })
}

/// Generate a URL-safe slug from business name
fn generate_slug(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Validate email format
fn validate_email(email: &str) -> bool {
    let email = email.trim();
    if email.is_empty() {
        return false;
    }
    // Basic email validation
    email.contains('@') && email.contains('.') && email.len() >= 5
}

/// Validate phone format (basic validation)
fn validate_phone(phone: &str) -> bool {
    let digits: String = phone.chars().filter(|c| c.is_ascii_digit()).collect();
    digits.len() >= 10
}

// ============================================================================
// Route Handlers
// ============================================================================

/// Send invitation to a walker (admin only)
/// POST /walker/invite
pub async fn invite_walker(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(req): Json<InviteWalkerRequest>,
) -> ApiResult<Json<InviteResponse>> {
    // Get organization ID from auth
    let org_id = auth_user.org_id.ok_or_else(|| {
        ApiError::from(DomainError::Unauthorized(
            "Organization context required".to_string(),
        ))
    })?;

    // Verify user is admin
    let user = UserRepository::find_by_id_unchecked(&state.pool, auth_user.user_id)
        .await?
        .ok_or_else(|| ApiError::from(DomainError::UserNotFound(auth_user.user_id.to_string())))?;

    if user.role != UserRole::Admin {
        return Err(ApiError::from(DomainError::Unauthorized(
            "Only admins can invite walkers".to_string(),
        )));
    }

    send_invitation(
        &state,
        org_id,
        auth_user.user_id,
        InvitationType::Walker,
        req.email.as_deref(),
        req.phone.as_deref(),
    )
    .await
}

/// Send invitation to a client (admin/walker)
/// POST /client/invite
pub async fn invite_client(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(req): Json<InviteClientRequest>,
) -> ApiResult<Json<InviteResponse>> {
    // Get organization ID from auth
    let org_id = auth_user.org_id.ok_or_else(|| {
        ApiError::from(DomainError::Unauthorized(
            "Organization context required".to_string(),
        ))
    })?;

    // Verify user is admin or walker
    let user = UserRepository::find_by_id_unchecked(&state.pool, auth_user.user_id)
        .await?
        .ok_or_else(|| ApiError::from(DomainError::UserNotFound(auth_user.user_id.to_string())))?;

    if user.role != UserRole::Admin && user.role != UserRole::Walker {
        return Err(ApiError::from(DomainError::Unauthorized(
            "Only admins and walkers can invite clients".to_string(),
        )));
    }

    send_invitation(
        &state,
        org_id,
        auth_user.user_id,
        InvitationType::Client,
        req.email.as_deref(),
        req.phone.as_deref(),
    )
    .await
}

/// Common invitation sending logic
async fn send_invitation(
    state: &AppState,
    org_id: OrganizationId,
    inviter_id: UserId,
    invitation_type: InvitationType,
    email: Option<&str>,
    phone: Option<&str>,
) -> ApiResult<Json<InviteResponse>> {
    // Validate input
    if email.is_none() && phone.is_none() {
        return Err(ApiError::from(DomainError::ValidationError(
            "Either email or phone is required".to_string(),
        )));
    }

    if let Some(email) = email {
        if !validate_email(email) {
            return Err(ApiError::from(DomainError::ValidationError(
                "Invalid email format".to_string(),
            )));
        }
    }

    if let Some(phone) = phone {
        if !validate_phone(phone) {
            return Err(ApiError::from(DomainError::ValidationError(
                "Invalid phone format".to_string(),
            )));
        }
    }

    // Rate limit check
    let recent_count =
        InvitationRepository::count_recent_by_inviter(&state.pool, inviter_id).await?;
    if recent_count >= MAX_INVITATIONS_PER_HOUR {
        return Err(ApiError::from(DomainError::RateLimitExceeded));
    }

    // Check for existing pending invitation
    let existing =
        InvitationRepository::find_existing_pending(&state.pool, org_id, email, phone).await?;

    if existing.is_some() {
        return Err(ApiError::from(DomainError::InvitationAlreadyExists));
    }

    // Generate token
    let token = generate_invite_token();
    let token_hash = hash_token(&token)?;

    // Create invitation
    let invitation = InvitationRepository::create(
        &state.pool,
        CreateInvitation {
            organization_id: org_id,
            invited_by: inviter_id,
            invitation_type,
            email: email.map(|e| e.to_lowercase()),
            phone: phone.map(|p| p.to_string()),
            token: token.clone(),
            token_hash,
            expires_at: Utc::now() + Duration::days(INVITATION_EXPIRY_DAYS),
        },
    )
    .await?;

    // Get organization info for the message
    let org = OrganizationRepository::find_by_id(&state.pool, org_id)
        .await?
        .ok_or_else(|| ApiError::from(DomainError::OrganizationNotFound(org_id.to_string())))?;

    // TODO: Send email or SMS with invite link
    // For now, just log it
    let invite_link = format!("offleash://invite/{}", token);
    tracing::info!(
        "Invitation created for {} to join {} as {}: {}",
        email.or(phone).unwrap_or("unknown"),
        org.name,
        invitation_type,
        invite_link
    );

    // Send notification based on contact method
    if let Some(email_addr) = email {
        // TODO: Integrate with email service (SendGrid, etc.)
        tracing::info!("Would send email to {} with token {}", email_addr, token);
    }

    if let Some(phone_num) = phone {
        // TODO: Integrate with SMS service (Twilio)
        tracing::info!("Would send SMS to {} with token {}", phone_num, token);
    }

    Ok(Json(InviteResponse {
        success: true,
        message: "Invitation sent successfully".to_string(),
        invitation_id: Some(invitation.id.to_string()),
    }))
}

/// Validate an invitation token (public endpoint)
/// POST /invitations/validate
pub async fn validate_invitation(
    State(state): State<AppState>,
    Json(req): Json<ValidateInviteRequest>,
) -> ApiResult<Json<ValidateInviteResponse>> {
    let invitation = InvitationRepository::find_by_token(&state.pool, &req.token).await?;

    match invitation {
        None => Ok(Json(ValidateInviteResponse {
            valid: false,
            organization_name: None,
            inviter_name: None,
            invitation_type: None,
            message: Some("Invalid invitation token".to_string()),
        })),
        Some(inv) => {
            // Check status
            if inv.status == InvitationStatus::Accepted {
                return Ok(Json(ValidateInviteResponse {
                    valid: false,
                    organization_name: None,
                    inviter_name: None,
                    invitation_type: None,
                    message: Some("This invitation has already been used".to_string()),
                }));
            }

            if inv.status == InvitationStatus::Revoked {
                return Ok(Json(ValidateInviteResponse {
                    valid: false,
                    organization_name: None,
                    inviter_name: None,
                    invitation_type: None,
                    message: Some("This invitation has been revoked".to_string()),
                }));
            }

            if inv.expires_at < Utc::now() {
                return Ok(Json(ValidateInviteResponse {
                    valid: false,
                    organization_name: None,
                    inviter_name: None,
                    invitation_type: None,
                    message: Some("This invitation has expired".to_string()),
                }));
            }

            // Get organization and inviter info
            let org = OrganizationRepository::find_by_id(&state.pool, inv.organization_id)
                .await?
                .map(|o| o.name)
                .unwrap_or_else(|| "Unknown Organization".to_string());

            let inviter = UserRepository::find_by_id_unchecked(&state.pool, inv.invited_by)
                .await?
                .map(|u| u.full_name())
                .unwrap_or_else(|| "Unknown".to_string());

            Ok(Json(ValidateInviteResponse {
                valid: true,
                organization_name: Some(org),
                inviter_name: Some(inviter),
                invitation_type: Some(inv.invitation_type.to_string()),
                message: None,
            }))
        }
    }
}

/// Accept invitation and create account
/// POST /invitations/accept
pub async fn accept_invitation(
    State(state): State<AppState>,
    Json(req): Json<AcceptInviteRequest>,
) -> ApiResult<Json<AuthResponse>> {
    // Find and validate invitation
    let invitation = InvitationRepository::find_by_token(&state.pool, &req.invite_token)
        .await?
        .ok_or_else(|| ApiError::from(DomainError::InvitationNotFound))?;

    // Check status
    match invitation.status {
        InvitationStatus::Accepted => {
            return Err(ApiError::from(DomainError::InvitationAlreadyAccepted));
        }
        InvitationStatus::Revoked => {
            return Err(ApiError::from(DomainError::InvitationRevoked));
        }
        InvitationStatus::Expired => {
            return Err(ApiError::from(DomainError::InvitationExpired));
        }
        InvitationStatus::Pending => {
            // Check expiration
            if invitation.expires_at < Utc::now() {
                return Err(ApiError::from(DomainError::InvitationExpired));
            }
        }
    }

    // Check if email already exists in this organization
    if UserRepository::find_by_email(&state.pool, invitation.organization_id, &req.email)
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

    // Determine role based on invitation type
    let (user_role, membership_role) = match invitation.invitation_type {
        InvitationType::Walker => (UserRole::Walker, MembershipRole::Walker),
        InvitationType::Client => (UserRole::Customer, MembershipRole::Customer),
    };

    // Create user (without organization_id - users are now org-agnostic)
    let user = UserRepository::create(
        &state.pool,
        CreateUser {
            organization_id: invitation.organization_id, // Keep for backwards compat
            email: req.email.to_lowercase(),
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
            organization_id: invitation.organization_id,
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

    // Mark invitation as accepted
    InvitationRepository::accept(&state.pool, invitation.id, user.id).await?;

    // Get organization info for the response
    let org = OrganizationRepository::find_by_id(&state.pool, invitation.organization_id)
        .await?
        .ok_or_else(|| {
            ApiError::from(DomainError::OrganizationNotFound(
                invitation.organization_id.to_string(),
            ))
        })?;

    // Create token with membership context
    let token = create_token(user.id, Some(invitation.organization_id), &state.jwt_secret)
        .map_err(|_| {
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
            organization_id: org.id.to_string(),
            organization_name: org.name,
            organization_slug: org.slug,
            role: membership_role.to_string(),
            is_default: true,
        }),
        memberships: None,
    }))
}

/// Join an organization using an invite token (for existing users)
/// POST /walker/join-tenant
pub async fn join_tenant(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(req): Json<JoinTenantRequest>,
) -> ApiResult<Json<AuthResponse>> {
    // Find and validate invitation
    let invitation = InvitationRepository::find_valid_by_token(&state.pool, &req.invite_token)
        .await?
        .ok_or_else(|| ApiError::from(DomainError::InvitationNotFound))?;

    // Get current user
    let user = UserRepository::find_by_id_unchecked(&state.pool, auth_user.user_id)
        .await?
        .ok_or_else(|| ApiError::from(DomainError::UserNotFound(auth_user.user_id.to_string())))?;

    // Determine membership role based on invitation type
    let membership_role = match invitation.invitation_type {
        InvitationType::Walker => MembershipRole::Walker,
        InvitationType::Client => MembershipRole::Customer,
    };

    // Check if user already has this role in this organization
    let existing_membership = MembershipRepository::find_by_user_org_role(
        &state.pool,
        auth_user.user_id,
        invitation.organization_id,
        membership_role,
    )
    .await?;

    if existing_membership.is_some() {
        return Err(ApiError::from(DomainError::AlreadyMember));
    }

    // Create membership for the user in this organization
    let membership = MembershipRepository::create(
        &state.pool,
        CreateMembership {
            user_id: auth_user.user_id,
            organization_id: invitation.organization_id,
            role: membership_role,
            status: Some(MembershipStatus::Active),
            title: None,
        },
    )
    .await?;

    // Mark invitation as accepted
    InvitationRepository::accept(&state.pool, invitation.id, auth_user.user_id).await?;

    // Get organization info for the response
    let org = OrganizationRepository::find_by_id(&state.pool, invitation.organization_id)
        .await?
        .ok_or_else(|| {
            ApiError::from(DomainError::OrganizationNotFound(
                invitation.organization_id.to_string(),
            ))
        })?;

    // Create new token with the new membership context
    let token = create_token(
        auth_user.user_id,
        Some(invitation.organization_id),
        &state.jwt_secret,
    )
    .map_err(|_| {
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
            role: membership.role.to_string(),
        },
        membership: Some(MembershipInfo {
            id: membership.id.to_string(),
            organization_id: org.id.to_string(),
            organization_name: org.name,
            organization_slug: org.slug,
            role: membership.role.to_string(),
            is_default: false,
        }),
        memberships: None,
    }))
}

/// Create a new tenant/organization (for walkers starting their own business)
/// POST /walker/create-tenant
pub async fn create_tenant(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(req): Json<CreateTenantRequest>,
) -> ApiResult<Json<CreateTenantResponse>> {
    // Generate slug from business name
    let base_slug = req
        .slug
        .unwrap_or_else(|| generate_slug(&req.business_name));

    // Make slug unique by adding random suffix if needed
    let mut slug = base_slug.clone();
    let mut attempts = 0;
    while OrganizationRepository::find_by_slug(&state.pool, &slug)
        .await?
        .is_some()
    {
        attempts += 1;
        if attempts > 10 {
            return Err(ApiError::from(DomainError::SlugAlreadyExists(base_slug)));
        }
        let mut rng = rand::thread_rng();
        let suffix: u32 = rng.gen_range(1000..9999);
        slug = format!("{}-{}", base_slug, suffix);
    }

    // Create organization
    let organization = OrganizationRepository::create(
        &state.pool,
        CreateOrganization {
            name: req.business_name.clone(),
            slug: slug.clone(),
            subdomain: Some(slug.clone()),
            custom_domain: None,
            settings: Some(OrganizationSettings::default()),
        },
    )
    .await?;

    // Create tenant database record (in production, you'd provision the actual DB)
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost/offleash".to_string());

    TenantDatabaseRepository::create(
        &state.pool,
        CreateTenantDatabase {
            organization_id: organization.id,
            connection_string: database_url,
            status: None, // Will default to Active
        },
    )
    .await?;

    // Get current user
    let user = UserRepository::find_by_id_unchecked(&state.pool, auth_user.user_id)
        .await?
        .ok_or_else(|| ApiError::from(DomainError::UserNotFound(auth_user.user_id.to_string())))?;

    // Create owner membership for the user in this new organization
    let membership = MembershipRepository::create(
        &state.pool,
        CreateMembership {
            user_id: auth_user.user_id,
            organization_id: organization.id,
            role: MembershipRole::Owner,
            status: Some(MembershipStatus::Active),
            title: Some("Business Owner".to_string()),
        },
    )
    .await?;

    // Set this as the user's default membership
    sqlx::query("UPDATE users SET default_membership_id = $1 WHERE id = $2")
        .bind(membership.id)
        .bind(auth_user.user_id)
        .execute(&state.pool)
        .await?;

    // Create new token with the new organization context
    let token = create_token(auth_user.user_id, Some(organization.id), &state.jwt_secret).map_err(
        |_| {
            ApiError::from(shared::AppError::Internal(
                "Token creation failed".to_string(),
            ))
        },
    )?;

    Ok(Json(CreateTenantResponse {
        success: true,
        organization_id: organization.id.to_string(),
        slug,
        token,
        user: UserResponse {
            id: user.id.to_string(),
            email: user.email,
            first_name: user.first_name,
            last_name: user.last_name,
            role: MembershipRole::Owner.to_string(),
        },
    }))
}

/// List invitations for current organization
/// GET /invitations
pub async fn list_invitations(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> ApiResult<Json<Vec<InvitationInfoResponse>>> {
    let org_id = auth_user.org_id.ok_or_else(|| {
        ApiError::from(DomainError::Unauthorized(
            "Organization context required".to_string(),
        ))
    })?;

    // Verify user is admin
    let user = UserRepository::find_by_id_unchecked(&state.pool, auth_user.user_id)
        .await?
        .ok_or_else(|| ApiError::from(DomainError::UserNotFound(auth_user.user_id.to_string())))?;

    if user.role != UserRole::Admin {
        return Err(ApiError::from(DomainError::Unauthorized(
            "Only admins can view invitations".to_string(),
        )));
    }

    let invitations = InvitationRepository::list_by_organization(&state.pool, org_id).await?;

    // Get organization name
    let org = OrganizationRepository::find_by_id(&state.pool, org_id)
        .await?
        .ok_or_else(|| ApiError::from(DomainError::OrganizationNotFound(org_id.to_string())))?;

    let mut responses = Vec::new();
    for inv in invitations {
        // Get inviter name
        let inviter_name = UserRepository::find_by_id_unchecked(&state.pool, inv.invited_by)
            .await?
            .map(|u| u.full_name())
            .unwrap_or_else(|| "Unknown".to_string());

        responses.push(InvitationInfoResponse {
            id: inv.id.to_string(),
            invitation_type: inv.invitation_type.to_string(),
            organization_name: org.name.clone(),
            inviter_name,
            email: inv.email,
            phone: inv.phone,
            status: inv.status.to_string(),
            expires_at: inv.expires_at.to_rfc3339(),
            created_at: inv.created_at.to_rfc3339(),
        });
    }

    Ok(Json(responses))
}

/// Revoke an invitation
/// DELETE /invitations/:id
pub async fn revoke_invitation(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(invitation_id): Path<Uuid>,
) -> ApiResult<Json<InviteResponse>> {
    let org_id = auth_user.org_id.ok_or_else(|| {
        ApiError::from(DomainError::Unauthorized(
            "Organization context required".to_string(),
        ))
    })?;

    // Verify user is admin
    let user = UserRepository::find_by_id_unchecked(&state.pool, auth_user.user_id)
        .await?
        .ok_or_else(|| ApiError::from(DomainError::UserNotFound(auth_user.user_id.to_string())))?;

    if user.role != UserRole::Admin {
        return Err(ApiError::from(DomainError::Unauthorized(
            "Only admins can revoke invitations".to_string(),
        )));
    }

    // Get invitation and verify it belongs to this organization
    let invitation = InvitationRepository::find_by_id(&state.pool, invitation_id)
        .await?
        .ok_or_else(|| ApiError::from(DomainError::InvitationNotFound))?;

    if invitation.organization_id != org_id {
        return Err(ApiError::from(DomainError::InvitationNotFound));
    }

    // Revoke it
    InvitationRepository::revoke(&state.pool, invitation_id).await?;

    Ok(Json(InviteResponse {
        success: true,
        message: "Invitation revoked successfully".to_string(),
        invitation_id: Some(invitation_id.to_string()),
    }))
}
