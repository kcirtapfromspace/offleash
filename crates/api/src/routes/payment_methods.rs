use axum::{
    extract::{Path, State},
    Json,
};
use db::{
    models::{
        CreateCustomerPaymentMethod, PaymentMethodType, PaymentProviderType,
        UpdateCustomerPaymentMethod,
    },
    CustomerPaymentMethodRepository,
};
use serde::{Deserialize, Serialize};
use shared::AppError;
use uuid::Uuid;

use crate::{
    auth::{AuthUser, TenantContext},
    error::{ApiError, ApiResult},
    state::AppState,
};

#[derive(Debug, Serialize)]
pub struct PaymentMethodResponse {
    pub id: String,
    pub method_type: String,
    pub display_name: String,
    pub last_four: Option<String>,
    pub brand: Option<String>,
    pub exp_month: Option<i32>,
    pub exp_year: Option<i32>,
    pub expiry: Option<String>,
    pub wallet_type: Option<String>,
    pub is_default: bool,
    pub is_expired: bool,
    pub icon: String,
    pub created_at: String,
}

/// List all payment methods for the authenticated user
pub async fn list_payment_methods(
    State(_state): State<AppState>,
    auth_user: AuthUser,
    tenant: TenantContext,
) -> ApiResult<Json<Vec<PaymentMethodResponse>>> {
    let methods = CustomerPaymentMethodRepository::list_for_user(
        &tenant.pool,
        tenant.org_id,
        auth_user.user_id,
    )
    .await?;

    let response: Vec<PaymentMethodResponse> = methods
        .into_iter()
        .map(|m| PaymentMethodResponse {
            id: m.id.to_string(),
            method_type: m.method_type.to_string(),
            display_name: m.display_name(),
            last_four: m.last_four.clone(),
            brand: m.brand.clone(),
            exp_month: m.exp_month,
            exp_year: m.exp_year,
            expiry: m.expiry_display(),
            wallet_type: m.wallet_type.clone(),
            is_default: m.is_default,
            is_expired: m.is_expired(),
            icon: m.icon().to_string(),
            created_at: m.created_at.to_rfc3339(),
        })
        .collect();

    Ok(Json(response))
}

#[derive(Debug, Deserialize)]
pub struct CreatePaymentMethodRequest {
    /// Payment method type: "card", "apple_pay", "google_pay", "shop_pay", "link", "bank_account"
    pub method_type: String,
    /// Provider type: "stripe", "square", "platform"
    pub provider_type: Option<String>,
    /// Stripe payment method ID (from Stripe Elements)
    pub stripe_payment_method_id: Option<String>,
    /// Stripe customer ID
    pub stripe_customer_id: Option<String>,
    /// Square card ID (from Square SDK)
    pub square_card_id: Option<String>,
    /// Square customer ID
    pub square_customer_id: Option<String>,
    /// For cards: last 4 digits
    pub last_four: Option<String>,
    /// For cards: brand (visa, mastercard, amex, discover)
    pub brand: Option<String>,
    /// For cards: expiration month (1-12)
    pub exp_month: Option<i32>,
    /// For cards: expiration year
    pub exp_year: Option<i32>,
    /// Cardholder name
    pub cardholder_name: Option<String>,
    /// For bank accounts: bank name
    pub bank_name: Option<String>,
    /// For bank accounts: last 4 of account number
    pub account_last_four: Option<String>,
    /// For wallets: wallet type
    pub wallet_type: Option<String>,
    /// Set as default payment method
    pub is_default: Option<bool>,
    /// Billing address
    pub billing_address: Option<serde_json::Value>,
}

