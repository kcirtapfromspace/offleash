use db::TenantPoolManager;
use integrations::GoogleMapsClient;
use metrics_exporter_prometheus::PrometheusHandle;
use sqlx::PgPool;
use std::sync::Arc;

/// Application state shared across all handlers
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub jwt_secret: String,
    /// Supabase JWT secret for verifying OAuth tokens (optional, enables Supabase Auth)
    pub supabase_jwt_secret: Option<String>,
    pub google_maps: Option<Arc<GoogleMapsClient>>,
    pub tenant_pool_manager: Arc<TenantPoolManager>,
    pub metrics_handle: PrometheusHandle,
}

impl AppState {
    pub fn new(
        pool: PgPool,
        jwt_secret: String,
        google_maps_key: Option<String>,
        metrics_handle: PrometheusHandle,
    ) -> Self {
        Self::with_supabase(pool, jwt_secret, None, google_maps_key, metrics_handle)
    }

    pub fn with_supabase(
        pool: PgPool,
        jwt_secret: String,
        supabase_jwt_secret: Option<String>,
        google_maps_key: Option<String>,
        metrics_handle: PrometheusHandle,
    ) -> Self {
        let google_maps = google_maps_key.map(|key| Arc::new(GoogleMapsClient::new(key)));
        let tenant_pool_manager = Arc::new(TenantPoolManager::new(pool.clone()));

        Self {
            pool,
            jwt_secret,
            supabase_jwt_secret,
            google_maps,
            tenant_pool_manager,
            metrics_handle,
        }
    }
}
