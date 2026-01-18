pub mod models;
pub mod repositories;
pub mod tenant_pool;

pub use models::*;
pub use repositories::*;
pub use tenant_pool::*;

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

/// Create a database connection pool
pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await
}
