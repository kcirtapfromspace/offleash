use axum::{extract::State, http::HeaderMap, Json};
use db::repositories::OrganizationRepository;
use serde::Serialize;
use shared::DomainError;

use crate::{error::ApiResult, state::AppState, ApiError};

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

/// GET /api/branding - Fetch tenant branding without authentication
pub async fn get_branding(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<BrandingResponse>> {
    // Get Host header
    let host = headers
        .get("host")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| {
            ApiError::from(DomainError::TenantNotFound(
                "missing host header".to_string(),
            ))
        })?;

    let tenant_identifier = extract_tenant_from_host(host)
        .ok_or_else(|| ApiError::from(DomainError::TenantNotFound(host.to_string())))?;

    // Try to find organization by subdomain first
    let org = OrganizationRepository::find_by_subdomain(&state.pool, &tenant_identifier).await?;

    // If not found by subdomain, try by custom domain (using the full host without port)
    let org = match org {
        Some(org) => org,
        None => {
            let host_without_port = host.split(':').next().unwrap_or(host);
            OrganizationRepository::find_by_custom_domain(&state.pool, host_without_port)
                .await?
                .ok_or_else(|| ApiError::from(DomainError::TenantNotFound(host.to_string())))?
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
