use chrono::{DateTime, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use shared::types::{OrganizationId, UserId};
use sqlx::FromRow;
use uuid::Uuid;

/// Subscription status enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "subscription_status", rename_all = "snake_case")]
pub enum SubscriptionStatus {
    Active,
    Paused,
    Canceled,
    PastDue,
    Trialing,
    Incomplete,
}

impl std::fmt::Display for SubscriptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SubscriptionStatus::Active => write!(f, "active"),
            SubscriptionStatus::Paused => write!(f, "paused"),
            SubscriptionStatus::Canceled => write!(f, "canceled"),
            SubscriptionStatus::PastDue => write!(f, "past_due"),
            SubscriptionStatus::Trialing => write!(f, "trialing"),
            SubscriptionStatus::Incomplete => write!(f, "incomplete"),
        }
    }
}

/// Plan tier enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "plan_tier", rename_all = "snake_case")]
pub enum PlanTier {
    #[default]
    Free,
    Professional,
    Business,
    Enterprise,
}

impl std::fmt::Display for PlanTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlanTier::Free => write!(f, "free"),
            PlanTier::Professional => write!(f, "professional"),
            PlanTier::Business => write!(f, "business"),
            PlanTier::Enterprise => write!(f, "enterprise"),
        }
    }
}

/// Platform fee tier configuration
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct PlatformFeeTier {
    pub id: Uuid,
    pub plan_tier: PlanTier,
    pub display_name: String,
    pub customer_fee_percent: rust_decimal::Decimal,
    pub provider_fee_percent: rust_decimal::Decimal,
    pub min_customer_fee_cents: i32,
    pub min_provider_fee_cents: i32,
    pub monthly_price_cents: i32,
    pub annual_price_cents: i32,
    pub features: Option<serde_json::Value>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl PlatformFeeTier {
    /// Get customer fee as percentage (e.g., 3.0 for 3%)
    pub fn customer_fee_display(&self) -> f64 {
        self.customer_fee_percent
            .to_string()
            .parse::<f64>()
            .unwrap_or(0.0)
            * 100.0
    }

    /// Get provider fee as percentage (e.g., 20.0 for 20%)
    pub fn provider_fee_display(&self) -> f64 {
        self.provider_fee_percent
            .to_string()
            .parse::<f64>()
            .unwrap_or(0.0)
            * 100.0
    }

    /// Get monthly price in dollars
    pub fn monthly_price_dollars(&self) -> f64 {
        self.monthly_price_cents as f64 / 100.0
    }
}

/// Tenant subscription for platform SaaS billing
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct TenantSubscription {
    pub id: Uuid,
    pub organization_id: OrganizationId,
    pub plan_tier: PlanTier,
    pub stripe_subscription_id: Option<String>,
    pub stripe_customer_id: Option<String>,
    pub status: SubscriptionStatus,
    pub current_period_start: Option<DateTime<Utc>>,
    pub current_period_end: Option<DateTime<Utc>>,
    pub trial_start: Option<DateTime<Utc>>,
    pub trial_end: Option<DateTime<Utc>>,
    pub cancel_at_period_end: bool,
    pub canceled_at: Option<DateTime<Utc>>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TenantSubscription {
    /// Check if subscription is active and can process payments
    pub fn is_active(&self) -> bool {
        matches!(
            self.status,
            SubscriptionStatus::Active | SubscriptionStatus::Trialing
        )
    }

    /// Check if subscription is in trial period
    pub fn is_trialing(&self) -> bool {
        if self.status != SubscriptionStatus::Trialing {
            return false;
        }
        if let Some(trial_end) = self.trial_end {
            trial_end > Utc::now()
        } else {
            false
        }
    }

    /// Get days remaining in current period
    pub fn days_remaining(&self) -> Option<i64> {
        self.current_period_end.map(|end| {
            let now = Utc::now();
            (end - now).num_days().max(0)
        })
    }
}

/// Input for creating a tenant subscription
#[derive(Debug, Clone, Deserialize)]
pub struct CreateTenantSubscription {
    pub plan_tier: PlanTier,
    pub stripe_subscription_id: Option<String>,
    pub stripe_customer_id: Option<String>,
    pub current_period_start: Option<DateTime<Utc>>,
    pub current_period_end: Option<DateTime<Utc>>,
    pub trial_end: Option<DateTime<Utc>>,
}

/// Input for updating a tenant subscription
#[derive(Debug, Clone, Deserialize, Default)]
pub struct UpdateTenantSubscription {
    pub plan_tier: Option<PlanTier>,
    pub stripe_subscription_id: Option<String>,
    pub status: Option<SubscriptionStatus>,
    pub current_period_start: Option<DateTime<Utc>>,
    pub current_period_end: Option<DateTime<Utc>>,
    pub cancel_at_period_end: Option<bool>,
    pub canceled_at: Option<DateTime<Utc>>,
    pub metadata: Option<serde_json::Value>,
}

/// Customer subscription for recurring service packages
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct CustomerSubscription {
    pub id: Uuid,
    pub organization_id: OrganizationId,
    pub user_id: UserId,
    pub service_id: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
    pub price_cents: i32,
    pub interval: String, // week, month, year
    pub interval_count: i32,
    pub stripe_subscription_id: Option<String>,
    pub stripe_price_id: Option<String>,
    pub square_subscription_id: Option<String>,
    pub status: SubscriptionStatus,
    pub current_period_start: Option<DateTime<Utc>>,
    pub current_period_end: Option<DateTime<Utc>>,
    pub cancel_at_period_end: bool,
    pub canceled_at: Option<DateTime<Utc>>,
    pub auto_create_bookings: bool,
    pub preferred_day_of_week: Option<i32>,
    pub preferred_time: Option<NaiveTime>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl CustomerSubscription {
    /// Check if subscription is active
    pub fn is_active(&self) -> bool {
        self.status == SubscriptionStatus::Active
    }

    /// Get price in dollars
    pub fn price_dollars(&self) -> f64 {
        self.price_cents as f64 / 100.0
    }

    /// Get display-friendly interval
    pub fn interval_display(&self) -> String {
        match (self.interval.as_str(), self.interval_count) {
            ("week", 1) => "Weekly".to_string(),
            ("week", n) => format!("Every {} weeks", n),
            ("month", 1) => "Monthly".to_string(),
            ("month", n) => format!("Every {} months", n),
            ("year", 1) => "Yearly".to_string(),
            ("year", n) => format!("Every {} years", n),
            _ => format!("Every {} {}", self.interval_count, self.interval),
        }
    }
}

/// Input for creating a customer subscription
#[derive(Debug, Clone, Deserialize)]
pub struct CreateCustomerSubscription {
    pub service_id: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
    pub price_cents: i32,
    pub interval: String,
    pub interval_count: i32,
    pub auto_create_bookings: bool,
    pub preferred_day_of_week: Option<i32>,
    pub preferred_time: Option<NaiveTime>,
}

/// Input for updating a customer subscription
#[derive(Debug, Clone, Deserialize, Default)]
pub struct UpdateCustomerSubscription {
    pub name: Option<String>,
    pub description: Option<String>,
    pub price_cents: Option<i32>,
    pub status: Option<SubscriptionStatus>,
    pub stripe_subscription_id: Option<String>,
    pub stripe_price_id: Option<String>,
    pub square_subscription_id: Option<String>,
    pub current_period_start: Option<DateTime<Utc>>,
    pub current_period_end: Option<DateTime<Utc>>,
    pub cancel_at_period_end: Option<bool>,
    pub canceled_at: Option<DateTime<Utc>>,
    pub auto_create_bookings: Option<bool>,
    pub preferred_day_of_week: Option<i32>,
    pub preferred_time: Option<NaiveTime>,
    pub metadata: Option<serde_json::Value>,
}
