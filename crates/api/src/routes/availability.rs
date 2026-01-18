use axum::{
    extract::{Path, Query, State},
    Json,
};
use chrono::{Datelike, Duration, NaiveDate, NaiveTime, Utc};
use db::{BlockRepository, BookingRepository, LocationRepository, ServiceRepository, UserRepository};
use domain::{AvailabilityConfig, AvailabilityEngine, BlockSlot, BookingSlot, DayHours, TravelTimeMatrix};
use serde::{Deserialize, Serialize};
use shared::{AppError, DomainError};

use crate::{
    error::{ApiError, ApiResult},
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct AvailabilityQuery {
    pub date: String,        // YYYY-MM-DD
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
    State(state): State<AppState>,
    Path(walker_id): Path<String>,
    Query(query): Query<AvailabilityQuery>,
) -> ApiResult<Json<AvailabilityResponse>> {
    // Parse walker ID
    let walker_id_parsed = walker_id
        .parse()
        .map_err(|_| ApiError::from(AppError::Validation("Invalid walker ID".to_string())))?;

    // Verify walker exists and is a walker
    let walker = UserRepository::find_by_id(&state.pool, walker_id_parsed)
        .await?
        .ok_or_else(|| ApiError::from(DomainError::WalkerNotFound(walker_id.clone())))?;

    if !walker.is_walker() {
        return Err(ApiError::from(DomainError::WalkerNotFound(walker_id.clone())));
    }

    // Parse date
    let date = NaiveDate::parse_from_str(&query.date, "%Y-%m-%d")
        .map_err(|_| ApiError::from(AppError::Validation("Invalid date format".to_string())))?;

    // Parse service ID and get service
    let service_id = query
        .service_id
        .parse()
        .map_err(|_| ApiError::from(AppError::Validation("Invalid service ID".to_string())))?;

    let service = ServiceRepository::find_by_id(&state.pool, service_id)
        .await?
        .ok_or_else(|| ApiError::from(DomainError::ServiceNotFound(query.service_id.clone())))?;

    // Parse location ID and get location
    let location_id = query
        .location_id
        .parse()
        .map_err(|_| ApiError::from(AppError::Validation("Invalid location ID".to_string())))?;

    let _location = LocationRepository::find_by_id(&state.pool, location_id)
        .await?
        .ok_or_else(|| ApiError::from(DomainError::LocationNotFound(query.location_id.clone())))?;

    // Get working hours for this day (simplified - using default 9-5 for now)
    // TODO: Load from working_hours table
    let day_of_week = date.weekday().num_days_from_sunday() as u8;
    let working_hours = if day_of_week >= 1 && day_of_week <= 5 {
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
        &state.pool,
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
        &state.pool,
        walker_id_parsed,
        start_of_day,
        end_of_day,
    )
    .await?;

    let block_slots: Vec<BlockSlot> = blocks
        .into_iter()
        .map(|b| BlockSlot::new(b.id, b.start_time, b.end_time))
        .collect();

    // Calculate availability
    let config = AvailabilityConfig::default();
    let travel_times = TravelTimeMatrix::new(); // Empty for now, would be populated from cache/API

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
