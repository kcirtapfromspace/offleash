use axum::{
    extract::{Path, Query, State},
    Json,
};
use chrono::{Datelike, Duration, NaiveDate, NaiveTime, Utc};
use db::{
    BlockRepository, BookingRepository, LocationRepository, ServiceAreaRepository,
    ServiceRepository, TravelTimeCacheRepository, UserRepository,
};
use domain::{
    walker_can_service_location, AvailabilityConfig, AvailabilityEngine, BlockSlot, BookingSlot,
    DayHours, PolygonPoint, ServiceAreaBoundary, TrafficConfig, TravelTimeMatrix,
};
use serde::{Deserialize, Serialize};
use shared::{types::DurationMinutes, AppError, DomainError};

use crate::{
    auth::TenantContext,
    error::{ApiError, ApiResult},
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct AvailabilityQuery {
    pub date: String, // YYYY-MM-DD
    pub service_id: String,
    pub location_id: String,
}

#[derive(Debug, Serialize)]
pub struct AvailabilityResponse {
    pub walker_id: String,
    pub date: String,
    pub service_id: String,
    pub slots: Vec<SlotResponse>,
}

#[derive(Debug, Serialize)]
pub struct SlotResponse {
    pub start: String,
    pub end: String,
    pub confidence: String,
}

pub async fn get_availability(
    State(_state): State<AppState>,
    tenant: TenantContext,
    Path(walker_id): Path<String>,
    Query(query): Query<AvailabilityQuery>,
) -> ApiResult<Json<AvailabilityResponse>> {
    // Parse walker ID
    let walker_id_parsed = walker_id
        .parse()
        .map_err(|_| ApiError::from(AppError::Validation("Invalid walker ID".to_string())))?;

    // Verify walker exists and is a walker within this organization
    let walker = UserRepository::find_by_id(&tenant.pool, tenant.org_id, walker_id_parsed)
        .await?
        .ok_or_else(|| ApiError::from(DomainError::WalkerNotFound(walker_id.clone())))?;

    if !walker.is_walker() {
        return Err(ApiError::from(DomainError::WalkerNotFound(
            walker_id.clone(),
        )));
    }

    // Parse date
    let date = NaiveDate::parse_from_str(&query.date, "%Y-%m-%d")
        .map_err(|_| ApiError::from(AppError::Validation("Invalid date format".to_string())))?;

    // Parse service ID and get service
    let service_id = query
        .service_id
        .parse()
        .map_err(|_| ApiError::from(AppError::Validation("Invalid service ID".to_string())))?;

    let service = ServiceRepository::find_by_id(&tenant.pool, tenant.org_id, service_id)
        .await?
        .ok_or_else(|| ApiError::from(DomainError::ServiceNotFound(query.service_id.clone())))?;

    // Parse location ID and get location
    let location_id = query
        .location_id
        .parse()
        .map_err(|_| ApiError::from(AppError::Validation("Invalid location ID".to_string())))?;

    let location = LocationRepository::find_by_id(&tenant.pool, tenant.org_id, location_id)
        .await?
        .ok_or_else(|| ApiError::from(DomainError::LocationNotFound(query.location_id.clone())))?;

    // Check if walker can service this location
    let walker_service_areas =
        ServiceAreaRepository::find_active_by_walker(&tenant.pool, tenant.org_id, walker_id_parsed)
            .await?;

    if !walker_service_areas.is_empty() {
        // Walker has defined service areas, check if location is within them
        let boundaries: Vec<ServiceAreaBoundary> = walker_service_areas
            .iter()
            .map(|sa| ServiceAreaBoundary {
                walker_id: sa.walker_id.to_string(),
                area_id: sa.id.to_string(),
                name: sa.name.clone(),
                polygon: sa
                    .polygon
                    .0
                    .iter()
                    .map(|p| PolygonPoint::new(p.lat, p.lng))
                    .collect(),
                min_lat: sa.min_latitude.unwrap_or(-90.0),
                max_lat: sa.max_latitude.unwrap_or(90.0),
                min_lng: sa.min_longitude.unwrap_or(-180.0),
                max_lng: sa.max_longitude.unwrap_or(180.0),
                priority: sa.priority.unwrap_or(0),
                price_adjustment_percent: sa.price_adjustment_percent.unwrap_or(0),
            })
            .collect();

        let coords = location.coordinates();

        if walker_can_service_location(&boundaries, &walker_id, &coords).is_none() {
            return Err(ApiError::from(AppError::Validation(
                "Location is outside walker's service area".to_string(),
            )));
        }
    }

    // Get working hours for this day (simplified - using default 9-5 for now)
    // TODO: Load from working_hours table
    let day_of_week = date.weekday().num_days_from_sunday() as u8;
    let working_hours = if (1..=5).contains(&day_of_week) {
        // Monday-Friday
        Some(DayHours {
            start: NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            end: NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
        })
    } else {
        None // Weekends off by default
    };

    // Get date range for querying bookings and blocks
    let start_of_day = date.and_hms_opt(0, 0, 0).unwrap().and_utc();
    let end_of_day = start_of_day + Duration::days(1);

    // Load existing bookings
    let bookings = BookingRepository::find_by_walker_in_range(
        &tenant.pool,
        tenant.org_id,
        walker_id_parsed,
        start_of_day,
        end_of_day,
    )
    .await?;

    let booking_slots: Vec<BookingSlot> = bookings
        .into_iter()
        .map(|b| BookingSlot::new(b.id, b.location_id, b.scheduled_start, b.scheduled_end))
        .collect();

    // Load blocks
    let blocks = BlockRepository::find_by_walker_in_range(
        &tenant.pool,
        tenant.org_id,
        walker_id_parsed,
        start_of_day,
        end_of_day,
    )
    .await?;

    let block_slots: Vec<BlockSlot> = blocks
        .into_iter()
        .map(|b| BlockSlot::new(b.id, b.start_time, b.end_time))
        .collect();

    // Calculate availability with traffic-aware travel times
    let config = AvailabilityConfig::default();
    let traffic_config = TrafficConfig::default();

    // Build travel time matrix from cache
    let mut travel_times = TravelTimeMatrix::new();

    // Collect all location IDs (target + booking locations)
    let mut location_ids: Vec<shared::types::LocationId> = vec![location_id];
    for booking in &booking_slots {
        if !location_ids.contains(&booking.location_id) {
            location_ids.push(booking.location_id);
        }
    }

    // Fetch cached travel times
    let cache_ttl = traffic_config.get_cache_ttl_minutes(start_of_day);
    let cached_times =
        TravelTimeCacheRepository::get_batch_fresh(&tenant.pool, &location_ids, cache_ttl).await?;

    // Populate travel matrix with traffic-adjusted times
    for cache_entry in cached_times {
        let base_minutes = cache_entry.travel_minutes();
        // Apply peak hour multiplier based on a representative time (midday of requested date)
        let representative_time = start_of_day + Duration::hours(12);
        let adjusted_minutes = traffic_config.adjust_travel_time(base_minutes, representative_time);

        travel_times.insert(
            cache_entry.origin_location_id,
            cache_entry.destination_location_id,
            DurationMinutes::new(adjusted_minutes),
        );
    }

    let available_slots = AvailabilityEngine::calculate_slots(
        working_hours.as_ref(),
        &booking_slots,
        &block_slots,
        &travel_times,
        location_id,
        service.duration_minutes,
        date,
        &walker.timezone,
        &config,
    );

    // Filter out slots in the past
    let now = Utc::now();
    let min_notice = Duration::hours(config.min_notice_hours as i64);
    let earliest_bookable = now + min_notice;

    let filtered_slots: Vec<SlotResponse> = available_slots
        .into_iter()
        .filter(|s| s.start >= earliest_bookable)
        .map(|s| SlotResponse {
            start: s.start.to_rfc3339(),
            end: s.end.to_rfc3339(),
            confidence: format!("{:?}", s.confidence),
        })
        .collect();

    Ok(Json(AvailabilityResponse {
        walker_id,
        date: query.date,
        service_id: query.service_id,
        slots: filtered_slots,
    }))
}
