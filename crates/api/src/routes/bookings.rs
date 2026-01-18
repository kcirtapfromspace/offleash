use axum::{
    extract::{Path, State},
    Json,
};
use chrono::DateTime;
use db::models::CreateBooking;
use db::{BookingRepository, LocationRepository, ServiceRepository, UserRepository};
use serde::{Deserialize, Serialize};
use shared::{AppError, DomainError};

use crate::{
    auth::AuthUser,
    error::{ApiError, ApiResult},
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct CreateBookingRequest {
    pub walker_id: String,
    pub service_id: String,
    pub location_id: String,
    pub start_time: String, // ISO 8601
    pub notes: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BookingResponse {
    pub id: String,
    pub customer_id: String,
    pub walker_id: String,
    pub service_id: String,
    pub location_id: String,
    pub status: String,
    pub scheduled_start: String,
    pub scheduled_end: String,
    pub price_cents: i64,
    pub price_display: String,
    pub notes: Option<String>,
}

pub async fn create_booking(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateBookingRequest>,
) -> ApiResult<Json<BookingResponse>> {
    // Parse IDs
    let walker_id = req
        .walker_id
        .parse()
        .map_err(|_| ApiError::from(AppError::Validation("Invalid walker ID".to_string())))?;

    let service_id = req
        .service_id
        .parse()
        .map_err(|_| ApiError::from(AppError::Validation("Invalid service ID".to_string())))?;

    let location_id = req
        .location_id
        .parse()
        .map_err(|_| ApiError::from(AppError::Validation("Invalid location ID".to_string())))?;

    // Parse start time
    let start_time = DateTime::parse_from_rfc3339(&req.start_time)
        .map_err(|_| {
            ApiError::from(AppError::Validation(
                "Invalid start time format".to_string(),
            ))
        })?
        .with_timezone(&chrono::Utc);

    // Verify walker exists
    let walker = UserRepository::find_by_id(&state.pool, walker_id)
        .await?
        .ok_or_else(|| ApiError::from(DomainError::WalkerNotFound(req.walker_id.clone())))?;

    if !walker.is_walker() {
        return Err(ApiError::from(DomainError::WalkerNotFound(
            req.walker_id.clone(),
        )));
    }

    // Get service for duration and price
    let service = ServiceRepository::find_by_id(&state.pool, service_id)
        .await?
        .ok_or_else(|| ApiError::from(DomainError::ServiceNotFound(req.service_id.clone())))?;

    // Verify location exists and belongs to customer
    let location = LocationRepository::find_by_id(&state.pool, location_id)
        .await?
        .ok_or_else(|| ApiError::from(DomainError::LocationNotFound(req.location_id.clone())))?;

    if location.user_id != auth.user_id {
        return Err(ApiError::from(AppError::Forbidden));
    }

    // Calculate end time
    let end_time = start_time + chrono::Duration::minutes(service.duration_minutes as i64);

    // Create booking
    let booking = BookingRepository::create(
        &state.pool,
        CreateBooking {
            customer_id: auth.user_id,
            walker_id,
            service_id,
            location_id,
            scheduled_start: start_time,
            scheduled_end: end_time,
            price_cents: service.base_price_cents,
            notes: req.notes,
        },
    )
    .await
    .map_err(|e| {
        if e.to_string().contains("conflict") {
            ApiError::from(DomainError::BookingConflict)
        } else {
            ApiError::from(e)
        }
    })?;

    Ok(Json(BookingResponse {
        id: booking.id.to_string(),
        customer_id: booking.customer_id.to_string(),
        walker_id: booking.walker_id.to_string(),
        service_id: booking.service_id.to_string(),
        location_id: booking.location_id.to_string(),
        status: booking.status.to_string(),
        scheduled_start: booking.scheduled_start.to_rfc3339(),
        scheduled_end: booking.scheduled_end.to_rfc3339(),
        price_cents: booking.price_cents,
        price_display: format!("${:.2}", booking.price_dollars()),
        notes: booking.notes,
    }))
}

pub async fn get_booking(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
) -> ApiResult<Json<BookingResponse>> {
    let booking_id = id
        .parse()
        .map_err(|_| ApiError::from(AppError::Validation("Invalid booking ID".to_string())))?;

    let booking = BookingRepository::find_by_id(&state.pool, booking_id)
        .await?
        .ok_or_else(|| ApiError::from(DomainError::BookingNotFound(id)))?;

    // Check authorization - must be customer or walker
    if booking.customer_id != auth.user_id && booking.walker_id != auth.user_id {
        return Err(ApiError::from(AppError::Forbidden));
    }

    Ok(Json(BookingResponse {
        id: booking.id.to_string(),
        customer_id: booking.customer_id.to_string(),
        walker_id: booking.walker_id.to_string(),
        service_id: booking.service_id.to_string(),
        location_id: booking.location_id.to_string(),
        status: booking.status.to_string(),
        scheduled_start: booking.scheduled_start.to_rfc3339(),
        scheduled_end: booking.scheduled_end.to_rfc3339(),
        price_cents: booking.price_cents,
        price_display: format!("${:.2}", booking.price_dollars()),
        notes: booking.notes,
    }))
}

pub async fn confirm_booking(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
) -> ApiResult<Json<BookingResponse>> {
    let booking_id = id
        .parse()
        .map_err(|_| ApiError::from(AppError::Validation("Invalid booking ID".to_string())))?;

    let booking = BookingRepository::find_by_id(&state.pool, booking_id)
        .await?
        .ok_or_else(|| ApiError::from(DomainError::BookingNotFound(id.clone())))?;

    // Only walker can confirm
    if booking.walker_id != auth.user_id {
        return Err(ApiError::from(AppError::Forbidden));
    }

    if !booking.can_confirm() {
        return Err(ApiError::from(DomainError::InvalidStateTransition(
            booking.status.to_string(),
        )));
    }

    let updated = BookingRepository::confirm(&state.pool, booking_id)
        .await?
        .ok_or_else(|| ApiError::from(DomainError::BookingNotFound(id)))?;

    Ok(Json(BookingResponse {
        id: updated.id.to_string(),
        customer_id: updated.customer_id.to_string(),
        walker_id: updated.walker_id.to_string(),
        service_id: updated.service_id.to_string(),
        location_id: updated.location_id.to_string(),
        status: updated.status.to_string(),
        scheduled_start: updated.scheduled_start.to_rfc3339(),
        scheduled_end: updated.scheduled_end.to_rfc3339(),
        price_cents: updated.price_cents,
        price_display: format!("${:.2}", updated.price_dollars()),
        notes: updated.notes,
    }))
}

pub async fn cancel_booking(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
) -> ApiResult<Json<BookingResponse>> {
    let booking_id = id
        .parse()
        .map_err(|_| ApiError::from(AppError::Validation("Invalid booking ID".to_string())))?;

    let booking = BookingRepository::find_by_id(&state.pool, booking_id)
        .await?
        .ok_or_else(|| ApiError::from(DomainError::BookingNotFound(id.clone())))?;

    // Customer or walker can cancel
    if booking.customer_id != auth.user_id && booking.walker_id != auth.user_id {
        return Err(ApiError::from(AppError::Forbidden));
    }

    if !booking.can_cancel() {
        return Err(ApiError::from(DomainError::InvalidStateTransition(
            booking.status.to_string(),
        )));
    }

    let updated = BookingRepository::cancel(&state.pool, booking_id)
        .await?
        .ok_or_else(|| ApiError::from(DomainError::BookingNotFound(id)))?;

    Ok(Json(BookingResponse {
        id: updated.id.to_string(),
        customer_id: updated.customer_id.to_string(),
        walker_id: updated.walker_id.to_string(),
        service_id: updated.service_id.to_string(),
        location_id: updated.location_id.to_string(),
        status: updated.status.to_string(),
        scheduled_start: updated.scheduled_start.to_rfc3339(),
        scheduled_end: updated.scheduled_end.to_rfc3339(),
        price_cents: updated.price_cents,
        price_display: format!("${:.2}", updated.price_dollars()),
        notes: updated.notes,
    }))
}
