//! Calendar API routes for events, blocks, and connections

use axum::{
    extract::{Path, Query, State},
    Json,
};
use chrono::{DateTime, Utc};
use db::models::{CalendarEventType, CreateCalendarEvent, SyncStatus, UpdateCalendarEvent};
use db::CalendarRepository;
use serde::{Deserialize, Serialize};
use shared::{AppError, DomainError};
use uuid::Uuid;

use crate::{
    auth::{AuthUser, TenantContext},
    error::{ApiError, ApiResult},
    state::AppState,
};

// ============ Request/Response Types ============

#[derive(Debug, Deserialize)]
pub struct CreateEventRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub start_time: String, // ISO 8601
    pub end_time: String,   // ISO 8601
    pub all_day: Option<bool>,
    pub event_type: String, // "block" | "personal"
    pub recurrence_rule: Option<String>,
    pub color: Option<String>,
    pub is_blocking: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateEventRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub all_day: Option<bool>,
    pub color: Option<String>,
    pub is_blocking: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ListEventsQuery {
    pub start: String, // ISO 8601
    pub end: String,   // ISO 8601
    pub event_type: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CalendarEventResponse {
    pub id: String,
    pub user_id: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub start_time: String,
    pub end_time: String,
    pub all_day: bool,
    pub event_type: String,
    pub calendar_connection_id: Option<String>,
    pub external_event_id: Option<String>,
    pub sync_status: String,
    pub recurrence_rule: Option<String>,
    pub recurrence_parent_id: Option<String>,
    pub color: Option<String>,
    pub is_blocking: bool,
    pub created_at: String,
    pub updated_at: String,
    // Optional join fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connection_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ListEventsResponse {
    pub events: Vec<CalendarEventResponse>,
    pub count: usize,
}

// ============ Event Handlers ============

/// List calendar events in a date range
pub async fn list_events(
    State(_state): State<AppState>,
    tenant: TenantContext,
    auth: AuthUser,
    Query(query): Query<ListEventsQuery>,
) -> ApiResult<Json<ListEventsResponse>> {
    let start = DateTime::parse_from_rfc3339(&query.start)
        .map_err(|_| {
            ApiError::from(AppError::Validation(
                "Invalid start time format".to_string(),
            ))
        })?
        .with_timezone(&Utc);

    let end = DateTime::parse_from_rfc3339(&query.end)
        .map_err(|_| ApiError::from(AppError::Validation("Invalid end time format".to_string())))?
        .with_timezone(&Utc);

    let events = if let Some(event_type) = query.event_type {
        let et = parse_event_type(&event_type)?;
        CalendarRepository::find_events_by_type(
            &tenant.pool,
            tenant.org_id,
            auth.user_id,
            et,
            start,
            end,
        )
        .await?
    } else {
        CalendarRepository::find_events_in_range(
            &tenant.pool,
            tenant.org_id,
            auth.user_id,
            start,
            end,
        )
        .await?
    };

    let count = events.len();
    let events: Vec<_> = events
        .into_iter()
        .map(|e| CalendarEventResponse {
            id: e.id.to_string(),
            user_id: e.user_id.to_string(),
            title: e.title,
            description: e.description,
            start_time: e.start_time.to_rfc3339(),
            end_time: e.end_time.to_rfc3339(),
            all_day: e.all_day,
            event_type: e.event_type.to_string(),
            calendar_connection_id: e.calendar_connection_id.map(|id| id.to_string()),
            external_event_id: e.external_event_id,
            sync_status: e.sync_status.to_string(),
            recurrence_rule: e.recurrence_rule,
            recurrence_parent_id: e.recurrence_parent_id.map(|id| id.to_string()),
            color: e.color,
            is_blocking: e.is_blocking,
            created_at: e.created_at.to_rfc3339(),
            updated_at: e.updated_at.to_rfc3339(),
            connection_name: None,
            provider: None,
        })
        .collect();

    Ok(Json(ListEventsResponse { events, count }))
}

/// Get a single calendar event
pub async fn get_event(
    State(_state): State<AppState>,
    tenant: TenantContext,
    auth: AuthUser,
    Path(id): Path<String>,
) -> ApiResult<Json<CalendarEventResponse>> {
    let event_id: Uuid = id
        .parse()
        .map_err(|_| ApiError::from(AppError::Validation("Invalid event ID".to_string())))?;

    let event = CalendarRepository::find_event_by_id(&tenant.pool, tenant.org_id, event_id)
        .await?
        .ok_or_else(|| ApiError::from(DomainError::BookingNotFound(id.clone())))?;

    // Verify event belongs to user
    if event.user_id != auth.user_id {
        return Err(ApiError::from(AppError::Forbidden));
    }

    Ok(Json(CalendarEventResponse {
        id: event.id.to_string(),
        user_id: event.user_id.to_string(),
        title: event.title,
        description: event.description,
        start_time: event.start_time.to_rfc3339(),
        end_time: event.end_time.to_rfc3339(),
        all_day: event.all_day,
        event_type: event.event_type.to_string(),
        calendar_connection_id: event.calendar_connection_id.map(|id| id.to_string()),
        external_event_id: event.external_event_id,
        sync_status: event.sync_status.to_string(),
        recurrence_rule: event.recurrence_rule,
        recurrence_parent_id: event.recurrence_parent_id.map(|id| id.to_string()),
        color: event.color,
        is_blocking: event.is_blocking,
        created_at: event.created_at.to_rfc3339(),
        updated_at: event.updated_at.to_rfc3339(),
        connection_name: None,
        provider: None,
    }))
}

/// Create a calendar event (block or personal)
pub async fn create_event(
    State(_state): State<AppState>,
    tenant: TenantContext,
    auth: AuthUser,
    Json(req): Json<CreateEventRequest>,
) -> ApiResult<Json<CalendarEventResponse>> {
    let start_time = DateTime::parse_from_rfc3339(&req.start_time)
        .map_err(|_| {
            ApiError::from(AppError::Validation(
                "Invalid start time format".to_string(),
            ))
        })?
        .with_timezone(&Utc);

    let end_time = DateTime::parse_from_rfc3339(&req.end_time)
        .map_err(|_| ApiError::from(AppError::Validation("Invalid end time format".to_string())))?
        .with_timezone(&Utc);

    if end_time <= start_time {
        return Err(ApiError::from(AppError::Validation(
            "End time must be after start time".to_string(),
        )));
    }

    let event_type = parse_event_type(&req.event_type)?;

    // Only allow creating blocks and personal events through this endpoint
    if matches!(
        event_type,
        CalendarEventType::Booking | CalendarEventType::Synced
    ) {
        return Err(ApiError::from(AppError::Validation(
            "Cannot create booking or synced events through this endpoint".to_string(),
        )));
    }

    let event = CalendarRepository::create_event(
        &tenant.pool,
        CreateCalendarEvent {
            organization_id: tenant.org_id,
            user_id: auth.user_id,
            title: req.title,
            description: req.description,
            start_time,
            end_time,
            all_day: req.all_day.unwrap_or(false),
            event_type,
            calendar_connection_id: None,
            external_event_id: None,
            recurrence_rule: req.recurrence_rule,
            recurrence_parent_id: None,
            color: req.color,
            is_blocking: req.is_blocking.unwrap_or(true),
        },
    )
    .await?;

    Ok(Json(CalendarEventResponse {
        id: event.id.to_string(),
        user_id: event.user_id.to_string(),
        title: event.title,
        description: event.description,
        start_time: event.start_time.to_rfc3339(),
        end_time: event.end_time.to_rfc3339(),
        all_day: event.all_day,
        event_type: event.event_type.to_string(),
        calendar_connection_id: None,
        external_event_id: None,
        sync_status: event.sync_status.to_string(),
        recurrence_rule: event.recurrence_rule,
        recurrence_parent_id: None,
        color: event.color,
        is_blocking: event.is_blocking,
        created_at: event.created_at.to_rfc3339(),
        updated_at: event.updated_at.to_rfc3339(),
        connection_name: None,
        provider: None,
    }))
}

/// Update a calendar event
pub async fn update_event(
    State(_state): State<AppState>,
    tenant: TenantContext,
    auth: AuthUser,
    Path(id): Path<String>,
    Json(req): Json<UpdateEventRequest>,
) -> ApiResult<Json<CalendarEventResponse>> {
    let event_id: Uuid = id
        .parse()
        .map_err(|_| ApiError::from(AppError::Validation("Invalid event ID".to_string())))?;

    // Verify event exists and belongs to user
    let existing = CalendarRepository::find_event_by_id(&tenant.pool, tenant.org_id, event_id)
        .await?
        .ok_or_else(|| ApiError::from(DomainError::BookingNotFound(id.clone())))?;

    if existing.user_id != auth.user_id {
        return Err(ApiError::from(AppError::Forbidden));
    }

    // Cannot edit synced events
    if matches!(existing.event_type, CalendarEventType::Synced) {
        return Err(ApiError::from(AppError::Validation(
            "Cannot edit synced events. Please edit in the source calendar.".to_string(),
        )));
    }

    // Parse optional times
    let start_time = if let Some(ref s) = req.start_time {
        Some(
            DateTime::parse_from_rfc3339(s)
                .map_err(|_| {
                    ApiError::from(AppError::Validation(
                        "Invalid start time format".to_string(),
                    ))
                })?
                .with_timezone(&Utc),
        )
    } else {
        None
    };

    let end_time = if let Some(ref e) = req.end_time {
        Some(
            DateTime::parse_from_rfc3339(e)
                .map_err(|_| {
                    ApiError::from(AppError::Validation("Invalid end time format".to_string()))
                })?
                .with_timezone(&Utc),
        )
    } else {
        None
    };

    // Validate time range if both provided
    let effective_start = start_time.unwrap_or(existing.start_time);
    let effective_end = end_time.unwrap_or(existing.end_time);
    if effective_end <= effective_start {
        return Err(ApiError::from(AppError::Validation(
            "End time must be after start time".to_string(),
        )));
    }

    let event = CalendarRepository::update_event(
        &tenant.pool,
        tenant.org_id,
        event_id,
        UpdateCalendarEvent {
            title: req.title.map(Some),
            description: req.description.map(Some),
            start_time,
            end_time,
            all_day: req.all_day,
            color: req.color.map(Some),
            is_blocking: req.is_blocking,
            sync_status: None,
            last_synced_at: None,
        },
    )
    .await?
    .ok_or_else(|| ApiError::from(DomainError::BookingNotFound(id)))?;

    Ok(Json(CalendarEventResponse {
        id: event.id.to_string(),
        user_id: event.user_id.to_string(),
        title: event.title,
        description: event.description,
        start_time: event.start_time.to_rfc3339(),
        end_time: event.end_time.to_rfc3339(),
        all_day: event.all_day,
        event_type: event.event_type.to_string(),
        calendar_connection_id: event.calendar_connection_id.map(|id| id.to_string()),
        external_event_id: event.external_event_id,
        sync_status: event.sync_status.to_string(),
        recurrence_rule: event.recurrence_rule,
        recurrence_parent_id: event.recurrence_parent_id.map(|id| id.to_string()),
        color: event.color,
        is_blocking: event.is_blocking,
        created_at: event.created_at.to_rfc3339(),
        updated_at: event.updated_at.to_rfc3339(),
        connection_name: None,
        provider: None,
    }))
}

/// Delete a calendar event
pub async fn delete_event(
    State(_state): State<AppState>,
    tenant: TenantContext,
    auth: AuthUser,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let event_id: Uuid = id
        .parse()
        .map_err(|_| ApiError::from(AppError::Validation("Invalid event ID".to_string())))?;

    // Verify event exists and belongs to user
    let existing = CalendarRepository::find_event_by_id(&tenant.pool, tenant.org_id, event_id)
        .await?
        .ok_or_else(|| ApiError::from(DomainError::BookingNotFound(id.clone())))?;

    if existing.user_id != auth.user_id {
        return Err(ApiError::from(AppError::Forbidden));
    }

    // Cannot delete synced events (must delete from source calendar)
    if matches!(existing.event_type, CalendarEventType::Synced) {
        return Err(ApiError::from(AppError::Validation(
            "Cannot delete synced events. Please delete in the source calendar.".to_string(),
        )));
    }

    CalendarRepository::delete_event(&tenant.pool, tenant.org_id, event_id).await?;

    Ok(Json(serde_json::json!({ "deleted": true })))
}

// ============ Helper Functions ============

fn parse_event_type(s: &str) -> Result<CalendarEventType, ApiError> {
    match s.to_lowercase().as_str() {
        "booking" => Ok(CalendarEventType::Booking),
        "block" => Ok(CalendarEventType::Block),
        "personal" => Ok(CalendarEventType::Personal),
        "synced" => Ok(CalendarEventType::Synced),
        _ => Err(ApiError::from(AppError::Validation(format!(
            "Invalid event type: {}. Must be one of: booking, block, personal, synced",
            s
        )))),
    }
}

#[allow(dead_code)]
fn parse_sync_status(s: &str) -> Result<SyncStatus, ApiError> {
    match s.to_lowercase().as_str() {
        "pending" => Ok(SyncStatus::Pending),
        "synced" => Ok(SyncStatus::Synced),
        "failed" => Ok(SyncStatus::Failed),
        "conflict" => Ok(SyncStatus::Conflict),
        _ => Err(ApiError::from(AppError::Validation(format!(
            "Invalid sync status: {}",
            s
        )))),
    }
}
