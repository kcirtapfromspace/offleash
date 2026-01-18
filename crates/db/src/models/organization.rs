use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared::types::OrganizationId;
use sqlx::FromRow;

/// Organization settings including branding configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OrganizationSettings {
    /// Primary brand color (hex format, e.g., "#FF5733")
    pub primary_color: Option<String>,
    /// Secondary brand color (hex format)
    pub secondary_color: Option<String>,
    /// URL to the organization's logo
    pub logo_url: Option<String>,
    /// URL to the organization's favicon
    pub favicon_url: Option<String>,
    /// Custom font family for the organization
    pub font_family: Option<String>,
}

/// Organization database model
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Organization {
    pub id: OrganizationId,
    pub name: String,
    pub slug: String,
    pub subdomain: String,
    pub custom_domain: Option<String>,
    pub settings: sqlx::types::Json<OrganizationSettings>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Input for creating a new organization
#[derive(Debug, Clone, Deserialize)]
pub struct CreateOrganization {
    pub name: String,
    pub slug: String,
    pub subdomain: Option<String>,
    pub custom_domain: Option<String>,
    pub settings: Option<OrganizationSettings>,
}

/// Input for updating an organization
#[derive(Debug, Clone, Deserialize, Default)]
pub struct UpdateOrganization {
    pub name: Option<String>,
    pub slug: Option<String>,
    pub custom_domain: Option<String>,
    pub settings: Option<OrganizationSettings>,
}
