use axum::{
    extract::{Path, State},
    Json,
};
use db::models::{CreateLocation, UpdateLocation};
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

#[derive(Debug, Deserialize)]
pub struct UpdateLocationRequest {
    pub name: Option<String>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub zip_code: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
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

    // Check for duplicate address
    let existing_locations =
        LocationRepository::find_by_user(&tenant.pool, tenant.org_id, auth.user_id).await?;
    let normalized_address = req.address.trim().to_lowercase();
    for loc in &existing_locations {
        if loc.address.trim().to_lowercase() == normalized_address {
            return Err(ApiError::from(AppError::Validation(
                "A location with this address already exists".to_string(),
            )));
        }
    }

    let is_default = req.is_default.unwrap_or(false);

    // If setting as default, unset any existing defaults for this user first
    if is_default {
        LocationRepository::unset_defaults_for_user(&tenant.pool, auth.user_id).await?;
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
            is_default,
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

pub async fn update_location(
    State(_state): State<AppState>,
    tenant: TenantContext,
    auth: AuthUser,
    Path(id): Path<String>,
    Json(req): Json<UpdateLocationRequest>,
) -> ApiResult<Json<LocationResponse>> {
    let location_id = id
        .parse()
        .map_err(|_| ApiError::from(AppError::Validation("Invalid location ID".to_string())))?;

    // Verify location exists and belongs to user
    let existing = LocationRepository::find_by_id(&tenant.pool, tenant.org_id, location_id)
        .await?
        .ok_or_else(|| ApiError::from(AppError::NotFound("Location not found".to_string())))?;

    if existing.user_id != auth.user_id {
        return Err(ApiError::from(AppError::Forbidden));
    }

    // Validate coordinates if provided
    if let Some(lat) = req.latitude {
        if !(-90.0..=90.0).contains(&lat) {
            return Err(ApiError::from(AppError::Validation(
                "Invalid latitude".to_string(),
            )));
        }
    }
    if let Some(lng) = req.longitude {
        if !(-180.0..=180.0).contains(&lng) {
            return Err(ApiError::from(AppError::Validation(
                "Invalid longitude".to_string(),
            )));
        }
    }

    // If setting as default, unset any existing defaults for this user first
    if req.is_default == Some(true) {
        LocationRepository::unset_defaults_for_user(&tenant.pool, auth.user_id).await?;
    }

    let input = UpdateLocation {
        name: req.name,
        address: req.address,
        city: req.city,
        state: req.state,
        zip_code: req.zip_code,
        latitude: req.latitude,
        longitude: req.longitude,
        notes: req.notes,
        is_default: req.is_default,
    };

    let updated = LocationRepository::update(&tenant.pool, tenant.org_id, location_id, input)
        .await?
        .ok_or_else(|| ApiError::from(AppError::NotFound("Location not found".to_string())))?;

    let full_address = updated.full_address();
    Ok(Json(LocationResponse {
        id: updated.id.to_string(),
        name: updated.name,
        address: updated.address,
        city: updated.city,
        state: updated.state,
        zip_code: updated.zip_code,
        full_address,
        latitude: updated.latitude,
        longitude: updated.longitude,
        notes: updated.notes,
        is_default: updated.is_default,
    }))
}

pub async fn delete_location(
    State(_state): State<AppState>,
    tenant: TenantContext,
    auth: AuthUser,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let location_id = id
        .parse()
        .map_err(|_| ApiError::from(AppError::Validation("Invalid location ID".to_string())))?;

    // Verify location exists and belongs to user
    let location = LocationRepository::find_by_id(&tenant.pool, tenant.org_id, location_id)
        .await?
        .ok_or_else(|| ApiError::from(AppError::NotFound("Location not found".to_string())))?;

    if location.user_id != auth.user_id {
        return Err(ApiError::from(AppError::Forbidden));
    }

    LocationRepository::delete(&tenant.pool, tenant.org_id, location_id).await?;

    Ok(Json(serde_json::json!({"success": true})))
}

pub async fn set_default_location(
    State(_state): State<AppState>,
    tenant: TenantContext,
    auth: AuthUser,
    Path(id): Path<String>,
) -> ApiResult<Json<LocationResponse>> {
    let location_id = id
        .parse()
        .map_err(|_| ApiError::from(AppError::Validation("Invalid location ID".to_string())))?;

    // Verify location exists and belongs to user
    let location = LocationRepository::find_by_id(&tenant.pool, tenant.org_id, location_id)
        .await?
        .ok_or_else(|| ApiError::from(AppError::NotFound("Location not found".to_string())))?;

    if location.user_id != auth.user_id {
        return Err(ApiError::from(AppError::Forbidden));
    }

    // Unset all other defaults for this user
    LocationRepository::unset_defaults_for_user(&tenant.pool, auth.user_id).await?;

    // Set this location as default
    let updated = LocationRepository::set_default(&tenant.pool, location_id).await?;

    let full_address = updated.full_address();
    Ok(Json(LocationResponse {
        id: updated.id.to_string(),
        name: updated.name,
        address: updated.address,
        city: updated.city,
        state: updated.state,
        zip_code: updated.zip_code,
        full_address,
        latitude: updated.latitude,
        longitude: updated.longitude,
        notes: updated.notes,
        is_default: updated.is_default,
    }))
}
