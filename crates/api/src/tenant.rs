use db::{models::Organization, repositories::OrganizationRepository};
use sqlx::PgPool;

/// Extract host without port from Host header.
/// E.g., "acme.offleash.app:3000" -> "acme.offleash.app"
fn extract_host_without_port(host: &str) -> &str {
    host.split(':').next().unwrap_or(host)
}

/// Extract subdomain from Host header.
/// Returns the subdomain part if present (e.g., "acme.offleash.app" -> "acme").
fn extract_subdomain_from_host(host: &str) -> Option<&str> {
    let host_without_port = extract_host_without_port(host);

    // Check if it's a subdomain pattern (e.g., "acme.offleash.app")
    let parts: Vec<&str> = host_without_port.split('.').collect();

    // Need at least 3 parts for subdomain (subdomain.domain.tld)
    if parts.len() >= 3 {
        Some(parts[0])
    } else {
        None
    }
}

/// Extract tenant organization from Host header.
///
/// Resolution order (custom domain takes precedence):
/// 1. Look up by custom_domain (full host without port)
/// 2. If not found, extract subdomain and look up by subdomain
///
/// Returns `None` if:
/// - No organization matches the custom domain
/// - No subdomain is present and no custom domain match
/// - No organization matches the subdomain
pub async fn extract_tenant_from_host(
    pool: &PgPool,
    host: &str,
) -> Result<Option<Organization>, sqlx::Error> {
    let host_without_port = extract_host_without_port(host);

    // First, try to find by custom domain (takes precedence)
    if let Some(org) =
        OrganizationRepository::find_by_custom_domain(pool, host_without_port).await?
    {
        return Ok(Some(org));
    }

    // Fall back to subdomain lookup
    let subdomain = match extract_subdomain_from_host(host) {
        Some(s) => s,
        None => return Ok(None),
    };

    // Lookup organization by subdomain
    OrganizationRepository::find_by_subdomain(pool, subdomain).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_host_without_port() {
        assert_eq!(extract_host_without_port("example.com"), "example.com");
        assert_eq!(extract_host_without_port("example.com:3000"), "example.com");
        assert_eq!(extract_host_without_port("localhost:8080"), "localhost");
        assert_eq!(extract_host_without_port("localhost"), "localhost");
        assert_eq!(
            extract_host_without_port("acme.offleash.app:443"),
            "acme.offleash.app"
        );
    }

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
