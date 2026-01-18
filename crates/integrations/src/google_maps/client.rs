use reqwest::Client;
use serde::Deserialize;
use shared::types::Coordinates;

/// Google Maps Distance Matrix API client
pub struct GoogleMapsClient {
    client: Client,
    api_key: String,
}

impl GoogleMapsClient {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
        }
    }

    /// Get travel time between two points in minutes
    pub async fn get_travel_time(
        &self,
        origin: &Coordinates,
        destination: &Coordinates,
    ) -> Result<TravelTimeResult, GoogleMapsError> {
        let url = format!(
            "https://maps.googleapis.com/maps/api/distancematrix/json?origins={}&destinations={}&key={}&mode=driving&departure_time=now",
            origin.to_lat_lng_string(),
            destination.to_lat_lng_string(),
            self.api_key
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| GoogleMapsError::Request(e.to_string()))?;

        if !response.status().is_success() {
            return Err(GoogleMapsError::Api(format!(
                "HTTP {}: {}",
                response.status(),
                response.text().await.unwrap_or_default()
            )));
        }

        let result: DistanceMatrixResponse = response
            .json()
            .await
            .map_err(|e| GoogleMapsError::Parse(e.to_string()))?;

        if result.status != "OK" {
            return Err(GoogleMapsError::Api(format!(
                "API error: {}",
                result.status
            )));
        }

        let element = result
            .rows
            .first()
            .and_then(|r| r.elements.first())
            .ok_or_else(|| GoogleMapsError::Api("No results returned".to_string()))?;

        if element.status != "OK" {
            return Err(GoogleMapsError::Api(format!(
                "Element error: {}",
                element.status
            )));
        }

        let duration = element
            .duration_in_traffic
            .as_ref()
            .or(element.duration.as_ref())
            .ok_or_else(|| GoogleMapsError::Api("No duration in response".to_string()))?;

        let distance = element
            .distance
            .as_ref()
            .ok_or_else(|| GoogleMapsError::Api("No distance in response".to_string()))?;

        Ok(TravelTimeResult {
            duration_minutes: duration.value / 60,
            distance_meters: distance.value,
        })
    }
}

/// Result of a travel time calculation
#[derive(Debug, Clone)]
pub struct TravelTimeResult {
    pub duration_minutes: i32,
    pub distance_meters: i32,
}

/// Errors from the Google Maps API
#[derive(Debug, thiserror::Error)]
pub enum GoogleMapsError {
    #[error("Request error: {0}")]
    Request(String),
    #[error("API error: {0}")]
    Api(String),
    #[error("Parse error: {0}")]
    Parse(String),
}

// Response types for Distance Matrix API
#[derive(Debug, Deserialize)]
struct DistanceMatrixResponse {
    status: String,
    rows: Vec<DistanceMatrixRow>,
}

#[derive(Debug, Deserialize)]
struct DistanceMatrixRow {
    elements: Vec<DistanceMatrixElement>,
}

#[derive(Debug, Deserialize)]
struct DistanceMatrixElement {
    status: String,
    duration: Option<DurationValue>,
    duration_in_traffic: Option<DurationValue>,
    distance: Option<DistanceValue>,
}

#[derive(Debug, Deserialize)]
struct DurationValue {
    value: i32, // seconds
    #[allow(dead_code)]
    text: String,
}

#[derive(Debug, Deserialize)]
struct DistanceValue {
    value: i32, // meters
    #[allow(dead_code)]
    text: String,
}
