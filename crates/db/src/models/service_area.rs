use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared::types::{OrganizationId, UserId};
use sqlx::FromRow;
use uuid::Uuid;

// MARK: - Polygon Point

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolygonPoint {
    pub lat: f64,
    pub lng: f64,
}

// MARK: - Service Area

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ServiceArea {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub walker_id: Uuid,
    pub name: String,
    pub color: Option<String>,
    pub polygon: sqlx::types::Json<Vec<PolygonPoint>>,
    pub min_latitude: Option<f64>,
    pub max_latitude: Option<f64>,
    pub min_longitude: Option<f64>,
    pub max_longitude: Option<f64>,
    pub is_active: bool,
    pub priority: Option<i32>,
    pub price_adjustment_percent: Option<i32>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateServiceArea {
    pub organization_id: OrganizationId,
    pub walker_id: UserId,
    pub name: String,
    pub color: Option<String>,
    pub polygon: Vec<PolygonPoint>,
    pub is_active: bool,
    pub priority: Option<i32>,
    pub price_adjustment_percent: Option<i32>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateServiceArea {
    pub name: Option<String>,
    pub color: Option<String>,
    pub polygon: Option<Vec<PolygonPoint>>,
    pub is_active: Option<bool>,
    pub priority: Option<i32>,
    pub price_adjustment_percent: Option<i32>,
    pub notes: Option<String>,
}

// MARK: - Service Area Response (for API)

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceAreaResponse {
    pub id: String,
    pub walker_id: String,
    pub name: String,
    pub color: String,
    pub polygon: Vec<PolygonPoint>,
    pub is_active: bool,
    pub priority: i32,
    pub price_adjustment_percent: i32,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<ServiceArea> for ServiceAreaResponse {
    fn from(area: ServiceArea) -> Self {
        Self {
            id: area.id.to_string(),
            walker_id: area.walker_id.to_string(),
            name: area.name,
            color: area.color.unwrap_or_else(|| "#3B82F6".to_string()),
            polygon: area.polygon.0,
            is_active: area.is_active,
            priority: area.priority.unwrap_or(0),
            price_adjustment_percent: area.price_adjustment_percent.unwrap_or(0),
            notes: area.notes,
            created_at: area.created_at.to_rfc3339(),
            updated_at: area.updated_at.to_rfc3339(),
        }
    }
}
