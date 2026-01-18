use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared::types::{OrganizationId, ServiceId};
use sqlx::FromRow;

/// Service database model
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Service {
    pub id: ServiceId,
    pub organization_id: OrganizationId,
    pub name: String,
    pub description: Option<String>,
    pub duration_minutes: i32,
    pub base_price_cents: i64,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Service {
    pub fn price_dollars(&self) -> f64 {
        self.base_price_cents as f64 / 100.0
    }
}

/// Input for creating a new service
#[derive(Debug, Clone, Deserialize)]
pub struct CreateService {
    pub organization_id: OrganizationId,
    pub name: String,
    pub description: Option<String>,
    pub duration_minutes: i32,
    pub base_price_cents: i64,
}

/// Input for updating a service
#[derive(Debug, Clone, Deserialize, Default)]
pub struct UpdateService {
    pub name: Option<String>,
    pub description: Option<String>,
    pub duration_minutes: Option<i32>,
    pub base_price_cents: Option<i64>,
    pub is_active: Option<bool>,
}
