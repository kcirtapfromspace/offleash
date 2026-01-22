pub mod auth;
pub mod error;
pub mod metrics;
pub mod routes;
pub mod state;
pub mod supabase_auth;
pub mod tenant;

pub use error::ApiError;
pub use metrics::init_metrics;
pub use state::AppState;

use axum::{
    routing::{delete, get, post, put},
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
        // Prometheus metrics
        .route("/metrics", get(routes::prometheus::metrics))
        // Public branding endpoint (no auth required)
        .route("/api/branding", get(routes::branding::get_branding))
        // Auth routes
        .route("/auth/register", post(routes::auth::register))
        .route("/auth/login", post(routes::auth::login))
        .route("/auth/validate", get(routes::auth::validate_token))
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
        .route("/bookings/customer", get(routes::bookings::list_customer_bookings))
        .route("/bookings/walker", get(routes::bookings::list_walker_bookings))
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
        // Recurring booking routes
        .route(
            "/bookings/recurring",
            get(routes::recurring_bookings::list_recurring_bookings)
                .post(routes::recurring_bookings::create_recurring_booking),
        )
        .route(
            "/bookings/recurring/:id",
            get(routes::recurring_bookings::get_recurring_booking),
        )
        .route(
            "/bookings/recurring/:id/cancel",
            post(routes::recurring_bookings::cancel_recurring_series),
        )
        // Location routes
        .route("/locations", post(routes::locations::create_location))
        .route("/locations", get(routes::locations::list_locations))
        .route("/locations/:id", delete(routes::locations::delete_location))
        .route("/locations/:id/default", put(routes::locations::set_default_location))
        // Payment method routes
        .route(
            "/payment-methods",
            get(routes::payment_methods::list_payment_methods)
                .post(routes::payment_methods::create_payment_method),
        )
        .route(
            "/payment-methods/:id",
            delete(routes::payment_methods::delete_payment_method)
                .patch(routes::payment_methods::update_payment_method),
        )
        .route(
            "/payment-methods/:id/default",
            put(routes::payment_methods::set_default_payment_method),
        )
        // Block routes
        .route(
            "/blocks",
            get(routes::blocks::list_blocks).post(routes::blocks::create_block),
        )
        .route(
            "/blocks/:id",
            axum::routing::delete(routes::blocks::delete_block),
        )
        // User routes
        .route("/users", get(routes::users::list_users))
        .route("/users/me", get(routes::users::get_me).put(routes::users::update_me))
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
        // Calendar routes
        .route(
            "/calendar/events",
            get(routes::calendar::list_events).post(routes::calendar::create_event),
        )
        .route(
            "/calendar/events/:id",
            get(routes::calendar::get_event)
                .put(routes::calendar::update_event)
                .delete(routes::calendar::delete_event),
        )
        // Travel time and location routes
        .route(
            "/walkers/:id/location",
            get(routes::travel_time::get_walker_location)
                .post(routes::travel_time::update_walker_location),
        )
        .route(
            "/walkers/:id/on-duty",
            post(routes::travel_time::set_walker_duty_status),
        )
        .route("/travel-time", get(routes::travel_time::get_travel_time))
        .route(
            "/availability/slots",
            get(routes::travel_time::get_availability_slots),
        )
        // Walker profile routes
        .route(
            "/walker/profile",
            get(routes::walker_profiles::get_my_profile)
                .put(routes::walker_profiles::update_my_profile),
        )
        .route(
            "/walker/specializations",
            get(routes::walker_profiles::list_specializations),
        )
        .route(
            "/admin/walkers/:walker_id/profile",
            get(routes::walker_profiles::get_walker_profile)
                .put(routes::walker_profiles::update_walker_profile),
        )
        // Service area routes (walker self-service)
        .route(
            "/walker/service-areas",
            get(routes::service_areas::get_my_service_areas)
                .post(routes::service_areas::create_my_service_area),
        )
        .route(
            "/walker/service-areas/:area_id",
            put(routes::service_areas::update_my_service_area)
                .delete(routes::service_areas::delete_my_service_area),
        )
        // Service area routes (admin)
        .route(
            "/admin/service-areas",
            get(routes::service_areas::list_all_service_areas),
        )
        .route(
            "/admin/walkers/:walker_id/service-areas",
            get(routes::service_areas::get_walker_service_areas)
                .post(routes::service_areas::create_walker_service_area),
        )
        .route(
            "/admin/service-areas/:area_id",
            put(routes::service_areas::update_service_area)
                .delete(routes::service_areas::delete_service_area),
        )
        // Add middleware
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}
