use chrono::{DateTime, Duration, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Duration in minutes (for service durations, travel times, buffers)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub struct DurationMinutes(i32);

impl DurationMinutes {
    pub fn new(minutes: i32) -> Self {
        Self(minutes.max(0))
    }

    pub fn as_minutes(&self) -> i32 {
        self.0
    }

    pub fn as_chrono_duration(&self) -> Duration {
        Duration::minutes(self.0 as i64)
    }

    pub fn zero() -> Self {
        Self(0)
    }

    pub fn is_zero(&self) -> bool {
        self.0 == 0
    }
}

impl From<i32> for DurationMinutes {
    fn from(minutes: i32) -> Self {
        Self::new(minutes)
    }
}

impl fmt::Display for DurationMinutes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 >= 60 {
            let hours = self.0 / 60;
            let mins = self.0 % 60;
            if mins == 0 {
                write!(f, "{}h", hours)
            } else {
                write!(f, "{}h {}m", hours, mins)
            }
        } else {
            write!(f, "{}m", self.0)
        }
    }
}

/// A time slot with start and end times in UTC
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeSlot {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

impl TimeSlot {
    /// Create a new time slot, validating that end is after start
    pub fn new(start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Self, TimeSlotError> {
        if end <= start {
            return Err(TimeSlotError::EndBeforeStart { start, end });
        }
        Ok(Self { start, end })
    }

    /// Create a time slot from a start time and duration
    pub fn from_start_and_duration(start: DateTime<Utc>, duration: DurationMinutes) -> Self {
        Self {
            start,
            end: start + duration.as_chrono_duration(),
        }
    }

    /// Get the duration of this time slot in minutes
    pub fn duration_minutes(&self) -> i64 {
        (self.end - self.start).num_minutes()
    }

    /// Check if this time slot overlaps with another
    pub fn overlaps(&self, other: &TimeSlot) -> bool {
        self.start < other.end && other.start < self.end
    }

    /// Check if this time slot contains a specific instant
    pub fn contains(&self, instant: DateTime<Utc>) -> bool {
        self.start <= instant && instant < self.end
    }

    /// Check if this time slot fully contains another
    pub fn contains_slot(&self, other: &TimeSlot) -> bool {
        self.start <= other.start && other.end <= self.end
    }

    /// Get the gap between this slot and another (if this ends before other starts)
    pub fn gap_to(&self, other: &TimeSlot) -> Option<TimeSlot> {
        if self.end <= other.start {
            Some(TimeSlot {
                start: self.end,
                end: other.start,
            })
        } else {
            None
        }
    }

    /// Extend the start time backwards by the given duration
    pub fn extend_start(&self, duration: DurationMinutes) -> Self {
        Self {
            start: self.start - duration.as_chrono_duration(),
            end: self.end,
        }
    }

    /// Extend the end time forwards by the given duration
    pub fn extend_end(&self, duration: DurationMinutes) -> Self {
        Self {
            start: self.start,
            end: self.end + duration.as_chrono_duration(),
        }
    }
}

/// Error for invalid time slot creation
#[derive(Debug, Clone, thiserror::Error)]
pub enum TimeSlotError {
    #[error("End time ({end}) must be after start time ({start})")]
    EndBeforeStart {
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    },
}

/// Working hours for a specific day of the week
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkingHours {
    /// Day of week (0 = Sunday, 6 = Saturday)
    pub day_of_week: u8,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
    pub is_active: bool,
}

impl WorkingHours {
    pub fn new(day_of_week: u8, start_time: NaiveTime, end_time: NaiveTime) -> Self {
        Self {
            day_of_week,
            start_time,
            end_time,
            is_active: true,
        }
    }

    /// Get the duration of working hours in minutes
    pub fn duration_minutes(&self) -> i64 {
        let duration = self.end_time.signed_duration_since(self.start_time);
        duration.num_minutes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_duration_minutes() {
        let d = DurationMinutes::new(90);
        assert_eq!(d.as_minutes(), 90);
        assert_eq!(d.to_string(), "1h 30m");

        let d2 = DurationMinutes::new(30);
        assert_eq!(d2.to_string(), "30m");

        let d3 = DurationMinutes::new(120);
        assert_eq!(d3.to_string(), "2h");
    }

    #[test]
    fn test_duration_negative_clamped() {
        let d = DurationMinutes::new(-10);
        assert_eq!(d.as_minutes(), 0);
    }

    #[test]
    fn test_time_slot_creation() {
        let start = Utc.with_ymd_and_hms(2024, 6, 15, 10, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2024, 6, 15, 11, 0, 0).unwrap();
        let slot = TimeSlot::new(start, end).unwrap();
        assert_eq!(slot.duration_minutes(), 60);
    }

    #[test]
    fn test_time_slot_invalid() {
        let start = Utc.with_ymd_and_hms(2024, 6, 15, 11, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2024, 6, 15, 10, 0, 0).unwrap();
        assert!(TimeSlot::new(start, end).is_err());
    }

    #[test]
    fn test_time_slot_overlaps() {
        let slot1 = TimeSlot::new(
            Utc.with_ymd_and_hms(2024, 6, 15, 10, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2024, 6, 15, 11, 0, 0).unwrap(),
        )
        .unwrap();

        let slot2 = TimeSlot::new(
            Utc.with_ymd_and_hms(2024, 6, 15, 10, 30, 0).unwrap(),
            Utc.with_ymd_and_hms(2024, 6, 15, 11, 30, 0).unwrap(),
        )
        .unwrap();

        let slot3 = TimeSlot::new(
            Utc.with_ymd_and_hms(2024, 6, 15, 11, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2024, 6, 15, 12, 0, 0).unwrap(),
        )
        .unwrap();

        assert!(slot1.overlaps(&slot2)); // Overlapping
        assert!(!slot1.overlaps(&slot3)); // Adjacent, not overlapping
    }

    #[test]
    fn test_time_slot_gap() {
        let slot1 = TimeSlot::new(
            Utc.with_ymd_and_hms(2024, 6, 15, 10, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2024, 6, 15, 11, 0, 0).unwrap(),
        )
        .unwrap();

        let slot2 = TimeSlot::new(
            Utc.with_ymd_and_hms(2024, 6, 15, 12, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2024, 6, 15, 13, 0, 0).unwrap(),
        )
        .unwrap();

        let gap = slot1.gap_to(&slot2).unwrap();
        assert_eq!(gap.duration_minutes(), 60);
    }
}
