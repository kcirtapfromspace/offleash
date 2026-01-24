use axum::{
    extract::{Path, State},
    Json,
};
use db::{
    models::{CreateTransaction, PaymentProviderType, TransactionFeeBreakdown, TransactionStatus},
    PaymentProviderRepository, SubscriptionRepository, TransactionRepository,
};
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};
use shared::types::{BookingId, UserId};
use shared::AppError;
use uuid::Uuid;

use crate::{
    auth::{AuthUser, TenantContext},
    error::{ApiError, ApiResult},
    state::AppState,
};

/// Checkout session response
#[derive(Debug, Serialize)]
pub struct CheckoutResponse {
    pub transaction_id: String,
    pub status: String,
    pub subtotal_cents: i32,
    pub customer_fee_cents: i32,
    pub tax_cents: i32,
    pub total_cents: i32,
    pub provider_type: String,
    /// Client secret for Stripe PaymentIntent (if using Stripe)
    pub client_secret: Option<String>,
    /// Payment ID for Square (if using Square)
    pub payment_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCheckoutRequest {
    /// The booking ID this payment is for
    pub booking_id: String,
    /// Payment method ID (from customer_payment_methods)
    pub payment_method_id: Option<String>,
    /// Subtotal in cents (service price)
    pub subtotal_cents: i32,
    /// Optional tip in cents
    pub tip_cents: Option<i32>,
    /// Walker/provider user ID to receive payment
    pub provider_user_id: String,
    /// Customer's address state for tax calculation
    pub customer_state: Option<String>,
    /// Customer's address zip for tax calculation
    pub customer_zip: Option<String>,
}

/// Create a checkout session / payment intent
pub async fn create_checkout(
    State(_state): State<AppState>,
    auth_user: AuthUser,
    tenant: TenantContext,
    Json(req): Json<CreateCheckoutRequest>,
) -> ApiResult<Json<CheckoutResponse>> {
    let booking_id: Uuid = req
        .booking_id
        .parse()
        .map_err(|_| ApiError::from(AppError::Validation("Invalid booking ID".to_string())))?;

    let provider_user_id: Uuid = req.provider_user_id.parse().map_err(|_| {
        ApiError::from(AppError::Validation("Invalid provider user ID".to_string()))
    })?;

    // Get the primary payment provider for this tenant
    let provider = PaymentProviderRepository::get_primary(&tenant.pool, tenant.org_id)
        .await?
        .ok_or_else(|| {
            ApiError::from(AppError::Validation(
                "No payment provider configured".to_string(),
            ))
        })?;

    // Get the platform fee tier for this tenant
    let fee_tier = SubscriptionRepository::get_org_fee_tier(&tenant.pool, tenant.org_id).await?;

    // Calculate tax (simplified - in production use TaxJar/Avalara)
    let tax_rate_percent =
        if let (Some(state), Some(_zip)) = (&req.customer_state, &req.customer_zip) {
            // Use simple state-based tax rate as fallback
            match state.to_uppercase().as_str() {
                "CA" => 7.25,
                "TX" => 6.25,
                "FL" => 6.00,
                "NY" => 8.00,
                "WA" => 6.50,
                "OR" | "MT" | "NH" | "DE" => 0.0, // No sales tax
                _ => 6.0,
            }
        } else {
            0.0 // No tax if address not provided
        };

    // Calculate fee breakdown
    let fee_breakdown = TransactionFeeBreakdown::calculate(
        req.subtotal_cents,
        fee_tier.customer_fee_percent.to_f64().unwrap_or(0.0),
        fee_tier.provider_fee_percent.to_f64().unwrap_or(0.0),
        tax_rate_percent,
        2.9, // Stripe/Square processing fee percentage
    );

    // Get payment method details if provided
    let payment_method_id = if let Some(pm_id) = &req.payment_method_id {
        Some(pm_id.parse::<Uuid>().map_err(|_| {
            ApiError::from(AppError::Validation(
                "Invalid payment method ID".to_string(),
            ))
        })?)
    } else {
        None
    };

    // Create the transaction record
    let transaction_input = CreateTransaction {
        booking_id: Some(BookingId::from_uuid(booking_id)),
        customer_user_id: auth_user.user_id,
        provider_user_id: UserId::from_uuid(provider_user_id),
        payment_method_id,
        provider_id: provider.id,
        subtotal_cents: req.subtotal_cents,
        tip_cents: req.tip_cents.unwrap_or(0),
        customer_fee_cents: fee_breakdown.customer_fee_cents,
        provider_fee_cents: fee_breakdown.provider_fee_cents,
        platform_fee_cents: fee_breakdown.platform_fee_cents,
        tax_cents: fee_breakdown.tax_cents,
        processing_fee_cents: fee_breakdown.processing_fee_cents,
        total_cents: fee_breakdown.total_cents,
        provider_payout_cents: fee_breakdown.provider_payout_cents,
        currency: "USD".to_string(),
        tax_rate_percent: None,
        tax_jurisdiction: None,
        description: None,
        metadata: None,
    };

    let transaction =
        TransactionRepository::create(&tenant.pool, tenant.org_id, transaction_input).await?;

    // Create payment with the provider
    let (client_secret, payment_id) = match provider.provider_type {
        PaymentProviderType::Stripe => {
            let secret =
                create_stripe_payment_intent(&provider, &transaction, &fee_breakdown).await?;
            (Some(secret), None)
        }
        PaymentProviderType::Square => {
            let id = create_square_payment(&provider, &transaction, &fee_breakdown).await?;
            (None, Some(id))
        }
        PaymentProviderType::Platform => {
            // Platform default - use Stripe
            let secret =
                create_stripe_payment_intent(&provider, &transaction, &fee_breakdown).await?;
            (Some(secret), None)
        }
    };

    // Update transaction with external payment ID
    let external_id = client_secret.clone().or(payment_id.clone());
    if let Some(ext_id) = &external_id {
        TransactionRepository::update_external_id(&tenant.pool, transaction.id, ext_id).await?;
    }

    Ok(Json(CheckoutResponse {
        transaction_id: transaction.id.to_string(),
        status: format!("{:?}", transaction.status).to_lowercase(),
        subtotal_cents: transaction.subtotal_cents,
        customer_fee_cents: transaction.customer_fee_cents,
        tax_cents: transaction.tax_cents,
        total_cents: transaction.total_cents,
        provider_type: format!("{:?}", provider.provider_type).to_lowercase(),
        client_secret,
        payment_id,
    }))
}

/// Get checkout/transaction details
pub async fn get_checkout(
    State(_state): State<AppState>,
    auth_user: AuthUser,
    tenant: TenantContext,
    Path(id): Path<String>,
) -> ApiResult<Json<CheckoutResponse>> {
    let transaction_id: Uuid = id
        .parse()
        .map_err(|_| ApiError::from(AppError::Validation("Invalid transaction ID".to_string())))?;

    let transaction = TransactionRepository::get_by_id(&tenant.pool, tenant.org_id, transaction_id)
        .await?
        .ok_or_else(|| ApiError::from(AppError::NotFound("Transaction not found".to_string())))?;

    // Verify user is either customer or provider
    if transaction.customer_user_id != auth_user.user_id
        && transaction.provider_user_id != auth_user.user_id
    {
        return Err(ApiError::from(AppError::NotFound(
            "Transaction not found".to_string(),
        )));
    }

    let provider =
        PaymentProviderRepository::get_by_id(&tenant.pool, tenant.org_id, transaction.provider_id)
            .await?;

    Ok(Json(CheckoutResponse {
        transaction_id: transaction.id.to_string(),
        status: format!("{:?}", transaction.status).to_lowercase(),
        subtotal_cents: transaction.subtotal_cents,
        customer_fee_cents: transaction.customer_fee_cents,
        tax_cents: transaction.tax_cents,
        total_cents: transaction.total_cents,
        provider_type: provider
            .map(|p| format!("{:?}", p.provider_type).to_lowercase())
            .unwrap_or_else(|| "unknown".to_string()),
        client_secret: None, // Don't expose client secret on GET
        payment_id: transaction.external_payment_id,
    }))
}

#[derive(Debug, Serialize)]
pub struct TransactionListItem {
    pub id: String,
    pub booking_id: Option<String>,
    pub status: String,
    pub subtotal_cents: i32,
    pub total_cents: i32,
    pub created_at: String,
    pub is_customer: bool,
}

/// List transactions for the current user
pub async fn list_transactions(
    State(_state): State<AppState>,
    auth_user: AuthUser,
    tenant: TenantContext,
) -> ApiResult<Json<Vec<TransactionListItem>>> {
    let transactions =
        TransactionRepository::list_for_user(&tenant.pool, tenant.org_id, auth_user.user_id)
            .await?;

    let response: Vec<TransactionListItem> = transactions
        .into_iter()
        .map(|t| TransactionListItem {
            id: t.id.to_string(),
            booking_id: t.booking_id.map(|id| id.to_string()),
            status: format!("{:?}", t.status).to_lowercase(),
            subtotal_cents: t.subtotal_cents,
            total_cents: t.total_cents,
            created_at: t.created_at.to_rfc3339(),
            is_customer: t.customer_user_id == auth_user.user_id,
        })
        .collect();

    Ok(Json(response))
}

#[derive(Debug, Deserialize)]
pub struct ConfirmPaymentRequest {
    /// Payment method ID to use (for new payments)
    pub payment_method_id: Option<String>,
}

/// Confirm a payment (after client-side confirmation)
pub async fn confirm_payment(
    State(_state): State<AppState>,
    auth_user: AuthUser,
    tenant: TenantContext,
    Path(id): Path<String>,
    Json(_req): Json<ConfirmPaymentRequest>,
) -> ApiResult<Json<CheckoutResponse>> {
    let transaction_id: Uuid = id
        .parse()
        .map_err(|_| ApiError::from(AppError::Validation("Invalid transaction ID".to_string())))?;

    let transaction = TransactionRepository::get_by_id(&tenant.pool, tenant.org_id, transaction_id)
        .await?
        .ok_or_else(|| ApiError::from(AppError::NotFound("Transaction not found".to_string())))?;

    // Verify user is the customer
    if transaction.customer_user_id != auth_user.user_id {
        return Err(ApiError::from(AppError::NotFound(
            "Transaction not found".to_string(),
        )));
    }

    // Update status to processing
    TransactionRepository::update_status(
        &tenant.pool,
        transaction.id,
        TransactionStatus::Processing,
    )
    .await?;

    let provider =
        PaymentProviderRepository::get_by_id(&tenant.pool, tenant.org_id, transaction.provider_id)
            .await?;

    Ok(Json(CheckoutResponse {
        transaction_id: transaction.id.to_string(),
        status: "processing".to_string(),
        subtotal_cents: transaction.subtotal_cents,
        customer_fee_cents: transaction.customer_fee_cents,
        tax_cents: transaction.tax_cents,
        total_cents: transaction.total_cents,
        provider_type: provider
            .map(|p| format!("{:?}", p.provider_type).to_lowercase())
            .unwrap_or_else(|| "unknown".to_string()),
        client_secret: None,
        payment_id: transaction.external_payment_id,
    }))
}

#[derive(Debug, Deserialize)]
pub struct RefundRequest {
    /// Amount to refund in cents (if partial refund)
    pub amount_cents: Option<i32>,
    /// Reason for refund
    pub reason: Option<String>,
}

/// Request a refund for a transaction
pub async fn request_refund(
    State(_state): State<AppState>,
    auth_user: AuthUser,
    tenant: TenantContext,
    Path(id): Path<String>,
    Json(req): Json<RefundRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let transaction_id: Uuid = id
        .parse()
        .map_err(|_| ApiError::from(AppError::Validation("Invalid transaction ID".to_string())))?;

    let transaction = TransactionRepository::get_by_id(&tenant.pool, tenant.org_id, transaction_id)
        .await?
        .ok_or_else(|| ApiError::from(AppError::NotFound("Transaction not found".to_string())))?;

    // Only customers can request refunds
    if transaction.customer_user_id != auth_user.user_id {
        return Err(ApiError::from(AppError::NotFound(
            "Transaction not found".to_string(),
        )));
    }

    // Check transaction is refundable
    if transaction.status != TransactionStatus::Succeeded {
        return Err(ApiError::from(AppError::Validation(
            "Transaction cannot be refunded".to_string(),
        )));
    }

    let refund_amount = req.amount_cents.unwrap_or(transaction.total_cents);

    // Get the payment provider
    let provider =
        PaymentProviderRepository::get_by_id(&tenant.pool, tenant.org_id, transaction.provider_id)
            .await?
            .ok_or_else(|| {
                ApiError::from(AppError::Internal("Payment provider not found".to_string()))
            })?;

    // Process refund through payment provider
    let refund_id = match provider.provider_type {
        PaymentProviderType::Stripe | PaymentProviderType::Platform => {
            process_stripe_refund(&transaction, refund_amount, req.reason.as_deref()).await?
        }
        PaymentProviderType::Square => {
            process_square_refund(
                &provider,
                &transaction,
                refund_amount,
                req.reason.as_deref(),
            )
            .await?
        }
    };

    let new_status = if refund_amount >= transaction.total_cents {
        TransactionStatus::Refunded
    } else {
        TransactionStatus::PartiallyRefunded
    };

    TransactionRepository::update_status(&tenant.pool, transaction.id, new_status).await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "refund_id": refund_id,
        "refund_amount_cents": refund_amount,
        "status": format!("{:?}", new_status).to_lowercase()
    })))
}

