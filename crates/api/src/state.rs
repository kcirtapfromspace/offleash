use db::TenantPoolManager;
use integrations::GoogleMapsClient;
use metrics_exporter_prometheus::PrometheusHandle;
use sqlx::PgPool;
use std::sync::Arc;

/// Configuration for external services
#[derive(Clone, Default)]
pub struct AppConfig {
    /// GitHub personal access token for creating issues
    pub github_token: Option<String>,
    /// GitHub repository for feedback issues (format: "owner/repo")
    pub github_feedback_repo: Option<String>,
}

/// Application state shared across all handlers
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub jwt_secret: String,
    pub google_maps: Option<Arc<GoogleMapsClient>>,
    pub tenant_pool_manager: Arc<TenantPoolManager>,
    pub metrics_handle: PrometheusHandle,
    pub config: AppConfig,
}

impl AppState {
    pub fn new(
        pool: PgPool,
        jwt_secret: String,
        google_maps_key: Option<String>,
        metrics_handle: PrometheusHandle,
    ) -> Self {
        Self::with_config(
            pool,
            jwt_secret,
            google_maps_key,
            metrics_handle,
            AppConfig::default(),
        )
    }

    pub fn with_config(
        pool: PgPool,
        jwt_secret: String,
        google_maps_key: Option<String>,
        metrics_handle: PrometheusHandle,
        config: AppConfig,
    ) -> Self {
        let google_maps = google_maps_key.map(|key| Arc::new(GoogleMapsClient::new(key)));
        let tenant_pool_manager = Arc::new(TenantPoolManager::new(pool.clone()));

        Self {
            pool,
            jwt_secret,
            google_maps,
            tenant_pool_manager,
            metrics_handle,
            config,
        }
    }
}
