use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared::types::{BookingId, LocationId, OrganizationId, ServiceId, UserId};
use sqlx::FromRow;

/// Booking status enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "booking_status", rename_all = "snake_case")]
pub enum BookingStatus {
    Pending,
    Confirmed,
    InProgress,
    Completed,
    Cancelled,
    NoShow,
}

impl std::fmt::Display for BookingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BookingStatus::Pending => write!(f, "pending"),
            BookingStatus::Confirmed => write!(f, "confirmed"),
            BookingStatus::InProgress => write!(f, "in_progress"),
            BookingStatus::Completed => write!(f, "completed"),
            BookingStatus::Cancelled => write!(f, "cancelled"),
            BookingStatus::NoShow => write!(f, "no_show"),
        }
    }
}

/// Booking database model
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Booking {
    pub id: BookingId,
    pub organization_id: Option<OrganizationId>,
    pub customer_id: UserId,
    pub walker_id: UserId,
    pub service_id: ServiceId,
    pub location_id: LocationId,
    pub status: BookingStatus,
    pub scheduled_start: DateTime<Utc>,
    pub scheduled_end: DateTime<Utc>,
    pub actual_start: Option<DateTime<Utc>>,
    pub actual_end: Option<DateTime<Utc>>,
    pub price_cents: i64,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Booking {
    pub fn price_dollars(&self) -> f64 {
        self.price_cents as f64 / 100.0
    }

    pub fn duration_minutes(&self) -> i64 {
        (self.scheduled_end - self.scheduled_start).num_minutes()
    }

    pub fn is_active(&self) -> bool {
        matches!(
            self.status,
            BookingStatus::Pending | BookingStatus::Confirmed | BookingStatus::InProgress
        )
    }

    pub fn can_cancel(&self) -> bool {
        matches!(
            self.status,
            BookingStatus::Pending | BookingStatus::Confirmed
        )
    }

    pub fn can_confirm(&self) -> bool {
        self.status == BookingStatus::Pending
    }
}

/// Input for creating a new booking
#[derive(Debug, Clone, Deserialize)]
pub struct CreateBooking {
    pub organization_id: OrganizationId,
    pub customer_id: UserId,
    pub walker_id: UserId,
    pub service_id: ServiceId,
    pub location_id: LocationId,
    pub scheduled_start: DateTime<Utc>,
    pub scheduled_end: DateTime<Utc>,
    pub price_cents: i64,
    pub notes: Option<String>,
}
