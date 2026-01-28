//! Traffic-aware availability calculations
//!
//! Provides peak hour multipliers and cache TTL configurations for
//! traffic-aware slot blocking.

use chrono::{DateTime, Timelike, Utc};

/// Traffic configuration for availability calculations
#[derive(Debug, Clone)]
pub struct TrafficConfig {
    /// Peak hour ranges (start_hour, end_hour) in 24h format
    /// Default: [(7,9), (16,18)] for morning and evening rush
    pub peak_hours: Vec<(u8, u8)>,
    /// Multiplier applied during peak hours (e.g., 1.3 = 30% longer travel times)
    pub peak_multiplier: f32,
    /// Cache TTL during peak hours (in minutes)
    pub cache_ttl_peak_minutes: i64,
    /// Cache TTL during off-peak hours (in minutes)
    pub cache_ttl_offpeak_minutes: i64,
}

impl Default for TrafficConfig {
    fn default() -> Self {
        Self {
            peak_hours: vec![(7, 9), (16, 18)],
            peak_multiplier: 1.3,
            cache_ttl_peak_minutes: 240,     // 4 hours during peak
            cache_ttl_offpeak_minutes: 1440, // 24 hours off-peak
        }
    }
}

impl TrafficConfig {
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if a given time falls within peak hours
    pub fn is_peak_hour(&self, time: DateTime<Utc>) -> bool {
        let hour = time.hour() as u8;
        self.peak_hours
            .iter()
            .any(|(start, end)| hour >= *start && hour < *end)
    }

    /// Get the traffic multiplier for a given time
    pub fn get_multiplier(&self, time: DateTime<Utc>) -> f32 {
        if self.is_peak_hour(time) {
            self.peak_multiplier
        } else {
            1.0
        }
    }

    /// Get the appropriate cache TTL for a given time
    pub fn get_cache_ttl_minutes(&self, time: DateTime<Utc>) -> i64 {
        if self.is_peak_hour(time) {
            self.cache_ttl_peak_minutes
        } else {
            self.cache_ttl_offpeak_minutes
        }
    }

    /// Apply traffic multiplier to a travel time in minutes
    pub fn adjust_travel_time(&self, base_minutes: i32, time: DateTime<Utc>) -> i32 {
        let multiplier = self.get_multiplier(time);
        (base_minutes as f32 * multiplier).ceil() as i32
    }
}

/// Calculate traffic-adjusted travel time
pub fn get_traffic_multiplier(time: DateTime<Utc>) -> f32 {
    let hour = time.hour();
    if (7..=9).contains(&hour) || (16..=18).contains(&hour) {
        1.3 // 30% longer during rush hour
    } else {
        1.0
    }
}

/// Determine slot confidence based on data freshness and traffic conditions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TravelConfidence {
    /// Travel time from fresh API/cache data
    High,
    /// Travel time from stale cache (>4 hours old during peak)
    Medium,
    /// Estimated using Haversine distance
    Low,
    /// No travel time data, using default buffer
    Unknown,
}

impl TravelConfidence {
    /// Get confidence based on cache age and traffic conditions
    pub fn from_cache_age(age_minutes: i64, is_peak: bool, traffic_config: &TrafficConfig) -> Self {
        let ttl = if is_peak {
            traffic_config.cache_ttl_peak_minutes
        } else {
            traffic_config.cache_ttl_offpeak_minutes
        };

        if age_minutes <= ttl / 2 {
            TravelConfidence::High
        } else if age_minutes <= ttl {
            TravelConfidence::Medium
        } else {
            TravelConfidence::Low
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_is_peak_hour() {
        let config = TrafficConfig::default();

        // 8am - peak
        let peak_morning = Utc.with_ymd_and_hms(2024, 6, 15, 8, 30, 0).unwrap();
        assert!(config.is_peak_hour(peak_morning));

        // 5pm - peak
        let peak_evening = Utc.with_ymd_and_hms(2024, 6, 15, 17, 0, 0).unwrap();
        assert!(config.is_peak_hour(peak_evening));

        // 2pm - off-peak
        let offpeak = Utc.with_ymd_and_hms(2024, 6, 15, 14, 0, 0).unwrap();
        assert!(!config.is_peak_hour(offpeak));

        // 10am - off-peak
        let late_morning = Utc.with_ymd_and_hms(2024, 6, 15, 10, 0, 0).unwrap();
        assert!(!config.is_peak_hour(late_morning));
    }

    #[test]
    fn test_traffic_multiplier() {
        let config = TrafficConfig::default();

        let peak_time = Utc.with_ymd_and_hms(2024, 6, 15, 8, 0, 0).unwrap();
        let offpeak_time = Utc.with_ymd_and_hms(2024, 6, 15, 14, 0, 0).unwrap();

        assert_eq!(config.get_multiplier(peak_time), 1.3);
        assert_eq!(config.get_multiplier(offpeak_time), 1.0);
    }

    #[test]
    fn test_adjust_travel_time() {
        let config = TrafficConfig::default();

        let peak_time = Utc.with_ymd_and_hms(2024, 6, 15, 8, 0, 0).unwrap();
        let offpeak_time = Utc.with_ymd_and_hms(2024, 6, 15, 14, 0, 0).unwrap();

        // 20 min base during peak = 26 min (20 * 1.3)
        assert_eq!(config.adjust_travel_time(20, peak_time), 26);

        // 20 min base off-peak = 20 min
        assert_eq!(config.adjust_travel_time(20, offpeak_time), 20);
    }

    #[test]
    fn test_travel_confidence() {
        let config = TrafficConfig::default();

        // During peak (4 hour TTL)
        assert_eq!(
            TravelConfidence::from_cache_age(60, true, &config),
            TravelConfidence::High
        );
        assert_eq!(
            TravelConfidence::from_cache_age(180, true, &config),
            TravelConfidence::Medium
        );
        assert_eq!(
            TravelConfidence::from_cache_age(300, true, &config),
            TravelConfidence::Low
        );

        // Off-peak (24 hour TTL)
        assert_eq!(
            TravelConfidence::from_cache_age(60, false, &config),
            TravelConfidence::High
        );
        assert_eq!(
            TravelConfidence::from_cache_age(800, false, &config),
            TravelConfidence::Medium
        );
    }
}
