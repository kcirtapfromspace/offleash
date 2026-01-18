use chrono::{DateTime, Duration, NaiveDate, NaiveTime, TimeZone, Utc};
use shared::types::{Coordinates, DurationMinutes, LocationId};
use std::collections::HashMap;

use super::{
    config::AvailabilityConfig,
    slot::{AvailableSlot, BlockSlot, BookingSlot, SlotConfidence},
};

/// Working hours for a specific day
#[derive(Debug, Clone)]
pub struct DayHours {
    pub start: NaiveTime,
    pub end: NaiveTime,
}

/// Travel time matrix between locations
#[derive(Debug, Clone, Default)]
pub struct TravelTimeMatrix {
    /// Key: (origin_id, destination_id) -> travel time in minutes
    times: HashMap<(LocationId, LocationId), DurationMinutes>,
}

impl TravelTimeMatrix {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(
        &mut self,
        origin: LocationId,
        destination: LocationId,
        duration: DurationMinutes,
    ) {
        self.times.insert((origin, destination), duration);
    }

    pub fn get(&self, origin: LocationId, destination: LocationId) -> Option<DurationMinutes> {
        self.times.get(&(origin, destination)).copied()
    }

    pub fn get_or_default(
        &self,
        origin: LocationId,
        destination: LocationId,
        default: DurationMinutes,
    ) -> DurationMinutes {
        self.get(origin, destination).unwrap_or(default)
    }
}

/// Location with coordinates for travel time estimation
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct LocationWithCoords {
    pub id: LocationId,
    pub coords: Coordinates,
}

/// Pure availability calculation engine
///
/// This is the core business logic for determining available booking slots.
/// It has NO I/O dependencies and is fully deterministic for a given input.
pub struct AvailabilityEngine;

impl AvailabilityEngine {
    /// Calculate available slots for a given date
    ///
    /// # Algorithm Overview:
    /// 1. Get working hours for the requested day
    /// 2. Generate all potential time slots within working hours
    /// 3. Remove slots that conflict with existing bookings
    /// 4. Remove slots that conflict with blocks
    /// 5. Adjust slots based on travel time from previous appointments
    /// 6. Return remaining valid slots
    #[allow(clippy::too_many_arguments)]
    pub fn calculate_slots(
        working_hours: Option<&DayHours>,
        existing_bookings: &[BookingSlot],
        blocks: &[BlockSlot],
        travel_times: &TravelTimeMatrix,
        target_location: LocationId,
        service_duration_minutes: i32,
        date: NaiveDate,
        timezone: &str,
        config: &AvailabilityConfig,
    ) -> Vec<AvailableSlot> {
        // If no working hours for this day, return empty
        let Some(hours) = working_hours else {
            return Vec::new();
        };

        // Parse timezone
        let tz: chrono_tz::Tz = timezone.parse().unwrap_or(chrono_tz::UTC);

        // Convert working hours to UTC for this date
        let work_start_local = date.and_time(hours.start);
        let work_end_local = date.and_time(hours.end);

        let work_start_utc = tz
            .from_local_datetime(&work_start_local)
            .single()
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(Utc::now);

        let work_end_utc = tz
            .from_local_datetime(&work_end_local)
            .single()
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(Utc::now);

        // Step 1: Generate potential slots
        let potential_slots = Self::generate_potential_slots(
            work_start_utc,
            work_end_utc,
            service_duration_minutes,
            config.slot_interval_minutes,
        );

        // Step 2: Filter out slots that conflict with bookings
        let after_booking_filter: Vec<_> = potential_slots
            .into_iter()
            .filter(|slot| !Self::conflicts_with_bookings(slot, existing_bookings))
            .collect();

        // Step 3: Filter out slots that conflict with blocks
        let after_block_filter: Vec<_> = after_booking_filter
            .into_iter()
            .filter(|slot| !Self::conflicts_with_blocks(slot, blocks))
            .collect();

        // Step 4: Apply travel time constraints
        Self::apply_travel_constraints(
            after_block_filter,
            existing_bookings,
            travel_times,
            target_location,
            config,
        )
    }

    /// Generate potential time slots within working hours
    fn generate_potential_slots(
        work_start: DateTime<Utc>,
        work_end: DateTime<Utc>,
        service_duration_minutes: i32,
        interval_minutes: i32,
    ) -> Vec<PotentialSlot> {
        let mut slots = Vec::new();
        let service_duration = Duration::minutes(service_duration_minutes as i64);
        let interval = Duration::minutes(interval_minutes as i64);

        let mut current = work_start;
        while current + service_duration <= work_end {
            slots.push(PotentialSlot {
                start: current,
                end: current + service_duration,
            });
            current += interval;
        }

        slots
    }

