/// Configuration for availability calculations
#[derive(Debug, Clone)]
pub struct AvailabilityConfig {
    /// Minimum gap between appointments (buffer time in minutes)
    pub min_buffer_minutes: i32,
    /// Default travel time when no cache/API data available
    pub default_travel_minutes: i32,
    /// Slot interval for generating time slots (e.g., 15 or 30 minutes)
    pub slot_interval_minutes: i32,
    /// Maximum advance booking days
    pub max_advance_days: i32,
    /// Minimum notice hours for booking
    pub min_notice_hours: i32,
}

impl Default for AvailabilityConfig {
    fn default() -> Self {
        Self {
            min_buffer_minutes: 15,
            default_travel_minutes: 20,
            slot_interval_minutes: 30,
            max_advance_days: 30,
            min_notice_hours: 2,
        }
    }
}

impl AvailabilityConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_buffer(mut self, minutes: i32) -> Self {
        self.min_buffer_minutes = minutes;
        self
    }

    pub fn with_default_travel(mut self, minutes: i32) -> Self {
        self.default_travel_minutes = minutes;
        self
    }

    pub fn with_slot_interval(mut self, minutes: i32) -> Self {
        self.slot_interval_minutes = minutes;
        self
    }

    pub fn with_max_advance_days(mut self, days: i32) -> Self {
        self.max_advance_days = days;
        self
    }

    pub fn with_min_notice_hours(mut self, hours: i32) -> Self {
        self.min_notice_hours = hours;
        self
    }
}
