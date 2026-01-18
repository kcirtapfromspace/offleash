use axum::{extract::State, Json};
use db::models::{OrganizationSettings, UpdateOrganization};
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

    // Merge with existing settings (only update provided fields)
    let new_settings = OrganizationSettings {
        primary_color: req.primary_color.or(current_settings.primary_color),
        secondary_color: req.secondary_color.or(current_settings.secondary_color),
        logo_url: req.logo_url.or(current_settings.logo_url),
        favicon_url: req.favicon_url.or(current_settings.favicon_url),
        font_family: req.font_family.or(current_settings.font_family),
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
