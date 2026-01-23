use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared::types::{OrganizationId, UserId};
use sqlx::FromRow;
use uuid::Uuid;

/// Invitation type enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "invitation_type", rename_all = "lowercase")]
pub enum InvitationType {
    Walker,
    Client,
}

impl std::fmt::Display for InvitationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InvitationType::Walker => write!(f, "walker"),
            InvitationType::Client => write!(f, "client"),
        }
    }
}

/// Invitation status enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "invitation_status", rename_all = "lowercase")]
pub enum InvitationStatus {
    Pending,
    Accepted,
    Expired,
    Revoked,
}

impl std::fmt::Display for InvitationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InvitationStatus::Pending => write!(f, "pending"),
            InvitationStatus::Accepted => write!(f, "accepted"),
            InvitationStatus::Expired => write!(f, "expired"),
            InvitationStatus::Revoked => write!(f, "revoked"),
        }
    }
}

/// Invitation database model
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Invitation {
    pub id: Uuid,
    pub organization_id: OrganizationId,
    pub invited_by: UserId,
    pub invitation_type: InvitationType,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub token: String,
    #[serde(skip_serializing)]
    pub token_hash: String,
    pub status: InvitationStatus,
    pub expires_at: DateTime<Utc>,
    pub accepted_at: Option<DateTime<Utc>>,
    pub accepted_by: Option<UserId>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Invitation {
    /// Check if the invitation is still valid (pending and not expired)
    pub fn is_valid(&self) -> bool {
        self.status == InvitationStatus::Pending && self.expires_at > Utc::now()
    }

    /// Get contact info (email or phone)
    pub fn contact_info(&self) -> String {
        self.email
            .clone()
            .or_else(|| self.phone.clone())
            .unwrap_or_else(|| "unknown".to_string())
    }
}

/// Input for creating a new invitation
#[derive(Debug, Clone)]
pub struct CreateInvitation {
    pub organization_id: OrganizationId,
    pub invited_by: UserId,
    pub invitation_type: InvitationType,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub token: String,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
}

/// Response for invitation info (safe to send to client)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvitationInfo {
    pub id: Uuid,
    pub organization_name: String,
    pub invitation_type: InvitationType,
    pub inviter_name: String,
    pub expires_at: DateTime<Utc>,
}
