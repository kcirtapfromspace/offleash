use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared::types::{BookingId, OrganizationId, UserId};
use sqlx::FromRow;
use uuid::Uuid;

/// Transaction status enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "transaction_status", rename_all = "snake_case")]
pub enum TransactionStatus {
    Pending,
    Processing,
    Succeeded,
    Failed,
    Refunded,
    PartiallyRefunded,
    Disputed,
}

impl std::fmt::Display for TransactionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionStatus::Pending => write!(f, "pending"),
            TransactionStatus::Processing => write!(f, "processing"),
            TransactionStatus::Succeeded => write!(f, "succeeded"),
            TransactionStatus::Failed => write!(f, "failed"),
            TransactionStatus::Refunded => write!(f, "refunded"),
            TransactionStatus::PartiallyRefunded => write!(f, "partially_refunded"),
            TransactionStatus::Disputed => write!(f, "disputed"),
        }
    }
}

/// Transaction database model - comprehensive payment transaction records
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Transaction {
    pub id: Uuid,
    pub organization_id: OrganizationId,
    pub booking_id: Option<BookingId>,
    pub customer_user_id: UserId,
    pub provider_user_id: UserId,
    pub payment_method_id: Option<Uuid>,
    pub provider_id: Uuid,

    // Amounts (all in cents)
    pub subtotal_cents: i32,
    pub tip_cents: i32,
    pub customer_fee_cents: i32,
    pub provider_fee_cents: i32,
    pub platform_fee_cents: i32,
    pub tax_cents: i32,
    pub processing_fee_cents: i32,
    pub total_cents: i32,
    pub provider_payout_cents: i32,

    pub currency: String,
    pub status: TransactionStatus,

    // External payment reference (provider-agnostic)
    pub external_payment_id: Option<String>,

    // Provider-specific references
    pub stripe_payment_intent_id: Option<String>,
    pub stripe_charge_id: Option<String>,
    pub stripe_transfer_id: Option<String>,
    pub square_payment_id: Option<String>,
    pub square_order_id: Option<String>,

    // Tax details
    pub tax_rate_percent: Option<rust_decimal::Decimal>,
    pub tax_jurisdiction: Option<String>,
    pub tax_calculation_id: Option<String>,

    // Refund tracking
    pub refunded_amount_cents: i32,

    // Failure info
    pub failure_code: Option<String>,
    pub failure_message: Option<String>,

    // Metadata
    pub description: Option<String>,
    pub metadata: Option<serde_json::Value>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Transaction {
    /// Convert cents to dollars for display
    pub fn total_dollars(&self) -> f64 {
        self.total_cents as f64 / 100.0
    }

    pub fn subtotal_dollars(&self) -> f64 {
        self.subtotal_cents as f64 / 100.0
    }

    pub fn provider_payout_dollars(&self) -> f64 {
        self.provider_payout_cents as f64 / 100.0
    }

    /// Check if transaction is successful
    pub fn is_successful(&self) -> bool {
        self.status == TransactionStatus::Succeeded
    }

    /// Check if transaction can be refunded
    pub fn can_refund(&self) -> bool {
        matches!(
            self.status,
            TransactionStatus::Succeeded | TransactionStatus::PartiallyRefunded
        ) && self.refunded_amount_cents < self.total_cents
    }

    /// Get remaining refundable amount
    pub fn refundable_amount_cents(&self) -> i32 {
        if self.can_refund() {
            self.total_cents - self.refunded_amount_cents
        } else {
            0
        }
    }
}

/// Input for creating a new transaction
#[derive(Debug, Clone, Deserialize)]
pub struct CreateTransaction {
    pub booking_id: Option<BookingId>,
    pub customer_user_id: UserId,
    pub provider_user_id: UserId,
    pub payment_method_id: Option<Uuid>,
    pub provider_id: Uuid,
    pub subtotal_cents: i32,
    pub tip_cents: i32,
    pub customer_fee_cents: i32,
    pub provider_fee_cents: i32,
    pub platform_fee_cents: i32,
    pub tax_cents: i32,
    pub processing_fee_cents: i32,
    pub total_cents: i32,
    pub provider_payout_cents: i32,
    pub currency: String,
    pub tax_rate_percent: Option<rust_decimal::Decimal>,
    pub tax_jurisdiction: Option<String>,
    pub description: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// Input for updating a transaction
#[derive(Debug, Clone, Deserialize, Default)]
pub struct UpdateTransaction {
    pub status: Option<TransactionStatus>,
    pub stripe_payment_intent_id: Option<String>,
    pub stripe_charge_id: Option<String>,
    pub stripe_transfer_id: Option<String>,
    pub square_payment_id: Option<String>,
    pub square_order_id: Option<String>,
    pub tax_calculation_id: Option<String>,
    pub refunded_amount_cents: Option<i32>,
    pub failure_code: Option<String>,
    pub failure_message: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// Transaction fee breakdown for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionFeeBreakdown {
    pub subtotal_cents: i32,
    pub customer_fee_cents: i32,
    pub customer_fee_percent: f64,
    pub provider_fee_cents: i32,
    pub provider_fee_percent: f64,
    pub platform_fee_cents: i32,
    pub tax_cents: i32,
    pub tax_rate_percent: f64,
    pub processing_fee_cents: i32,
    pub total_cents: i32,
    pub provider_payout_cents: i32,
}

impl TransactionFeeBreakdown {
    /// Calculate fee breakdown from amounts
    ///
    /// Fee model:
    /// - Customer pays: subtotal + customer_fee + tax = total
    /// - Provider receives: subtotal - provider_fee - processing_fee = provider_payout
    /// - Platform earns: customer_fee + provider_fee (platform_fee)
    pub fn calculate(
        subtotal_cents: i32,
        customer_fee_percent: f64,
        provider_fee_percent: f64,
        tax_rate_percent: f64,
        processing_fee_percent: f64,
    ) -> Self {
        // Customer service fee (paid by customer)
        let customer_fee_cents = ((subtotal_cents as f64) * customer_fee_percent).round() as i32;

        // Tax on subtotal + customer fee
        let taxable_amount = subtotal_cents + customer_fee_cents;
        let tax_cents = ((taxable_amount as f64) * (tax_rate_percent / 100.0)).round() as i32;

        // Total paid by customer
        let total_cents = subtotal_cents + customer_fee_cents + tax_cents;

        // Provider fee (taken from provider's share)
        let provider_fee_cents = ((subtotal_cents as f64) * provider_fee_percent).round() as i32;

        // Processing fee (Stripe/Square - typically 2.9% + $0.30)
        let processing_fee_cents =
            ((total_cents as f64) * (processing_fee_percent / 100.0)).round() as i32 + 30;

        // Platform total revenue (customer fee + provider fee)
        let platform_fee_cents = customer_fee_cents + provider_fee_cents;

        // What provider receives
        let provider_payout_cents = subtotal_cents - provider_fee_cents;

        Self {
            subtotal_cents,
            customer_fee_cents,
            customer_fee_percent,
            provider_fee_cents,
            provider_fee_percent,
            platform_fee_cents,
            tax_cents,
            tax_rate_percent,
            processing_fee_cents,
            total_cents,
            provider_payout_cents,
        }
    }
}