    /// Check if a potential slot conflicts with any existing booking
    fn conflicts_with_bookings(slot: &PotentialSlot, bookings: &[BookingSlot]) -> bool {
        bookings.iter().any(|booking| {
            // Overlap: slot starts before booking ends AND slot ends after booking starts
            slot.start < booking.end && slot.end > booking.start
        })
    }

    /// Check if a potential slot conflicts with any block
    fn conflicts_with_blocks(slot: &PotentialSlot, blocks: &[BlockSlot]) -> bool {
        blocks
            .iter()
            .any(|block| slot.start < block.end && slot.end > block.start)
    }

    /// Apply travel time constraints to filter and adjust slots
    fn apply_travel_constraints(
        slots: Vec<PotentialSlot>,
        bookings: &[BookingSlot],
        travel_times: &TravelTimeMatrix,
        target_location: LocationId,
        config: &AvailabilityConfig,
    ) -> Vec<AvailableSlot> {
        // Sort bookings by start time for efficient lookup
        let mut sorted_bookings = bookings.to_vec();
        sorted_bookings.sort_by_key(|b| b.start);

        let buffer = Duration::minutes(config.min_buffer_minutes as i64);
        let default_travel = DurationMinutes::new(config.default_travel_minutes);

        slots
            .into_iter()
            .filter_map(|slot| {
                // Find the booking that ends most recently before this slot
                let previous_booking = sorted_bookings.iter().rev().find(|b| b.end <= slot.start);

                // Find the booking that starts soonest after this slot
                let next_booking = sorted_bookings.iter().find(|b| b.start >= slot.end);

                // Calculate travel from previous
                let (travel_from_prev, confidence) = if let Some(prev) = previous_booking {
                    match travel_times.get(prev.location_id, target_location) {
                        Some(duration) => (duration, SlotConfidence::High),
                        None => (default_travel, SlotConfidence::Low),
                    }
                } else {
                    (DurationMinutes::zero(), SlotConfidence::High)
                };

                // Calculate travel to next
                let travel_to_next = if let Some(next) = next_booking {
                    travel_times
                        .get(target_location, next.location_id)
                        .unwrap_or(default_travel)
                } else {
                    DurationMinutes::zero()
                };

                // Validate timing constraints
                let travel_from_duration = travel_from_prev.as_chrono_duration();
                let travel_to_duration = travel_to_next.as_chrono_duration();

                // Check if there's enough time after previous booking
                if let Some(prev) = previous_booking {
                    let earliest_possible_start = prev.end + travel_from_duration + buffer;
                    if slot.start < earliest_possible_start {
                        return None;
                    }
                }

                // Check if there's enough time before next booking
                if let Some(next) = next_booking {
                    let latest_possible_end = next.start - travel_to_duration - buffer;
                    if slot.end > latest_possible_end {
                        return None;
                    }
                }

                let mut available_slot = AvailableSlot::new(slot.start, slot.end);
                if previous_booking.is_some() {
                    available_slot = available_slot.with_travel(travel_from_prev, confidence);
                }

                Some(available_slot)
            })
            .collect()
    }

    /// Estimate travel time between two locations using Haversine distance
    /// Used as fallback when API/cache data is unavailable
    pub fn estimate_travel_time(from: &Coordinates, to: &Coordinates) -> DurationMinutes {
        DurationMinutes::new(from.estimate_travel_minutes(to))
    }

    /// Find gaps in the schedule that could accommodate a booking
    pub fn find_schedule_gaps(
        work_start: DateTime<Utc>,
        work_end: DateTime<Utc>,
        bookings: &[BookingSlot],
        blocks: &[BlockSlot],
    ) -> Vec<ScheduleGap> {
        // Combine bookings and blocks into occupied intervals
        let mut occupied: Vec<(DateTime<Utc>, DateTime<Utc>)> = Vec::new();

        for booking in bookings {
            occupied.push((booking.start, booking.end));
        }
        for block in blocks {
            occupied.push((block.start, block.end));
        }

        // Sort by start time
        occupied.sort_by_key(|(start, _)| *start);

        // Merge overlapping intervals
        let merged = Self::merge_intervals(occupied);

        // Find gaps
        let mut gaps = Vec::new();
        let mut current = work_start;

        for (start, end) in merged {
            if start > current && start <= work_end {
                let gap_end = start.min(work_end);
                if gap_end > current {
                    gaps.push(ScheduleGap {
                        start: current,
                        end: gap_end,
                        duration_minutes: (gap_end - current).num_minutes(),
                    });
                }
            }
            current = end.max(current);
        }

        // Gap after last occupied interval
        if current < work_end {
            gaps.push(ScheduleGap {
                start: current,
                end: work_end,
                duration_minutes: (work_end - current).num_minutes(),
            });
        }

        gaps
    }

