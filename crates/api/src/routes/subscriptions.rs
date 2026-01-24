use axum::{
    extract::{Path, State},
    Json,
};
use db::{
    models::{
        CreateCustomerSubscription, CreateTenantSubscription, PlanTier, UpdateTenantSubscription,
    },
    SubscriptionRepository,
};
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};
use shared::AppError;
use uuid::Uuid;

use crate::{
    auth::{AuthUser, TenantContext},
    error::{ApiError, ApiResult},
    state::AppState,
};

/// Tenant subscription response
#[derive(Debug, Serialize)]
pub struct TenantSubscriptionResponse {
    pub id: String,
    pub plan_tier: String,
    pub status: String,
    pub monthly_price_cents: i32,
    pub annual_price_cents: i32,
    pub customer_fee_percent: f64,
    pub provider_fee_percent: f64,
    pub current_period_start: Option<String>,
    pub current_period_end: Option<String>,
    pub cancel_at_period_end: bool,
    pub created_at: String,
}

/// Fee tier info response
#[derive(Debug, Serialize)]
pub struct FeeTierResponse {
    pub plan_tier: String,
    pub display_name: String,
    pub customer_fee_percent: f64,
    pub provider_fee_percent: f64,
    pub min_customer_fee_cents: i32,
    pub min_provider_fee_cents: i32,
    pub monthly_price_cents: i32,
    pub annual_price_cents: i32,
    pub features: Vec<String>,
}

/// Get current tenant subscription
pub async fn get_tenant_subscription(
    State(_state): State<AppState>,
    _auth_user: AuthUser,
    tenant: TenantContext,
) -> ApiResult<Json<Option<TenantSubscriptionResponse>>> {
    let subscription =
        SubscriptionRepository::get_tenant_subscription(&tenant.pool, tenant.org_id).await?;

    let response = if let Some(s) = subscription {
        // Get the fee tier for the plan to include pricing info
        let fee_tier = SubscriptionRepository::get_fee_tier(&tenant.pool, s.plan_tier)
            .await?
            .ok_or_else(|| ApiError::from(AppError::Internal("Fee tier not found".to_string())))?;

        Some(TenantSubscriptionResponse {
            id: s.id.to_string(),
            plan_tier: s.plan_tier.to_string(),
            status: s.status.to_string(),
            monthly_price_cents: fee_tier.monthly_price_cents,
            annual_price_cents: fee_tier.annual_price_cents,
            customer_fee_percent: fee_tier.customer_fee_percent.to_f64().unwrap_or(0.0) * 100.0,
            provider_fee_percent: fee_tier.provider_fee_percent.to_f64().unwrap_or(0.0) * 100.0,
            current_period_start: s.current_period_start.map(|dt| dt.to_rfc3339()),
            current_period_end: s.current_period_end.map(|dt| dt.to_rfc3339()),
            cancel_at_period_end: s.cancel_at_period_end,
            created_at: s.created_at.to_rfc3339(),
        })
    } else {
        None
    };

    Ok(Json(response))
}

/// Get available fee tiers/plans
pub async fn list_fee_tiers(
    State(_state): State<AppState>,
    _auth_user: AuthUser,
    tenant: TenantContext,
) -> ApiResult<Json<Vec<FeeTierResponse>>> {
    let tiers = SubscriptionRepository::list_fee_tiers(&tenant.pool).await?;

    let response: Vec<FeeTierResponse> = tiers
        .into_iter()
        .map(|t| {
            let features = get_tier_features(&t.plan_tier);
            FeeTierResponse {
                plan_tier: t.plan_tier.to_string(),
                display_name: t.display_name,
                customer_fee_percent: t.customer_fee_percent.to_f64().unwrap_or(0.0) * 100.0,
                provider_fee_percent: t.provider_fee_percent.to_f64().unwrap_or(0.0) * 100.0,
                min_customer_fee_cents: t.min_customer_fee_cents,
                min_provider_fee_cents: t.min_provider_fee_cents,
                monthly_price_cents: t.monthly_price_cents,
                annual_price_cents: t.annual_price_cents,
                features,
            }
        })
        .collect();

    Ok(Json(response))
}

