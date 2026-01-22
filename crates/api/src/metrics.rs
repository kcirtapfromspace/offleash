//! Prometheus metrics for booking operations
//!
//! This module defines and records metrics for observability of the booking system.

use metrics::{counter, gauge, histogram};
use std::time::Instant;

/// Metric names for booking operations
pub mod names {
    pub const RECURRING_SERIES_CREATED: &str = "recurring_booking_series_created_total";
    pub const RECURRING_INSTANCES_CREATED: &str = "recurring_booking_instances_created_total";
    pub const RECURRING_CREATION_DURATION: &str = "recurring_booking_creation_duration_seconds";
    pub const ACTIVE_SERIES_COUNT: &str = "recurring_booking_active_series_count";
    pub const BOOKING_CONFLICTS: &str = "recurring_booking_conflicts_total";
    pub const IDEMPOTENCY_HITS: &str = "recurring_booking_idempotency_hits_total";
}

/// Record a successful recurring series creation
pub fn record_series_created(org_id: &str, instances_count: i64) {
    counter!(names::RECURRING_SERIES_CREATED, "status" => "success", "org_id" => org_id.to_string())
        .increment(1);
    counter!(names::RECURRING_INSTANCES_CREATED, "org_id" => org_id.to_string())
        .increment(instances_count as u64);
}

/// Record a failed recurring series creation
pub fn record_series_creation_failed(org_id: &str, reason: &str) {
    counter!(names::RECURRING_SERIES_CREATED, "status" => "failure", "org_id" => org_id.to_string(), "reason" => reason.to_string())
        .increment(1);
}

/// Record conflicts detected during booking creation
pub fn record_conflicts(org_id: &str, count: i64) {
    counter!(names::BOOKING_CONFLICTS, "org_id" => org_id.to_string())
        .increment(count as u64);
}

/// Record an idempotency cache hit (duplicate request)
pub fn record_idempotency_hit(org_id: &str) {
    counter!(names::IDEMPOTENCY_HITS, "org_id" => org_id.to_string())
        .increment(1);
}

/// Record the duration of a recurring booking creation operation
pub fn record_creation_duration(org_id: &str, duration_secs: f64) {
    histogram!(names::RECURRING_CREATION_DURATION, "org_id" => org_id.to_string())
        .record(duration_secs);
}

/// Update the active series count gauge for an organization
pub fn set_active_series_count(org_id: &str, count: f64) {
    gauge!(names::ACTIVE_SERIES_COUNT, "org_id" => org_id.to_string())
        .set(count);
}

/// Helper struct for timing operations
pub struct Timer {
    start: Instant,
    org_id: String,
}

impl Timer {
    pub fn start(org_id: &str) -> Self {
        Self {
            start: Instant::now(),
            org_id: org_id.to_string(),
        }
    }

    pub fn record(self) {
        let duration = self.start.elapsed().as_secs_f64();
        record_creation_duration(&self.org_id, duration);
    }
}

/// Initialize the Prometheus metrics exporter
/// Returns a handle to the PrometheusBuilder that can be used to render metrics
pub fn init_metrics() -> metrics_exporter_prometheus::PrometheusHandle {
    let builder = metrics_exporter_prometheus::PrometheusBuilder::new();
    builder
        .install_recorder()
        .expect("Failed to install Prometheus metrics recorder")
}
