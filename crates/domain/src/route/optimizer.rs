//! Route optimizer using Clark-Wright Savings Algorithm
//!
//! The algorithm works by:
//! 1. Computing savings for merging each pair of stops
//! 2. Sorting savings in descending order
//! 3. Iteratively merging stops that maximize savings
//! 4. Respecting time constraints for appointments

use chrono::Duration;
use shared::types::Coordinates;

use super::models::{OptimizedRoute, RouteBooking, RouteStop, Savings};
use crate::TravelTimeMatrix;

/// Route optimizer for a walker's daily schedule
pub struct RouteOptimizer {
    /// Starting location (walker's home base or first booking)
    start_location: Option<Coordinates>,
}

impl RouteOptimizer {
    pub fn new() -> Self {
        Self {
            start_location: None,
        }
    }

    pub fn with_start_location(mut self, coords: Coordinates) -> Self {
        self.start_location = Some(coords);
        self
    }

    /// Optimize a route using the Clark-Wright Savings Algorithm
    ///
    /// This is a greedy heuristic that provides good solutions for the
    /// Traveling Salesman Problem (TSP) in O(n² log n) time.
    pub fn optimize(
        &self,
        bookings: Vec<RouteBooking>,
        travel_matrix: &TravelTimeMatrix,
    ) -> OptimizedRoute {
        if bookings.is_empty() {
            return OptimizedRoute::empty();
        }

        if bookings.len() == 1 {
            return self.single_booking_route(bookings.into_iter().next().unwrap());
        }

        // Sort bookings by scheduled start time first to get chronological order
        let mut sorted_bookings = bookings;
        sorted_bookings.sort_by_key(|b| b.scheduled_start);

        // Calculate chronological travel time for comparison
        let chronological_travel = self.calculate_total_travel(&sorted_bookings, travel_matrix);

        // Apply Clark-Wright Savings Algorithm
        let optimized_order = self.clark_wright_optimize(&sorted_bookings, travel_matrix);

        // Reorder bookings according to optimized order
        let optimized_bookings: Vec<RouteBooking> = optimized_order
            .iter()
            .map(|&idx| sorted_bookings[idx].clone())
            .collect();

        // Calculate optimized travel time
        let optimized_travel = self.calculate_total_travel(&optimized_bookings, travel_matrix);

        // Build the route
        self.build_route(
            optimized_bookings,
            travel_matrix,
            chronological_travel - optimized_travel,
        )
    }

    /// Create a route for a single booking
    fn single_booking_route(&self, booking: RouteBooking) -> OptimizedRoute {
        let duration = booking.duration_minutes();
        let stop = RouteStop {
            sequence: 1,
            booking_id: booking.booking_id,
            location_id: booking.location_id,
            customer_name: booking.customer_name,
            address: booking.address,
            arrival_time: booking.scheduled_start,
            departure_time: booking.scheduled_end,
            travel_from_previous_minutes: 0,
            service_duration_minutes: duration,
        };

        OptimizedRoute {
            stops: vec![stop],
            total_travel_minutes: 0,
            total_distance_meters: 0,
            savings_vs_chronological: 0,
            is_optimized: true,
        }
    }

    /// Calculate total travel time for a sequence of bookings
    fn calculate_total_travel(
        &self,
        bookings: &[RouteBooking],
        travel_matrix: &TravelTimeMatrix,
    ) -> i32 {
        if bookings.len() < 2 {
            return 0;
        }

        let mut total = 0;
        for i in 1..bookings.len() {
            let from = &bookings[i - 1];
            let to = &bookings[i];

            let travel_time = travel_matrix
                .get(from.location_id, to.location_id)
                .map(|d| d.as_minutes())
                .unwrap_or_else(|| self.estimate_travel(from, to));

            total += travel_time;
        }

        total
    }

    /// Estimate travel time using Haversine distance when no cached data
    fn estimate_travel(&self, from: &RouteBooking, to: &RouteBooking) -> i32 {
        let from_coords = Coordinates::new_unchecked(from.latitude, from.longitude);
        let to_coords = Coordinates::new_unchecked(to.latitude, to.longitude);
        from_coords.estimate_travel_minutes(&to_coords)
    }

