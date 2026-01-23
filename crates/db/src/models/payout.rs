use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared::types::OrganizationId;
use sqlx::FromRow;
use uuid::Uuid;

/// Payout status enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "payout_status", rename_all = "snake_case")]
pub enum PayoutStatus {
    Pending,
    InTransit,
    Paid,
    Failed,
    Canceled,
}

impl std::fmt::Display for PayoutStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PayoutStatus::Pending => write!(f, "pending"),
            PayoutStatus::InTransit => write!(f, "in_transit"),
            PayoutStatus::Paid => write!(f, "paid"),
            PayoutStatus::Failed => write!(f, "failed"),
            PayoutStatus::Canceled => write!(f, "canceled"),
        }
    }
}

/// Payout settings for tenants using platform default
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct PayoutSettings {
    pub id: Uuid,
    pub organization_id: OrganizationId,
    pub payout_method: String, // bank, debit
    pub stripe_bank_account_id: Option<String>,
    pub square_bank_account_id: Option<String>,
    pub bank_name: Option<String>,
    pub bank_account_last_four: Option<String>,
    pub bank_routing_last_four: Option<String>,
    pub payout_schedule: String, // daily, weekly, monthly
    pub payout_day_of_week: Option<i32>,
    pub payout_day_of_month: Option<i32>,
    pub minimum_payout_cents: i32,
    pub is_verified: bool,
    pub verification_status: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl PayoutSettings {
    /// Get display-friendly schedule description
    pub fn schedule_display(&self) -> String {
        match self.payout_schedule.as_str() {
            "daily" => "Daily".to_string(),
            "weekly" => {
                let day = self.payout_day_of_week.unwrap_or(1);
                let day_name = match day {
                    0 => "Sunday",
                    1 => "Monday",
                    2 => "Tuesday",
                    3 => "Wednesday",
                    4 => "Thursday",
                    5 => "Friday",
                    6 => "Saturday",
                    _ => "Monday",
                };
                format!("Weekly on {}", day_name)
            }
            "monthly" => {
                let day = self.payout_day_of_month.unwrap_or(1);
                format!("Monthly on the {}th", day)
            }
            _ => self.payout_schedule.clone(),
        }
    }

    /// Get minimum payout in dollars
    pub fn minimum_payout_dollars(&self) -> f64 {
        self.minimum_payout_cents as f64 / 100.0
    }

    /// Check if payout settings are complete and verified
    pub fn is_ready(&self) -> bool {
        self.is_verified
            && (self.stripe_bank_account_id.is_some() || self.square_bank_account_id.is_some())
    }
}

/// Input for creating payout settings
#[derive(Debug, Clone, Deserialize)]
pub struct CreatePayoutSettings {
    pub payout_method: String,
    pub payout_schedule: String,
    pub payout_day_of_week: Option<i32>,
    pub payout_day_of_month: Option<i32>,
    pub minimum_payout_cents: Option<i32>,
}

/// Input for updating payout settings
#[derive(Debug, Clone, Deserialize, Default)]
pub struct UpdatePayoutSettings {
    pub payout_method: Option<String>,
    pub stripe_bank_account_id: Option<String>,
    pub square_bank_account_id: Option<String>,
    pub bank_name: Option<String>,
    pub bank_account_last_four: Option<String>,
    pub bank_routing_last_four: Option<String>,
    pub payout_schedule: Option<String>,
    pub payout_day_of_week: Option<i32>,
    pub payout_day_of_month: Option<i32>,
    pub minimum_payout_cents: Option<i32>,
    pub is_verified: Option<bool>,
    pub verification_status: Option<String>,
}

/// Payout record
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Payout {
    pub id: Uuid,
    pub organization_id: OrganizationId,
    pub amount_cents: i32,
    pub fee_cents: i32,
    pub net_amount_cents: i32,
    pub currency: String,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub stripe_payout_id: Option<String>,
    pub stripe_transfer_id: Option<String>,
    pub square_payout_id: Option<String>,
    pub status: PayoutStatus,
    pub initiated_at: Option<DateTime<Utc>>,
    pub arrival_date: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub failure_code: Option<String>,
    pub failure_message: Option<String>,
    pub transaction_count: i32,
    pub transaction_ids: Vec<Uuid>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Payout {
    /// Get net amount in dollars
    pub fn net_amount_dollars(&self) -> f64 {
        self.net_amount_cents as f64 / 100.0
    }

    /// Check if payout was successful
    pub fn is_successful(&self) -> bool {
        self.status == PayoutStatus::Paid
    }

    /// Check if payout can be retried
    pub fn can_retry(&self) -> bool {
        self.status == PayoutStatus::Failed
    }
}

/// Input for creating a payout
#[derive(Debug, Clone, Deserialize)]
pub struct CreatePayout {
    pub amount_cents: i32,
    pub fee_cents: i32,
    pub net_amount_cents: i32,
    pub currency: String,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub transaction_count: i32,
    pub transaction_ids: Vec<Uuid>,
}

/// Input for updating a payout
#[derive(Debug, Clone, Deserialize, Default)]
pub struct UpdatePayout {
    pub stripe_payout_id: Option<String>,
    pub stripe_transfer_id: Option<String>,
    pub square_payout_id: Option<String>,
    pub status: Option<PayoutStatus>,
    pub initiated_at: Option<DateTime<Utc>>,
    pub arrival_date: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub failure_code: Option<String>,
    pub failure_message: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// Payout summary for dashboard
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct PayoutSummary {
    pub total_payouts: i64,
    pub total_amount_cents: i64,
    pub pending_amount_cents: i64,
    pub last_payout_date: Option<DateTime<Utc>>,
    pub next_payout_date: Option<DateTime<Utc>>,
}
