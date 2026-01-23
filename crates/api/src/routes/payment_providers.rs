use axum::{
    extract::{Path, Query, State},
    Json,
};
use db::{
    models::{CreatePaymentProvider, PaymentProvider, PaymentProviderType, UpdatePaymentProvider},
    PaymentProviderRepository,
};
use serde::{Deserialize, Serialize};
use shared::AppError;
use uuid::Uuid;

use crate::{
    auth::{AuthUser, TenantContext},
    error::{ApiError, ApiResult},
    state::AppState,
};

/// Payment provider response
#[derive(Debug, Serialize)]
pub struct PaymentProviderResponse {
    pub id: String,
    pub provider_type: String,
    pub is_active: bool,
    pub is_primary: bool,
    pub is_verified: bool,
    pub merchant_id: Option<String>,
    pub connected_at: Option<String>,
    pub account_name: Option<String>,
    pub charges_enabled: bool,
    pub payouts_enabled: bool,
}

fn provider_to_response(p: PaymentProvider) -> PaymentProviderResponse {
    PaymentProviderResponse {
        id: p.id.to_string(),
        provider_type: format!("{:?}", p.provider_type).to_lowercase(),
        is_active: p.is_active,
        is_primary: p.is_primary,
        is_verified: p.is_verified,
        merchant_id: p.merchant_id.or(p.stripe_account_id).or(p.square_merchant_id),
        connected_at: p.created_at.map(|dt| dt.to_rfc3339()),
        account_name: p.account_name,
        charges_enabled: p.charges_enabled.unwrap_or(false),
        payouts_enabled: p.payouts_enabled.unwrap_or(false),
    }
}

/// List all payment providers for the tenant
pub async fn list_payment_providers(
    State(_state): State<AppState>,
    _auth_user: AuthUser,
    tenant: TenantContext,
) -> ApiResult<Json<Vec<PaymentProviderResponse>>> {
    let providers = PaymentProviderRepository::list_for_org(&tenant.pool, tenant.org_id).await?;

    let response: Vec<PaymentProviderResponse> = providers
        .into_iter()
        .map(provider_to_response)
        .collect();

    Ok(Json(response))
}

/// Get the primary payment provider
pub async fn get_primary_provider(
    State(_state): State<AppState>,
    _auth_user: AuthUser,
    tenant: TenantContext,
) -> ApiResult<Json<Option<PaymentProviderResponse>>> {
    let provider = PaymentProviderRepository::get_primary(&tenant.pool, tenant.org_id).await?;
    let response = provider.map(provider_to_response);
    Ok(Json(response))
}

#[derive(Debug, Deserialize)]
pub struct StripeConnectQuery {
    pub redirect_uri: String,
}

/// Get Stripe Connect OAuth URL
pub async fn get_stripe_connect_url(
    State(_state): State<AppState>,
    _auth_user: AuthUser,
    tenant: TenantContext,
    Query(query): Query<StripeConnectQuery>,
) -> ApiResult<Json<serde_json::Value>> {
    // Get Stripe client ID from env/config
    let client_id = std::env::var("STRIPE_CLIENT_ID")
        .map_err(|_| ApiError::from(AppError::Internal("Stripe not configured".to_string())))?;

    // Create state parameter with org_id for security
    let state_param = format!("{}:{}", tenant.org_id.as_uuid(), uuid::Uuid::new_v4());

    // Generate the OAuth URL
    let oauth_url = format!(
        "https://connect.stripe.com/oauth/authorize?response_type=code&client_id={}&scope=read_write&redirect_uri={}&state={}",
        client_id,
        urlencoding::encode(&query.redirect_uri),
        urlencoding::encode(&state_param)
    );

    Ok(Json(serde_json::json!({
        "url": oauth_url,
        "state": state_param
    })))
}

#[derive(Debug, Deserialize)]
pub struct StripeOAuthCallback {
    pub code: String,
    pub state: String,
}