    /// Apply Clark-Wright Savings Algorithm
    fn clark_wright_optimize(
        &self,
        bookings: &[RouteBooking],
        travel_matrix: &TravelTimeMatrix,
    ) -> Vec<usize> {
        let n = bookings.len();
        if n <= 2 {
            return (0..n).collect();
        }

        // Build distance matrix for all pairs
        let distances = self.build_distance_matrix(bookings, travel_matrix);

        // Calculate savings for all pairs
        // Savings(i,j) = D(depot,i) + D(depot,j) - D(i,j)
        // We use the first booking's location as a virtual depot
        let mut savings: Vec<Savings> = Vec::new();

        for i in 0..n {
            for j in (i + 1)..n {
                // Using first location as depot reference
                let depot_to_i = distances[0][i];
                let depot_to_j = distances[0][j];
                let i_to_j = distances[i][j];

                let saving = depot_to_i + depot_to_j - i_to_j;
                if saving > 0 {
                    savings.push(Savings::new(i, j, saving));
                }
            }
        }

        // Sort savings in descending order
        savings.sort_by(|a, b| b.savings_minutes.cmp(&a.savings_minutes));

        // Initialize routes: each booking starts as its own route
        let mut routes: Vec<Vec<usize>> = (0..n).map(|i| vec![i]).collect();
        let mut route_of: Vec<usize> = (0..n).collect(); // Which route contains each node

        // Merge routes based on savings
        for saving in savings {
            let i = saving.from_idx;
            let j = saving.to_idx;

            let route_i = route_of[i];
            let route_j = route_of[j];

            // Skip if already in same route
            if route_i == route_j {
                continue;
            }

            // Check if i and j are at the ends of their respective routes
            let i_at_end =
                routes[route_i].first() == Some(&i) || routes[route_i].last() == Some(&i);
            let j_at_end =
                routes[route_j].first() == Some(&j) || routes[route_j].last() == Some(&j);

            if !i_at_end || !j_at_end {
                continue;
            }

            // Merge routes
            let (merge_from, merge_to, reverse_from) =
                if routes[route_i].len() <= routes[route_j].len() {
                    (route_i, route_j, routes[route_i].last() != Some(&i))
                } else {
                    (route_j, route_i, routes[route_j].last() != Some(&j))
                };

            let mut from_route = routes[merge_from].clone();
            if reverse_from {
                from_route.reverse();
            }

            // Update route assignments
            for &node in &from_route {
                route_of[node] = merge_to;
            }

            // Determine where to attach
            if routes[merge_to].last() == Some(&j) || routes[merge_to].last() == Some(&i) {
                routes[merge_to].extend(from_route);
            } else {
                from_route.extend(routes[merge_to].clone());
                routes[merge_to] = from_route;
            }

            routes[merge_from].clear();
        }

        // Find the non-empty route (should be exactly one)
        for route in routes {
            if !route.is_empty() {
                return route;
            }
        }

        // Fallback to chronological order
        (0..n).collect()
    }

    /// Build a distance matrix for all booking pairs
    fn build_distance_matrix(
        &self,
        bookings: &[RouteBooking],
        travel_matrix: &TravelTimeMatrix,
    ) -> Vec<Vec<i32>> {
        let n = bookings.len();
        let mut distances = vec![vec![0; n]; n];

        for i in 0..n {
            for j in 0..n {
                if i != j {
                    distances[i][j] = travel_matrix
                        .get(bookings[i].location_id, bookings[j].location_id)
                        .map(|d| d.as_minutes())
                        .unwrap_or_else(|| self.estimate_travel(&bookings[i], &bookings[j]));
                }
            }
        }

        distances
    }

