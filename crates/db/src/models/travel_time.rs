use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared::types::{LocationId, UserId};
use sqlx::FromRow;
use uuid::Uuid;

/// Cached travel time between two locations
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct TravelTimeCache {
    pub id: Uuid,
    pub origin_location_id: LocationId,
    pub destination_location_id: LocationId,
    pub travel_seconds: i32,
    pub distance_meters: i32,
    pub calculated_at: DateTime<Utc>,
}

impl TravelTimeCache {
    /// Get travel time in minutes (rounded up)
    pub fn travel_minutes(&self) -> i32 {
        (self.travel_seconds + 59) / 60
    }

    /// Check if cache entry is stale (older than given minutes)
    pub fn is_stale(&self, max_age_minutes: i64) -> bool {
        let age = Utc::now() - self.calculated_at;
        age.num_minutes() > max_age_minutes
    }
}

/// Walker's live location
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct WalkerLocation {
    pub walker_id: UserId,
    pub latitude: f64,
    pub longitude: f64,
    pub accuracy_meters: Option<f64>,
    pub heading: Option<f64>,
    pub speed_mps: Option<f64>,
    pub is_on_duty: bool,
    pub updated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl WalkerLocation {
    /// Check if location is stale (older than given minutes)
    pub fn is_stale(&self, max_age_minutes: i64) -> bool {
        let age = Utc::now() - self.updated_at;
        age.num_minutes() > max_age_minutes
    }

    /// Get coordinates as a tuple
    pub fn coordinates(&self) -> (f64, f64) {
        (self.latitude, self.longitude)
    }
}

/// Input for updating walker location
#[derive(Debug, Clone, Deserialize)]
pub struct WalkerLocationUpdate {
    pub latitude: f64,
    pub longitude: f64,
    pub accuracy_meters: Option<f64>,
    pub heading: Option<f64>,
    pub speed_mps: Option<f64>,
    pub is_on_duty: bool,
}

/// Travel time between two points (calculated or cached)
#[derive(Debug, Clone, Serialize)]
pub struct TravelTime {
    pub origin_location_id: Option<LocationId>,
    pub destination_location_id: LocationId,
    pub travel_minutes: i32,
    pub distance_meters: i32,
    pub is_cached: bool,
    pub is_live_location: bool,
}

/// Availability slot with travel time information
#[derive(Debug, Clone, Serialize)]
pub struct AvailabilitySlot {
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub travel_minutes: Option<i32>,
    pub travel_from: Option<String>, // Description of origin
    pub is_tight: bool,              // True if buffer is minimal
    pub warning: Option<String>,
}
