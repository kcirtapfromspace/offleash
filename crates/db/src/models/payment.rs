use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared::types::{BookingId, OrganizationId, PaymentId, UserId};
use sqlx::FromRow;

/// Payment status enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "payment_status", rename_all = "snake_case")]
pub enum PaymentStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Refunded,
    PartiallyRefunded,
}

impl std::fmt::Display for PaymentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PaymentStatus::Pending => write!(f, "pending"),
            PaymentStatus::Processing => write!(f, "processing"),
            PaymentStatus::Completed => write!(f, "completed"),
            PaymentStatus::Failed => write!(f, "failed"),
            PaymentStatus::Refunded => write!(f, "refunded"),
            PaymentStatus::PartiallyRefunded => write!(f, "partially_refunded"),
        }
    }
}

/// Payment method enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "payment_method", rename_all = "snake_case")]
pub enum PaymentMethod {
    Card,
    ApplePay,
    GooglePay,
}

/// Payment database model
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Payment {
    pub id: PaymentId,
    pub organization_id: Option<OrganizationId>,
    pub booking_id: BookingId,
    pub customer_id: UserId,
    pub amount_cents: i64,
    pub status: PaymentStatus,
    pub square_payment_id: Option<String>,
    pub square_order_id: Option<String>,
    pub payment_method: PaymentMethod,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Payment {
    pub fn amount_dollars(&self) -> f64 {
        self.amount_cents as f64 / 100.0
    }

    pub fn is_successful(&self) -> bool {
        self.status == PaymentStatus::Completed
    }
}

/// Input for creating a new payment
#[derive(Debug, Clone, Deserialize)]
pub struct CreatePayment {
    pub organization_id: OrganizationId,
    pub booking_id: BookingId,
    pub customer_id: UserId,
    pub amount_cents: i64,
    pub payment_method: PaymentMethod,
}
