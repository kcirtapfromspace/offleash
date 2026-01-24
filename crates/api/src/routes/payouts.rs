use axum::{
    extract::{Path, Query, State},
    Json,
};
use db::{
    models::{
        CreatePayout, PaymentProviderType, Payout, PayoutStatus, UpdatePayout, UpdatePayoutSettings,
    },
    PaymentProviderRepository, PayoutRepository,
};
use serde::{Deserialize, Serialize};
use shared::types::OrganizationId;
use shared::AppError;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    auth::{AuthUser, TenantContext},
    error::{ApiError, ApiResult},
    state::AppState,
};

/// Payout settings response
#[derive(Debug, Serialize)]
pub struct PayoutSettingsResponse {
    pub id: String,
    pub payout_schedule: String,
    pub payout_day_of_week: Option<i32>,
    pub payout_day_of_month: Option<i32>,
    pub minimum_payout_cents: i32,
    pub payout_method: String,
    pub bank_name: Option<String>,
    pub bank_account_last_four: Option<String>,
    pub is_verified: bool,
}

/// Payout response
#[derive(Debug, Serialize)]
pub struct PayoutResponse {
    pub id: String,
    pub amount_cents: i32,
    pub fee_cents: i32,
    pub net_amount_cents: i32,
    pub status: String,
    pub initiated_at: Option<String>,
    pub completed_at: Option<String>,
    pub period_start: String,
    pub period_end: String,
    pub transaction_count: i32,
    pub created_at: String,
}

/// Payout summary response
#[derive(Debug, Serialize)]
pub struct PayoutSummaryResponse {
    pub total_payouts: i64,
    pub total_amount_cents: i64,
    pub pending_amount_cents: i64,
    pub last_payout_date: Option<String>,
    pub next_payout_date: Option<String>,
}

/// Get payout settings for the organization
pub async fn get_payout_settings(
    State(_state): State<AppState>,
    _auth_user: AuthUser,
    tenant: TenantContext,
) -> ApiResult<Json<Option<PayoutSettingsResponse>>> {
    let settings = PayoutRepository::get_settings(&tenant.pool, tenant.org_id).await?;

    let response = settings.map(|s| PayoutSettingsResponse {
        id: s.id.to_string(),
        payout_schedule: s.payout_schedule,
        payout_day_of_week: s.payout_day_of_week,
        payout_day_of_month: s.payout_day_of_month,
        minimum_payout_cents: s.minimum_payout_cents,
        payout_method: s.payout_method,
        bank_name: s.bank_name,
        bank_account_last_four: s.bank_account_last_four,
        is_verified: s.is_verified,
    });

    Ok(Json(response))
}

#[derive(Debug, Deserialize)]
pub struct UpdatePayoutSettingsRequest {
    /// "daily", "weekly", "monthly"
    pub payout_schedule: Option<String>,
    /// Day of week (0-6, Sunday-Saturday) for weekly payouts
    pub payout_day_of_week: Option<i32>,
    /// Day of month (1-31) for monthly payouts
    pub payout_day_of_month: Option<i32>,
    /// Minimum amount required for automatic payout
    pub minimum_payout_cents: Option<i32>,
    /// "bank_account", "debit_card"
    pub payout_method: Option<String>,
}

/// Update payout settings
pub async fn update_payout_settings(
    State(_state): State<AppState>,
    _auth_user: AuthUser,
    tenant: TenantContext,
    Json(req): Json<UpdatePayoutSettingsRequest>,
) -> ApiResult<Json<PayoutSettingsResponse>> {
    // Validate payout schedule if provided
    if let Some(schedule) = &req.payout_schedule {
        if !["daily", "weekly", "monthly"].contains(&schedule.as_str()) {
            return Err(ApiError::from(AppError::Validation(
                "Invalid payout schedule. Must be 'daily', 'weekly', or 'monthly'".to_string(),
            )));
        }
    }

    // Validate payout method if provided
    if let Some(method) = &req.payout_method {
        if !["bank_account", "debit_card", "bank", "debit"].contains(&method.as_str()) {
            return Err(ApiError::from(AppError::Validation(
                "Invalid payout method. Must be 'bank_account' or 'debit_card'".to_string(),
            )));
        }
    }

    let input = UpdatePayoutSettings {
        payout_method: req.payout_method,
        payout_schedule: req.payout_schedule,
        payout_day_of_week: req.payout_day_of_week,
        payout_day_of_month: req.payout_day_of_month,
        minimum_payout_cents: req.minimum_payout_cents,
        ..Default::default()
    };

    let settings = PayoutRepository::update_settings(&tenant.pool, tenant.org_id, input)
        .await?
        .ok_or_else(|| {
            ApiError::from(AppError::NotFound("Payout settings not found".to_string()))
        })?;

    Ok(Json(PayoutSettingsResponse {
        id: settings.id.to_string(),
        payout_schedule: settings.payout_schedule,
        payout_day_of_week: settings.payout_day_of_week,
        payout_day_of_month: settings.payout_day_of_month,
        minimum_payout_cents: settings.minimum_payout_cents,
        payout_method: settings.payout_method,
        bank_name: settings.bank_name,
        bank_account_last_four: settings.bank_account_last_four,
        is_verified: settings.is_verified,
    }))
}

