use axum::{
    extract::{Path, State},
    http::HeaderMap,
    Json,
};
use chrono::{Datelike, Duration, NaiveDate, NaiveTime};
use db::models::{
    CreateBooking, CreateRecurringBookingSeries, OccurrenceConflict,
    RecurrenceFrequency,
};
use db::{
    check_conflicts_batch, generate_occurrence_dates, to_utc_datetime, BookingRepository,
    LocationRepository, RecurringBookingRepository, ServiceRepository, UserRepository,
};
use serde::{Deserialize, Serialize};
use shared::{AppError, DomainError};
use tracing::{info, warn, instrument, Span};
use uuid::Uuid;

use crate::{
    auth::{AuthUser, TenantContext},
    error::{ApiError, ApiResult},
    metrics,
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct CreateRecurringBookingRequest {
    pub walker_id: String,
    pub service_id: String,
    pub location_id: String,
    pub frequency: String,      // "weekly", "bi_weekly", "monthly"
    pub start_date: String,     // YYYY-MM-DD
    pub time_of_day: String,    // HH:MM
    pub end_condition: EndConditionRequest,
    pub notes: Option<String>,
    #[serde(default)]
    pub preview_only: bool,     // If true, just return preview without creating
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum EndConditionRequest {
    #[serde(rename = "occurrences")]
    Occurrences(i32),
    #[serde(rename = "date")]
    Date(String), // YYYY-MM-DD
}

#[derive(Debug, Serialize)]
pub struct RecurringBookingSeriesResponse {
    pub id: String,
    pub customer_id: String,
    pub walker_id: String,
    pub service_id: String,
    pub location_id: String,
    pub frequency: String,
    pub day_of_week: i32,
    pub day_of_week_name: String,
    pub time_of_day: String,
    pub timezone: String,
    pub end_date: Option<String>,
    pub total_occurrences: Option<i32>,
    pub is_active: bool,
    pub price_cents_per_booking: i64,
    pub price_display: String,
    pub default_notes: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct CreateRecurringResponse {
    pub series: Option<RecurringBookingSeriesResponse>,
    pub bookings_created: i32,
    pub total_planned: i32,
    pub conflicts: Vec<OccurrenceConflict>,
    pub preview_dates: Vec<String>, // First 5 dates for preview
}

#[derive(Debug, Serialize)]
pub struct RecurringBookingListItem {
    pub id: String,
    pub walker_id: String,
    pub walker_name: String,
    pub service_id: String,
    pub service_name: String,
    pub frequency: String,
    pub day_of_week_name: String,
    pub time_of_day: String,
    pub is_active: bool,
    pub price_display: String,
    pub next_occurrence: Option<String>,
    pub total_bookings: i32,
}

#[derive(Debug, Serialize)]
pub struct RecurringSeriesDetailResponse {
    pub series: RecurringBookingSeriesResponse,
    pub walker_name: String,
    pub service_name: String,
    pub location_address: String,
    pub bookings: Vec<SeriesBookingItem>,
}

#[derive(Debug, Serialize)]
pub struct SeriesBookingItem {
    pub id: String,
    pub occurrence_number: i32,
    pub scheduled_start: String,
    pub scheduled_end: String,
    pub status: String,
    pub price_display: String,
}

#[derive(Debug, Deserialize)]
pub struct CancelSeriesRequest {
    pub scope: String, // "all_future" or "entire_series"
}

#[derive(Debug, Serialize)]
pub struct CancelSeriesResponse {
    pub bookings_cancelled: i64,
    pub series_deactivated: bool,
}

/// Create a recurring booking series with atomic transaction
#[instrument(skip(_state, tenant, auth, headers, req), fields(customer_id = %auth.user_id, org_id = %tenant.org_id))]
pub async fn create_recurring_booking(
    State(_state): State<AppState>,
    tenant: TenantContext,
    auth: AuthUser,
    headers: HeaderMap,
    Json(req): Json<CreateRecurringBookingRequest>,
) -> ApiResult<Json<CreateRecurringResponse>> {
    // Extract idempotency key from header
    let idempotency_key: Option<Uuid> = headers
        .get("X-Idempotency-Key")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse().ok());

    // Check for existing series with same idempotency key
    if let Some(key) = idempotency_key {
        if let Some(existing) = RecurringBookingRepository::find_by_idempotency_key(
            &tenant.pool,
            tenant.org_id,
            key,
        ).await? {
            // Record idempotency hit
            metrics::record_idempotency_hit(&tenant.org_id.to_string());
            info!(
                series_id = %existing.id,
                idempotency_key = %key,
                "Returning existing series for idempotency key"
            );

            // Return the existing series (idempotent response)
            let bookings = BookingRepository::find_by_series(&tenant.pool, tenant.org_id, existing.id).await?;
            return Ok(Json(CreateRecurringResponse {
                series: Some(RecurringBookingSeriesResponse {
                    id: existing.id.to_string(),
                    customer_id: existing.customer_id.to_string(),
                    walker_id: existing.walker_id.to_string(),
                    service_id: existing.service_id.to_string(),
                    location_id: existing.location_id.to_string(),
                    frequency: existing.frequency.to_string(),
                    day_of_week: existing.day_of_week,
                    day_of_week_name: existing.day_of_week_name().to_string(),
                    time_of_day: existing.time_of_day.format("%H:%M").to_string(),
                    timezone: existing.timezone.clone(),
                    end_date: existing.end_date.map(|d| d.to_string()),
                    total_occurrences: existing.total_occurrences,
                    is_active: existing.is_active,
                    price_cents_per_booking: existing.price_cents_per_booking,
                    price_display: format!("${:.2}", existing.price_dollars()),
                    default_notes: existing.default_notes.clone(),
                    created_at: existing.created_at.to_rfc3339(),
                }),
                bookings_created: bookings.len() as i32,
                total_planned: bookings.len() as i32,
                conflicts: vec![],
                preview_dates: vec![],
            }));
        }
    }

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

    // Parse frequency
    let frequency = match req.frequency.as_str() {
        "weekly" => RecurrenceFrequency::Weekly,
        "bi_weekly" => RecurrenceFrequency::BiWeekly,
        "monthly" => RecurrenceFrequency::Monthly,
        _ => {
            return Err(ApiError::from(AppError::Validation(
                "Invalid frequency. Must be weekly, bi_weekly, or monthly".to_string(),
            )))
        }
    };

    // Parse start date
    let start_date = NaiveDate::parse_from_str(&req.start_date, "%Y-%m-%d").map_err(|_| {
        ApiError::from(AppError::Validation(
            "Invalid start date format. Use YYYY-MM-DD".to_string(),
        ))
    })?;

    // Parse time of day
    let time_of_day = NaiveTime::parse_from_str(&req.time_of_day, "%H:%M").map_err(|_| {
        ApiError::from(AppError::Validation(
            "Invalid time format. Use HH:MM".to_string(),
        ))
    })?;

    // Parse end condition
    let (end_date, total_occurrences) = match &req.end_condition {
        EndConditionRequest::Occurrences(n) => {
            if *n < 1 || *n > 52 {
                return Err(ApiError::from(AppError::Validation(
                    "Occurrences must be between 1 and 52".to_string(),
                )));
            }
            (None, Some(*n))
        }
        EndConditionRequest::Date(d) => {
            let date = NaiveDate::parse_from_str(d, "%Y-%m-%d").map_err(|_| {
                ApiError::from(AppError::Validation(
                    "Invalid end date format. Use YYYY-MM-DD".to_string(),
                ))
            })?;
            if date <= start_date {
                return Err(ApiError::from(AppError::Validation(
                    "End date must be after start date".to_string(),
                )));
            }
            (Some(date), None)
        }
    };

    // Verify walker exists and is a walker
    let walker = UserRepository::find_by_id(&tenant.pool, tenant.org_id, walker_id)
        .await?
        .ok_or_else(|| ApiError::from(DomainError::WalkerNotFound(req.walker_id.clone())))?;

    if !walker.is_walker() {
        return Err(ApiError::from(DomainError::WalkerNotFound(
            req.walker_id.clone(),
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

    // Get day of week from start date
    let day_of_week = start_date.weekday().num_days_from_sunday() as i32;

    // Default timezone - could be made configurable
    let timezone = "America/Denver".to_string();

    // Generate occurrence dates
    let dates = generate_occurrence_dates(
        start_date,
        frequency,
        day_of_week,
        end_date,
        total_occurrences,
    );

    let total_planned = dates.len() as i32;

    // Check for conflicts using batch query (2 queries instead of N*2)
    let conflicts = check_conflicts_batch(
        &tenant.pool,
        tenant.org_id,
        walker_id,
        &dates,
        time_of_day,
        service.duration_minutes,
        &timezone,
    )
    .await?;

    // Log and record conflicts
    if !conflicts.is_empty() {
        metrics::record_conflicts(&tenant.org_id.to_string(), conflicts.len() as i64);
        warn!(
            conflicts_count = conflicts.len(),
            total_dates = dates.len(),
            "Found conflicts during recurring booking creation"
        );
        for conflict in &conflicts {
            info!(date = %conflict.date, reason = %conflict.reason, "Conflict detected");
        }
    }

    // Get conflict-free dates
    let conflict_dates: std::collections::HashSet<_> = conflicts.iter().map(|c| c.date).collect();
    let valid_dates: Vec<_> = dates
        .iter()
        .filter(|d| !conflict_dates.contains(*d))
        .cloned()
        .collect();

    // Preview first 5 dates
    let preview_dates: Vec<String> = dates.iter().take(5).map(|d| d.to_string()).collect();

    // If preview only, return without creating
    if req.preview_only {
        info!(
            total_planned = total_planned,
            conflicts_count = conflicts.len(),
            valid_dates = valid_dates.len(),
            "Preview mode - returning without creating"
        );
        return Ok(Json(CreateRecurringResponse {
            series: None,
            bookings_created: 0,
            total_planned,
            conflicts,
            preview_dates,
        }));
    }

    // Start transaction for atomic creation
    let timer = metrics::Timer::start(&tenant.org_id.to_string());
    let mut tx = tenant.pool.begin().await.map_err(|e| {
        warn!(error = %e, "Failed to start transaction");
        metrics::record_series_creation_failed(&tenant.org_id.to_string(), "transaction_start");
        ApiError::from(AppError::Internal("Failed to start transaction".to_string()))
    })?;

    // Create the series within transaction
    let series = RecurringBookingRepository::create_in_tx(
        &mut tx,
        CreateRecurringBookingSeries {
            organization_id: tenant.org_id,
            customer_id: auth.user_id,
            walker_id,
            service_id,
            location_id,
            frequency,
            day_of_week,
            time_of_day,
            timezone: timezone.clone(),
            end_date,
            total_occurrences,
            price_cents_per_booking: service.base_price_cents,
            default_notes: req.notes.clone(),
            idempotency_key,
        },
    )
    .await
    .map_err(|e| {
        warn!(error = %e, "Failed to create recurring series");
        metrics::record_series_creation_failed(&tenant.org_id.to_string(), "series_create");
        ApiError::from(AppError::Internal("Failed to create recurring series".to_string()))
    })?;

    Span::current().record("series_id", series.id.to_string());

    // Create individual bookings for valid dates within same transaction
    let mut bookings_created = 0;
    let mut failed_bookings = 0;

    for (idx, date) in valid_dates.iter().enumerate() {
        let start = match to_utc_datetime(*date, time_of_day, &timezone) {
            Some(dt) => dt,
            None => {
                warn!(date = %date, "Failed to convert date to UTC");
                failed_bookings += 1;
                continue;
            }
        };
        let end = start + Duration::minutes(service.duration_minutes as i64);

        match BookingRepository::create_in_tx(
            &mut tx,
            CreateBooking {
                organization_id: tenant.org_id,
                customer_id: auth.user_id,
                walker_id,
                service_id,
                location_id,
                scheduled_start: start,
                scheduled_end: end,
                price_cents: service.base_price_cents,
                notes: req.notes.clone(),
                recurring_series_id: Some(series.id),
                occurrence_number: Some((idx + 1) as i32),
            },
        )
        .await {
            Ok(_) => bookings_created += 1,
            Err(e) => {
                // Check if it's a uniqueness constraint violation (duplicate)
                let err_str = e.to_string();
                if err_str.contains("idx_booking_uniqueness") || err_str.contains("duplicate") {
                    warn!(date = %date, error = %e, "Duplicate booking detected, skipping");
                } else {
                    warn!(date = %date, error = %e, "Failed to create booking");
                }
                failed_bookings += 1;
            }
        }
    }

    // If no bookings were created successfully and we expected some, rollback
    if bookings_created == 0 && !valid_dates.is_empty() {
        warn!(
            valid_dates = valid_dates.len(),
            failed_bookings = failed_bookings,
            "All booking creations failed, rolling back transaction"
        );
        tx.rollback().await.ok();
        metrics::record_series_creation_failed(&tenant.org_id.to_string(), "all_bookings_failed");
        return Err(ApiError::from(AppError::Internal(
            "Failed to create any bookings. Please try again.".to_string(),
        )));
    }

    // Commit the transaction
    tx.commit().await.map_err(|e| {
        warn!(error = %e, "Failed to commit transaction");
        metrics::record_series_creation_failed(&tenant.org_id.to_string(), "commit_failed");
        ApiError::from(AppError::Internal("Failed to save bookings".to_string()))
    })?;

    // Record successful creation metrics
    timer.record();
    metrics::record_series_created(&tenant.org_id.to_string(), bookings_created as i64);

    info!(
        series_id = %series.id,
        bookings_created = bookings_created,
        total_planned = total_planned,
        conflicts_count = conflicts.len(),
        "Successfully created recurring booking series"
    );

    Ok(Json(CreateRecurringResponse {
        series: Some(RecurringBookingSeriesResponse {
            id: series.id.to_string(),
            customer_id: series.customer_id.to_string(),
            walker_id: series.walker_id.to_string(),
            service_id: series.service_id.to_string(),
            location_id: series.location_id.to_string(),
            frequency: series.frequency.to_string(),
            day_of_week: series.day_of_week,
            day_of_week_name: series.day_of_week_name().to_string(),
            time_of_day: series.time_of_day.format("%H:%M").to_string(),
            timezone: series.timezone.clone(),
            end_date: series.end_date.map(|d| d.to_string()),
            total_occurrences: series.total_occurrences,
            is_active: series.is_active,
            price_cents_per_booking: series.price_cents_per_booking,
            price_display: format!("${:.2}", series.price_dollars()),
            default_notes: series.default_notes.clone(),
            created_at: series.created_at.to_rfc3339(),
        }),
        bookings_created,
        total_planned,
        conflicts,
        preview_dates,
    }))
}

/// List customer's recurring series
pub async fn list_recurring_bookings(
    State(_state): State<AppState>,
    tenant: TenantContext,
    auth: AuthUser,
) -> ApiResult<Json<Vec<RecurringBookingListItem>>> {
    let series_list =
        RecurringBookingRepository::find_by_customer(&tenant.pool, tenant.org_id, auth.user_id)
            .await?;

    let mut responses = Vec::with_capacity(series_list.len());
    for s in series_list {
        let walker = UserRepository::find_by_id(&tenant.pool, tenant.org_id, s.walker_id)
            .await?
            .map(|u| u.full_name())
            .unwrap_or_else(|| "Unknown".to_string());

        let service = ServiceRepository::find_by_id(&tenant.pool, tenant.org_id, s.service_id)
            .await?
            .map(|svc| svc.name)
            .unwrap_or_else(|| "Unknown".to_string());

        // Get bookings for this series to find next occurrence and count
        let bookings = BookingRepository::find_by_series(&tenant.pool, tenant.org_id, s.id).await?;

        let now = chrono::Utc::now();
        let next_occurrence = bookings
            .iter()
            .filter(|b| b.scheduled_start > now && b.is_active())
            .min_by_key(|b| b.scheduled_start)
            .map(|b| b.scheduled_start.to_rfc3339());

        responses.push(RecurringBookingListItem {
            id: s.id.to_string(),
            walker_id: s.walker_id.to_string(),
            walker_name: walker,
            service_id: s.service_id.to_string(),
            service_name: service,
            frequency: s.frequency.display_name().to_string(),
            day_of_week_name: s.day_of_week_name().to_string(),
            time_of_day: s.time_of_day.format("%H:%M").to_string(),
            is_active: s.is_active,
            price_display: format!("${:.2}", s.price_dollars()),
            next_occurrence,
            total_bookings: bookings.len() as i32,
        });
    }

    Ok(Json(responses))
}

/// Get recurring series details
pub async fn get_recurring_booking(
    State(_state): State<AppState>,
    tenant: TenantContext,
    auth: AuthUser,
    Path(id): Path<String>,
) -> ApiResult<Json<RecurringSeriesDetailResponse>> {
    let series_id = id.parse().map_err(|_| {
        ApiError::from(AppError::Validation(
            "Invalid series ID".to_string(),
        ))
    })?;

    let series = RecurringBookingRepository::find_by_id(&tenant.pool, tenant.org_id, series_id)
        .await?
        .ok_or_else(|| {
            ApiError::from(DomainError::BookingNotFound(id.clone()))
        })?;

    // Check authorization
    if series.customer_id != auth.user_id {
        return Err(ApiError::from(AppError::Forbidden));
    }

    let walker = UserRepository::find_by_id(&tenant.pool, tenant.org_id, series.walker_id)
        .await?
        .map(|u| u.full_name())
        .unwrap_or_else(|| "Unknown".to_string());

    let service = ServiceRepository::find_by_id(&tenant.pool, tenant.org_id, series.service_id)
        .await?
        .map(|s| s.name)
        .unwrap_or_else(|| "Unknown".to_string());

    let location = LocationRepository::find_by_id(&tenant.pool, tenant.org_id, series.location_id)
        .await?
        .map(|l| format!("{}, {}", l.address, l.city))
        .unwrap_or_else(|| "Unknown".to_string());

    let bookings = BookingRepository::find_by_series(&tenant.pool, tenant.org_id, series_id).await?;

    let booking_items: Vec<SeriesBookingItem> = bookings
        .iter()
        .map(|b| SeriesBookingItem {
            id: b.id.to_string(),
            occurrence_number: b.occurrence_number.unwrap_or(0),
            scheduled_start: b.scheduled_start.to_rfc3339(),
            scheduled_end: b.scheduled_end.to_rfc3339(),
            status: b.status.to_string(),
            price_display: format!("${:.2}", b.price_dollars()),
        })
        .collect();

    Ok(Json(RecurringSeriesDetailResponse {
        series: RecurringBookingSeriesResponse {
            id: series.id.to_string(),
            customer_id: series.customer_id.to_string(),
            walker_id: series.walker_id.to_string(),
            service_id: series.service_id.to_string(),
            location_id: series.location_id.to_string(),
            frequency: series.frequency.display_name().to_string(),
            day_of_week: series.day_of_week,
            day_of_week_name: series.day_of_week_name().to_string(),
            time_of_day: series.time_of_day.format("%H:%M").to_string(),
            timezone: series.timezone.clone(),
            end_date: series.end_date.map(|d| d.to_string()),
            total_occurrences: series.total_occurrences,
            is_active: series.is_active,
            price_cents_per_booking: series.price_cents_per_booking,
            price_display: format!("${:.2}", series.price_dollars()),
            default_notes: series.default_notes.clone(),
            created_at: series.created_at.to_rfc3339(),
        },
        walker_name: walker,
        service_name: service,
        location_address: location,
        bookings: booking_items,
    }))
}

/// Cancel a recurring series
pub async fn cancel_recurring_series(
    State(_state): State<AppState>,
    tenant: TenantContext,
    auth: AuthUser,
    Path(id): Path<String>,
    Json(req): Json<CancelSeriesRequest>,
) -> ApiResult<Json<CancelSeriesResponse>> {
    let series_id = id.parse().map_err(|_| {
        ApiError::from(AppError::Validation(
            "Invalid series ID".to_string(),
        ))
    })?;

    let series = RecurringBookingRepository::find_by_id(&tenant.pool, tenant.org_id, series_id)
        .await?
        .ok_or_else(|| {
            ApiError::from(DomainError::BookingNotFound(id.clone()))
        })?;

    // Check authorization
    if series.customer_id != auth.user_id {
        return Err(ApiError::from(AppError::Forbidden));
    }

    if !series.can_cancel() {
        return Err(ApiError::from(DomainError::InvalidStateTransition(
            "inactive".to_string(),
        )));
    }

    let bookings_cancelled = match req.scope.as_str() {
        "all_future" => {
            BookingRepository::cancel_future_by_series(&tenant.pool, tenant.org_id, series_id)
                .await?
        }
        "entire_series" => {
            BookingRepository::cancel_all_by_series(&tenant.pool, tenant.org_id, series_id).await?
        }
        _ => {
            return Err(ApiError::from(AppError::Validation(
                "Invalid scope. Must be all_future or entire_series".to_string(),
            )))
        }
    };

    // Deactivate the series
    RecurringBookingRepository::deactivate(&tenant.pool, tenant.org_id, series_id).await?;

    Ok(Json(CancelSeriesResponse {
        bookings_cancelled,
        series_deactivated: true,
    }))
}