// Helper functions for creating payments with providers

async fn create_stripe_payment_intent(
    provider: &db::models::PaymentProvider,
    transaction: &db::models::Transaction,
    _fee_breakdown: &TransactionFeeBreakdown,
) -> Result<String, ApiError> {
    let stripe_secret = std::env::var("STRIPE_SECRET_KEY")
        .map_err(|_| ApiError::from(AppError::Internal("Stripe not configured".to_string())))?;

    let client = reqwest::Client::new();

    // Build params
    let mut params = vec![
        ("amount", transaction.total_cents.to_string()),
        ("currency", "usd".to_string()),
        ("automatic_payment_methods[enabled]", "true".to_string()),
    ];

    // Add application fee for platform revenue
    if transaction.platform_fee_cents > 0 {
        params.push((
            "application_fee_amount",
            transaction.platform_fee_cents.to_string(),
        ));
    }

    // Add transfer data for connected account
    if let Some(merchant_id) = &provider.merchant_id {
        params.push(("transfer_data[destination]", merchant_id.clone()));
    }

    let response = client
        .post("https://api.stripe.com/v1/payment_intents")
        .header("Authorization", format!("Bearer {}", stripe_secret))
        .form(&params)
        .send()
        .await
        .map_err(|e| ApiError::from(AppError::Internal(e.to_string())))?;

    if !response.status().is_success() {
        let error_body = response.text().await.unwrap_or_default();
        return Err(ApiError::from(AppError::Internal(format!(
            "Stripe error: {}",
            error_body
        ))));
    }

    #[derive(Deserialize)]
    struct PaymentIntentResponse {
        client_secret: String,
    }

    let pi: PaymentIntentResponse = response
        .json()
        .await
        .map_err(|e| ApiError::from(AppError::Internal(e.to_string())))?;

    Ok(pi.client_secret)
}