    /// Merge overlapping time intervals
    fn merge_intervals(
        mut intervals: Vec<(DateTime<Utc>, DateTime<Utc>)>,
    ) -> Vec<(DateTime<Utc>, DateTime<Utc>)> {
        if intervals.is_empty() {
            return Vec::new();
        }

        intervals.sort_by_key(|(start, _)| *start);
        let mut merged = vec![intervals[0]];

        for (start, end) in intervals.into_iter().skip(1) {
            let last = merged.last_mut().unwrap();
            if start <= last.1 {
                // Overlapping, extend the end
                last.1 = last.1.max(end);
            } else {
                merged.push((start, end));
            }
        }

        merged
    }
}

/// Internal type for slot generation before travel adjustments
#[derive(Debug, Clone)]
struct PotentialSlot {
    start: DateTime<Utc>,
    end: DateTime<Utc>,
}

/// A gap in the schedule that could be filled
#[derive(Debug, Clone)]
pub struct ScheduleGap {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub duration_minutes: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Timelike;
    use shared::types::{BlockId, BookingId, LocationId};
    use uuid::Uuid;

    fn make_booking(
        id: u32,
        start_hour: u32,
        start_min: u32,
        end_hour: u32,
        end_min: u32,
        location_id: u32,
    ) -> BookingSlot {
        let date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        BookingSlot {
            id: BookingId::from_uuid(Uuid::from_u128(id as u128)),
            location_id: LocationId::from_uuid(Uuid::from_u128(location_id as u128)),
            start: Utc.from_utc_datetime(&date.and_hms_opt(start_hour, start_min, 0).unwrap()),
            end: Utc.from_utc_datetime(&date.and_hms_opt(end_hour, end_min, 0).unwrap()),
        }
    }

    fn make_block(id: u32, start_hour: u32, end_hour: u32) -> BlockSlot {
        let date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        BlockSlot {
            id: BlockId::from_uuid(Uuid::from_u128(id as u128)),
            start: Utc.from_utc_datetime(&date.and_hms_opt(start_hour, 0, 0).unwrap()),
            end: Utc.from_utc_datetime(&date.and_hms_opt(end_hour, 0, 0).unwrap()),
        }
    }

    fn default_config() -> AvailabilityConfig {
        AvailabilityConfig {
            min_buffer_minutes: 15,
            default_travel_minutes: 20,
            slot_interval_minutes: 30,
            max_advance_days: 30,
            min_notice_hours: 2,
        }
    }

    #[test]
    fn test_empty_day_returns_all_slots() {
        let working_hours = DayHours {
            start: NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            end: NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
        };

        let date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let service_duration = 60;
        let target_location = LocationId::from_uuid(Uuid::from_u128(1));

        let slots = AvailabilityEngine::calculate_slots(
            Some(&working_hours),
            &[],
            &[],
            &TravelTimeMatrix::new(),
            target_location,
            service_duration,
            date,
            "UTC",
            &default_config(),
        );

        // With 30-min intervals from 9am-5pm for 60-min service
        // Slots: 9:00, 9:30, 10:00, ..., 15:30, 16:00 = 15 slots
        assert_eq!(slots.len(), 15);
        assert_eq!(slots[0].start.hour(), 9);
        assert_eq!(slots[0].start.minute(), 0);
    }

    #[test]
    fn test_no_working_hours_returns_empty() {
        let date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let target_location = LocationId::from_uuid(Uuid::from_u128(1));

        let slots = AvailabilityEngine::calculate_slots(
            None,
            &[],
            &[],
            &TravelTimeMatrix::new(),
            target_location,
            60,
            date,
            "UTC",
            &default_config(),
        );

        assert!(slots.is_empty());
    }