/// Add a new payment method
pub async fn create_payment_method(
    State(_state): State<AppState>,
    auth_user: AuthUser,
    tenant: TenantContext,
    Json(req): Json<CreatePaymentMethodRequest>,
) -> ApiResult<Json<PaymentMethodResponse>> {
    // Parse method type
    let method_type = match req.method_type.to_lowercase().as_str() {
        "card" => PaymentMethodType::Card,
        "apple_pay" => PaymentMethodType::ApplePay,
        "google_pay" => PaymentMethodType::GooglePay,
        "shop_pay" => PaymentMethodType::ShopPay,
        "link" => PaymentMethodType::Link,
        "bank_account" => PaymentMethodType::BankAccount,
        _ => {
            return Err(ApiError::from(AppError::Validation(
                "Invalid method type. Must be 'card', 'apple_pay', 'google_pay', 'shop_pay', 'link', or 'bank_account'".to_string(),
            )))
        }
    };

    // Parse provider type
    let provider_type = match req
        .provider_type
        .as_deref()
        .unwrap_or("stripe")
        .to_lowercase()
        .as_str()
    {
        "stripe" => PaymentProviderType::Stripe,
        "square" => PaymentProviderType::Square,
        "platform" => PaymentProviderType::Platform,
        _ => PaymentProviderType::Stripe,
    };

    // For cards, validate required fields
    if method_type == PaymentMethodType::Card {
        if req.last_four.is_none() {
            return Err(ApiError::from(AppError::Validation(
                "last_four is required for card payment methods".to_string(),
            )));
        }
        if req.stripe_payment_method_id.is_none() && req.square_card_id.is_none() {
            return Err(ApiError::from(AppError::Validation(
                "Either stripe_payment_method_id or square_card_id is required".to_string(),
            )));
        }
    }

    let input = CreateCustomerPaymentMethod {
        provider_type,
        method_type,
        stripe_payment_method_id: req.stripe_payment_method_id,
        stripe_customer_id: req.stripe_customer_id,
        square_card_id: req.square_card_id,
        square_customer_id: req.square_customer_id,
        last_four: req.last_four,
        brand: req.brand,
        exp_month: req.exp_month,
        exp_year: req.exp_year,
        cardholder_name: req.cardholder_name,
        bank_name: req.bank_name,
        account_last_four: req.account_last_four,
        wallet_type: req.wallet_type,
        is_default: req.is_default.unwrap_or(false),
        billing_address: req.billing_address,
    };

    let method = CustomerPaymentMethodRepository::create(
        &tenant.pool,
        tenant.org_id,
        auth_user.user_id,
        input,
    )
    .await?;

    Ok(Json(method_to_response(method)))
}

fn method_to_response(method: db::models::CustomerPaymentMethod) -> PaymentMethodResponse {
    PaymentMethodResponse {
        id: method.id.to_string(),
        method_type: method.method_type.to_string(),
        display_name: method.display_name(),
        last_four: method.last_four.clone(),
        brand: method.brand.clone(),
        exp_month: method.exp_month,
        exp_year: method.exp_year,
        expiry: method.expiry_display(),
        wallet_type: method.wallet_type.clone(),
        is_default: method.is_default,
        is_expired: method.is_expired(),
        icon: method.icon().to_string(),
        created_at: method.created_at.to_rfc3339(),
    }
}

/// Set a payment method as the default
pub async fn set_default_payment_method(
    State(_state): State<AppState>,
    auth_user: AuthUser,
    tenant: TenantContext,
    Path(id): Path<String>,
) -> ApiResult<Json<PaymentMethodResponse>> {
    let method_id: Uuid = id.parse().map_err(|_| {
        ApiError::from(AppError::Validation(
            "Invalid payment method ID".to_string(),
        ))
    })?;

    let method = CustomerPaymentMethodRepository::set_default(
        &tenant.pool,
        tenant.org_id,
        auth_user.user_id,
        method_id,
    )
    .await?
    .ok_or_else(|| ApiError::from(AppError::NotFound("Payment method not found".to_string())))?;

    Ok(Json(method_to_response(method)))
}

/// Delete a payment method
pub async fn delete_payment_method(
    State(_state): State<AppState>,
    auth_user: AuthUser,
    tenant: TenantContext,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let method_id: Uuid = id.parse().map_err(|_| {
        ApiError::from(AppError::Validation(
            "Invalid payment method ID".to_string(),
        ))
    })?;

    let deleted = CustomerPaymentMethodRepository::delete(
        &tenant.pool,
        tenant.org_id,
        auth_user.user_id,
        method_id,
    )
    .await?;

    if !deleted {
        return Err(ApiError::from(AppError::NotFound(
            "Payment method not found".to_string(),
        )));
    }

    Ok(Json(serde_json::json!({ "success": true })))
}

#[derive(Debug, Deserialize)]
pub struct UpdatePaymentMethodRequest {
    pub is_default: Option<bool>,
    pub billing_address: Option<serde_json::Value>,
}

/// Update a payment method
pub async fn update_payment_method(
    State(_state): State<AppState>,
    auth_user: AuthUser,
    tenant: TenantContext,
    Path(id): Path<String>,
    Json(req): Json<UpdatePaymentMethodRequest>,
) -> ApiResult<Json<PaymentMethodResponse>> {
    let method_id: Uuid = id.parse().map_err(|_| {
        ApiError::from(AppError::Validation(
            "Invalid payment method ID".to_string(),
        ))
    })?;

    let input = UpdateCustomerPaymentMethod {
        is_default: req.is_default,
        is_active: None,
        billing_address: req.billing_address,
        metadata: None,
    };

    let method = CustomerPaymentMethodRepository::update(
        &tenant.pool,
        tenant.org_id,
        auth_user.user_id,
        method_id,
        input,
    )
    .await?
    .ok_or_else(|| ApiError::from(AppError::NotFound("Payment method not found".to_string())))?;

    Ok(Json(method_to_response(method)))
}
