use db::{models::Organization, repositories::OrganizationRepository};
use sqlx::PgPool;

/// Extract subdomain from Host header.
/// Returns the subdomain part if present (e.g., "acme.offleash.app" -> "acme").
fn extract_subdomain_from_host(host: &str) -> Option<&str> {
    // Remove port if present (e.g., "acme.offleash.app:3000" -> "acme.offleash.app")
    let host_without_port = host.split(':').next().unwrap_or(host);

    // Check if it's a subdomain pattern (e.g., "acme.offleash.app")
    let parts: Vec<&str> = host_without_port.split('.').collect();

    // Need at least 3 parts for subdomain (subdomain.domain.tld)
    if parts.len() >= 3 {
        Some(parts[0])
    } else {
        None
    }
}

/// Extract tenant organization from Host header by parsing subdomain and looking up by slug.
///
/// Parses the subdomain from the Host header (e.g., "acme.offleash.app" -> "acme")
/// and looks up the organization by slug.
///
/// Returns `None` if:
/// - No subdomain is present
/// - No organization matches the slug
pub async fn extract_tenant_from_host(
    pool: &PgPool,
    host: &str,
) -> Result<Option<Organization>, sqlx::Error> {
    let subdomain = match extract_subdomain_from_host(host) {
        Some(s) => s,
        None => return Ok(None),
    };

    // Lookup organization by slug (subdomain is used as the slug)
    OrganizationRepository::find_by_slug(pool, subdomain).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_subdomain_from_host() {
        // Standard subdomain pattern
        assert_eq!(
            extract_subdomain_from_host("acme.offleash.app"),
            Some("acme")
        );

        // With port
        assert_eq!(
            extract_subdomain_from_host("acme.offleash.app:3000"),
            Some("acme")
        );

        // Deep subdomain (still extracts first part)
        assert_eq!(
            extract_subdomain_from_host("acme.staging.offleash.app"),
            Some("acme")
        );

        // No subdomain (just domain.tld)
        assert_eq!(extract_subdomain_from_host("offleash.app"), None);

        // Single part (localhost)
        assert_eq!(extract_subdomain_from_host("localhost"), None);

        // With port, no subdomain
        assert_eq!(extract_subdomain_from_host("localhost:3000"), None);
    }
}
