pub mod auth;
pub mod error;
pub mod routes;
pub mod state;
pub mod tenant;

pub use error::ApiError;
pub use state::AppState;

use axum::{
    routing::{get, post},
    Router,
};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

/// Create the application router
pub fn create_app(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        // Health check
        .route("/health", get(routes::health::health_check))
        // Public branding endpoint (no auth required)
        .route("/api/branding", get(routes::branding::get_branding))
        // Auth routes
        .route("/auth/register", post(routes::auth::register))
        .route("/auth/login", post(routes::auth::login))
        // Platform admin auth routes
        .route(
            "/platform/auth/login",
            post(routes::platform_auth::platform_login),
        )
        // Admin tenant management routes (platform admin only)
        .route(
            "/admin/tenants",
            get(routes::admin_tenants::list_tenants).post(routes::admin_tenants::create_tenant),
        )
        .route(
            "/admin/tenants/:id",
            get(routes::admin_tenants::get_tenant)
                .patch(routes::admin_tenants::update_tenant)
                .delete(routes::admin_tenants::delete_tenant),
        )
        // Tenant admin branding management routes
        .route(
            "/admin/branding",
            get(routes::admin_branding::get_branding).put(routes::admin_branding::update_branding),
        )
        // Dashboard metrics route (tenant admin)
        .route(
            "/admin/dashboard/metrics",
            get(routes::dashboard::get_metrics),
        )
        // Service routes
        .route(
            "/services",
            get(routes::services::list_services).post(routes::services::create_service),
        )
        .route(
            "/services/:id",
            get(routes::services::get_service).patch(routes::services::update_service),
        )
        // Availability routes
        .route(
            "/availability/:walker_id",
            get(routes::availability::get_availability),
        )
        // Booking routes
        .route(
            "/bookings",
            get(routes::bookings::list_bookings).post(routes::bookings::create_booking),
        )
        .route("/bookings/:id", get(routes::bookings::get_booking))
        .route(
            "/bookings/:id/confirm",
            post(routes::bookings::confirm_booking),
        )
        .route(
            "/bookings/:id/cancel",
            post(routes::bookings::cancel_booking),
        )
        .route(
            "/bookings/:id/complete",
            post(routes::bookings::complete_booking),
        )
        // Location routes
        .route("/locations", post(routes::locations::create_location))
        .route("/locations", get(routes::locations::list_locations))
        // Block routes
        .route("/blocks", post(routes::blocks::create_block))
        .route(
            "/blocks/:id",
            axum::routing::delete(routes::blocks::delete_block),
        )
        // User routes
        .route("/users", get(routes::users::list_users))
        .route("/users/:id", get(routes::users::get_user))
        // Admin user management routes
        .route("/admin/walkers", post(routes::admin_users::create_walker))
        // Working hours routes
        .route(
            "/working-hours/:walker_id",
            get(routes::working_hours::get_walker_hours)
                .put(routes::working_hours::update_walker_hours)
                .delete(routes::working_hours::delete_walker_hours),
        )
        // Add middleware
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}