/// Handle Stripe Connect OAuth callback
pub async fn stripe_connect_callback(
    State(_state): State<AppState>,
    _auth_user: AuthUser,
    tenant: TenantContext,
    Json(req): Json<StripeOAuthCallback>,
) -> ApiResult<Json<PaymentProviderResponse>> {
    // Verify state parameter contains our org_id
    let state_parts: Vec<&str> = req.state.split(':').collect();
    if state_parts.len() != 2 {
        return Err(ApiError::from(AppError::Validation(
            "Invalid state parameter".to_string(),
        )));
    }

    let state_org_id: Uuid = state_parts[0]
        .parse()
        .map_err(|_| ApiError::from(AppError::Validation("Invalid state parameter".to_string())))?;

    if state_org_id != *tenant.org_id.as_uuid() {
        return Err(ApiError::from(AppError::Validation(
            "State mismatch".to_string(),
        )));
    }

    // Exchange the code for tokens using Stripe API
    let client_secret = std::env::var("STRIPE_SECRET_KEY")
        .map_err(|_| ApiError::from(AppError::Internal("Stripe not configured".to_string())))?;

    let client = reqwest::Client::new();
    let response = client
        .post("https://connect.stripe.com/oauth/token")
        .form(&[
            ("grant_type", "authorization_code"),
            ("code", &req.code),
            ("client_secret", &client_secret),
        ])
        .send()
        .await
        .map_err(|e| ApiError::from(AppError::Internal(e.to_string())))?;

    if !response.status().is_success() {
        let error_body = response.text().await.unwrap_or_default();
        return Err(ApiError::from(AppError::Internal(format!(
            "Stripe OAuth failed: {}",
            error_body
        ))));
    }

    #[derive(Deserialize)]
    #[allow(dead_code)]
    struct StripeOAuthResponse {
        access_token: String,
        refresh_token: Option<String>,
        stripe_user_id: String,
        stripe_publishable_key: Option<String>,
        livemode: bool,
    }

    let oauth_response: StripeOAuthResponse = response
        .json()
        .await
        .map_err(|e| ApiError::from(AppError::Internal(e.to_string())))?;

    // Store the provider credentials
    let input = CreatePaymentProvider {
        provider_type: PaymentProviderType::Stripe,
        stripe_account_id: Some(oauth_response.stripe_user_id.clone()),
        stripe_account_type: Some("standard".to_string()),
        square_merchant_id: None,
        access_token_encrypted: Some(oauth_response.access_token), // TODO: encrypt before storing
        refresh_token_encrypted: oauth_response.refresh_token,
        token_expires_at: None,
    };

    let provider =
        PaymentProviderRepository::create(&tenant.pool, tenant.org_id, input).await?;

    Ok(Json(provider_to_response(provider)))
}

#[derive(Debug, Deserialize)]
pub struct SquareOAuthQuery {
    pub redirect_uri: String,
}

/// Get Square OAuth URL
pub async fn get_square_oauth_url(
    State(_state): State<AppState>,
    _auth_user: AuthUser,
    tenant: TenantContext,
    Query(query): Query<SquareOAuthQuery>,
) -> ApiResult<Json<serde_json::Value>> {
    let app_id = std::env::var("SQUARE_APPLICATION_ID")
        .map_err(|_| ApiError::from(AppError::Internal("Square not configured".to_string())))?;

    let sandbox = std::env::var("SQUARE_SANDBOX")
        .map(|v| v == "true")
        .unwrap_or(false);

    // Create state parameter with org_id for security
    let state_param = format!("{}:{}", tenant.org_id.as_uuid(), uuid::Uuid::new_v4());

    let scopes = [
        "MERCHANT_PROFILE_READ",
        "PAYMENTS_WRITE",
        "PAYMENTS_READ",
        "ORDERS_WRITE",
        "ORDERS_READ",
        "CUSTOMERS_WRITE",
        "CUSTOMERS_READ",
        "ITEMS_READ",
        "ITEMS_WRITE",
    ];

    let base = if sandbox {
        "https://connect.squareupsandbox.com/oauth2"
    } else {
        "https://connect.squareup.com/oauth2"
    };

    let oauth_url = format!(
        "{}/authorize?client_id={}&scope={}&session=false&state={}&redirect_uri={}",
        base,
        app_id,
        scopes.join("+"),
        urlencoding::encode(&state_param),
        urlencoding::encode(&query.redirect_uri)
    );

    Ok(Json(serde_json::json!({
        "url": oauth_url,
        "state": state_param
    })))
}

#[derive(Debug, Deserialize)]
pub struct SquareOAuthCallback {
    pub code: String,
    pub state: String,
}

