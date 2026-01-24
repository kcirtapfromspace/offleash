use axum::{
    extract::{Path, Query, State},
    Json,
};
use chrono::DateTime;
use db::models::{BookingStatus, CreateBooking};
use db::{BookingRepository, LocationRepository, ServiceRepository, UserRepository};
use serde::{Deserialize, Serialize};
use shared::{AppError, DomainError};

use crate::{
    auth::{AuthUser, TenantContext},
    error::{ApiError, ApiResult},
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct ListBookingsQuery {
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateBookingRequest {
    pub walker_id: Option<String>, // Optional - backend can assign if not provided
    pub service_id: String,
    pub location_id: String,
    pub start_time: String, // ISO 8601
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RescheduleBookingRequest {
    pub scheduled_start: String, // ISO 8601
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

/// Enriched booking response with resolved names for admin list view
#[derive(Debug, Serialize)]
pub struct BookingListItem {
    pub id: String,
    pub customer_id: String,
    pub customer_name: String,
    pub walker_id: String,
    pub walker_name: String,
    pub service_id: String,
    pub service_name: String,
    pub location_id: String,
    pub location_address: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub status: String,
    pub scheduled_start: String,
    pub scheduled_end: String,
    pub price_cents: i64,
    pub price_display: String,
    pub notes: Option<String>,
    pub customer_phone: Option<String>,
    pub pet_name: Option<String>,
    pub pet_breed: Option<String>,
}

pub async fn create_booking(
    State(_state): State<AppState>,
    tenant: TenantContext,
    auth: AuthUser,
    Json(req): Json<CreateBookingRequest>,
) -> ApiResult<Json<BookingResponse>> {
    // Parse IDs
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

    // Get or find walker
    let walker_id = if let Some(ref wid) = req.walker_id {
        wid.parse()
            .map_err(|_| ApiError::from(AppError::Validation("Invalid walker ID".to_string())))?
    } else {
        // Find first available walker in the organization
        let walkers =
            UserRepository::list_by_role(&tenant.pool, tenant.org_id, db::models::UserRole::Walker)
                .await?;
        walkers.first().map(|w| w.id).ok_or_else(|| {
            ApiError::from(AppError::Validation("No walkers available".to_string()))
        })?
    };

    // Verify walker exists within this organization
    let walker = UserRepository::find_by_id(&tenant.pool, tenant.org_id, walker_id)
        .await?
        .ok_or_else(|| {
            ApiError::from(DomainError::WalkerNotFound(
                req.walker_id.clone().unwrap_or_default(),
            ))
        })?;

    if !walker.is_walker() {
        return Err(ApiError::from(DomainError::WalkerNotFound(
            req.walker_id.clone().unwrap_or_default(),
        )));
    }

    // Get service for duration and price
    let service = ServiceRepository::find_by_id(&tenant.pool, tenant.org_id, service_id)
        .await?
        .ok_or_else(|| ApiError::from(DomainError::ServiceNotFound(req.service_id.clone())))?;

    // Verify location exists and belongs to customer
    let location = LocationRepository::find_by_id(&tenant.pool, tenant.org_id, location_id)
        .await?
        .ok_or_else(|| ApiError::from(DomainError::LocationNotFound(req.location_id.clone())))?;

    if location.user_id != auth.user_id {
        return Err(ApiError::from(AppError::Forbidden));
    }

    // Calculate end time
    let end_time = start_time + chrono::Duration::minutes(service.duration_minutes as i64);

    // Create booking
    let booking = BookingRepository::create(
        &tenant.pool,
        CreateBooking {
            organization_id: tenant.org_id,
            customer_id: auth.user_id,
            walker_id,
            service_id,
            location_id,
            scheduled_start: start_time,
            scheduled_end: end_time,
            price_cents: service.base_price_cents,
            notes: req.notes,
            recurring_series_id: None,
            occurrence_number: None,
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
    State(_state): State<AppState>,
    tenant: TenantContext,
    auth: AuthUser,
    Path(id): Path<String>,
) -> ApiResult<Json<BookingResponse>> {
    let booking_id = id
        .parse()
        .map_err(|_| ApiError::from(AppError::Validation("Invalid booking ID".to_string())))?;

    let booking = BookingRepository::find_by_id(&tenant.pool, tenant.org_id, booking_id)
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
    State(_state): State<AppState>,
    tenant: TenantContext,
    auth: AuthUser,
    Path(id): Path<String>,
) -> ApiResult<Json<BookingResponse>> {
    let booking_id = id
        .parse()
        .map_err(|_| ApiError::from(AppError::Validation("Invalid booking ID".to_string())))?;

    let booking = BookingRepository::find_by_id(&tenant.pool, tenant.org_id, booking_id)
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

    let updated = BookingRepository::confirm(&tenant.pool, tenant.org_id, booking_id)
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
    State(_state): State<AppState>,
    tenant: TenantContext,
    auth: AuthUser,
    Path(id): Path<String>,
) -> ApiResult<Json<BookingResponse>> {
    let booking_id = id
        .parse()
        .map_err(|_| ApiError::from(AppError::Validation("Invalid booking ID".to_string())))?;

    let booking = BookingRepository::find_by_id(&tenant.pool, tenant.org_id, booking_id)
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

    let updated = BookingRepository::cancel(&tenant.pool, tenant.org_id, booking_id)
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

/// Reschedule a booking to a new time
pub async fn reschedule_booking(
    State(_state): State<AppState>,
    tenant: TenantContext,
    auth: AuthUser,
    Path(id): Path<String>,
    Json(req): Json<RescheduleBookingRequest>,
) -> ApiResult<Json<BookingResponse>> {
    let booking_id = id
        .parse()
        .map_err(|_| ApiError::from(AppError::Validation("Invalid booking ID".to_string())))?;

    let booking = BookingRepository::find_by_id(&tenant.pool, tenant.org_id, booking_id)
        .await?
        .ok_or_else(|| ApiError::from(DomainError::BookingNotFound(id.clone())))?;

    // Only customer can reschedule their own booking
    if booking.customer_id != auth.user_id {
        return Err(ApiError::from(AppError::Forbidden));
    }

    // Can only reschedule pending or confirmed bookings
    if !booking.can_cancel() {
        return Err(ApiError::from(DomainError::InvalidStateTransition(
            booking.status.to_string(),
        )));
    }

    // Parse the new scheduled start time
    let new_start = DateTime::parse_from_rfc3339(&req.scheduled_start)
        .map_err(|_| ApiError::from(AppError::Validation("Invalid date format".to_string())))?
        .with_timezone(&chrono::Utc);

    // Calculate new end time based on original duration
    let duration = booking.scheduled_end - booking.scheduled_start;
    let new_end = new_start + duration;

    // Update the booking
    let updated =
        BookingRepository::reschedule(&tenant.pool, tenant.org_id, booking_id, new_start, new_end)
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

/// List all bookings (admin only)
pub async fn list_bookings(
    State(_state): State<AppState>,
    tenant: TenantContext,
    auth: AuthUser,
    Query(query): Query<ListBookingsQuery>,
) -> ApiResult<Json<Vec<BookingListItem>>> {
    // Verify user is admin
    let user = UserRepository::find_by_id(&tenant.pool, tenant.org_id, auth.user_id)
        .await?
        .ok_or_else(|| ApiError::from(AppError::Forbidden))?;

    if !user.is_admin() {
        return Err(ApiError::from(AppError::Forbidden));
    }

    // Parse status filter
    let status_filter = query.status.and_then(|s| match s.as_str() {
        "pending" => Some(BookingStatus::Pending),
        "confirmed" => Some(BookingStatus::Confirmed),
        "in_progress" => Some(BookingStatus::InProgress),
        "completed" => Some(BookingStatus::Completed),
        "cancelled" => Some(BookingStatus::Cancelled),
        "no_show" => Some(BookingStatus::NoShow),
        _ => None,
    });

    let bookings = BookingRepository::list_all(&tenant.pool, tenant.org_id, status_filter).await?;

    // Enrich bookings with related data
    let mut responses = Vec::with_capacity(bookings.len());
    for b in bookings {
        let customer = UserRepository::find_by_id(&tenant.pool, tenant.org_id, b.customer_id)
            .await?
            .map(|u| u.full_name())
            .unwrap_or_else(|| "Unknown".to_string());

        let walker = UserRepository::find_by_id(&tenant.pool, tenant.org_id, b.walker_id)
            .await?
            .map(|u| u.full_name())
            .unwrap_or_else(|| "Unknown".to_string());

        let service = ServiceRepository::find_by_id(&tenant.pool, tenant.org_id, b.service_id)
            .await?
            .map(|s| s.name)
            .unwrap_or_else(|| "Unknown".to_string());

        let location =
            LocationRepository::find_by_id(&tenant.pool, tenant.org_id, b.location_id).await?;

        let location_address = location
            .as_ref()
            .map(|l| format!("{}, {}", l.address, l.city))
            .unwrap_or_else(|| "Unknown".to_string());
        let latitude = location.as_ref().map(|l| l.latitude);
        let longitude = location.as_ref().map(|l| l.longitude);

        responses.push(BookingListItem {
            id: b.id.to_string(),
            customer_id: b.customer_id.to_string(),
            customer_name: customer,
            walker_id: b.walker_id.to_string(),
            walker_name: walker,
            service_id: b.service_id.to_string(),
            service_name: service,
            location_id: b.location_id.to_string(),
            location_address,
            latitude,
            longitude,
            status: b.status.to_string(),
            scheduled_start: b.scheduled_start.to_rfc3339(),
            scheduled_end: b.scheduled_end.to_rfc3339(),
            price_cents: b.price_cents,
            price_display: format!("${:.2}", b.price_dollars()),
            notes: b.notes.clone(),
            customer_phone: None, // TODO: Add customer phone to booking
            pet_name: None,       // TODO: Add pet info to booking
            pet_breed: None,
        });
    }

    Ok(Json(responses))
}

/// List bookings for the authenticated customer
pub async fn list_customer_bookings(
    State(_state): State<AppState>,
    tenant: TenantContext,
    auth: AuthUser,
) -> ApiResult<Json<Vec<BookingListItem>>> {
    let bookings =
        BookingRepository::find_by_customer(&tenant.pool, tenant.org_id, auth.user_id).await?;

    // Enrich bookings with related data
    let mut responses = Vec::with_capacity(bookings.len());
    for b in bookings {
        let walker = UserRepository::find_by_id(&tenant.pool, tenant.org_id, b.walker_id)
            .await?
            .map(|u| u.full_name())
            .unwrap_or_else(|| "Unknown".to_string());

        let service = ServiceRepository::find_by_id(&tenant.pool, tenant.org_id, b.service_id)
            .await?
            .map(|s| s.name)
            .unwrap_or_else(|| "Unknown".to_string());

        let location =
            LocationRepository::find_by_id(&tenant.pool, tenant.org_id, b.location_id).await?;

        let location_address = location
            .as_ref()
            .map(|l| format!("{}, {}", l.address, l.city))
            .unwrap_or_else(|| "Unknown".to_string());
        let latitude = location.as_ref().map(|l| l.latitude);
        let longitude = location.as_ref().map(|l| l.longitude);

        responses.push(BookingListItem {
            id: b.id.to_string(),
            customer_id: b.customer_id.to_string(),
            customer_name: "".to_string(), // Not needed for customer view
            walker_id: b.walker_id.to_string(),
            walker_name: walker,
            service_id: b.service_id.to_string(),
            service_name: service,
            location_id: b.location_id.to_string(),
            location_address,
            latitude,
            longitude,
            status: b.status.to_string(),
            scheduled_start: b.scheduled_start.to_rfc3339(),
            scheduled_end: b.scheduled_end.to_rfc3339(),
            price_cents: b.price_cents,
            price_display: format!("${:.2}", b.price_dollars()),
            notes: b.notes.clone(),
            customer_phone: None,
            pet_name: None,
            pet_breed: None,
        });
    }

    Ok(Json(responses))
}

/// List bookings assigned to the authenticated walker
pub async fn list_walker_bookings(
    State(_state): State<AppState>,
    tenant: TenantContext,
    auth: AuthUser,
) -> ApiResult<Json<Vec<BookingListItem>>> {
    let bookings =
        BookingRepository::find_by_walker(&tenant.pool, tenant.org_id, auth.user_id).await?;

    // Enrich bookings with related data
    let mut responses = Vec::with_capacity(bookings.len());
    for b in bookings {
        let customer =
            UserRepository::find_by_id(&tenant.pool, tenant.org_id, b.customer_id).await?;

        let customer_name = customer
            .as_ref()
            .map(|u| u.full_name())
            .unwrap_or_else(|| "Unknown".to_string());

        let customer_phone = customer.and_then(|u| u.phone);

        let service = ServiceRepository::find_by_id(&tenant.pool, tenant.org_id, b.service_id)
            .await?
            .map(|s| s.name)
            .unwrap_or_else(|| "Unknown".to_string());

        let location =
            LocationRepository::find_by_id(&tenant.pool, tenant.org_id, b.location_id).await?;

        let location_address = location
            .as_ref()
            .map(|l| format!("{}, {}", l.address, l.city))
            .unwrap_or_else(|| "Unknown".to_string());
        let latitude = location.as_ref().map(|l| l.latitude);
        let longitude = location.as_ref().map(|l| l.longitude);

        responses.push(BookingListItem {
            id: b.id.to_string(),
            customer_id: b.customer_id.to_string(),
            customer_name,
            walker_id: b.walker_id.to_string(),
            walker_name: "".to_string(), // Not needed for walker view
            service_id: b.service_id.to_string(),
            service_name: service,
            location_id: b.location_id.to_string(),
            location_address,
            latitude,
            longitude,
            status: b.status.to_string(),
            scheduled_start: b.scheduled_start.to_rfc3339(),
            scheduled_end: b.scheduled_end.to_rfc3339(),
            price_cents: b.price_cents,
            price_display: format!("${:.2}", b.price_dollars()),
            notes: b.notes.clone(),
            customer_phone,
            pet_name: None,
            pet_breed: None,
        });
    }

    Ok(Json(responses))
}

/// Complete a booking (admin only)
pub async fn complete_booking(
    State(_state): State<AppState>,
    tenant: TenantContext,
    auth: AuthUser,
    Path(id): Path<String>,
) -> ApiResult<Json<BookingResponse>> {
    let booking_id = id
        .parse()
        .map_err(|_| ApiError::from(AppError::Validation("Invalid booking ID".to_string())))?;

    // Verify user is admin
    let user = UserRepository::find_by_id(&tenant.pool, tenant.org_id, auth.user_id)
        .await?
        .ok_or_else(|| ApiError::from(AppError::Forbidden))?;

    if !user.is_admin() {
        return Err(ApiError::from(AppError::Forbidden));
    }

    let booking = BookingRepository::find_by_id(&tenant.pool, tenant.org_id, booking_id)
        .await?
        .ok_or_else(|| ApiError::from(DomainError::BookingNotFound(id.clone())))?;

    // Can only complete confirmed or in_progress bookings
    if !matches!(
        booking.status,
        BookingStatus::Confirmed | BookingStatus::InProgress
    ) {
        return Err(ApiError::from(DomainError::InvalidStateTransition(
            booking.status.to_string(),
        )));
    }

    let updated = BookingRepository::complete(&tenant.pool, tenant.org_id, booking_id)
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