fn get_tier_features(tier: &PlanTier) -> Vec<String> {
    match tier {
        PlanTier::Free => vec![
            "Basic booking management".to_string(),
            "Customer payments".to_string(),
            "3% customer fee + 20% provider fee".to_string(),
        ],
        PlanTier::Professional => vec![
            "Everything in Free".to_string(),
            "Custom branding".to_string(),
            "Priority support".to_string(),
            "2% customer fee + 15% provider fee".to_string(),
            "Analytics dashboard".to_string(),
        ],
        PlanTier::Business => vec![
            "Everything in Professional".to_string(),
            "Multiple locations".to_string(),
            "Team management".to_string(),
            "1% customer fee + 10% provider fee".to_string(),
            "Advanced analytics".to_string(),
            "API access".to_string(),
        ],
        PlanTier::Enterprise => vec![
            "Everything in Business".to_string(),
            "Custom integrations".to_string(),
            "Dedicated support".to_string(),
            "Custom fee structure".to_string(),
            "SLA guarantee".to_string(),
            "White-label option".to_string(),
        ],
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateSubscriptionRequest {
    pub plan_tier: String,
    /// "monthly" or "annual"
    pub billing_period: String,
    /// Payment method ID to use for subscription
    pub payment_method_id: Option<String>,
}

/// Create/upgrade tenant subscription
pub async fn create_tenant_subscription(
    State(_state): State<AppState>,
    _auth_user: AuthUser,
    tenant: TenantContext,
    Json(req): Json<CreateSubscriptionRequest>,
) -> ApiResult<Json<TenantSubscriptionResponse>> {
    let plan_tier = match req.plan_tier.to_lowercase().as_str() {
        "free" => PlanTier::Free,
        "professional" => PlanTier::Professional,
        "business" => PlanTier::Business,
        "enterprise" => PlanTier::Enterprise,
        _ => {
            return Err(ApiError::from(AppError::Validation(
                "Invalid plan tier".to_string(),
            )))
        }
    };

    // Get the fee tier info
    let fee_tier = SubscriptionRepository::get_fee_tier(&tenant.pool, plan_tier)
        .await?
        .ok_or_else(|| ApiError::from(AppError::Internal("Fee tier not found".to_string())))?;

    // Calculate period dates
    let now = chrono::Utc::now();
    let period_end = if req.billing_period == "annual" {
        now + chrono::Duration::days(365)
    } else {
        now + chrono::Duration::days(30)
    };

    // Create the subscription
    let input = CreateTenantSubscription {
        plan_tier,
        stripe_subscription_id: None,
        stripe_customer_id: None,
        current_period_start: Some(now),
        current_period_end: Some(period_end),
        trial_end: None,
    };

    let subscription =
        SubscriptionRepository::create_tenant_subscription(&tenant.pool, tenant.org_id, input)
            .await?;

    // TODO: Create payment/charge for the subscription via Stripe/Square

    Ok(Json(TenantSubscriptionResponse {
        id: subscription.id.to_string(),
        plan_tier: subscription.plan_tier.to_string(),
        status: subscription.status.to_string(),
        monthly_price_cents: fee_tier.monthly_price_cents,
        annual_price_cents: fee_tier.annual_price_cents,
        customer_fee_percent: fee_tier.customer_fee_percent.to_f64().unwrap_or(0.0) * 100.0,
        provider_fee_percent: fee_tier.provider_fee_percent.to_f64().unwrap_or(0.0) * 100.0,
        current_period_start: subscription.current_period_start.map(|dt| dt.to_rfc3339()),
        current_period_end: subscription.current_period_end.map(|dt| dt.to_rfc3339()),
        cancel_at_period_end: subscription.cancel_at_period_end,
        created_at: subscription.created_at.to_rfc3339(),
    }))
}

/// Cancel tenant subscription (at period end)
pub async fn cancel_tenant_subscription(
    State(_state): State<AppState>,
    _auth_user: AuthUser,
    tenant: TenantContext,
) -> ApiResult<Json<TenantSubscriptionResponse>> {
    // Update subscription to cancel at period end
    let input = UpdateTenantSubscription {
        cancel_at_period_end: Some(true),
        canceled_at: Some(chrono::Utc::now()),
        ..Default::default()
    };

    let subscription =
        SubscriptionRepository::update_tenant_subscription(&tenant.pool, tenant.org_id, input)
            .await?
            .ok_or_else(|| {
                ApiError::from(AppError::NotFound(
                    "No active subscription found".to_string(),
                ))
            })?;

    // Get fee tier for response
    let fee_tier = SubscriptionRepository::get_fee_tier(&tenant.pool, subscription.plan_tier)
        .await?
        .ok_or_else(|| ApiError::from(AppError::Internal("Fee tier not found".to_string())))?;

    Ok(Json(TenantSubscriptionResponse {
        id: subscription.id.to_string(),
        plan_tier: subscription.plan_tier.to_string(),
        status: subscription.status.to_string(),
        monthly_price_cents: fee_tier.monthly_price_cents,
        annual_price_cents: fee_tier.annual_price_cents,
        customer_fee_percent: fee_tier.customer_fee_percent.to_f64().unwrap_or(0.0) * 100.0,
        provider_fee_percent: fee_tier.provider_fee_percent.to_f64().unwrap_or(0.0) * 100.0,
        current_period_start: subscription.current_period_start.map(|dt| dt.to_rfc3339()),
        current_period_end: subscription.current_period_end.map(|dt| dt.to_rfc3339()),
        cancel_at_period_end: subscription.cancel_at_period_end,
        created_at: subscription.created_at.to_rfc3339(),
    }))
}

// Customer subscriptions (recurring service packages)

/// Customer subscription response
#[derive(Debug, Serialize)]
pub struct CustomerSubscriptionResponse {
    pub id: String,
    pub service_id: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub price_cents: i32,
    pub interval: String,
    pub interval_count: i32,
    pub current_period_start: Option<String>,
    pub current_period_end: Option<String>,
    pub cancel_at_period_end: bool,
    pub auto_create_bookings: bool,
    pub created_at: String,
}

/// Get customer's subscriptions
pub async fn list_customer_subscriptions(
    State(_state): State<AppState>,
    auth_user: AuthUser,
    tenant: TenantContext,
) -> ApiResult<Json<Vec<CustomerSubscriptionResponse>>> {
    let subscriptions = SubscriptionRepository::list_customer_subscriptions(
        &tenant.pool,
        tenant.org_id,
        auth_user.user_id,
    )
    .await?;

    let response: Vec<CustomerSubscriptionResponse> = subscriptions
        .into_iter()
        .map(|s| CustomerSubscriptionResponse {
            id: s.id.to_string(),
            service_id: s.service_id.map(|id| id.to_string()),
            name: s.name,
            description: s.description,
            status: s.status.to_string(),
            price_cents: s.price_cents,
            interval: s.interval,
            interval_count: s.interval_count,
            current_period_start: s.current_period_start.map(|dt| dt.to_rfc3339()),
            current_period_end: s.current_period_end.map(|dt| dt.to_rfc3339()),
            cancel_at_period_end: s.cancel_at_period_end,
            auto_create_bookings: s.auto_create_bookings,
            created_at: s.created_at.to_rfc3339(),
        })
        .collect();

    Ok(Json(response))
}

#[derive(Debug, Deserialize)]
pub struct CreateCustomerSubscriptionRequest {
    /// Service this subscription is for (optional)
    pub service_id: Option<String>,
    /// Display name for the subscription
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Price in cents
    pub price_cents: i32,
    /// Interval: "week", "month", or "year"
    pub interval: String,
    /// How many intervals between charges
    pub interval_count: Option<i32>,
    /// Auto-create bookings from this subscription
    pub auto_create_bookings: Option<bool>,
    /// Preferred day of week (0-6, Sunday-Saturday)
    pub preferred_day_of_week: Option<i32>,
    /// Payment method to use
    pub payment_method_id: Option<String>,
}

/// Create a customer subscription (service package)
pub async fn create_customer_subscription(
    State(_state): State<AppState>,
    auth_user: AuthUser,
    tenant: TenantContext,
    Json(req): Json<CreateCustomerSubscriptionRequest>,
) -> ApiResult<Json<CustomerSubscriptionResponse>> {
    let service_id =
        if let Some(sid) = &req.service_id {
            Some(sid.parse::<Uuid>().map_err(|_| {
                ApiError::from(AppError::Validation("Invalid service ID".to_string()))
            })?)
        } else {
            None
        };

    // Validate interval
    if !["week", "month", "year"].contains(&req.interval.as_str()) {
        return Err(ApiError::from(AppError::Validation(
            "Invalid interval. Must be 'week', 'month', or 'year'".to_string(),
        )));
    }

    let input = CreateCustomerSubscription {
        service_id,
        name: req.name,
        description: req.description,
        price_cents: req.price_cents,
        interval: req.interval,
        interval_count: req.interval_count.unwrap_or(1),
        auto_create_bookings: req.auto_create_bookings.unwrap_or(false),
        preferred_day_of_week: req.preferred_day_of_week,
        preferred_time: None,
    };

    let subscription = SubscriptionRepository::create_customer_subscription(
        &tenant.pool,
        tenant.org_id,
        auth_user.user_id,
        input,
    )
    .await?;

    // TODO: Process initial payment

    Ok(Json(CustomerSubscriptionResponse {
        id: subscription.id.to_string(),
        service_id: subscription.service_id.map(|id| id.to_string()),
        name: subscription.name,
        description: subscription.description,
        status: subscription.status.to_string(),
        price_cents: subscription.price_cents,
        interval: subscription.interval,
        interval_count: subscription.interval_count,
        current_period_start: subscription.current_period_start.map(|dt| dt.to_rfc3339()),
        current_period_end: subscription.current_period_end.map(|dt| dt.to_rfc3339()),
        cancel_at_period_end: subscription.cancel_at_period_end,
        auto_create_bookings: subscription.auto_create_bookings,
        created_at: subscription.created_at.to_rfc3339(),
    }))
}

/// Cancel a customer subscription
pub async fn cancel_customer_subscription(
    State(_state): State<AppState>,
    _auth_user: AuthUser,
    tenant: TenantContext,
    Path(id): Path<String>,
) -> ApiResult<Json<CustomerSubscriptionResponse>> {
    let subscription_id: Uuid = id
        .parse()
        .map_err(|_| ApiError::from(AppError::Validation("Invalid subscription ID".to_string())))?;

    // Cancel at period end (not immediately)
    let subscription = SubscriptionRepository::cancel_customer_subscription(
        &tenant.pool,
        tenant.org_id,
        subscription_id,
        false, // cancel at period end
    )
    .await?
    .ok_or_else(|| ApiError::from(AppError::NotFound("Subscription not found".to_string())))?;

    Ok(Json(CustomerSubscriptionResponse {
        id: subscription.id.to_string(),
        service_id: subscription.service_id.map(|id| id.to_string()),
        name: subscription.name,
        description: subscription.description,
        status: subscription.status.to_string(),
        price_cents: subscription.price_cents,
        interval: subscription.interval,
        interval_count: subscription.interval_count,
        current_period_start: subscription.current_period_start.map(|dt| dt.to_rfc3339()),
        current_period_end: subscription.current_period_end.map(|dt| dt.to_rfc3339()),
        cancel_at_period_end: subscription.cancel_at_period_end,
        auto_create_bookings: subscription.auto_create_bookings,
        created_at: subscription.created_at.to_rfc3339(),
    }))
}

/// Get a specific customer subscription
pub async fn get_customer_subscription(
    State(_state): State<AppState>,
    _auth_user: AuthUser,
    tenant: TenantContext,
    Path(id): Path<String>,
) -> ApiResult<Json<CustomerSubscriptionResponse>> {
    let subscription_id: Uuid = id
        .parse()
        .map_err(|_| ApiError::from(AppError::Validation("Invalid subscription ID".to_string())))?;

    let subscription = SubscriptionRepository::get_customer_subscription(
        &tenant.pool,
        tenant.org_id,
        subscription_id,
    )
    .await?
    .ok_or_else(|| ApiError::from(AppError::NotFound("Subscription not found".to_string())))?;

    Ok(Json(CustomerSubscriptionResponse {
        id: subscription.id.to_string(),
        service_id: subscription.service_id.map(|id| id.to_string()),
        name: subscription.name,
        description: subscription.description,
        status: subscription.status.to_string(),
        price_cents: subscription.price_cents,
        interval: subscription.interval,
        interval_count: subscription.interval_count,
        current_period_start: subscription.current_period_start.map(|dt| dt.to_rfc3339()),
        current_period_end: subscription.current_period_end.map(|dt| dt.to_rfc3339()),
        cancel_at_period_end: subscription.cancel_at_period_end,
        auto_create_bookings: subscription.auto_create_bookings,
        created_at: subscription.created_at.to_rfc3339(),
    }))
}
