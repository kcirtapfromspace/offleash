use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared::types::OrganizationId;
use sqlx::FromRow;

/// Business model type for the organization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum BusinessModel {
    /// Booking management only - no customer payments processed
    #[default]
    BookingOnly,
    /// Full service with customer payments
    FullService,
}

/// Fee structure for organizations using customer payments
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum FeeStructure {
    /// Service fees passed to customer (owner pays no fees)
    #[default]
    CustomerPays,
    /// Fees split between customer and owner
    SplitFees,
    /// Owner pays subscription, no fees to customer
    OwnerSubscription,
}

/// Billing frequency for booking-only plans
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum BillingFrequency {
    #[default]
    Monthly,
    Yearly,
}

/// Payment configuration for the organization
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PaymentConfig {
    /// Business model type
    #[serde(default)]
    pub business_model: BusinessModel,
    /// Fee structure (only relevant for full_service)
    #[serde(default)]
    pub fee_structure: FeeStructure,
    /// Billing frequency (only relevant for booking_only)
    #[serde(default)]
    pub billing_frequency: BillingFrequency,
    /// Whether Apple Pay is enabled
    #[serde(default)]
    pub apple_pay_enabled: bool,
    /// Whether Google Pay is enabled
    #[serde(default)]
    pub google_pay_enabled: bool,
    /// Preferred payment provider (stripe, square)
    pub preferred_provider: Option<String>,
}

/// Organization settings including branding and payment configuration
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
    /// Payment and business model configuration
    #[serde(default)]
    pub payment_config: PaymentConfig,
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
