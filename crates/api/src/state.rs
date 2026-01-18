use db::TenantPoolManager;
use integrations::GoogleMapsClient;
use sqlx::PgPool;
use std::sync::Arc;

/// Application state shared across all handlers
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub jwt_secret: String,
    pub google_maps: Option<Arc<GoogleMapsClient>>,
    pub tenant_pool_manager: Arc<TenantPoolManager>,
}

impl AppState {
    pub fn new(pool: PgPool, jwt_secret: String, google_maps_key: Option<String>) -> Self {
        let google_maps = google_maps_key.map(|key| Arc::new(GoogleMapsClient::new(key)));
        let tenant_pool_manager = Arc::new(TenantPoolManager::new(pool.clone()));

        Self {
            pool,
            jwt_secret,
            google_maps,
            tenant_pool_manager,
        }
    }
}
