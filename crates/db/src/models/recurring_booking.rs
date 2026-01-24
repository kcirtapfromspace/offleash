use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use shared::types::{LocationId, OrganizationId, RecurringBookingSeriesId, ServiceId, UserId};
use sqlx::FromRow;
use uuid::Uuid;

/// Recurrence frequency enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "recurrence_frequency", rename_all = "snake_case")]
pub enum RecurrenceFrequency {
    Weekly,
    BiWeekly,
    Monthly,
}

impl std::fmt::Display for RecurrenceFrequency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RecurrenceFrequency::Weekly => write!(f, "weekly"),
            RecurrenceFrequency::BiWeekly => write!(f, "bi_weekly"),
            RecurrenceFrequency::Monthly => write!(f, "monthly"),
        }
    }
}

impl RecurrenceFrequency {
    pub fn display_name(&self) -> &'static str {
        match self {
            RecurrenceFrequency::Weekly => "Weekly",
            RecurrenceFrequency::BiWeekly => "Every 2 weeks",
            RecurrenceFrequency::Monthly => "Monthly",
        }
    }
}

/// Recurring booking series database model
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct RecurringBookingSeries {
    pub id: RecurringBookingSeriesId,
    pub organization_id: OrganizationId,
    pub customer_id: UserId,
    pub walker_id: UserId,
    pub service_id: ServiceId,
    pub location_id: LocationId,
    pub frequency: RecurrenceFrequency,
    pub day_of_week: i32,
    pub time_of_day: NaiveTime,
    pub timezone: String,
    pub end_date: Option<NaiveDate>,
    pub total_occurrences: Option<i32>,
    pub is_active: bool,
    pub price_cents_per_booking: i64,
    pub default_notes: Option<String>,
    pub idempotency_key: Option<Uuid>,
    pub idempotency_expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl RecurringBookingSeries {
    pub fn price_dollars(&self) -> f64 {
        self.price_cents_per_booking as f64 / 100.0
    }

    pub fn day_of_week_name(&self) -> &'static str {
        match self.day_of_week {
            0 => "Sunday",
            1 => "Monday",
            2 => "Tuesday",
            3 => "Wednesday",
            4 => "Thursday",
            5 => "Friday",
            6 => "Saturday",
            _ => "Unknown",
        }
    }

    pub fn can_cancel(&self) -> bool {
        self.is_active
    }
}

/// Input for creating a new recurring booking series
#[derive(Debug, Clone, Deserialize)]
pub struct CreateRecurringBookingSeries {
    pub organization_id: OrganizationId,
    pub customer_id: UserId,
    pub walker_id: UserId,
    pub service_id: ServiceId,
    pub location_id: LocationId,
    pub frequency: RecurrenceFrequency,
    pub day_of_week: i32,
    pub time_of_day: NaiveTime,
    pub timezone: String,
    pub end_date: Option<NaiveDate>,
    pub total_occurrences: Option<i32>,
    pub price_cents_per_booking: i64,
    pub default_notes: Option<String>,
    pub idempotency_key: Option<Uuid>,
}

/// End condition for recurring series
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum EndCondition {
    #[serde(rename = "occurrences")]
    Occurrences(i32),
    #[serde(rename = "date")]
    Date(NaiveDate),
}

/// Conflict information for a specific occurrence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OccurrenceConflict {
    pub date: NaiveDate,
    pub reason: String,
}

/// Result of creating a recurring booking series
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRecurringResult {
    pub series_id: RecurringBookingSeriesId,
    pub bookings_created: i32,
    pub conflicts: Vec<OccurrenceConflict>,
}
