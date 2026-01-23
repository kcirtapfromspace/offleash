use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared::types::OrganizationId;
use sqlx::FromRow;
use uuid::Uuid;

/// Payment provider type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "payment_provider_type", rename_all = "snake_case")]
pub enum PaymentProviderType {
    Stripe,
    Square,
    Platform,
}

impl std::fmt::Display for PaymentProviderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PaymentProviderType::Stripe => write!(f, "stripe"),
            PaymentProviderType::Square => write!(f, "square"),
            PaymentProviderType::Platform => write!(f, "platform"),
        }
    }
}

/// Payment provider database model - stores tenant payment processor configurations
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct PaymentProvider {
    pub id: Uuid,
    pub organization_id: OrganizationId,
    pub provider_type: PaymentProviderType,
    // Stripe Connect
    pub stripe_account_id: Option<String>,
    pub stripe_account_type: Option<String>,
    // Square
    pub square_merchant_id: Option<String>,
    // Encrypted tokens
    pub access_token_encrypted: Option<String>,
    pub refresh_token_encrypted: Option<String>,
    pub token_expires_at: Option<DateTime<Utc>>,
    // Webhook secret for signature verification
    pub webhook_secret: Option<String>,
    // Status
    pub is_active: bool,
    pub is_primary: bool,
    pub is_verified: bool,
    pub verification_status: Option<String>,
    // Capabilities
    pub charges_enabled: Option<bool>,
    pub payouts_enabled: Option<bool>,
    // Display info
    pub account_name: Option<String>,
    pub merchant_id: Option<String>,
    // Metadata
    pub metadata: Option<serde_json::Value>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

impl PaymentProvider {
    /// Check if this provider can process payments
    pub fn can_process_payments(&self) -> bool {
        self.is_active && self.is_verified && self.charges_enabled.unwrap_or(false)
    }

    /// Check if this provider can receive payouts
    pub fn can_receive_payouts(&self) -> bool {
        self.is_active && self.is_verified && self.payouts_enabled.unwrap_or(false)
    }

    /// Get display name for the provider
    pub fn display_name(&self) -> &'static str {
        match self.provider_type {
            PaymentProviderType::Stripe => "Stripe",
            PaymentProviderType::Square => "Square",
            PaymentProviderType::Platform => "Platform Default",
        }
    }
}

/// Input for creating a new payment provider
#[derive(Debug, Clone, Deserialize)]
pub struct CreatePaymentProvider {
    pub provider_type: PaymentProviderType,
    pub stripe_account_id: Option<String>,
    pub stripe_account_type: Option<String>,
    pub square_merchant_id: Option<String>,
    pub access_token_encrypted: Option<String>,
    pub refresh_token_encrypted: Option<String>,
    pub token_expires_at: Option<DateTime<Utc>>,
}

/// Input for updating a payment provider
#[derive(Debug, Clone, Deserialize, Default)]
pub struct UpdatePaymentProvider {
    pub access_token_encrypted: Option<String>,
    pub refresh_token_encrypted: Option<String>,
    pub token_expires_at: Option<DateTime<Utc>>,
    pub is_active: Option<bool>,
    pub is_verified: Option<bool>,
    pub verification_status: Option<String>,
    pub charges_enabled: Option<bool>,
    pub payouts_enabled: Option<bool>,
    pub metadata: Option<serde_json::Value>,
}

/// Stripe Connect OAuth state for secure callback verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StripeConnectState {
    pub organization_id: OrganizationId,
    pub account_type: String, // "standard" or "express"
    pub redirect_url: String,
    pub created_at: DateTime<Utc>,
}

/// Square OAuth state for secure callback verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SquareOAuthState {
    pub organization_id: OrganizationId,
    pub redirect_url: String,
    pub created_at: DateTime<Utc>,
}
