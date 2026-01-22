use std::net::SocketAddr;

use api::{create_app, init_metrics, AppState};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    // Load environment variables
    dotenvy::dotenv().ok();

    // Check for --migrate-only flag
    let migrate_only = std::env::args().any(|arg| arg == "--migrate-only");

    // Initialize Prometheus metrics
    let metrics_handle = init_metrics();
    tracing::info!("Prometheus metrics initialized");

    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "api=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Get configuration from environment
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // Create database pool
    let pool = db::create_pool(&database_url)
        .await
        .expect("Failed to create database pool");

    // Run migrations
    tracing::info!("Running database migrations...");
    sqlx::migrate!("../../migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    tracing::info!("Database migrations complete");

    // Exit early if only running migrations
    if migrate_only {
        tracing::info!("Migration-only mode, exiting");
        return;
    }

    // Get remaining configuration
    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let google_maps_key = std::env::var("GOOGLE_MAPS_API_KEY").ok();

    // Create app state
    let state = AppState::new(pool, jwt_secret, google_maps_key, metrics_handle);

    // Create the app
    let app = create_app(state);

    // Get port from environment or default to 8080
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Listening on {}", addr);

    // Start server
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
