use axum::{extract::State, Json};
use db::models::CreateLocation;
use db::LocationRepository;
use serde::{Deserialize, Serialize};
use shared::AppError;

use crate::{
    auth::{AuthUser, TenantContext},
    error::{ApiError, ApiResult},
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct CreateLocationRequest {
    pub name: String,
    pub address: String,
    pub city: String,
    pub state: String,
    pub zip_code: String,
    pub latitude: f64,
    pub longitude: f64,
    pub notes: Option<String>,
    pub is_default: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct LocationResponse {
    pub id: String,
    pub name: String,
    pub address: String,
    pub city: String,
    pub state: String,
    pub zip_code: String,
    pub full_address: String,
    pub latitude: f64,
    pub longitude: f64,
    pub notes: Option<String>,
    pub is_default: bool,
}

pub async fn create_location(
    State(_state): State<AppState>,
    tenant: TenantContext,
    auth: AuthUser,
    Json(req): Json<CreateLocationRequest>,
) -> ApiResult<Json<LocationResponse>> {
    // Validate coordinates
    if req.latitude < -90.0 || req.latitude > 90.0 {
        return Err(ApiError::from(AppError::Validation(
            "Invalid latitude".to_string(),
        )));
    }
    if req.longitude < -180.0 || req.longitude > 180.0 {
        return Err(ApiError::from(AppError::Validation(
            "Invalid longitude".to_string(),
        )));
    }

    let location = LocationRepository::create(
        &tenant.pool,
        CreateLocation {
            organization_id: tenant.org_id,
            user_id: auth.user_id,
            name: req.name,
            address: req.address,
            city: req.city,
            state: req.state,
            zip_code: req.zip_code,
            latitude: req.latitude,
            longitude: req.longitude,
            notes: req.notes,
            is_default: req.is_default.unwrap_or(false),
        },
    )
    .await?;

    let full_address = location.full_address();
    Ok(Json(LocationResponse {
        id: location.id.to_string(),
        name: location.name,
        address: location.address,
        city: location.city,
        state: location.state,
        zip_code: location.zip_code,
        full_address,
        latitude: location.latitude,
        longitude: location.longitude,
        notes: location.notes,
        is_default: location.is_default,
    }))
}

pub async fn list_locations(
    State(_state): State<AppState>,
    tenant: TenantContext,
    auth: AuthUser,
) -> ApiResult<Json<Vec<LocationResponse>>> {
    let locations =
        LocationRepository::find_by_user(&tenant.pool, tenant.org_id, auth.user_id).await?;

    let response: Vec<LocationResponse> = locations
        .into_iter()
        .map(|l| {
            let full_address = l.full_address();
            LocationResponse {
                id: l.id.to_string(),
                name: l.name,
                address: l.address,
                city: l.city,
                state: l.state,
                zip_code: l.zip_code,
                full_address,
                latitude: l.latitude,
                longitude: l.longitude,
                notes: l.notes,
                is_default: l.is_default,
            }
        })
        .collect();

    Ok(Json(response))
}
