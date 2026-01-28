use serde::{Deserialize, Serialize};

/// Geographic coordinates (latitude, longitude)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Coordinates {
    pub latitude: f64,
    pub longitude: f64,
}

impl Coordinates {
    /// Create new coordinates with validation
    pub fn new(latitude: f64, longitude: f64) -> Result<Self, CoordinatesError> {
        if !(-90.0..=90.0).contains(&latitude) {
            return Err(CoordinatesError::InvalidLatitude(latitude));
        }
        if !(-180.0..=180.0).contains(&longitude) {
            return Err(CoordinatesError::InvalidLongitude(longitude));
        }
        Ok(Self {
            latitude,
            longitude,
        })
    }

    /// Create coordinates without validation (use when data is trusted)
    pub fn new_unchecked(latitude: f64, longitude: f64) -> Self {
        Self {
            latitude,
            longitude,
        }
    }

    /// Calculate the Haversine distance to another point in kilometers
    /// This is a great-circle distance approximation
    pub fn distance_km(&self, other: &Coordinates) -> f64 {
        const EARTH_RADIUS_KM: f64 = 6371.0;

        let lat1 = self.latitude.to_radians();
        let lat2 = other.latitude.to_radians();
        let dlat = (other.latitude - self.latitude).to_radians();
        let dlon = (other.longitude - self.longitude).to_radians();

        let a = (dlat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (dlon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().asin();

        EARTH_RADIUS_KM * c
    }

    /// Calculate the Haversine distance in miles
    pub fn distance_miles(&self, other: &Coordinates) -> f64 {
        self.distance_km(other) * 0.621371
    }

    /// Estimate travel time in minutes using a simple heuristic
    /// Assumes average driving speed based on distance
    pub fn estimate_travel_minutes(&self, other: &Coordinates) -> i32 {
        let distance_km = self.distance_km(other);

        // Use different average speeds based on distance
        // Shorter distances have more stops/traffic
        let avg_speed_kmh = if distance_km < 5.0 {
            25.0 // City driving
        } else if distance_km < 20.0 {
            35.0 // Suburban
        } else {
            50.0 // Highway mix
        };

        let hours = distance_km / avg_speed_kmh;
        let minutes = (hours * 60.0).ceil() as i32;

        // Minimum 5 minutes for any trip
        minutes.max(5)
    }

    /// Format as "lat,lng" string (useful for API calls)
    pub fn to_lat_lng_string(&self) -> String {
        format!("{},{}", self.latitude, self.longitude)
    }

    /// Check if point is within a polygon using ray casting algorithm
    /// Polygon should be a slice of (latitude, longitude) pairs
    pub fn is_within_polygon(&self, polygon: &[(f64, f64)]) -> bool {
        if polygon.len() < 3 {
            return false;
        }

        let mut inside = false;
        let n = polygon.len();

        let mut j = n - 1;
        for i in 0..n {
            let (lat_i, lng_i) = polygon[i];
            let (lat_j, lng_j) = polygon[j];

            // Ray casting algorithm
            if ((lng_i > self.longitude) != (lng_j > self.longitude))
                && (self.latitude
                    < (lat_j - lat_i) * (self.longitude - lng_i) / (lng_j - lng_i) + lat_i)
            {
                inside = !inside;
            }
            j = i;
        }

        inside
    }

    /// Check if point is within a bounding box (quick pre-filter)
    pub fn is_within_bounds(&self, min_lat: f64, max_lat: f64, min_lng: f64, max_lng: f64) -> bool {
        self.latitude >= min_lat
            && self.latitude <= max_lat
            && self.longitude >= min_lng
            && self.longitude <= max_lng
    }
}

/// Error for invalid coordinates
#[derive(Debug, Clone, thiserror::Error)]
pub enum CoordinatesError {
    #[error("Invalid latitude: {0}. Must be between -90 and 90")]
    InvalidLatitude(f64),
    #[error("Invalid longitude: {0}. Must be between -180 and 180")]
    InvalidLongitude(f64),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_coordinates() {
        let coords = Coordinates::new(40.7128, -74.0060).unwrap();
        assert_eq!(coords.latitude, 40.7128);
        assert_eq!(coords.longitude, -74.0060);
    }

    #[test]
    fn test_invalid_latitude() {
        assert!(Coordinates::new(91.0, 0.0).is_err());
        assert!(Coordinates::new(-91.0, 0.0).is_err());
    }

    #[test]
    fn test_invalid_longitude() {
        assert!(Coordinates::new(0.0, 181.0).is_err());
        assert!(Coordinates::new(0.0, -181.0).is_err());
    }

    #[test]
    fn test_distance_calculation() {
        // New York to Los Angeles: approximately 3944 km
        let nyc = Coordinates::new(40.7128, -74.0060).unwrap();
        let la = Coordinates::new(34.0522, -118.2437).unwrap();

        let distance = nyc.distance_km(&la);
        assert!(distance > 3900.0 && distance < 4000.0);
    }

    #[test]
    fn test_same_point_distance() {
        let point = Coordinates::new(40.7128, -74.0060).unwrap();
        assert!(point.distance_km(&point) < 0.001);
    }

    #[test]
    fn test_travel_estimate() {
        // Short distance should estimate slower speed
        let a = Coordinates::new(40.7128, -74.0060).unwrap();
        let b = Coordinates::new(40.7200, -74.0100).unwrap();

        let minutes = a.estimate_travel_minutes(&b);
        assert!(minutes >= 5); // Minimum 5 minutes
    }

    #[test]
    fn test_lat_lng_string() {
        let coords = Coordinates::new(40.7128, -74.0060).unwrap();
        assert_eq!(coords.to_lat_lng_string(), "40.7128,-74.006");
    }

    #[test]
    fn test_point_in_polygon() {
        // Denver area polygon (Downtown/RiNo/LoHi)
        let polygon = [
            (39.7800, -105.0300),
            (39.7800, -104.9600),
            (39.7300, -104.9600),
            (39.7300, -105.0300),
        ];

        // Point inside (Downtown Denver)
        let inside = Coordinates::new(39.7456, -104.9894).unwrap();
        assert!(inside.is_within_polygon(&polygon));

        // Point outside (Boulder)
        let outside = Coordinates::new(40.0176, -105.2789).unwrap();
        assert!(!outside.is_within_polygon(&polygon));
    }

    #[test]
    fn test_point_in_bounds() {
        let point = Coordinates::new(39.75, -105.0).unwrap();

        assert!(point.is_within_bounds(39.7, 39.8, -105.1, -104.9));
        assert!(!point.is_within_bounds(40.0, 40.5, -105.1, -104.9));
    }
}