async fn create_square_payment(
    provider: &db::models::PaymentProvider,
    transaction: &db::models::Transaction,
    _fee_breakdown: &TransactionFeeBreakdown,
) -> Result<String, ApiError> {
    let access_token = provider
        .access_token_encrypted
        .as_ref()
        .ok_or_else(|| ApiError::from(AppError::Internal("Square not configured".to_string())))?;

    let sandbox = std::env::var("SQUARE_SANDBOX")
        .map(|v| v == "true")
        .unwrap_or(false);

    let base_url = if sandbox {
        "https://connect.squareupsandbox.com/v2"
    } else {
        "https://connect.squareup.com/v2"
    };

    let client = reqwest::Client::new();

    // First create an order
    let order_response = client
        .post(&format!("{}/orders", base_url))
        .header("Authorization", format!("Bearer {}", access_token))
        .header("Content-Type", "application/json")
        .header("Square-Version", "2024-01-18")
        .json(&serde_json::json!({
            "idempotency_key": transaction.id.to_string(),
            "order": {
                "location_id": provider.metadata.as_ref()
                        .and_then(|m| m.get("location_id"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("main"),
                "line_items": [{
                    "name": "Service Payment",
                    "quantity": "1",
                    "base_price_money": {
                        "amount": transaction.subtotal_cents as i64,
                        "currency": "USD"
                    }
                }],
                "service_charges": [{
                    "name": "Service Fee",
                    "amount_money": {
                        "amount": transaction.customer_fee_cents as i64,
                        "currency": "USD"
                    },
                    "calculation_phase": "TOTAL_PHASE"
                }],
                "taxes": [{
                    "name": "Sales Tax",
                    "percentage": format!("{:.2}", (transaction.tax_cents as f64 / transaction.subtotal_cents as f64) * 100.0),
                    "scope": "ORDER"
                }]
            }
        }))
        .send()
        .await
        .map_err(|e| ApiError::from(AppError::Internal(e.to_string())))?;

    if !order_response.status().is_success() {
        let error_body = order_response.text().await.unwrap_or_default();
        return Err(ApiError::from(AppError::Internal(format!(
            "Square order error: {}",
            error_body
        ))));
    }

    #[derive(Deserialize)]
    struct SquareOrderResponse {
        order: SquareOrder,
    }

    #[derive(Deserialize)]
    struct SquareOrder {
        id: String,
    }

    let order: SquareOrderResponse = order_response
        .json()
        .await
        .map_err(|e| ApiError::from(AppError::Internal(e.to_string())))?;

    // Return the order ID - client will use Square Web Payments SDK to complete
    Ok(order.order.id)
}

/// Calculate fees preview without creating a transaction
#[derive(Debug, Deserialize)]
pub struct FeePreviewRequest {
    pub subtotal_cents: i32,
    pub tip_cents: Option<i32>,
    pub customer_state: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct FeePreviewResponse {
    pub subtotal_cents: i32,
    pub tip_cents: i32,
    pub customer_fee_cents: i32,
    pub tax_cents: i32,
    pub total_cents: i32,
    pub customer_fee_percent: f64,
    pub tax_rate_percent: f64,
}

pub async fn preview_fees(
    State(_state): State<AppState>,
    _auth_user: AuthUser,
    tenant: TenantContext,
    Json(req): Json<FeePreviewRequest>,
) -> ApiResult<Json<FeePreviewResponse>> {
    // Get the fee tier for this tenant
    let fee_tier = SubscriptionRepository::get_org_fee_tier(&tenant.pool, tenant.org_id).await?;

    // Calculate tax rate
    let tax_rate_percent = if let Some(state) = &req.customer_state {
        match state.to_uppercase().as_str() {
            "CA" => 7.25,
            "TX" => 6.25,
            "FL" => 6.00,
            "NY" => 8.00,
            "WA" => 6.50,
            "OR" | "MT" | "NH" | "DE" => 0.0,
            _ => 6.0,
        }
    } else {
        0.0
    };

    let customer_fee_pct = fee_tier.customer_fee_percent.to_f64().unwrap_or(0.0);
    let provider_fee_pct = fee_tier.provider_fee_percent.to_f64().unwrap_or(0.0);

    let fee_breakdown = TransactionFeeBreakdown::calculate(
        req.subtotal_cents,
        customer_fee_pct,
        provider_fee_pct,
        tax_rate_percent,
        2.9,
    );

    let tip = req.tip_cents.unwrap_or(0);
    let total = fee_breakdown.total_cents + tip;

    Ok(Json(FeePreviewResponse {
        subtotal_cents: req.subtotal_cents,
        tip_cents: tip,
        customer_fee_cents: fee_breakdown.customer_fee_cents,
        tax_cents: fee_breakdown.tax_cents,
        total_cents: total,
        customer_fee_percent: customer_fee_pct * 100.0,
        tax_rate_percent,
    }))
}

// Refund helper functions

async fn process_stripe_refund(
    transaction: &db::models::Transaction,
    amount_cents: i32,
    reason: Option<&str>,
) -> Result<String, ApiError> {
    let stripe_secret = std::env::var("STRIPE_SECRET_KEY")
        .map_err(|_| ApiError::from(AppError::Internal("Stripe not configured".to_string())))?;

    let payment_intent_id = transaction
        .external_payment_id
        .as_ref()
        .ok_or_else(|| ApiError::from(AppError::Validation("No payment to refund".to_string())))?;

    // Extract the payment intent ID from client secret if needed
    let pi_id = if payment_intent_id.contains("_secret_") {
        payment_intent_id
            .split("_secret_")
            .next()
            .unwrap_or(payment_intent_id)
    } else {
        payment_intent_id.as_str()
    };

    let client = reqwest::Client::new();

    let mut params = vec![
        ("payment_intent", pi_id.to_string()),
        ("amount", amount_cents.to_string()),
    ];

    if let Some(r) = reason {
        let stripe_reason = match r.to_lowercase().as_str() {
            "duplicate" => "duplicate",
            "fraudulent" => "fraudulent",
            _ => "requested_by_customer",
        };
        params.push(("reason", stripe_reason.to_string()));
    }

    let response = client
        .post("https://api.stripe.com/v1/refunds")
        .header("Authorization", format!("Bearer {}", stripe_secret))
        .form(&params)
        .send()
        .await
        .map_err(|e| ApiError::from(AppError::Internal(e.to_string())))?;

    if !response.status().is_success() {
        let error_body = response.text().await.unwrap_or_default();
        return Err(ApiError::from(AppError::Internal(format!(
            "Stripe refund error: {}",
            error_body
        ))));
    }

    #[derive(Deserialize)]
    struct RefundResponse {
        id: String,
    }

    let refund: RefundResponse = response
        .json()
        .await
        .map_err(|e| ApiError::from(AppError::Internal(e.to_string())))?;

    Ok(refund.id)
}

async fn process_square_refund(
    provider: &db::models::PaymentProvider,
    transaction: &db::models::Transaction,
    amount_cents: i32,
    reason: Option<&str>,
) -> Result<String, ApiError> {
    let access_token = provider
        .access_token_encrypted
        .as_ref()
        .ok_or_else(|| ApiError::from(AppError::Internal("Square not configured".to_string())))?;

    let payment_id = transaction
        .external_payment_id
        .as_ref()
        .ok_or_else(|| ApiError::from(AppError::Validation("No payment to refund".to_string())))?;

    let sandbox = std::env::var("SQUARE_SANDBOX")
        .map(|v| v == "true")
        .unwrap_or(false);

    let base_url = if sandbox {
        "https://connect.squareupsandbox.com/v2"
    } else {
        "https://connect.squareup.com/v2"
    };

    let client = reqwest::Client::new();

    let response = client
        .post(&format!("{}/refunds", base_url))
        .header("Authorization", format!("Bearer {}", access_token))
        .header("Content-Type", "application/json")
        .header("Square-Version", "2024-01-18")
        .json(&serde_json::json!({
            "idempotency_key": format!("refund_{}", transaction.id),
            "payment_id": payment_id,
            "amount_money": {
                "amount": amount_cents as i64,
                "currency": "USD"
            },
            "reason": reason.unwrap_or("Requested by customer")
        }))
        .send()
        .await
        .map_err(|e| ApiError::from(AppError::Internal(e.to_string())))?;

    if !response.status().is_success() {
        let error_body = response.text().await.unwrap_or_default();
        return Err(ApiError::from(AppError::Internal(format!(
            "Square refund error: {}",
            error_body
        ))));
    }

    #[derive(Deserialize)]
    struct SquareRefundResponse {
        refund: SquareRefund,
    }

    #[derive(Deserialize)]
    struct SquareRefund {
        id: String,
    }

    let refund: SquareRefundResponse = response
        .json()
        .await
        .map_err(|e| ApiError::from(AppError::Internal(e.to_string())))?;

    Ok(refund.refund.id)
}
