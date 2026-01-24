use axum::{
    extract::{Path, Query, State},
    Json,
};
use chrono::{Datelike, Duration, Utc};
use db::{
    BookingRepository, LocationRepository, TravelTimeCacheRepository, WalkerLocationRepository,
};
use serde::{Deserialize, Serialize};
use shared::{types::Coordinates, AppError};

use crate::{
    auth::TenantContext,
    error::{ApiError, ApiResult},
    state::AppState,
};

// ============================================================================
// Walker Location Endpoints
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct UpdateLocationRequest {
    pub latitude: f64,
    pub longitude: f64,
    pub accuracy_meters: Option<f64>,
    pub heading: Option<f64>,
    pub speed_mps: Option<f64>,
    pub is_on_duty: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct LocationResponse {
    pub walker_id: String,
    pub latitude: f64,
    pub longitude: f64,
    pub accuracy_meters: Option<f64>,
    pub is_on_duty: bool,
    pub updated_at: String,
    pub is_stale: bool,
}

/// POST /walkers/:id/location - Update walker's live location (from iOS app)
pub async fn update_walker_location(
    State(_state): State<AppState>,
    tenant: TenantContext,
    Path(walker_id): Path<String>,
    Json(req): Json<UpdateLocationRequest>,
) -> ApiResult<Json<LocationResponse>> {
    let walker_id = walker_id
        .parse()
        .map_err(|_| ApiError::from(AppError::Validation("Invalid walker ID".to_string())))?;

    // Validate coordinates
    if req.latitude < -90.0 || req.latitude > 90.0 {
        return Err(ApiError::from(AppError::Validation(
            "Invalid latitude".to_string(),
        )));
    }
    if req.longitude < -180.0 || req.longitude > 180.0 {
        return Err(ApiError::from(AppError::Validation(
            "Invalid longitude".to_string(),
        )));
    }

    let update = db::models::WalkerLocationUpdate {
        latitude: req.latitude,
        longitude: req.longitude,
        accuracy_meters: req.accuracy_meters,
        heading: req.heading,
        speed_mps: req.speed_mps,
        is_on_duty: req.is_on_duty.unwrap_or(true),
    };

    let location = WalkerLocationRepository::upsert(&tenant.pool, walker_id, &update).await?;

    Ok(Json(LocationResponse {
        walker_id: location.walker_id.to_string(),
        latitude: location.latitude,
        longitude: location.longitude,
        accuracy_meters: location.accuracy_meters,
        is_on_duty: location.is_on_duty,
        updated_at: location.updated_at.to_rfc3339(),
        is_stale: location.is_stale(30), // 30 minute threshold
    }))
}

/// GET /walkers/:id/location - Get walker's current location
pub async fn get_walker_location(
    State(_state): State<AppState>,
    tenant: TenantContext,
    Path(walker_id): Path<String>,
) -> ApiResult<Json<Option<LocationResponse>>> {
    let walker_id = walker_id
        .parse()
        .map_err(|_| ApiError::from(AppError::Validation("Invalid walker ID".to_string())))?;

    let location = WalkerLocationRepository::get(&tenant.pool, walker_id).await?;

    Ok(Json(location.map(|loc| LocationResponse {
        walker_id: loc.walker_id.to_string(),
        latitude: loc.latitude,
        longitude: loc.longitude,
        accuracy_meters: loc.accuracy_meters,
        is_on_duty: loc.is_on_duty,
        updated_at: loc.updated_at.to_rfc3339(),
        is_stale: loc.is_stale(30),
    })))
}

/// POST /walkers/:id/on-duty - Toggle walker on/off duty
pub async fn set_walker_duty_status(
    State(_state): State<AppState>,
    tenant: TenantContext,
    Path(walker_id): Path<String>,
    Json(req): Json<SetDutyRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let walker_id = walker_id
        .parse()
        .map_err(|_| ApiError::from(AppError::Validation("Invalid walker ID".to_string())))?;

    WalkerLocationRepository::set_on_duty(&tenant.pool, walker_id, req.is_on_duty).await?;

    Ok(Json(serde_json::json!({
        "is_on_duty": req.is_on_duty
    })))
}

#[derive(Debug, Deserialize)]
pub struct SetDutyRequest {
    pub is_on_duty: bool,
}

// ============================================================================
// Travel Time Endpoints
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct TravelTimeQuery {
    pub origin_location_id: Option<String>,
    pub origin_lat: Option<f64>,
    pub origin_lng: Option<f64>,
    pub destination_location_id: String,
}

#[derive(Debug, Serialize)]
pub struct TravelTimeResponse {
    pub travel_minutes: i32,
    pub distance_meters: i32,
    pub is_cached: bool,
    pub calculated_at: String,
}

/// GET /travel-time - Calculate travel time between two points
pub async fn get_travel_time(
    State(state): State<AppState>,
    tenant: TenantContext,
    Query(query): Query<TravelTimeQuery>,
) -> ApiResult<Json<TravelTimeResponse>> {
    let dest_location_id = query.destination_location_id.parse().map_err(|_| {
        ApiError::from(AppError::Validation(
            "Invalid destination location ID".to_string(),
        ))
    })?;

    // Get destination location coordinates
    let dest_location =
        LocationRepository::find_by_id(&tenant.pool, tenant.org_id, dest_location_id)
            .await?
            .ok_or_else(|| {
                ApiError::from(AppError::Validation(
                    "Destination location not found".to_string(),
                ))
            })?;

    let dest_coords =
        Coordinates::new(dest_location.latitude, dest_location.longitude).map_err(|_| {
            ApiError::from(AppError::Validation(
                "Invalid destination coordinates".to_string(),
            ))
        })?;

    // Determine origin coordinates
    let (origin_coords, origin_location_id) = if let Some(origin_id) = &query.origin_location_id {
        let origin_id = origin_id.parse().map_err(|_| {
            ApiError::from(AppError::Validation(
                "Invalid origin location ID".to_string(),
            ))
        })?;

        // Check cache first
        if let Some(cached) = TravelTimeCacheRepository::get_if_fresh(
            &tenant.pool,
            origin_id,
            dest_location_id,
            15, // 15 minute cache TTL
        )
        .await?
        {
            return Ok(Json(TravelTimeResponse {
                travel_minutes: cached.travel_minutes(),
                distance_meters: cached.distance_meters,
                is_cached: true,
                calculated_at: cached.calculated_at.to_rfc3339(),
            }));
        }

        let origin_location =
            LocationRepository::find_by_id(&tenant.pool, tenant.org_id, origin_id)
                .await?
                .ok_or_else(|| {
                    ApiError::from(AppError::Validation(
                        "Origin location not found".to_string(),
                    ))
                })?;

        let coords = Coordinates::new(origin_location.latitude, origin_location.longitude)
            .map_err(|_| {
                ApiError::from(AppError::Validation(
                    "Invalid origin coordinates".to_string(),
                ))
            })?;

        (coords, Some(origin_id))
    } else if let (Some(lat), Some(lng)) = (query.origin_lat, query.origin_lng) {
        let coords = Coordinates::new(lat, lng).map_err(|_| {
            ApiError::from(AppError::Validation(
                "Invalid origin coordinates".to_string(),
            ))
        })?;
        (coords, None)
    } else {
        return Err(ApiError::from(AppError::Validation(
            "Must provide origin_location_id or origin_lat/origin_lng".to_string(),
        )));
    };

    // Calculate travel time via Google Maps (if available)
    let google_maps = state.google_maps.as_ref().ok_or_else(|| {
        ApiError::from(AppError::ExternalApi(
            "Google Maps not configured".to_string(),
        ))
    })?;

    let result = google_maps
        .get_travel_time(&origin_coords, &dest_coords)
        .await
        .map_err(|e| ApiError::from(AppError::ExternalApi(format!("Google Maps error: {}", e))))?;

    // Cache the result if we have location IDs
    if let Some(origin_id) = origin_location_id {
        TravelTimeCacheRepository::upsert(
            &tenant.pool,
            origin_id,
            dest_location_id,
            result.duration_minutes * 60,
            result.distance_meters,
        )
        .await?;
    }

    Ok(Json(TravelTimeResponse {
        travel_minutes: result.duration_minutes,
        distance_meters: result.distance_meters,
        is_cached: false,
        calculated_at: Utc::now().to_rfc3339(),
    }))
}

// ============================================================================
// Enhanced Availability Endpoint
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct AvailabilityQuery {
    pub walker_id: String,
    pub location_id: String,
    pub service_id: String,
    pub date: String, // YYYY-MM-DD
}

#[derive(Debug, Serialize)]
pub struct AvailabilitySlotResponse {
    pub start_time: String,
    pub end_time: String,
    pub travel_minutes: Option<i32>,
    pub travel_from: Option<String>,
    pub is_tight: bool,
    pub warning: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AvailabilityResponse {
    pub date: String,
    pub walker_id: String,
    pub walker_name: String,
    pub slots: Vec<AvailabilitySlotResponse>,
    pub travel_buffer_minutes: i32,
}

/// GET /availability/slots - Get available booking slots with travel time info
pub async fn get_availability_slots(
    State(state): State<AppState>,
    tenant: TenantContext,
    Query(query): Query<AvailabilityQuery>,
) -> ApiResult<Json<AvailabilityResponse>> {
    use chrono::NaiveDate;

    let walker_id = query
        .walker_id
        .parse()
        .map_err(|_| ApiError::from(AppError::Validation("Invalid walker ID".to_string())))?;

    let location_id = query
        .location_id
        .parse()
        .map_err(|_| ApiError::from(AppError::Validation("Invalid location ID".to_string())))?;

    let service_id = query
        .service_id
        .parse()
        .map_err(|_| ApiError::from(AppError::Validation("Invalid service ID".to_string())))?;

    let date = NaiveDate::parse_from_str(&query.date, "%Y-%m-%d").map_err(|_| {
        ApiError::from(AppError::Validation(
            "Invalid date format, use YYYY-MM-DD".to_string(),
        ))
    })?;

    // Get walker info
    let walker = db::UserRepository::find_by_id(&tenant.pool, tenant.org_id, walker_id)
        .await?
        .ok_or_else(|| ApiError::from(AppError::Validation("Walker not found".to_string())))?;

    // Get service duration
    let service = db::ServiceRepository::find_by_id(&tenant.pool, tenant.org_id, service_id)
        .await?
        .ok_or_else(|| ApiError::from(AppError::Validation("Service not found".to_string())))?;

    // Get destination location
    let destination = LocationRepository::find_by_id(&tenant.pool, tenant.org_id, location_id)
        .await?
        .ok_or_else(|| ApiError::from(AppError::Validation("Location not found".to_string())))?;

    let dest_coords =
        Coordinates::new(destination.latitude, destination.longitude).map_err(|_| {
            ApiError::from(AppError::Validation(
                "Invalid destination coordinates".to_string(),
            ))
        })?;

    // Get working hours for this day
    let day_of_week = date.weekday().num_days_from_sunday() as i16;
    let working_hours = db::WorkingHoursRepository::find_by_walker(&tenant.pool, walker_id)
        .await?
        .into_iter()
        .find(|wh| wh.day_of_week == day_of_week && wh.is_active);

    let Some(working_hours) = working_hours else {
        return Ok(Json(AvailabilityResponse {
            date: query.date,
            walker_id: query.walker_id,
            walker_name: format!("{} {}", walker.first_name, walker.last_name),
            slots: vec![],
            travel_buffer_minutes: 15,
        }));
    };

    // Get existing bookings for this day
    let day_start = date.and_hms_opt(0, 0, 0).unwrap().and_utc();
    let day_end = date.and_hms_opt(23, 59, 59).unwrap().and_utc();

    let bookings = BookingRepository::find_by_walker_in_range(
        &tenant.pool,
        tenant.org_id,
        walker_id,
        day_start,
        day_end,
    )
    .await?;

    // Get organization travel buffer setting (default 15 min)
    let travel_buffer = 15; // TODO: Get from org settings

    // Check if this is today and within 2 hours (use live location)
    let now = Utc::now();
    let is_same_day = date == now.date_naive();
    let use_live_location = is_same_day;

    // Get walker's live location if same-day booking
    let live_location = if use_live_location {
        WalkerLocationRepository::get_if_fresh(&tenant.pool, walker_id, 30).await?
    } else {
        None
    };

    // Generate time slots based on working hours
    let slot_duration = service.duration_minutes as i64;
    let mut slots = Vec::new();

    let work_start = working_hours.start_time;
    let work_end = working_hours.end_time;

    // Generate slots every 30 minutes
    let mut current_time = work_start;
    while current_time < work_end {
        let slot_start = date.and_time(current_time).and_utc();
        let slot_end = slot_start + Duration::minutes(slot_duration);

        // Skip if slot end is past working hours
        if slot_end.time() > work_end {
            current_time =
                (chrono::NaiveDateTime::new(date, current_time) + Duration::minutes(30)).time();
            continue;
        }

        // Skip if slot is in the past
        if slot_start < now {
            current_time =
                (chrono::NaiveDateTime::new(date, current_time) + Duration::minutes(30)).time();
            continue;
        }

        // Check for conflicts with existing bookings
        let has_conflict = bookings.iter().any(|b| {
            let booking_start = b.scheduled_start;
            let booking_end = b.scheduled_end;
            // Overlap check
            slot_start < booking_end && slot_end > booking_start
        });

        if has_conflict {
            current_time =
                (chrono::NaiveDateTime::new(date, current_time) + Duration::minutes(30)).time();
            continue;
        }

        // Calculate travel time from previous booking or live location
        let (travel_minutes, travel_from) = calculate_travel_info(
            &state,
            &tenant.pool,
            tenant.org_id,
            &bookings,
            slot_start,
            location_id,
            &dest_coords,
            live_location.as_ref(),
        )
        .await;

        // Check if there's enough time including travel
        let prev_booking_end = bookings
            .iter()
            .filter(|b| b.scheduled_end <= slot_start)
            .max_by_key(|b| b.scheduled_end)
            .map(|b| b.scheduled_end);

        let is_tight = if let (Some(prev_end), Some(travel)) = (prev_booking_end, travel_minutes) {
            let available_gap = (slot_start - prev_end).num_minutes();
            let needed = travel as i64 + travel_buffer as i64;
            available_gap < needed + 10 // Less than 10 min buffer is tight
        } else {
            false
        };

        // Generate warning if tight
        let warning = if is_tight {
            Some("Schedule is tight - walker may be slightly delayed".to_string())
        } else {
            None
        };

        slots.push(AvailabilitySlotResponse {
            start_time: slot_start.to_rfc3339(),
            end_time: slot_end.to_rfc3339(),
            travel_minutes,
            travel_from,
            is_tight,
            warning,
        });

        current_time =
            (chrono::NaiveDateTime::new(date, current_time) + Duration::minutes(30)).time();
    }

    Ok(Json(AvailabilityResponse {
        date: query.date,
        walker_id: query.walker_id,
        walker_name: format!("{} {}", walker.first_name, walker.last_name),
        slots,
        travel_buffer_minutes: travel_buffer,
    }))
}

/// Helper function to calculate travel time from previous location
async fn calculate_travel_info(
    state: &AppState,
    pool: &sqlx::PgPool,
    org_id: shared::types::OrganizationId,
    bookings: &[db::models::Booking],
    slot_start: chrono::DateTime<Utc>,
    dest_location_id: shared::types::LocationId,
    dest_coords: &Coordinates,
    live_location: Option<&db::models::WalkerLocation>,
) -> (Option<i32>, Option<String>) {
    // Find the most recent booking before this slot
    let prev_booking = bookings
        .iter()
        .filter(|b| b.scheduled_end <= slot_start)
        .max_by_key(|b| b.scheduled_end);

    // If we have live location and it's a same-day early slot, use live location
    if let Some(live_loc) = live_location {
        if !live_loc.is_stale(30) {
            let (lat, lng) = live_loc.coordinates();
            if let Ok(origin) = Coordinates::new(lat, lng) {
                // Try Google Maps for live location (no cache for arbitrary coordinates)
                if let Some(google_maps) = state.google_maps.as_ref() {
                    if let Ok(result) = google_maps.get_travel_time(&origin, dest_coords).await {
                        return (
                            Some(result.duration_minutes),
                            Some("Current location".to_string()),
                        );
                    }
                }
            }
        }
    }

    // Otherwise, calculate from previous booking location
    if let Some(prev) = prev_booking {
        // Get the location from the previous booking
        if let Ok(Some(prev_location)) =
            LocationRepository::find_by_id(pool, org_id, prev.location_id).await
        {
            // First, check the travel_time_cache for cached travel time
            if let Ok(Some(cached)) = TravelTimeCacheRepository::get_if_fresh(
                pool,
                prev.location_id,
                dest_location_id,
                60, // 60 minute cache TTL
            )
            .await
            {
                return (
                    Some(cached.travel_minutes()),
                    Some(format!("Previous: {}", prev_location.address)),
                );
            }

            // Fall back to Google Maps if available and no cache
            if let Some(google_maps) = state.google_maps.as_ref() {
                if let Ok(origin) =
                    Coordinates::new(prev_location.latitude, prev_location.longitude)
                {
                    if let Ok(result) = google_maps.get_travel_time(&origin, dest_coords).await {
                        // Cache the result for future use
                        let _ = TravelTimeCacheRepository::upsert(
                            pool,
                            prev.location_id,
                            dest_location_id,
                            result.duration_minutes * 60,
                            result.distance_meters,
                        )
                        .await;

                        return (
                            Some(result.duration_minutes),
                            Some(format!("Previous: {}", prev_location.address)),
                        );
                    }
                }
            }
        }
    }

    (None, None)
}
