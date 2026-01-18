//! Tenant connection pool manager for multi-tenant database routing.
//!
//! This module provides the `TenantPoolManager` struct which manages database
//! connection pools for each tenant organization, enabling lazy initialization
//! and caching of connections.

use shared::types::OrganizationId;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::models::TenantDbStatus;
use crate::repositories::TenantDatabaseRepository;

/// Error type for tenant pool operations
#[derive(Debug, thiserror::Error)]
pub enum TenantPoolError {
    #[error("Tenant database not found for organization: {0}")]
    TenantNotFound(OrganizationId),

    #[error("Tenant database is not active for organization: {0}")]
    TenantNotActive(OrganizationId),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}

/// Manages connection pools for tenant databases.
///
/// The `TenantPoolManager` caches connection pools in a `HashMap` keyed by
/// `OrganizationId`. Pools are created lazily on first access and reused
/// for subsequent requests.
pub struct TenantPoolManager {
    /// The master database pool used to look up tenant connection strings
    master_pool: PgPool,
    /// Cache of tenant connection pools
    pools: Arc<RwLock<HashMap<OrganizationId, PgPool>>>,
    /// Maximum connections per tenant pool
    max_connections: u32,
}

impl TenantPoolManager {
    /// Creates a new `TenantPoolManager` with the given master database pool.
    ///
    /// # Arguments
    ///
    /// * `master_pool` - The master database pool for looking up tenant configurations
    pub fn new(master_pool: PgPool) -> Self {
        Self {
            master_pool,
            pools: Arc::new(RwLock::new(HashMap::new())),
            max_connections: 5,
        }
    }

    /// Creates a new `TenantPoolManager` with a custom max connections setting.
    ///
    /// # Arguments
    ///
    /// * `master_pool` - The master database pool for looking up tenant configurations
    /// * `max_connections` - Maximum number of connections per tenant pool
    pub fn with_max_connections(master_pool: PgPool, max_connections: u32) -> Self {
        Self {
            master_pool,
            pools: Arc::new(RwLock::new(HashMap::new())),
            max_connections,
        }
    }

    /// Gets or creates a connection pool for the specified organization.
    ///
    /// This method first checks the cache for an existing pool. If not found,
    /// it looks up the tenant database configuration in the master database
    /// and creates a new connection pool.
    ///
    /// # Arguments
    ///
    /// * `org_id` - The organization ID to get the pool for
    ///
    /// # Returns
    ///
    /// Returns the `PgPool` for the tenant database, or an error if the
    /// tenant is not found or not active.
    ///
    /// # Errors
    ///
    /// * `TenantPoolError::TenantNotFound` - If no tenant database is configured
    /// * `TenantPoolError::TenantNotActive` - If the tenant database is not active
    /// * `TenantPoolError::Database` - If a database error occurs
    pub async fn get_pool(&self, org_id: OrganizationId) -> Result<PgPool, TenantPoolError> {
        // First, try to get from cache with a read lock
        {
            let pools = self.pools.read().await;
            if let Some(pool) = pools.get(&org_id) {
                return Ok(pool.clone());
            }
        }

        // Not in cache, need to create a new pool
        // Use write lock for the creation
        let mut pools = self.pools.write().await;

        // Double-check after acquiring write lock (another task might have created it)
        if let Some(pool) = pools.get(&org_id) {
            return Ok(pool.clone());
        }

        // Look up tenant database configuration
        let tenant_db = TenantDatabaseRepository::find_by_org_id(&self.master_pool, org_id)
            .await?
            .ok_or(TenantPoolError::TenantNotFound(org_id))?;

        // Verify tenant is active
        if tenant_db.status != TenantDbStatus::Active {
            return Err(TenantPoolError::TenantNotActive(org_id));
        }

        // Create new connection pool
        let pool = PgPoolOptions::new()
            .max_connections(self.max_connections)
            .connect(&tenant_db.connection_string)
            .await?;

        // Cache the pool
        pools.insert(org_id, pool.clone());

        Ok(pool)
    }

    /// Removes a pool from the cache, closing all its connections.
    ///
    /// This is useful when a tenant database configuration changes or
    /// when you need to force reconnection.
    ///
    /// # Arguments
    ///
    /// * `org_id` - The organization ID whose pool should be removed
    pub async fn remove_pool(&self, org_id: &OrganizationId) {
        let mut pools = self.pools.write().await;
        if let Some(pool) = pools.remove(org_id) {
            pool.close().await;
        }
    }

    /// Returns the number of cached pools.
    pub async fn pool_count(&self) -> usize {
        self.pools.read().await.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Integration tests would require a running database
    // These are placeholder tests for the module structure

    #[test]
    fn test_tenant_pool_error_display() {
        let org_id = OrganizationId::new();
        let error = TenantPoolError::TenantNotFound(org_id);
        assert!(error.to_string().contains("not found"));

        let error = TenantPoolError::TenantNotActive(org_id);
        assert!(error.to_string().contains("not active"));
    }
}