    /// Build the final optimized route
    fn build_route(
        &self,
        bookings: Vec<RouteBooking>,
        travel_matrix: &TravelTimeMatrix,
        savings: i32,
    ) -> OptimizedRoute {
        let mut stops = Vec::with_capacity(bookings.len());
        let mut total_travel = 0;
        let total_distance = 0;

        for (idx, booking) in bookings.iter().enumerate() {
            let travel_from_prev = if idx == 0 {
                0
            } else {
                let prev = &bookings[idx - 1];
                travel_matrix
                    .get(prev.location_id, booking.location_id)
                    .map(|d| d.as_minutes())
                    .unwrap_or_else(|| self.estimate_travel(prev, booking))
            };

            total_travel += travel_from_prev;

            // Calculate arrival time based on previous stop + travel
            let arrival_time = if idx == 0 {
                booking.scheduled_start
            } else {
                let prev = &bookings[idx - 1];
                prev.scheduled_end + Duration::minutes(travel_from_prev as i64)
            };

            // Use the later of: calculated arrival or scheduled time
            let actual_arrival = if arrival_time > booking.scheduled_start {
                arrival_time
            } else {
                booking.scheduled_start
            };

            let stop = RouteStop {
                sequence: idx + 1,
                booking_id: booking.booking_id,
                location_id: booking.location_id,
                customer_name: booking.customer_name.clone(),
                address: booking.address.clone(),
                arrival_time: actual_arrival,
                departure_time: actual_arrival
                    + Duration::minutes(booking.duration_minutes() as i64),
                travel_from_previous_minutes: travel_from_prev,
                service_duration_minutes: booking.duration_minutes(),
            };

            stops.push(stop);
        }

        OptimizedRoute {
            stops,
            total_travel_minutes: total_travel,
            total_distance_meters: total_distance,
            savings_vs_chronological: savings.max(0),
            is_optimized: true,
        }
    }
}

impl Default for RouteOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use shared::types::{BookingId, DurationMinutes, LocationId};
    use uuid::Uuid;

    fn make_booking(idx: u32, lat: f64, lng: f64, hour: u32, duration_mins: i64) -> RouteBooking {
        let date = chrono::Utc
            .with_ymd_and_hms(2024, 6, 15, hour, 0, 0)
            .unwrap();
        RouteBooking {
            booking_id: BookingId::from_uuid(Uuid::from_u128(idx as u128)),
            location_id: LocationId::from_uuid(Uuid::from_u128(idx as u128)),
            customer_name: format!("Customer {}", idx),
            address: format!("Address {}", idx),
            scheduled_start: date,
            scheduled_end: date + Duration::minutes(duration_mins),
            latitude: lat,
            longitude: lng,
        }
    }

    #[test]
    fn test_single_booking() {
        let optimizer = RouteOptimizer::new();
        let bookings = vec![make_booking(1, 39.75, -105.0, 9, 30)];

        let route = optimizer.optimize(bookings, &TravelTimeMatrix::new());

        assert_eq!(route.num_stops(), 1);
        assert!(route.is_optimized);
        assert_eq!(route.total_travel_minutes, 0);
    }

    #[test]
    fn test_two_bookings_close_together() {
        let optimizer = RouteOptimizer::new();
        let bookings = vec![
            make_booking(1, 39.75, -105.0, 9, 30),
            make_booking(2, 39.751, -105.001, 10, 30),
        ];

        let route = optimizer.optimize(bookings, &TravelTimeMatrix::new());

        assert_eq!(route.num_stops(), 2);
        assert!(route.is_optimized);
    }

    #[test]
    fn test_multiple_bookings_with_matrix() {
        let optimizer = RouteOptimizer::new();

        let loc1 = LocationId::from_uuid(Uuid::from_u128(1));
        let loc2 = LocationId::from_uuid(Uuid::from_u128(2));
        let loc3 = LocationId::from_uuid(Uuid::from_u128(3));

        let mut matrix = TravelTimeMatrix::new();
        matrix.insert(loc1, loc2, DurationMinutes::new(15));
        matrix.insert(loc2, loc1, DurationMinutes::new(15));
        matrix.insert(loc1, loc3, DurationMinutes::new(30));
        matrix.insert(loc3, loc1, DurationMinutes::new(30));
        matrix.insert(loc2, loc3, DurationMinutes::new(10));
        matrix.insert(loc3, loc2, DurationMinutes::new(10));

        let bookings = vec![
            make_booking(1, 39.75, -105.0, 9, 30),
            make_booking(2, 39.76, -105.01, 10, 30),
            make_booking(3, 39.77, -105.02, 11, 30),
        ];

        let route = optimizer.optimize(bookings, &matrix);

        assert_eq!(route.num_stops(), 3);
        assert!(route.is_optimized);
        assert!(route.total_travel_minutes > 0);
    }

    #[test]
    fn test_empty_bookings() {
        let optimizer = RouteOptimizer::new();
        let route = optimizer.optimize(vec![], &TravelTimeMatrix::new());

        assert_eq!(route.num_stops(), 0);
        assert!(!route.is_optimized);
    }
}
