use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared::types::OrganizationId;
use sqlx::FromRow;
use uuid::Uuid;

/// Dispute status enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "dispute_status", rename_all = "snake_case")]
pub enum DisputeStatus {
    NeedsResponse,
    UnderReview,
    Won,
    Lost,
}

impl std::fmt::Display for DisputeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DisputeStatus::NeedsResponse => write!(f, "needs_response"),
            DisputeStatus::UnderReview => write!(f, "under_review"),
            DisputeStatus::Won => write!(f, "won"),
            DisputeStatus::Lost => write!(f, "lost"),
        }
    }
}

/// Dispute database model
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Dispute {
    pub id: Uuid,
    pub organization_id: OrganizationId,
    pub transaction_id: Uuid,
    pub amount_cents: i32,
    pub currency: String,
    pub stripe_dispute_id: Option<String>,
    pub square_dispute_id: Option<String>,
    pub reason: String,
    pub status: DisputeStatus,
    pub evidence_submitted: bool,
    pub evidence_due_by: Option<DateTime<Utc>>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub outcome: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Dispute {
    /// Get amount in dollars
    pub fn amount_dollars(&self) -> f64 {
        self.amount_cents as f64 / 100.0
    }

    /// Check if dispute needs immediate attention
    pub fn needs_attention(&self) -> bool {
        if self.status != DisputeStatus::NeedsResponse {
            return false;
        }

        if let Some(due_by) = self.evidence_due_by {
            let now = Utc::now();
            let days_remaining = (due_by - now).num_days();
            return days_remaining <= 3; // Alert if 3 or fewer days remaining
        }

        true
    }

    /// Get days until evidence is due
    pub fn days_until_due(&self) -> Option<i64> {
        self.evidence_due_by.map(|due_by| {
            let now = Utc::now();
            (due_by - now).num_days().max(0)
        })
    }

    /// Check if dispute is resolved
    pub fn is_resolved(&self) -> bool {
        matches!(self.status, DisputeStatus::Won | DisputeStatus::Lost)
    }

    /// Get display-friendly reason
    pub fn reason_display(&self) -> &str {
        match self.reason.as_str() {
            "duplicate" => "Duplicate charge",
            "fraudulent" => "Fraudulent transaction",
            "subscription_canceled" => "Subscription canceled",
            "product_unacceptable" => "Service unacceptable",
            "product_not_received" => "Service not received",
            "unrecognized" => "Unrecognized charge",
            "credit_not_processed" => "Refund not processed",
            "general" => "General dispute",
            other => other,
        }
    }
}

/// Input for creating a dispute (usually from webhook)
#[derive(Debug, Clone, Deserialize)]
pub struct CreateDispute {
    pub transaction_id: Uuid,
    pub amount_cents: i32,
    pub currency: String,
    pub stripe_dispute_id: Option<String>,
    pub square_dispute_id: Option<String>,
    pub reason: String,
    pub evidence_due_by: Option<DateTime<Utc>>,
}

/// Input for updating a dispute
#[derive(Debug, Clone, Deserialize, Default)]
pub struct UpdateDispute {
    pub status: Option<DisputeStatus>,
    pub evidence_submitted: Option<bool>,
    pub evidence_due_by: Option<DateTime<Utc>>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub outcome: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// Dispute evidence submission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisputeEvidence {
    pub customer_communication: Option<String>,
    pub service_documentation: Option<String>,
    pub receipt: Option<String>,
    pub cancellation_policy: Option<String>,
    pub customer_signature: Option<String>,
    pub service_date: Option<String>,
    pub additional_documentation: Option<Vec<String>>,
}

/// Payment webhook event for audit logging
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct PaymentWebhookEvent {
    pub id: Uuid,
    pub provider: String,
    pub event_id: String,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub processed: bool,
    pub processed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Input for creating a webhook event record
#[derive(Debug, Clone, Deserialize)]
pub struct CreateWebhookEvent {
    pub provider: String,
    pub event_id: String,
    pub event_type: String,
    pub payload: serde_json::Value,
}
