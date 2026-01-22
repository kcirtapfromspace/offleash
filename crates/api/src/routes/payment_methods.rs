use axum::{
    extract::{Path, State},
    Json,
};
use db::{
    models::{CardBrand, CreateCustomerPaymentMethod, PaymentMethodType},
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
    pub card_last_four: Option<String>,
    pub card_brand: Option<String>,
    pub card_exp_month: Option<i32>,
    pub card_exp_year: Option<i32>,
    pub nickname: Option<String>,
    pub is_default: bool,
    pub is_expired: bool,
    pub created_at: String,
}

/// List all payment methods for the authenticated user
pub async fn list_payment_methods(
    State(_state): State<AppState>,
    auth_user: AuthUser,
    tenant: TenantContext,
) -> ApiResult<Json<Vec<PaymentMethodResponse>>> {
    let methods =
        CustomerPaymentMethodRepository::list_for_customer(&tenant.pool, tenant.org_id, auth_user.user_id)
            .await?;

    let response: Vec<PaymentMethodResponse> = methods
        .into_iter()
        .map(|m| PaymentMethodResponse {
            id: m.id.to_string(),
            method_type: m.method_type.to_string(),
            display_name: m.display_name(),
            card_last_four: m.card_last_four.clone(),
            card_brand: m.card_brand.map(|b| b.to_string()),
            card_exp_month: m.card_exp_month,
            card_exp_year: m.card_exp_year,
            nickname: m.nickname.clone(),
            is_default: m.is_default,
            is_expired: m.is_expired(),
            created_at: m.created_at.to_rfc3339(),
        })
        .collect();

    Ok(Json(response))
}

#[derive(Debug, Deserialize)]
pub struct CreatePaymentMethodRequest {
    /// "card", "apple_pay", or "bank_account"
    pub method_type: String,
    /// Square card nonce from SDK
    pub card_nonce: Option<String>,
    /// For cards: last 4 digits
    pub card_last_four: Option<String>,
    /// For cards: brand (visa, mastercard, amex, discover)
    pub card_brand: Option<String>,
    /// For cards: expiration month (1-12)
    pub card_exp_month: Option<i32>,
    /// For cards: expiration year
    pub card_exp_year: Option<i32>,
    /// Optional nickname
    pub nickname: Option<String>,
    /// Set as default payment method
    pub is_default: Option<bool>,
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
        "bank_account" => PaymentMethodType::BankAccount,
        _ => {
            return Err(ApiError::from(AppError::Validation(
                "Invalid method type. Must be 'card', 'apple_pay', or 'bank_account'".to_string(),
            )))
        }
    };

    // Parse card brand if provided
    let card_brand = if let Some(brand_str) = &req.card_brand {
        Some(match brand_str.to_lowercase().as_str() {
            "visa" => CardBrand::Visa,
            "mastercard" => CardBrand::Mastercard,
            "amex" | "american_express" => CardBrand::Amex,
            "discover" => CardBrand::Discover,
            _ => CardBrand::Other,
        })
    } else {
        None
    };

    // For cards, validate required fields
    if method_type == PaymentMethodType::Card {
        if req.card_last_four.is_none() {
            return Err(ApiError::from(AppError::Validation(
                "card_last_four is required for card payment methods".to_string(),
            )));
        }
    }

    // TODO: In production, use Square SDK to create the card on file
    // from the card_nonce and get back a square_card_id
    let square_card_id = req.card_nonce.clone(); // Placeholder - would be Square's card ID

    let input = CreateCustomerPaymentMethod {
        method_type,
        card_last_four: req.card_last_four,
        card_brand,
        card_exp_month: req.card_exp_month,
        card_exp_year: req.card_exp_year,
        square_card_id,
        nickname: req.nickname,
        is_default: req.is_default.unwrap_or(false),
    };

    let method = CustomerPaymentMethodRepository::create(
        &tenant.pool,
        tenant.org_id,
        auth_user.user_id,
        input,
    )
    .await?;

    let is_expired = method.is_expired();
    Ok(Json(PaymentMethodResponse {
        id: method.id.to_string(),
        method_type: method.method_type.to_string(),
        display_name: method.display_name(),
        card_last_four: method.card_last_four,
        card_brand: method.card_brand.map(|b| b.to_string()),
        card_exp_month: method.card_exp_month,
        card_exp_year: method.card_exp_year,
        nickname: method.nickname,
        is_default: method.is_default,
        is_expired,
        created_at: method.created_at.to_rfc3339(),
    }))
}

/// Set a payment method as the default
pub async fn set_default_payment_method(
    State(_state): State<AppState>,
    auth_user: AuthUser,
    tenant: TenantContext,
    Path(id): Path<String>,
) -> ApiResult<Json<PaymentMethodResponse>> {
    let method_id: Uuid = id
        .parse()
        .map_err(|_| ApiError::from(AppError::Validation("Invalid payment method ID".to_string())))?;

    let method = CustomerPaymentMethodRepository::set_default(
        &tenant.pool,
        tenant.org_id,
        auth_user.user_id,
        method_id,
    )
    .await?
    .ok_or_else(|| ApiError::from(AppError::NotFound("Payment method not found".to_string())))?;

    let is_expired = method.is_expired();
    Ok(Json(PaymentMethodResponse {
        id: method.id.to_string(),
        method_type: method.method_type.to_string(),
        display_name: method.display_name(),
        card_last_four: method.card_last_four,
        card_brand: method.card_brand.map(|b| b.to_string()),
        card_exp_month: method.card_exp_month,
        card_exp_year: method.card_exp_year,
        nickname: method.nickname,
        is_default: method.is_default,
        is_expired,
        created_at: method.created_at.to_rfc3339(),
    }))
}

/// Delete a payment method
pub async fn delete_payment_method(
    State(_state): State<AppState>,
    auth_user: AuthUser,
    tenant: TenantContext,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let method_id: Uuid = id
        .parse()
        .map_err(|_| ApiError::from(AppError::Validation("Invalid payment method ID".to_string())))?;

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
    pub nickname: Option<String>,
}

/// Update a payment method (currently only nickname)
pub async fn update_payment_method(
    State(_state): State<AppState>,
    auth_user: AuthUser,
    tenant: TenantContext,
    Path(id): Path<String>,
    Json(req): Json<UpdatePaymentMethodRequest>,
) -> ApiResult<Json<PaymentMethodResponse>> {
    let method_id: Uuid = id
        .parse()
        .map_err(|_| ApiError::from(AppError::Validation("Invalid payment method ID".to_string())))?;

    let method = CustomerPaymentMethodRepository::update_nickname(
        &tenant.pool,
        tenant.org_id,
        auth_user.user_id,
        method_id,
        req.nickname.as_deref(),
    )
    .await?
    .ok_or_else(|| ApiError::from(AppError::NotFound("Payment method not found".to_string())))?;

    let is_expired = method.is_expired();
    Ok(Json(PaymentMethodResponse {
        id: method.id.to_string(),
        method_type: method.method_type.to_string(),
        display_name: method.display_name(),
        card_last_four: method.card_last_four,
        card_brand: method.card_brand.map(|b| b.to_string()),
        card_exp_month: method.card_exp_month,
        card_exp_year: method.card_exp_year,
        nickname: method.nickname,
        is_default: method.is_default,
        is_expired,
        created_at: method.created_at.to_rfc3339(),
    }))
}