/// Handle Square OAuth callback
pub async fn square_oauth_callback(
    State(_state): State<AppState>,
    _auth_user: AuthUser,
    tenant: TenantContext,
    Json(req): Json<SquareOAuthCallback>,
) -> ApiResult<Json<PaymentProviderResponse>> {
    // Verify state parameter contains our org_id
    let state_parts: Vec<&str> = req.state.split(':').collect();
    if state_parts.len() != 2 {
        return Err(ApiError::from(AppError::Validation(
            "Invalid state parameter".to_string(),
        )));
    }

    let state_org_id: Uuid = state_parts[0]
        .parse()
        .map_err(|_| ApiError::from(AppError::Validation("Invalid state parameter".to_string())))?;

    if state_org_id != *tenant.org_id.as_uuid() {
        return Err(ApiError::from(AppError::Validation(
            "State mismatch".to_string(),
        )));
    }

    // Exchange the code for tokens
    let app_id = std::env::var("SQUARE_APPLICATION_ID")
        .map_err(|_| ApiError::from(AppError::Internal("Square not configured".to_string())))?;
    let app_secret = std::env::var("SQUARE_APPLICATION_SECRET")
        .map_err(|_| ApiError::from(AppError::Internal("Square not configured".to_string())))?;

    let sandbox = std::env::var("SQUARE_SANDBOX")
        .map(|v| v == "true")
        .unwrap_or(false);

    let base = if sandbox {
        "https://connect.squareupsandbox.com"
    } else {
        "https://connect.squareup.com"
    };

    let client = reqwest::Client::new();
    let response = client
        .post(&format!("{}/oauth2/token", base))
        .json(&serde_json::json!({
            "client_id": app_id,
            "client_secret": app_secret,
            "code": req.code,
            "grant_type": "authorization_code"
        }))
        .send()
        .await
        .map_err(|e| ApiError::from(AppError::Internal(e.to_string())))?;

    if !response.status().is_success() {
        let error_body = response.text().await.unwrap_or_default();
        return Err(ApiError::from(AppError::Internal(format!(
            "Square OAuth failed: {}",
            error_body
        ))));
    }

    #[derive(Deserialize)]
    #[allow(dead_code)]
    struct SquareOAuthResponse {
        access_token: String,
        token_type: String,
        expires_at: String,
        merchant_id: String,
        refresh_token: Option<String>,
    }

    let oauth_response: SquareOAuthResponse = response
        .json()
        .await
        .map_err(|e| ApiError::from(AppError::Internal(e.to_string())))?;

    // Parse token expiration
    let token_expires_at = chrono::DateTime::parse_from_rfc3339(&oauth_response.expires_at)
        .ok()
        .map(|dt| dt.with_timezone(&chrono::Utc));

    // Store the provider credentials
    let input = CreatePaymentProvider {
        provider_type: PaymentProviderType::Square,
        stripe_account_id: None,
        stripe_account_type: None,
        square_merchant_id: Some(oauth_response.merchant_id.clone()),
        access_token_encrypted: Some(oauth_response.access_token), // TODO: encrypt before storing
        refresh_token_encrypted: oauth_response.refresh_token,
        token_expires_at,
    };

    let provider =
        PaymentProviderRepository::create(&tenant.pool, tenant.org_id, input).await?;

    Ok(Json(provider_to_response(provider)))
}

#[derive(Debug, Deserialize)]
pub struct UpdateProviderRequest {
    pub is_active: Option<bool>,
}

/// Update a payment provider
pub async fn update_payment_provider(
    State(_state): State<AppState>,
    _auth_user: AuthUser,
    tenant: TenantContext,
    Path(id): Path<String>,
    Json(req): Json<UpdateProviderRequest>,
) -> ApiResult<Json<PaymentProviderResponse>> {
    let provider_id: Uuid = id
        .parse()
        .map_err(|_| ApiError::from(AppError::Validation("Invalid provider ID".to_string())))?;

    let input = UpdatePaymentProvider {
        is_active: req.is_active,
        ..Default::default()
    };

    let provider =
        PaymentProviderRepository::update(&tenant.pool, tenant.org_id, provider_id, input)
            .await?
            .ok_or_else(|| {
                ApiError::from(AppError::NotFound("Payment provider not found".to_string()))
            })?;

    Ok(Json(provider_to_response(provider)))
}

/// Disconnect a payment provider
pub async fn disconnect_payment_provider(
    State(_state): State<AppState>,
    _auth_user: AuthUser,
    tenant: TenantContext,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let provider_id: Uuid = id
        .parse()
        .map_err(|_| ApiError::from(AppError::Validation("Invalid provider ID".to_string())))?;

    PaymentProviderRepository::deactivate(&tenant.pool, tenant.org_id, provider_id)
        .await?;

    Ok(Json(serde_json::json!({ "success": true })))
}

// URL encoding helper
mod urlencoding {
    pub fn encode(s: &str) -> String {
        let mut result = String::new();
        for c in s.chars() {
            match c {
                'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => result.push(c),
                _ => {
                    for b in c.to_string().as_bytes() {
                        result.push_str(&format!("%{:02X}", b));
                    }
                }
            }
        }
        result
    }
}