/// Get payout summary for the organization
pub async fn get_payout_summary(
    State(_state): State<AppState>,
    _auth_user: AuthUser,
    tenant: TenantContext,
) -> ApiResult<Json<PayoutSummaryResponse>> {
    let summary = PayoutRepository::get_summary(&tenant.pool, tenant.org_id).await?;

    Ok(Json(PayoutSummaryResponse {
        total_payouts: summary.total_payouts,
        total_amount_cents: summary.total_amount_cents,
        pending_amount_cents: summary.pending_amount_cents,
        last_payout_date: summary.last_payout_date.map(|dt| dt.to_rfc3339()),
        next_payout_date: summary.next_payout_date.map(|dt| dt.to_rfc3339()),
    }))
}

#[derive(Debug, Deserialize)]
pub struct ListPayoutsQuery {
    pub status: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// List payouts for the organization
pub async fn list_payouts(
    State(_state): State<AppState>,
    _auth_user: AuthUser,
    tenant: TenantContext,
    Query(query): Query<ListPayoutsQuery>,
) -> ApiResult<Json<Vec<PayoutResponse>>> {
    let status = if let Some(s) = &query.status {
        Some(match s.to_lowercase().as_str() {
            "pending" => PayoutStatus::Pending,
            "in_transit" => PayoutStatus::InTransit,
            "paid" => PayoutStatus::Paid,
            "failed" => PayoutStatus::Failed,
            "canceled" | "cancelled" => PayoutStatus::Canceled,
            _ => {
                return Err(ApiError::from(AppError::Validation(
                    "Invalid status filter. Must be 'pending', 'in_transit', 'paid', 'failed', or 'canceled'".to_string(),
                )))
            }
        })
    } else {
        None
    };

    let payouts = PayoutRepository::list_for_org(
        &tenant.pool,
        tenant.org_id,
        status,
        query.limit.unwrap_or(50),
        query.offset.unwrap_or(0),
    )
    .await?;

    let response: Vec<PayoutResponse> = payouts
        .into_iter()
        .map(|p| PayoutResponse {
            id: p.id.to_string(),
            amount_cents: p.amount_cents,
            fee_cents: p.fee_cents,
            net_amount_cents: p.net_amount_cents,
            status: p.status.to_string(),
            initiated_at: p.initiated_at.map(|dt| dt.to_rfc3339()),
            completed_at: p.completed_at.map(|dt| dt.to_rfc3339()),
            period_start: p.period_start.to_rfc3339(),
            period_end: p.period_end.to_rfc3339(),
            transaction_count: p.transaction_count,
            created_at: p.created_at.to_rfc3339(),
        })
        .collect();

    Ok(Json(response))
}

/// Get a specific payout
pub async fn get_payout(
    State(_state): State<AppState>,
    _auth_user: AuthUser,
    tenant: TenantContext,
    Path(id): Path<String>,
) -> ApiResult<Json<PayoutResponse>> {
    let payout_id: Uuid = id
        .parse()
        .map_err(|_| ApiError::from(AppError::Validation("Invalid payout ID".to_string())))?;

    let payout = PayoutRepository::get_by_id(&tenant.pool, tenant.org_id, payout_id)
        .await?
        .ok_or_else(|| ApiError::from(AppError::NotFound("Payout not found".to_string())))?;

    Ok(Json(PayoutResponse {
        id: payout.id.to_string(),
        amount_cents: payout.amount_cents,
        fee_cents: payout.fee_cents,
        net_amount_cents: payout.net_amount_cents,
        status: payout.status.to_string(),
        initiated_at: payout.initiated_at.map(|dt| dt.to_rfc3339()),
        completed_at: payout.completed_at.map(|dt| dt.to_rfc3339()),
        period_start: payout.period_start.to_rfc3339(),
        period_end: payout.period_end.to_rfc3339(),
        transaction_count: payout.transaction_count,
        created_at: payout.created_at.to_rfc3339(),
    }))
}

/// Request an instant payout
pub async fn request_instant_payout(
    State(_state): State<AppState>,
    _auth_user: AuthUser,
    tenant: TenantContext,
) -> ApiResult<Json<PayoutResponse>> {
    // Get the org's payout summary to check available balance
    let summary = PayoutRepository::get_summary(&tenant.pool, tenant.org_id).await?;

    // For now, use pending_amount_cents as available amount
    // In production, this would be calculated from settled transactions
    let available_amount = summary.pending_amount_cents as i32;

    if available_amount <= 0 {
        return Err(ApiError::from(AppError::Validation(
            "No available balance for payout".to_string(),
        )));
    }

    // Get payout settings to check minimum
    let settings = PayoutRepository::get_settings(&tenant.pool, tenant.org_id).await?;

    if let Some(settings) = &settings {
        if available_amount < settings.minimum_payout_cents {
            return Err(ApiError::from(AppError::Validation(format!(
                "Available balance is below minimum payout amount of ${:.2}",
                settings.minimum_payout_cents as f64 / 100.0
            ))));
        }
    }

    // Calculate payout fee (instant payouts typically have a fee)
    let instant_fee_percent = 1.0; // 1% for instant payouts
    let fee_cents = ((available_amount as f64) * (instant_fee_percent / 100.0)).round() as i32;
    let net_amount = available_amount - fee_cents;

    // Create the payout
    let now = chrono::Utc::now();
    let input = CreatePayout {
        amount_cents: available_amount,
        fee_cents,
        net_amount_cents: net_amount,
        currency: "USD".to_string(),
        period_start: now - chrono::Duration::days(30), // Period start (last 30 days)
        period_end: now,                                // Period end
        transaction_count: 0, // Would be calculated from actual transactions
        transaction_ids: vec![],
    };

    let payout = PayoutRepository::create(&tenant.pool, tenant.org_id, input).await?;

    // Initiate payout through payment provider
    if let Ok((stripe_id, square_id)) =
        initiate_provider_payout(&tenant.pool, tenant.org_id, &payout).await
    {
        // Update payout with external ID
        let update = UpdatePayout {
            stripe_payout_id: stripe_id,
            square_payout_id: square_id,
            initiated_at: Some(chrono::Utc::now()),
            ..Default::default()
        };
        let _ = PayoutRepository::update(&tenant.pool, payout.id, update).await;
    }

    Ok(Json(PayoutResponse {
        id: payout.id.to_string(),
        amount_cents: payout.amount_cents,
        fee_cents: payout.fee_cents,
        net_amount_cents: payout.net_amount_cents,
        status: payout.status.to_string(),
        initiated_at: payout.initiated_at.map(|dt| dt.to_rfc3339()),
        completed_at: payout.completed_at.map(|dt| dt.to_rfc3339()),
        period_start: payout.period_start.to_rfc3339(),
        period_end: payout.period_end.to_rfc3339(),
        transaction_count: payout.transaction_count,
        created_at: payout.created_at.to_rfc3339(),
    }))
}

// Helper function to initiate payouts through payment providers

async fn initiate_provider_payout(
    pool: &PgPool,
    org_id: OrganizationId,
    payout: &Payout,
) -> Result<(Option<String>, Option<String>), ApiError> {
    // Get the primary payment provider
    let provider = PaymentProviderRepository::get_primary(pool, org_id)
        .await?
        .ok_or_else(|| {
            ApiError::from(AppError::Internal(
                "No payment provider configured".to_string(),
            ))
        })?;

    match provider.provider_type {
        PaymentProviderType::Stripe | PaymentProviderType::Platform => {
            let stripe_id = initiate_stripe_payout(&provider, payout).await?;
            Ok((Some(stripe_id), None))
        }
        PaymentProviderType::Square => {
            // Square doesn't support payouts the same way - funds go directly to merchant
            // Return a placeholder ID
            let square_id = format!("square_payout_{}", payout.id);
            Ok((None, Some(square_id)))
        }
    }
}

async fn initiate_stripe_payout(
    provider: &db::models::PaymentProvider,
    payout: &Payout,
) -> Result<String, ApiError> {
    let stripe_secret = std::env::var("STRIPE_SECRET_KEY")
        .map_err(|_| ApiError::from(AppError::Internal("Stripe not configured".to_string())))?;

    let connected_account = provider.stripe_account_id.as_ref().ok_or_else(|| {
        ApiError::from(AppError::Internal(
            "Stripe account not connected".to_string(),
        ))
    })?;

    let client = reqwest::Client::new();

    let params = vec![
        ("amount", payout.net_amount_cents.to_string()),
        ("currency", "usd".to_string()),
        ("method", "instant".to_string()), // or "standard" for regular speed
    ];

    let response = client
        .post("https://api.stripe.com/v1/payouts")
        .header("Authorization", format!("Bearer {}", stripe_secret))
        .header("Stripe-Account", connected_account)
        .form(&params)
        .send()
        .await
        .map_err(|e| ApiError::from(AppError::Internal(e.to_string())))?;

    if !response.status().is_success() {
        let error_body = response.text().await.unwrap_or_default();
        return Err(ApiError::from(AppError::Internal(format!(
            "Stripe payout error: {}",
            error_body
        ))));
    }

    #[derive(serde::Deserialize)]
    struct PayoutResponse {
        id: String,
    }

    let stripe_payout: PayoutResponse = response
        .json()
        .await
        .map_err(|e| ApiError::from(AppError::Internal(e.to_string())))?;

    Ok(stripe_payout.id)
}
