use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared::types::{Coordinates, LocationId, UserId};
use sqlx::FromRow;

/// Location database model
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Location {
    pub id: LocationId,
    pub user_id: UserId,
    pub name: String,
    pub address: String,
    pub city: String,
    pub state: String,
    pub zip_code: String,
    pub latitude: f64,
    pub longitude: f64,
    pub notes: Option<String>,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Location {
    /// Get the coordinates for this location
    pub fn coordinates(&self) -> Coordinates {
        Coordinates::new_unchecked(self.latitude, self.longitude)
    }

    /// Get the full formatted address
    pub fn full_address(&self) -> String {
        format!(
            "{}, {}, {} {}",
            self.address, self.city, self.state, self.zip_code
        )
    }
}

/// Input for creating a new location
#[derive(Debug, Clone, Deserialize)]
pub struct CreateLocation {
    pub user_id: UserId,
    pub name: String,
    pub address: String,
    pub city: String,
    pub state: String,
    pub zip_code: String,
    pub latitude: f64,
    pub longitude: f64,
    pub notes: Option<String>,
    pub is_default: bool,
}

/// Input for updating a location
#[derive(Debug, Clone, Deserialize, Default)]
pub struct UpdateLocation {
    pub name: Option<String>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub zip_code: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub notes: Option<String>,
    pub is_default: Option<bool>,
}
