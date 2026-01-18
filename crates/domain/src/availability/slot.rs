use chrono::{DateTime, Utc};
use shared::types::{BookingId, BlockId, DurationMinutes, LocationId};

/// An available time slot that can be booked
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AvailableSlot {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    /// Travel time from previous appointment (if any)
    pub travel_from_previous: Option<DurationMinutes>,
    /// Confidence level based on travel time source
    pub confidence: SlotConfidence,
}

impl AvailableSlot {
    pub fn new(start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        Self {
            start,
            end,
            travel_from_previous: None,
            confidence: SlotConfidence::High,
        }
    }

    pub fn with_travel(mut self, travel_minutes: DurationMinutes, confidence: SlotConfidence) -> Self {
        self.travel_from_previous = Some(travel_minutes);
        self.confidence = confidence;
        self
    }

    pub fn duration_minutes(&self) -> i64 {
        (self.end - self.start).num_minutes()
    }
}

/// Confidence level for slot availability
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SlotConfidence {
    /// Travel time from API/cache, high confidence
    #[default]
    High,
    /// Travel time estimated (Haversine fallback), medium confidence
    Medium,
    /// No travel time data, using default buffer
    Low,
}

/// An existing booking that occupies time
#[derive(Debug, Clone)]
pub struct BookingSlot {
    pub id: BookingId,
    pub location_id: LocationId,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

impl BookingSlot {
    pub fn new(
        id: BookingId,
        location_id: LocationId,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            location_id,
            start,
            end,
        }
    }

    pub fn duration_minutes(&self) -> i64 {
        (self.end - self.start).num_minutes()
    }
}

/// A blocked time period (lunch, personal, etc.)
#[derive(Debug, Clone)]
pub struct BlockSlot {
    pub id: BlockId,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

impl BlockSlot {
    pub fn new(id: BlockId, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        Self { id, start, end }
    }

    pub fn duration_minutes(&self) -> i64 {
        (self.end - self.start).num_minutes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use uuid::Uuid;

    #[test]
    fn test_available_slot_duration() {
        let start = Utc.with_ymd_and_hms(2024, 6, 15, 10, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2024, 6, 15, 11, 0, 0).unwrap();
        let slot = AvailableSlot::new(start, end);
        assert_eq!(slot.duration_minutes(), 60);
    }

    #[test]
    fn test_booking_slot_duration() {
        let start = Utc.with_ymd_and_hms(2024, 6, 15, 10, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2024, 6, 15, 10, 30, 0).unwrap();
        let slot = BookingSlot::new(
            BookingId::from_uuid(Uuid::new_v4()),
            LocationId::from_uuid(Uuid::new_v4()),
            start,
            end,
        );
        assert_eq!(slot.duration_minutes(), 30);
    }
}
