use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared::types::{OrganizationId, TenantDatabaseId};
use sqlx::FromRow;

/// Status of a tenant database
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "tenant_db_status", rename_all = "lowercase")]
pub enum TenantDbStatus {
    Active,
    Inactive,
    Provisioning,
}

/// Tenant database record for multi-tenant database connections
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct TenantDatabase {
    pub id: TenantDatabaseId,
    pub organization_id: OrganizationId,
    pub connection_string: String,
    pub status: TenantDbStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Input for creating a new tenant database record
#[derive(Debug, Clone, Deserialize)]
pub struct CreateTenantDatabase {
    pub organization_id: OrganizationId,
    pub connection_string: String,
    pub status: Option<TenantDbStatus>,
}
