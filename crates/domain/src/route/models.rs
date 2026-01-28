//! Route optimization models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared::types::{BookingId, LocationId};

/// A stop in an optimized route
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteStop {
    /// Position in the optimized route (1-indexed)
    pub sequence: usize,
    /// Booking ID for this stop
    pub booking_id: BookingId,
    /// Location ID for this stop
    pub location_id: LocationId,
    /// Customer name (for display)
    pub customer_name: String,
    /// Address (for display)
    pub address: String,
    /// Scheduled arrival time
    pub arrival_time: DateTime<Utc>,
    /// Scheduled departure time (after service completion)
    pub departure_time: DateTime<Utc>,
    /// Travel time from previous stop in minutes
    pub travel_from_previous_minutes: i32,
    /// Service duration in minutes
    pub service_duration_minutes: i32,
}

/// An optimized route for a walker's day
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizedRoute {
    /// Ordered list of stops
    pub stops: Vec<RouteStop>,
    /// Total travel time in minutes
    pub total_travel_minutes: i32,
    /// Total distance in meters
    pub total_distance_meters: i32,
    /// Travel time saved vs chronological order (in minutes)
    pub savings_vs_chronological: i32,
    /// Whether the route has been optimized
    pub is_optimized: bool,
}

impl OptimizedRoute {
    /// Create an empty route
    pub fn empty() -> Self {
        Self {
            stops: Vec::new(),
            total_travel_minutes: 0,
            total_distance_meters: 0,
            savings_vs_chronological: 0,
            is_optimized: false,
        }
    }

    /// Get total number of stops
    pub fn num_stops(&self) -> usize {
        self.stops.len()
    }

    /// Get the first stop
    pub fn first_stop(&self) -> Option<&RouteStop> {
        self.stops.first()
    }

    /// Get the last stop
    pub fn last_stop(&self) -> Option<&RouteStop> {
        self.stops.last()
    }

    /// Calculate total service time in minutes
    pub fn total_service_minutes(&self) -> i32 {
        self.stops.iter().map(|s| s.service_duration_minutes).sum()
    }

    /// Calculate total working time (service + travel) in minutes
    pub fn total_working_minutes(&self) -> i32 {
        self.total_service_minutes() + self.total_travel_minutes
    }
}

/// Input booking for route optimization
#[derive(Debug, Clone)]
pub struct RouteBooking {
    pub booking_id: BookingId,
    pub location_id: LocationId,
    pub customer_name: String,
    pub address: String,
    pub scheduled_start: DateTime<Utc>,
    pub scheduled_end: DateTime<Utc>,
    pub latitude: f64,
    pub longitude: f64,
}

impl RouteBooking {
    pub fn duration_minutes(&self) -> i32 {
        (self.scheduled_end - self.scheduled_start).num_minutes() as i32
    }
}

/// A savings entry for the Clark-Wright algorithm
#[derive(Debug, Clone)]
pub(crate) struct Savings {
    pub from_idx: usize,
    pub to_idx: usize,
    pub savings_minutes: i32,
}

impl Savings {
    pub fn new(from_idx: usize, to_idx: usize, savings_minutes: i32) -> Self {
        Self {
            from_idx,
            to_idx,
            savings_minutes,
        }
    }
}
