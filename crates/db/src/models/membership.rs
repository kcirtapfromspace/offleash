use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared::types::{OrganizationId, UserId};
use sqlx::FromRow;
use uuid::Uuid;

/// Membership role - more granular than the old UserRole
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "membership_role", rename_all = "lowercase")]
pub enum MembershipRole {
    Owner,    // Created the organization
    Admin,    // Full management access
    Walker,   // Service provider
    Customer, // Books services
}

impl std::fmt::Display for MembershipRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MembershipRole::Owner => write!(f, "owner"),
            MembershipRole::Admin => write!(f, "admin"),
            MembershipRole::Walker => write!(f, "walker"),
            MembershipRole::Customer => write!(f, "customer"),
        }
    }
}

impl MembershipRole {
    /// Check if this role has management privileges
    pub fn is_manager(&self) -> bool {
        matches!(self, MembershipRole::Owner | MembershipRole::Admin)
    }

    /// Check if this role can provide services
    pub fn is_service_provider(&self) -> bool {
        matches!(
            self,
            MembershipRole::Owner | MembershipRole::Admin | MembershipRole::Walker
        )
    }

    /// Check if this role can invite others
    pub fn can_invite(&self, target_role: MembershipRole) -> bool {
        match self {
            MembershipRole::Owner | MembershipRole::Admin => true,
            MembershipRole::Walker => target_role == MembershipRole::Customer,
            MembershipRole::Customer => false,
        }
    }
}

/// Membership status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "membership_status", rename_all = "lowercase")]
pub enum MembershipStatus {
    Active,
    Suspended,
    Pending,
    Declined,
}

impl std::fmt::Display for MembershipStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MembershipStatus::Active => write!(f, "active"),
            MembershipStatus::Suspended => write!(f, "suspended"),
            MembershipStatus::Pending => write!(f, "pending"),
            MembershipStatus::Declined => write!(f, "declined"),
        }
    }
}

/// Membership database model
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Membership {
    pub id: Uuid,
    pub user_id: UserId,
    pub organization_id: OrganizationId,
    pub role: MembershipRole,
    pub status: MembershipStatus,
    pub title: Option<String>,
    pub permissions: serde_json::Value,
    pub joined_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Membership {
    pub fn is_active(&self) -> bool {
        self.status == MembershipStatus::Active
    }
}

/// Input for creating a new membership
#[derive(Debug, Clone)]
pub struct CreateMembership {
    pub user_id: UserId,
    pub organization_id: OrganizationId,
    pub role: MembershipRole,
    pub status: Option<MembershipStatus>,
    pub title: Option<String>,
}

/// Input for updating a membership
#[derive(Debug, Clone, Default)]
pub struct UpdateMembership {
    pub role: Option<MembershipRole>,
    pub status: Option<MembershipStatus>,
    pub title: Option<String>,
    pub permissions: Option<serde_json::Value>,
}

/// Membership with organization details (for listing user's memberships)
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct MembershipWithOrg {
    pub id: Uuid,
    pub user_id: UserId,
    pub organization_id: OrganizationId,
    pub organization_name: String,
    pub organization_slug: String,
    pub role: MembershipRole,
    pub status: MembershipStatus,
    pub title: Option<String>,
    pub joined_at: DateTime<Utc>,
}

/// Context for authenticated requests - replaces the old AuthUser org_id
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserContext {
    pub user_id: UserId,
    pub membership_id: Option<Uuid>,
    pub organization_id: Option<OrganizationId>,
    pub role: Option<MembershipRole>,
}

impl UserContext {
    /// Create context without organization (browsing, account management)
    pub fn without_org(user_id: UserId) -> Self {
        Self {
            user_id,
            membership_id: None,
            organization_id: None,
            role: None,
        }
    }

    /// Create context with organization membership
    pub fn with_membership(user_id: UserId, membership: &Membership) -> Self {
        Self {
            user_id,
            membership_id: Some(membership.id),
            organization_id: Some(membership.organization_id),
            role: Some(membership.role),
        }
    }

    /// Check if user has organization context
    pub fn has_org_context(&self) -> bool {
        self.organization_id.is_some()
    }

    /// Check if user is a manager in current context
    pub fn is_manager(&self) -> bool {
        self.role.map(|r| r.is_manager()).unwrap_or(false)
    }

    /// Check if user is a service provider in current context
    pub fn is_service_provider(&self) -> bool {
        self.role.map(|r| r.is_service_provider()).unwrap_or(false)
    }
}
