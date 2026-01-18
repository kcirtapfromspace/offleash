pub mod models;
pub mod repositories;

pub use models::*;
pub use repositories::*;

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

/// Create a database connection pool
pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await
}
