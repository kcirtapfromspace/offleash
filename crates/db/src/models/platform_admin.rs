use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared::types::PlatformAdminId;
use sqlx::FromRow;

/// Platform admin database model
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct PlatformAdmin {
    pub id: PlatformAdminId,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub first_name: String,
    pub last_name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Input for creating a new platform admin
#[derive(Debug, Clone, Deserialize)]
pub struct CreatePlatformAdmin {
    pub email: String,
    pub password_hash: String,
    pub first_name: String,
    pub last_name: String,
}