    #[test]
    fn test_booking_blocks_overlapping_slots() {
        let working_hours = DayHours {
            start: NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            end: NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
        };

        let date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let target_location = LocationId::from_uuid(Uuid::from_u128(1));

        // Existing booking from 10:00 - 11:00
        let bookings = vec![make_booking(1, 10, 0, 11, 0, 2)];

        let slots = AvailabilityEngine::calculate_slots(
            Some(&working_hours),
            &bookings,
            &[],
            &TravelTimeMatrix::new(),
            target_location,
            60,
            date,
            "UTC",
            &default_config(),
        );

        // Slots starting at 9:30, 10:00, 10:30 should be blocked
        // because a 60-min service would overlap with 10:00-11:00
        let blocked_count = slots
            .iter()
            .filter(|s| s.start.hour() == 10 || (s.start.hour() == 9 && s.start.minute() == 30))
            .count();
        assert_eq!(blocked_count, 0);
    }

    #[test]
    fn test_block_removes_slots() {
        let working_hours = DayHours {
            start: NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            end: NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
        };

        let date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let target_location = LocationId::from_uuid(Uuid::from_u128(1));

        // Block from 12:00 - 13:00 (lunch)
        let blocks = vec![make_block(1, 12, 13)];

        let slots = AvailabilityEngine::calculate_slots(
            Some(&working_hours),
            &[],
            &blocks,
            &TravelTimeMatrix::new(),
            target_location,
            60,
            date,
            "UTC",
            &default_config(),
        );

        // No slots should overlap with 12:00-13:00
        let overlapping = slots
            .iter()
            .filter(|s| s.start.hour() == 12 || (s.start.hour() == 11 && s.start.minute() == 30))
            .count();
        assert_eq!(overlapping, 0);
    }

    #[test]
    fn test_travel_time_removes_insufficient_gaps() {
        let working_hours = DayHours {
            start: NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            end: NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
        };

        let date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let target_location = LocationId::from_uuid(Uuid::from_u128(1));
        let location_2 = LocationId::from_uuid(Uuid::from_u128(2));

        // Existing booking from 10:00 - 11:00 at location 2
        let bookings = vec![make_booking(1, 10, 0, 11, 0, 2)];

        // Travel time from location 2 to location 1 is 30 minutes
        let mut travel_times = TravelTimeMatrix::new();
        travel_times.insert(location_2, target_location, DurationMinutes::new(30));

        let config = AvailabilityConfig {
            min_buffer_minutes: 15,
            default_travel_minutes: 20,
            slot_interval_minutes: 30,
            max_advance_days: 30,
            min_notice_hours: 2,
        };

        let slots = AvailabilityEngine::calculate_slots(
            Some(&working_hours),
            &bookings,
            &[],
            &travel_times,
            target_location,
            60,
            date,
            "UTC",
            &config,
        );

        // After 11:00 booking, need 30 min travel + 15 min buffer = 45 min
        // So earliest available slot start is 11:45, rounded to 12:00 (30 min intervals)
        let first_after_booking = slots.iter().find(|s| s.start.hour() >= 11);

        if let Some(slot) = first_after_booking {
            // Should start at or after 11:45 (rounded to 12:00)
            let minutes_after_eleven = (slot.start.hour() - 11) * 60 + slot.start.minute();
            assert!(
                minutes_after_eleven >= 45,
                "Slot should start at least 45 min after 11:00"
            );
        }
    }

    #[test]
    fn test_find_schedule_gaps() {
        let date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let work_start = Utc.from_utc_datetime(&date.and_hms_opt(9, 0, 0).unwrap());
        let work_end = Utc.from_utc_datetime(&date.and_hms_opt(17, 0, 0).unwrap());

        let bookings = vec![
            make_booking(1, 10, 0, 11, 0, 1),
            make_booking(2, 14, 0, 15, 0, 1),
        ];

        let gaps = AvailabilityEngine::find_schedule_gaps(work_start, work_end, &bookings, &[]);

        // Should have 3 gaps: 9-10, 11-14, 15-17
        assert_eq!(gaps.len(), 3);
        assert_eq!(gaps[0].duration_minutes, 60); // 9-10
        assert_eq!(gaps[1].duration_minutes, 180); // 11-14
        assert_eq!(gaps[2].duration_minutes, 120); // 15-17
    }

    #[test]
    fn test_merge_overlapping_intervals() {
        let date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let intervals = vec![
            (
                Utc.from_utc_datetime(&date.and_hms_opt(10, 0, 0).unwrap()),
                Utc.from_utc_datetime(&date.and_hms_opt(11, 0, 0).unwrap()),
            ),
            (
                Utc.from_utc_datetime(&date.and_hms_opt(10, 30, 0).unwrap()),
                Utc.from_utc_datetime(&date.and_hms_opt(12, 0, 0).unwrap()),
            ),
        ];

        let merged = AvailabilityEngine::merge_intervals(intervals);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].0.hour(), 10);
        assert_eq!(merged[0].1.hour(), 12);
    }
}
