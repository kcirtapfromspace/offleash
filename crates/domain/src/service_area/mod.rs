//! Service area domain logic
//!
//! Provides functions for checking if locations are within walker service areas.

use shared::types::Coordinates;

/// A polygon point for service area boundaries
#[derive(Debug, Clone, Copy)]
pub struct PolygonPoint {
    pub lat: f64,
    pub lng: f64,
}

impl PolygonPoint {
    pub fn new(lat: f64, lng: f64) -> Self {
        Self { lat, lng }
    }
}

/// Service area with boundary information
#[derive(Debug, Clone)]
pub struct ServiceAreaBoundary {
    pub walker_id: String,
    pub area_id: String,
    pub name: String,
    pub polygon: Vec<PolygonPoint>,
    pub min_lat: f64,
    pub max_lat: f64,
    pub min_lng: f64,
    pub max_lng: f64,
    pub priority: i32,
    pub price_adjustment_percent: i32,
}

impl ServiceAreaBoundary {
    /// Check if a location is within this service area
    pub fn contains(&self, coords: &Coordinates) -> bool {
        // Quick bounding box check first
        if !coords.is_within_bounds(self.min_lat, self.max_lat, self.min_lng, self.max_lng) {
            return false;
        }

        // Full polygon check
        let polygon_points: Vec<(f64, f64)> = self.polygon.iter().map(|p| (p.lat, p.lng)).collect();

        coords.is_within_polygon(&polygon_points)
    }
}

/// Result of service area check
#[derive(Debug, Clone)]
pub struct ServiceAreaMatch {
    pub walker_id: String,
    pub area_id: String,
    pub area_name: String,
    pub priority: i32,
    pub price_adjustment_percent: i32,
}

/// Check which walkers can service a given location
pub fn find_walkers_for_location(
    service_areas: &[ServiceAreaBoundary],
    coords: &Coordinates,
) -> Vec<ServiceAreaMatch> {
    let mut matches: Vec<ServiceAreaMatch> = service_areas
        .iter()
        .filter(|area| area.contains(coords))
        .map(|area| ServiceAreaMatch {
            walker_id: area.walker_id.clone(),
            area_id: area.area_id.clone(),
            area_name: area.name.clone(),
            priority: area.priority,
            price_adjustment_percent: area.price_adjustment_percent,
        })
        .collect();

    // Sort by priority (lower = higher priority)
    matches.sort_by_key(|m| m.priority);

    matches
}

/// Check if a specific walker can service a location
pub fn walker_can_service_location(
    service_areas: &[ServiceAreaBoundary],
    walker_id: &str,
    coords: &Coordinates,
) -> Option<ServiceAreaMatch> {
    service_areas
        .iter()
        .filter(|area| area.walker_id == walker_id && area.contains(coords))
        .min_by_key(|area| area.priority)
        .map(|area| ServiceAreaMatch {
            walker_id: area.walker_id.clone(),
            area_id: area.area_id.clone(),
            area_name: area.name.clone(),
            priority: area.priority,
            price_adjustment_percent: area.price_adjustment_percent,
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn downtown_denver_area() -> ServiceAreaBoundary {
        ServiceAreaBoundary {
            walker_id: "walker-1".to_string(),
            area_id: "area-1".to_string(),
            name: "Downtown Denver".to_string(),
            polygon: vec![
                PolygonPoint::new(39.7800, -105.0300),
                PolygonPoint::new(39.7800, -104.9600),
                PolygonPoint::new(39.7300, -104.9600),
                PolygonPoint::new(39.7300, -105.0300),
            ],
            min_lat: 39.7300,
            max_lat: 39.7800,
            min_lng: -105.0300,
            max_lng: -104.9600,
            priority: 0,
            price_adjustment_percent: 0,
        }
    }

    fn boulder_area() -> ServiceAreaBoundary {
        ServiceAreaBoundary {
            walker_id: "walker-2".to_string(),
            area_id: "area-2".to_string(),
            name: "Boulder".to_string(),
            polygon: vec![
                PolygonPoint::new(40.0500, -105.3000),
                PolygonPoint::new(40.0500, -105.0800),
                PolygonPoint::new(39.9300, -105.0800),
                PolygonPoint::new(39.9300, -105.3000),
            ],
            min_lat: 39.9300,
            max_lat: 40.0500,
            min_lng: -105.3000,
            max_lng: -105.0800,
            priority: 0,
            price_adjustment_percent: 10,
        }
    }

    #[test]
    fn test_area_contains_point() {
        let area = downtown_denver_area();
        let inside = Coordinates::new_unchecked(39.7456, -104.9894);
        let outside = Coordinates::new_unchecked(40.0176, -105.2789);

        assert!(area.contains(&inside));
        assert!(!area.contains(&outside));
    }

    #[test]
    fn test_find_walkers_for_location() {
        let areas = vec![downtown_denver_area(), boulder_area()];

        // Downtown Denver location
        let denver_coords = Coordinates::new_unchecked(39.7456, -104.9894);
        let denver_matches = find_walkers_for_location(&areas, &denver_coords);
        assert_eq!(denver_matches.len(), 1);
        assert_eq!(denver_matches[0].walker_id, "walker-1");

        // Boulder location
        let boulder_coords = Coordinates::new_unchecked(40.0176, -105.2789);
        let boulder_matches = find_walkers_for_location(&areas, &boulder_coords);
        assert_eq!(boulder_matches.len(), 1);
        assert_eq!(boulder_matches[0].walker_id, "walker-2");
        assert_eq!(boulder_matches[0].price_adjustment_percent, 10);

        // Location outside all areas
        let outside_coords = Coordinates::new_unchecked(41.0, -106.0);
        let no_matches = find_walkers_for_location(&areas, &outside_coords);
        assert!(no_matches.is_empty());
    }

    #[test]
    fn test_walker_can_service_location() {
        let areas = vec![downtown_denver_area(), boulder_area()];
        let denver_coords = Coordinates::new_unchecked(39.7456, -104.9894);

        let walker1_match = walker_can_service_location(&areas, "walker-1", &denver_coords);
        assert!(walker1_match.is_some());

        let walker2_match = walker_can_service_location(&areas, "walker-2", &denver_coords);
        assert!(walker2_match.is_none());
    }
}
