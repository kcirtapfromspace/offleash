use axum::{extract::State, Json};
use db::models::{
    BillingFrequency, BusinessModel, FeeStructure, OrganizationSettings, PaymentConfig,
    UpdateOrganization,
};
use db::OrganizationRepository;
use serde::{Deserialize, Serialize};
use shared::DomainError;

use crate::{
    auth::AuthUser,
    error::{ApiError, ApiResult},
    state::AppState,
};

/// Validates a hex color code matches the #XXXXXX pattern
fn validate_hex_color(color: &str) -> bool {
    if color.len() != 7 {
        return false;
    }

    let mut chars = color.chars();

    // First char must be '#'
    if chars.next() != Some('#') {
        return false;
    }

    // Remaining 6 chars must be hex digits (0-9, A-F, a-f)
    chars.all(|c| c.is_ascii_hexdigit())
}

#[derive(Debug, Serialize)]
pub struct AdminBrandingResponse {
    pub primary_color: Option<String>,
    pub secondary_color: Option<String>,
    pub logo_url: Option<String>,
    pub favicon_url: Option<String>,
    pub font_family: Option<String>,
}

/// GET /admin/branding - Fetch current tenant's branding settings
pub async fn get_branding(
    auth: AuthUser,
    State(state): State<AppState>,
) -> ApiResult<Json<AdminBrandingResponse>> {
    let org_id = auth.org_id.ok_or_else(|| {
        ApiError::from(shared::AppError::Validation(
            "Organization context required".to_string(),
        ))
    })?;

    let org = OrganizationRepository::find_by_id(&state.pool, org_id)
        .await?
        .ok_or_else(|| ApiError::from(DomainError::TenantNotFound(org_id.to_string())))?;

    let settings = org.settings.0;

    Ok(Json(AdminBrandingResponse {
        primary_color: settings.primary_color,
        secondary_color: settings.secondary_color,
        logo_url: settings.logo_url,
        favicon_url: settings.favicon_url,
        font_family: settings.font_family,
    }))
}

#[derive(Debug, Deserialize)]
pub struct UpdateBrandingRequest {
    pub primary_color: Option<String>,
    pub secondary_color: Option<String>,
    pub logo_url: Option<String>,
    pub favicon_url: Option<String>,
    pub font_family: Option<String>,
}

/// PUT /admin/branding - Update current tenant's branding settings
pub async fn update_branding(
    auth: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<UpdateBrandingRequest>,
) -> ApiResult<Json<AdminBrandingResponse>> {
    let org_id = auth.org_id.ok_or_else(|| {
        ApiError::from(shared::AppError::Validation(
            "Organization context required".to_string(),
        ))
    })?;

    // Validate hex color codes if provided
    if let Some(ref color) = req.primary_color {
        if !validate_hex_color(color) {
            return Err(ApiError::from(shared::AppError::Validation(
                "primary_color must be a valid hex color code (e.g., #FF5733)".to_string(),
            )));
        }
    }

    if let Some(ref color) = req.secondary_color {
        if !validate_hex_color(color) {
            return Err(ApiError::from(shared::AppError::Validation(
                "secondary_color must be a valid hex color code (e.g., #FF5733)".to_string(),
            )));
        }
    }

    // Fetch current organization to get existing settings
    let org = OrganizationRepository::find_by_id(&state.pool, org_id)
        .await?
        .ok_or_else(|| ApiError::from(DomainError::TenantNotFound(org_id.to_string())))?;

    let current_settings = org.settings.0;

    // Merge with existing settings (only update provided fields, preserve payment_config)
    let new_settings = OrganizationSettings {
        primary_color: req.primary_color.or(current_settings.primary_color),
        secondary_color: req.secondary_color.or(current_settings.secondary_color),
        logo_url: req.logo_url.or(current_settings.logo_url),
        favicon_url: req.favicon_url.or(current_settings.favicon_url),
        font_family: req.font_family.or(current_settings.font_family),
        payment_config: current_settings.payment_config, // Preserve payment config
    };

    // Update the organization settings
    let updated_org = OrganizationRepository::update(
        &state.pool,
        org_id,
        UpdateOrganization {
            name: None,
            slug: None,
            custom_domain: None,
            settings: Some(new_settings),
        },
    )
    .await?
    .ok_or_else(|| ApiError::from(DomainError::TenantNotFound(org_id.to_string())))?;

    let settings = updated_org.settings.0;

    Ok(Json(AdminBrandingResponse {
        primary_color: settings.primary_color,
        secondary_color: settings.secondary_color,
        logo_url: settings.logo_url,
        favicon_url: settings.favicon_url,
        font_family: settings.font_family,
    }))
}

// ============================================================================
// Payment Configuration Endpoints
// ============================================================================

#[derive(Debug, Serialize)]
pub struct PaymentConfigResponse {
    pub business_model: String,
    pub fee_structure: String,
    pub billing_frequency: String,
    pub apple_pay_enabled: bool,
    pub google_pay_enabled: bool,
    pub preferred_provider: Option<String>,
}

