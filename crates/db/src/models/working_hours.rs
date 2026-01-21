use chrono::{DateTime, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use shared::types::{UserId, WorkingHoursId};
use sqlx::FromRow;

/// Working hours database model
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct WorkingHours {
    pub id: WorkingHoursId,
    pub walker_id: UserId,
    pub day_of_week: i16,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Input for creating working hours
#[derive(Debug, Clone, Deserialize)]
pub struct CreateWorkingHours {
    pub walker_id: UserId,
    pub day_of_week: i16,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
}

/// Input for updating working hours
#[derive(Debug, Clone, Deserialize, Default)]
pub struct UpdateWorkingHours {
    pub start_time: Option<NaiveTime>,
    pub end_time: Option<NaiveTime>,
    pub is_active: Option<bool>,
}

/// Batch update for a walker's entire schedule
#[derive(Debug, Clone, Deserialize)]
pub struct WorkingHoursSchedule {
    pub walker_id: UserId,
    pub schedule: Vec<DaySchedule>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DaySchedule {
    pub day_of_week: i16,
    pub start_time: String, // HH:MM format
    pub end_time: String,   // HH:MM format
    pub is_active: bool,
}
