use axum::{extract::State, http::HeaderMap, Json};
use db::repositories::OrganizationRepository;
use serde::Serialize;

use crate::{error::ApiResult, state::AppState};

#[derive(Debug, Serialize)]
pub struct BrandingResponse {
    pub company_name: String,
    pub logo_url: Option<String>,
    pub primary_color: Option<String>,
    pub secondary_color: Option<String>,
    pub accent_color: Option<String>,
    pub support_email: Option<String>,
}

/// Extract tenant identifier from Host header.
/// Supports both subdomain format (tenant.example.com) and custom domains.
fn extract_tenant_from_host(host: &str) -> Option<String> {
    // Remove port if present (e.g., "tenant.example.com:3000" -> "tenant.example.com")
    let host_without_port = host.split(':').next().unwrap_or(host);

    // Check if it's a subdomain pattern (e.g., "tenant.example.com")
    // We extract the first part as the subdomain
    let parts: Vec<&str> = host_without_port.split('.').collect();

    if parts.len() >= 3 {
        // Has subdomain: "tenant.example.com" -> "tenant"
        Some(parts[0].to_string())
    } else if parts.len() == 2 || parts.len() == 1 {
        // Custom domain or simple domain: return the full domain
        Some(host_without_port.to_string())
    } else {
        None
    }
}

/// Default platform branding when no tenant is found
fn default_branding() -> BrandingResponse {
    BrandingResponse {
        company_name: "OFFLEASH".to_string(),
        logo_url: None,
        primary_color: Some("#3b82f6".to_string()),
        secondary_color: Some("#6b7280".to_string()),
        accent_color: Some("#10b981".to_string()),
        support_email: Some("support@offleash.world".to_string()),
    }
}

/// GET /api/branding - Fetch tenant branding without authentication
/// Returns tenant-specific branding if a valid tenant is found in the Host header,
/// otherwise returns default platform branding.
pub async fn get_branding(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<BrandingResponse>> {
    // Get Host header - if missing, return default branding
    let host = match headers.get("host").and_then(|h| h.to_str().ok()) {
        Some(h) => h,
        None => return Ok(Json(default_branding())),
    };

    // Extract tenant identifier from host
    let tenant_identifier = match extract_tenant_from_host(host) {
        Some(t) => t,
        None => return Ok(Json(default_branding())),
    };

    // Try to find organization by subdomain first
    let org = OrganizationRepository::find_by_subdomain(&state.pool, &tenant_identifier).await?;

    // If not found by subdomain, try by custom domain (using the full host without port)
    let org = match org {
        Some(org) => org,
        None => {
            let host_without_port = host.split(':').next().unwrap_or(host);
            match OrganizationRepository::find_by_custom_domain(&state.pool, host_without_port)
                .await?
            {
                Some(org) => org,
                // No tenant found - return default platform branding
                None => return Ok(Json(default_branding())),
            }
        }
    };

    let settings = org.settings.0;

    Ok(Json(BrandingResponse {
        company_name: org.name,
        logo_url: settings.logo_url,
        primary_color: settings.primary_color,
        secondary_color: settings.secondary_color,
        accent_color: None,  // Not in current schema
        support_email: None, // Not in current schema
    }))
}