/// GET /admin/payment-config - Fetch current tenant's payment configuration
pub async fn get_payment_config(
    auth: AuthUser,
    State(state): State<AppState>,
) -> ApiResult<Json<PaymentConfigResponse>> {
    let org_id = auth.org_id.ok_or_else(|| {
        ApiError::from(shared::AppError::Validation(
            "Organization context required".to_string(),
        ))
    })?;

    let org = OrganizationRepository::find_by_id(&state.pool, org_id)
        .await?
        .ok_or_else(|| ApiError::from(DomainError::TenantNotFound(org_id.to_string())))?;

    let config = org.settings.0.payment_config;

    Ok(Json(PaymentConfigResponse {
        business_model: match config.business_model {
            BusinessModel::BookingOnly => "booking_only".to_string(),
            BusinessModel::FullService => "full_service".to_string(),
        },
        fee_structure: match config.fee_structure {
            FeeStructure::CustomerPays => "customer_pays".to_string(),
            FeeStructure::SplitFees => "split_fees".to_string(),
            FeeStructure::OwnerSubscription => "owner_subscription".to_string(),
        },
        billing_frequency: match config.billing_frequency {
            BillingFrequency::Monthly => "monthly".to_string(),
            BillingFrequency::Yearly => "yearly".to_string(),
        },
        apple_pay_enabled: config.apple_pay_enabled,
        google_pay_enabled: config.google_pay_enabled,
        preferred_provider: config.preferred_provider,
    }))
}

#[derive(Debug, Deserialize)]
pub struct UpdatePaymentConfigRequest {
    pub business_model: Option<String>,
    pub fee_structure: Option<String>,
    pub billing_frequency: Option<String>,
    pub apple_pay_enabled: Option<bool>,
    pub google_pay_enabled: Option<bool>,
    pub preferred_provider: Option<String>,
}

/// PUT /admin/payment-config - Update current tenant's payment configuration
pub async fn update_payment_config(
    auth: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<UpdatePaymentConfigRequest>,
) -> ApiResult<Json<PaymentConfigResponse>> {
    let org_id = auth.org_id.ok_or_else(|| {
        ApiError::from(shared::AppError::Validation(
            "Organization context required".to_string(),
        ))
    })?;

    // Fetch current organization
    let org = OrganizationRepository::find_by_id(&state.pool, org_id)
        .await?
        .ok_or_else(|| ApiError::from(DomainError::TenantNotFound(org_id.to_string())))?;

    let current_settings = org.settings.0;
    let current_config = current_settings.payment_config.clone();

    // Parse and validate business model
    let business_model = if let Some(ref bm) = req.business_model {
        match bm.as_str() {
            "booking_only" => BusinessModel::BookingOnly,
            "full_service" => BusinessModel::FullService,
            _ => {
                return Err(ApiError::from(shared::AppError::Validation(
                    "business_model must be 'booking_only' or 'full_service'".to_string(),
                )))
            }
        }
    } else {
        current_config.business_model
    };

    // Parse and validate fee structure
    let fee_structure = if let Some(ref fs) = req.fee_structure {
        match fs.as_str() {
            "customer_pays" => FeeStructure::CustomerPays,
            "split_fees" => FeeStructure::SplitFees,
            "owner_subscription" => FeeStructure::OwnerSubscription,
            _ => {
                return Err(ApiError::from(shared::AppError::Validation(
                    "fee_structure must be 'customer_pays', 'split_fees', or 'owner_subscription'"
                        .to_string(),
                )))
            }
        }
    } else {
        current_config.fee_structure
    };

    // Parse and validate billing frequency
    let billing_frequency = if let Some(ref bf) = req.billing_frequency {
        match bf.as_str() {
            "monthly" => BillingFrequency::Monthly,
            "yearly" => BillingFrequency::Yearly,
            _ => {
                return Err(ApiError::from(shared::AppError::Validation(
                    "billing_frequency must be 'monthly' or 'yearly'".to_string(),
                )))
            }
        }
    } else {
        current_config.billing_frequency
    };

    // Validate preferred provider if provided
    if let Some(ref provider) = req.preferred_provider {
        if !["stripe", "square"].contains(&provider.as_str()) {
            return Err(ApiError::from(shared::AppError::Validation(
                "preferred_provider must be 'stripe' or 'square'".to_string(),
            )));
        }
    }

    // Build new payment config
    let new_payment_config = PaymentConfig {
        business_model,
        fee_structure,
        billing_frequency,
        apple_pay_enabled: req.apple_pay_enabled.unwrap_or(current_config.apple_pay_enabled),
        google_pay_enabled: req.google_pay_enabled.unwrap_or(current_config.google_pay_enabled),
        preferred_provider: req.preferred_provider.or(current_config.preferred_provider),
    };

    // Preserve branding settings, update payment config
    let new_settings = OrganizationSettings {
        primary_color: current_settings.primary_color,
        secondary_color: current_settings.secondary_color,
        logo_url: current_settings.logo_url,
        favicon_url: current_settings.favicon_url,
        font_family: current_settings.font_family,
        payment_config: new_payment_config,
    };

    // Update the organization
    let updated_org = OrganizationRepository::update(
        &state.pool,
        org_id,
        UpdateOrganization {
            name: None,
            slug: None,
            custom_domain: None,
            settings: Some(new_settings),
        },
    )
    .await?
    .ok_or_else(|| ApiError::from(DomainError::TenantNotFound(org_id.to_string())))?;

    let config = updated_org.settings.0.payment_config;

    Ok(Json(PaymentConfigResponse {
        business_model: match config.business_model {
            BusinessModel::BookingOnly => "booking_only".to_string(),
            BusinessModel::FullService => "full_service".to_string(),
        },
        fee_structure: match config.fee_structure {
            FeeStructure::CustomerPays => "customer_pays".to_string(),
            FeeStructure::SplitFees => "split_fees".to_string(),
            FeeStructure::OwnerSubscription => "owner_subscription".to_string(),
        },
        billing_frequency: match config.billing_frequency {
            BillingFrequency::Monthly => "monthly".to_string(),
            BillingFrequency::Yearly => "yearly".to_string(),
        },
        apple_pay_enabled: config.apple_pay_enabled,
        google_pay_enabled: config.google_pay_enabled,
        preferred_provider: config.preferred_provider,
    }))
}
