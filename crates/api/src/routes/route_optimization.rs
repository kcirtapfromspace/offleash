//! Route optimization API endpoints
//!
//! Provides endpoints for walkers to view and optimize their daily routes.

use axum::{
    extract::{Path, State},
    Json,
};
use chrono::{Duration, NaiveDate};
use db::{BookingRepository, LocationRepository, TravelTimeCacheRepository, UserRepository};
use domain::{RouteBooking, RouteOptimizer, TrafficConfig, TravelTimeMatrix};
use serde::{Deserialize, Serialize};
use shared::{types::DurationMinutes, AppError, DomainError};

use crate::{
    auth::TenantContext,
    error::{ApiError, ApiResult},
    state::AppState,
};

/// Query parameters for route endpoints
#[derive(Debug, Deserialize)]
pub struct RouteQuery {
    pub date: String, // YYYY-MM-DD
}

/// Response for a single route stop
#[derive(Debug, Serialize)]
pub struct RouteStopResponse {
    pub sequence: usize,
    pub booking_id: String,
    pub customer_name: String,
    pub address: String,
    pub arrival_time: String,
    pub departure_time: String,
    pub travel_from_previous_minutes: i32,
    pub service_duration_minutes: i32,
}

/// Response for the optimized route
#[derive(Debug, Serialize)]
pub struct OptimizedRouteResponse {
    pub date: String,
    pub is_optimized: bool,
    pub stops: Vec<RouteStopResponse>,
    pub total_travel_minutes: i32,
    pub total_distance_meters: i32,
    pub savings_minutes: i32,
}

/// GET /walkers/:id/route?date=YYYY-MM-DD
///
/// Get the optimized route for a walker on a specific date.
pub async fn get_route(
    State(_state): State<AppState>,
    tenant: TenantContext,
    Path(walker_id): Path<String>,
    axum::extract::Query(query): axum::extract::Query<RouteQuery>,
) -> ApiResult<Json<OptimizedRouteResponse>> {
    // Parse walker ID
    let walker_id_parsed = walker_id
        .parse()
        .map_err(|_| ApiError::from(AppError::Validation("Invalid walker ID".to_string())))?;

    // Verify walker exists
    let walker = UserRepository::find_by_id(&tenant.pool, tenant.org_id, walker_id_parsed)
        .await?
        .ok_or_else(|| ApiError::from(DomainError::WalkerNotFound(walker_id.clone())))?;

    if !walker.is_walker() {
        return Err(ApiError::from(DomainError::WalkerNotFound(walker_id)));
    }

    // Parse date
    let date = NaiveDate::parse_from_str(&query.date, "%Y-%m-%d")
        .map_err(|_| ApiError::from(AppError::Validation("Invalid date format".to_string())))?;

    // Get bookings for the date
    let start_of_day = date.and_hms_opt(0, 0, 0).unwrap().and_utc();
    let end_of_day = start_of_day + Duration::days(1);

    let bookings = BookingRepository::find_by_walker_in_range(
        &tenant.pool,
        tenant.org_id,
        walker_id_parsed,
        start_of_day,
        end_of_day,
    )
    .await?;

    // Filter to only confirmed/in_progress bookings
    let active_bookings: Vec<_> = bookings.into_iter().filter(|b| b.is_active()).collect();

    if active_bookings.is_empty() {
        return Ok(Json(OptimizedRouteResponse {
            date: query.date,
            is_optimized: false,
            stops: vec![],
            total_travel_minutes: 0,
            total_distance_meters: 0,
            savings_minutes: 0,
        }));
    }

    // Fetch location details for each booking
    let mut route_bookings = Vec::with_capacity(active_bookings.len());
    for booking in &active_bookings {
        let location =
            LocationRepository::find_by_id(&tenant.pool, tenant.org_id, booking.location_id)
                .await?
                .ok_or_else(|| {
                    ApiError::from(DomainError::LocationNotFound(
                        booking.location_id.to_string(),
                    ))
                })?;

        // Get customer name
        let customer =
            UserRepository::find_by_id(&tenant.pool, tenant.org_id, booking.customer_id).await?;
        let customer_name = customer
            .map(|c| {
                format!(
                    "{} {}",
                    c.first_name,
                    c.last_name.chars().next().unwrap_or('.')
                )
            })
            .unwrap_or_else(|| "Unknown".to_string());

        route_bookings.push(RouteBooking {
            booking_id: booking.id,
            location_id: booking.location_id,
            customer_name,
            address: location.full_address(),
            scheduled_start: booking.scheduled_start,
            scheduled_end: booking.scheduled_end,
            latitude: location.latitude,
            longitude: location.longitude,
        });
    }

    // Build travel time matrix
    let traffic_config = TrafficConfig::default();
    let mut travel_matrix = TravelTimeMatrix::new();

    let location_ids: Vec<_> = route_bookings.iter().map(|b| b.location_id).collect();
    let cache_ttl = traffic_config.get_cache_ttl_minutes(start_of_day);
    let cached_times =
        TravelTimeCacheRepository::get_batch_fresh(&tenant.pool, &location_ids, cache_ttl).await?;

    for cache_entry in cached_times {
        let base_minutes = cache_entry.travel_minutes();
        let adjusted_minutes =
            traffic_config.adjust_travel_time(base_minutes, start_of_day + Duration::hours(12));

        travel_matrix.insert(
            cache_entry.origin_location_id,
            cache_entry.destination_location_id,
            DurationMinutes::new(adjusted_minutes),
        );
    }

    // Optimize route
    let optimizer = RouteOptimizer::new();
    let optimized = optimizer.optimize(route_bookings, &travel_matrix);

    // Build response
    let stops: Vec<RouteStopResponse> = optimized
        .stops
        .iter()
        .map(|stop| RouteStopResponse {
            sequence: stop.sequence,
            booking_id: stop.booking_id.to_string(),
            customer_name: stop.customer_name.clone(),
            address: stop.address.clone(),
            arrival_time: stop.arrival_time.to_rfc3339(),
            departure_time: stop.departure_time.to_rfc3339(),
            travel_from_previous_minutes: stop.travel_from_previous_minutes,
            service_duration_minutes: stop.service_duration_minutes,
        })
        .collect();

    Ok(Json(OptimizedRouteResponse {
        date: query.date,
        is_optimized: optimized.is_optimized,
        stops,
        total_travel_minutes: optimized.total_travel_minutes,
        total_distance_meters: optimized.total_distance_meters,
        savings_minutes: optimized.savings_vs_chronological,
    }))
}

/// POST /walkers/:id/route/optimize?date=YYYY-MM-DD
///
/// Trigger re-optimization of a walker's route for a specific date.
/// This is the same as get_route but explicitly signals an optimization request.
pub async fn optimize_route(
    state: State<AppState>,
    tenant: TenantContext,
    path: Path<String>,
    query: axum::extract::Query<RouteQuery>,
) -> ApiResult<Json<OptimizedRouteResponse>> {
    // Optimization is always done on-the-fly, so this is equivalent to get_route
    get_route(state, tenant, path, query).await
}
