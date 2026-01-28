pub mod auth;
pub mod error;
pub mod metrics;
pub mod routes;
pub mod state;
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
        .route("/auth/login/universal", post(routes::auth::universal_login))
        .route("/auth/validate", get(routes::auth::validate_token))
        .route("/auth/refresh", post(routes::auth::refresh_token))
        .route("/auth/session", get(routes::auth::session_info))
        // OAuth routes
        .route("/auth/google", post(routes::oauth::google_auth))
        .route("/auth/apple", post(routes::oauth::apple_auth))
        // Phone auth routes
        .route("/auth/phone/send-code", post(routes::phone_auth::send_code))
        .route(
            "/auth/phone/verify",
            post(routes::phone_auth::verify_code_endpoint),
        )
        // Wallet auth routes
        .route(
            "/auth/wallet/challenge",
            post(routes::wallet_auth::get_challenge),
        )
        .route(
            "/auth/wallet/verify",
            post(routes::wallet_auth::verify_signature),
        )
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
        .route(
            "/bookings/customer",
            get(routes::bookings::list_customer_bookings),
        )
        .route(
            "/bookings/walker",
            get(routes::bookings::list_walker_bookings),
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
            "/bookings/:id/reschedule",
            post(routes::bookings::reschedule_booking),
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
        .route(
            "/locations/:id",
            put(routes::locations::update_location).delete(routes::locations::delete_location),
        )
        .route(
            "/locations/:id/default",
            put(routes::locations::set_default_location),
        )
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
        // Pet routes
        .route(
            "/pets",
            get(routes::pets::list_pets).post(routes::pets::create_pet),
        )
        .route(
            "/pets/:id",
            get(routes::pets::get_pet)
                .put(routes::pets::update_pet)
                .delete(routes::pets::delete_pet),
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
        .route(
            "/users/me",
            get(routes::users::get_me).put(routes::users::update_me),
        )
        .route("/users/:id", get(routes::users::get_user))
        // User identity management routes
        .route(
            "/users/me/identities",
            get(routes::user_identities::list_identities),
        )
        .route(
            "/users/me/identities/:id",
            delete(routes::user_identities::unlink_identity),
        )
        // Link identity routes
        .route(
            "/users/me/identities/google",
            post(routes::user_identities::link_google),
        )
        .route(
            "/users/me/identities/apple",
            post(routes::user_identities::link_apple),
        )
        .route(
            "/users/me/identities/email",
            post(routes::user_identities::link_email),
        )
        .route(
            "/users/me/password",
            put(routes::user_identities::change_password),
        )
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
        // Route optimization
        .route(
            "/walkers/:id/route",
            get(routes::route_optimization::get_route),
        )
        .route(
            "/walkers/:id/route/optimize",
            post(routes::route_optimization::optimize_route),
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
        // Walker/Client invitation routes
        .route("/walker/invite", post(routes::invitations::invite_walker))
        .route("/client/invite", post(routes::invitations::invite_client))
        .route(
            "/walker/join-tenant",
            post(routes::invitations::join_tenant),
        )
        .route(
            "/walker/create-tenant",
            post(routes::invitations::create_tenant),
        )
        // Invitation management routes
        .route("/invitations", get(routes::invitations::list_invitations))
        .route(
            "/invitations/validate",
            post(routes::invitations::validate_invitation),
        )
        .route(
            "/invitations/accept",
            post(routes::invitations::accept_invitation),
        )
        .route(
            "/invitations/:id",
            delete(routes::invitations::revoke_invitation),
        )
        // Context management routes (multi-membership support)
        .route("/contexts", get(routes::contexts::list_contexts))
        .route("/contexts/switch", post(routes::contexts::switch_context))
        .route("/contexts/clear", post(routes::contexts::clear_context))
        .route(
            "/contexts/default",
            put(routes::contexts::set_default_context),
        )
        .route(
            "/contexts/join-as-customer/:org_slug",
            post(routes::contexts::join_as_customer),
        )
        // Organization deletion (owner-only)
        .route(
            "/contexts/organization",
            delete(routes::contexts::delete_organization),
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
        // Feedback routes (for bug reports and feature requests)
        .route("/feedback", post(routes::feedback::submit_feedback))
        // Payment provider routes (Stripe/Square OAuth connections)
        .route(
            "/payment-providers",
            get(routes::payment_providers::list_payment_providers),
        )
        .route(
            "/payment-providers/primary",
            get(routes::payment_providers::get_primary_provider),
        )
        .route(
            "/payment-providers/stripe/connect",
            get(routes::payment_providers::get_stripe_connect_url),
        )
        .route(
            "/payment-providers/stripe/callback",
            post(routes::payment_providers::stripe_connect_callback),
        )
        .route(
            "/payment-providers/square/connect",
            get(routes::payment_providers::get_square_oauth_url),
        )
        .route(
            "/payment-providers/square/callback",
            post(routes::payment_providers::square_oauth_callback),
        )
        .route(
            "/payment-providers/:id",
            put(routes::payment_providers::update_payment_provider)
                .delete(routes::payment_providers::disconnect_payment_provider),
        )
        // Checkout and transaction routes
        .route("/checkout", post(routes::checkout::create_checkout))
        .route(
            "/checkout/preview-fees",
            post(routes::checkout::preview_fees),
        )
        .route("/checkout/:id", get(routes::checkout::get_checkout))
        .route(
            "/checkout/:id/confirm",
            post(routes::checkout::confirm_payment),
        )
        .route(
            "/checkout/:id/refund",
            post(routes::checkout::request_refund),
        )
        .route("/transactions", get(routes::checkout::list_transactions))
        // Subscription routes
        .route(
            "/subscriptions/tenant",
            get(routes::subscriptions::get_tenant_subscription)
                .post(routes::subscriptions::create_tenant_subscription),
        )
        .route(
            "/subscriptions/tenant/cancel",
            post(routes::subscriptions::cancel_tenant_subscription),
        )
        .route(
            "/subscriptions/tiers",
            get(routes::subscriptions::list_fee_tiers),
        )
        .route(
            "/subscriptions/customer",
            get(routes::subscriptions::list_customer_subscriptions)
                .post(routes::subscriptions::create_customer_subscription),
        )
        .route(
            "/subscriptions/customer/:id",
            get(routes::subscriptions::get_customer_subscription)
                .delete(routes::subscriptions::cancel_customer_subscription),
        )
        // Payout routes
        .route(
            "/payouts/settings",
            get(routes::payouts::get_payout_settings).put(routes::payouts::update_payout_settings),
        )
        .route("/payouts/summary", get(routes::payouts::get_payout_summary))
        .route("/payouts", get(routes::payouts::list_payouts))
        .route(
            "/payouts/instant",
            post(routes::payouts::request_instant_payout),
        )
        .route("/payouts/:id", get(routes::payouts::get_payout))
        // Webhook routes (no auth - verified by signature)
        .route(
            "/webhooks/stripe/:org_id",
            post(routes::webhooks::stripe_webhook),
        )
        .route(
            "/webhooks/square/:org_id",
            post(routes::webhooks::square_webhook),
        )
        // Add middleware
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}
