use axum::{
    extract::{Path, State},
    Json,
};
use chrono::NaiveTime;
use db::WorkingHoursRepository;
use serde::{Deserialize, Serialize};
use shared::{types::UserId, AppError, DomainError};

use crate::{
    auth::TenantContext,
    error::{ApiError, ApiResult},
    state::AppState,
};

#[derive(Debug, Serialize)]
pub struct WorkingHoursResponse {
    pub id: String,
    pub walker_id: String,
    pub day_of_week: i16,
    pub day_name: String,
    pub start_time: String,
    pub end_time: String,
    pub is_active: bool,
}

#[derive(Debug, Deserialize)]
pub struct DayScheduleInput {
    pub day_of_week: i16,
    pub start_time: String, // HH:MM format
    pub end_time: String,   // HH:MM format
    pub is_active: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateScheduleRequest {
    pub schedule: Vec<DayScheduleInput>,
}

fn day_name(day: i16) -> String {
    match day {
        0 => "Sunday".to_string(),
        1 => "Monday".to_string(),
        2 => "Tuesday".to_string(),
        3 => "Wednesday".to_string(),
        4 => "Thursday".to_string(),
        5 => "Friday".to_string(),
        6 => "Saturday".to_string(),
        _ => "Unknown".to_string(),
    }
}

fn parse_time(time_str: &str) -> Result<NaiveTime, ApiError> {
    NaiveTime::parse_from_str(time_str, "%H:%M")
        .map_err(|_| ApiError::from(AppError::Validation(
            format!("Invalid time format '{}'. Use HH:MM format.", time_str)
        )))
}

pub async fn get_walker_hours(
    State(_state): State<AppState>,
    tenant: TenantContext,
    Path(walker_id): Path<String>,
) -> ApiResult<Json<Vec<WorkingHoursResponse>>> {
    let walker_id: UserId = walker_id
        .parse()
        .map_err(|_| ApiError::from(AppError::Validation("Invalid walker ID".to_string())))?;

    // Verify walker exists and belongs to this org
    let walker = db::UserRepository::find_by_id(&tenant.pool, tenant.org_id, walker_id)
        .await?
        .ok_or_else(|| ApiError::from(DomainError::UserNotFound(walker_id.to_string())))?;

    if !walker.is_walker() {
        return Err(ApiError::from(AppError::Validation(
            "User is not a walker".to_string(),
        )));
    }

    let hours = WorkingHoursRepository::find_by_walker(&tenant.pool, walker_id).await?;

    let response: Vec<WorkingHoursResponse> = hours
        .into_iter()
        .map(|h| WorkingHoursResponse {
            id: h.id.to_string(),
            walker_id: h.walker_id.to_string(),
            day_of_week: h.day_of_week,
            day_name: day_name(h.day_of_week),
            start_time: h.start_time.format("%H:%M").to_string(),
            end_time: h.end_time.format("%H:%M").to_string(),
            is_active: h.is_active,
        })
        .collect();

    Ok(Json(response))
}

pub async fn update_walker_hours(
    State(_state): State<AppState>,
    tenant: TenantContext,
    Path(walker_id): Path<String>,
    Json(req): Json<UpdateScheduleRequest>,
) -> ApiResult<Json<Vec<WorkingHoursResponse>>> {
    let walker_id: UserId = walker_id
        .parse()
        .map_err(|_| ApiError::from(AppError::Validation("Invalid walker ID".to_string())))?;

    // Verify walker exists and belongs to this org
    let walker = db::UserRepository::find_by_id(&tenant.pool, tenant.org_id, walker_id)
        .await?
        .ok_or_else(|| ApiError::from(DomainError::UserNotFound(walker_id.to_string())))?;

    if !walker.is_walker() {
        return Err(ApiError::from(AppError::Validation(
            "User is not a walker".to_string(),
        )));
    }

    // Validate day_of_week values
    for day in &req.schedule {
        if day.day_of_week < 0 || day.day_of_week > 6 {
            return Err(ApiError::from(AppError::Validation(
                format!("Invalid day_of_week {}. Must be 0-6 (Sunday-Saturday).", day.day_of_week)
            )));
        }
    }

    // Upsert each day's schedule
    let mut results = Vec::new();
    for day in req.schedule {
        let start_time = parse_time(&day.start_time)?;
        let end_time = parse_time(&day.end_time)?;

        if start_time >= end_time {
            return Err(ApiError::from(AppError::Validation(
                format!("Start time must be before end time for {}", day_name(day.day_of_week))
            )));
        }

        let hours = WorkingHoursRepository::upsert(
            &tenant.pool,
            walker_id,
            day.day_of_week,
            start_time,
            end_time,
            day.is_active,
        )
        .await?;

        results.push(WorkingHoursResponse {
            id: hours.id.to_string(),
            walker_id: hours.walker_id.to_string(),
            day_of_week: hours.day_of_week,
            day_name: day_name(hours.day_of_week),
            start_time: hours.start_time.format("%H:%M").to_string(),
            end_time: hours.end_time.format("%H:%M").to_string(),
            is_active: hours.is_active,
        });
    }

    Ok(Json(results))
}

pub async fn delete_walker_hours(
    State(_state): State<AppState>,
    tenant: TenantContext,
    Path(walker_id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let walker_id: UserId = walker_id
        .parse()
        .map_err(|_| ApiError::from(AppError::Validation("Invalid walker ID".to_string())))?;

    // Verify walker exists and belongs to this org
    let walker = db::UserRepository::find_by_id(&tenant.pool, tenant.org_id, walker_id)
        .await?
        .ok_or_else(|| ApiError::from(DomainError::UserNotFound(walker_id.to_string())))?;

    if !walker.is_walker() {
        return Err(ApiError::from(AppError::Validation(
            "User is not a walker".to_string(),
        )));
    }

    let deleted = WorkingHoursRepository::delete_by_walker(&tenant.pool, walker_id).await?;

    Ok(Json(serde_json::json!({
        "deleted": deleted,
        "message": format!("Deleted {} working hours entries", deleted)
    })))
}
