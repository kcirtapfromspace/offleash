//! Route optimization module
//!
//! Provides route optimization for walker schedules using the Clark-Wright
//! savings algorithm for TSP approximation.

pub mod models;
mod optimizer;

pub use models::{OptimizedRoute, RouteBooking, RouteStop};
pub use optimizer::RouteOptimizer;
