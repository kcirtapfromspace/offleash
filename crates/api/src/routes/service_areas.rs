use axum::{
    extract::{Path, State},
    Json,
};
use db::models::{CreateServiceArea, PolygonPoint, ServiceAreaResponse, UpdateServiceArea};
use db::{ServiceAreaRepository, UserRepository};
use serde::{Deserialize, Serialize};
use shared::AppError;

use crate::{
    auth::{AuthUser, TenantContext},
    error::{ApiError, ApiResult},
    state::AppState,
};

// MARK: - Request Types

#[derive(Debug, Deserialize)]
pub struct CreateServiceAreaRequest {
    pub name: String,
    pub color: Option<String>,
    pub polygon: Vec<PolygonPointRequest>,
    pub is_active: Option<bool>,
    pub priority: Option<i32>,
    pub price_adjustment_percent: Option<i32>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PolygonPointRequest {
    pub lat: f64,
    pub lng: f64,
}

#[derive(Debug, Deserialize)]
pub struct UpdateServiceAreaRequest {
    pub name: Option<String>,
    pub color: Option<String>,
    pub polygon: Option<Vec<PolygonPointRequest>>,
    pub is_active: Option<bool>,
    pub priority: Option<i32>,
    pub price_adjustment_percent: Option<i32>,
    pub notes: Option<String>,
}

// MARK: - Walker Endpoints

/// Get current walker's service areas
pub async fn get_my_service_areas(
    State(_state): State<AppState>,
    tenant: TenantContext,
    auth: AuthUser,
) -> ApiResult<Json<Vec<ServiceAreaResponse>>> {
    // Verify user is a walker
    let user = UserRepository::find_by_id(&tenant.pool, tenant.org_id, auth.user_id)
        .await?
        .ok_or_else(|| ApiError::from(AppError::Forbidden))?;

    if !user.is_walker() {
        return Err(ApiError::from(AppError::Forbidden));
    }

    let areas = ServiceAreaRepository::find_by_walker(&tenant.pool, tenant.org_id, auth.user_id).await?;

    let responses: Vec<ServiceAreaResponse> = areas.into_iter().map(ServiceAreaResponse::from).collect();

    Ok(Json(responses))
}

/// Create a service area for current walker
pub async fn create_my_service_area(
    State(_state): State<AppState>,
    tenant: TenantContext,
    auth: AuthUser,
    Json(req): Json<CreateServiceAreaRequest>,
) -> ApiResult<Json<ServiceAreaResponse>> {
    // Verify user is a walker
    let user = UserRepository::find_by_id(&tenant.pool, tenant.org_id, auth.user_id)
        .await?
        .ok_or_else(|| ApiError::from(AppError::Forbidden))?;

    if !user.is_walker() {
        return Err(ApiError::from(AppError::Forbidden));
    }

    // Validate polygon has at least 3 points
    if req.polygon.len() < 3 {
        return Err(ApiError::from(AppError::Validation(
            "Polygon must have at least 3 points".to_string(),
        )));
    }

    let polygon: Vec<PolygonPoint> = req
        .polygon
        .into_iter()
        .map(|p| PolygonPoint { lat: p.lat, lng: p.lng })
        .collect();

    let area = ServiceAreaRepository::create(
        &tenant.pool,
        CreateServiceArea {
            organization_id: tenant.org_id,
            walker_id: auth.user_id,
            name: req.name,
            color: req.color,
            polygon,
            is_active: req.is_active.unwrap_or(true),
            priority: req.priority,
            price_adjustment_percent: req.price_adjustment_percent,
            notes: req.notes,
        },
    )
    .await?;

    Ok(Json(ServiceAreaResponse::from(area)))
}

/// Update a service area for current walker
pub async fn update_my_service_area(
    State(_state): State<AppState>,
    tenant: TenantContext,
    auth: AuthUser,
    Path(area_id): Path<String>,
    Json(req): Json<UpdateServiceAreaRequest>,
) -> ApiResult<Json<ServiceAreaResponse>> {
    // Verify user is a walker
    let user = UserRepository::find_by_id(&tenant.pool, tenant.org_id, auth.user_id)
        .await?
        .ok_or_else(|| ApiError::from(AppError::Forbidden))?;

    if !user.is_walker() {
        return Err(ApiError::from(AppError::Forbidden));
    }

    let area_uuid = area_id
        .parse()
        .map_err(|_| ApiError::from(AppError::Validation("Invalid area ID".to_string())))?;

    // Verify area belongs to this walker
    let existing = ServiceAreaRepository::find_by_id(&tenant.pool, tenant.org_id, area_uuid)
        .await?
        .ok_or_else(|| ApiError::from(AppError::NotFound("Service area not found".to_string())))?;

    if existing.walker_id != *auth.user_id.as_uuid() {
        return Err(ApiError::from(AppError::Forbidden));
    }

    // Validate polygon if provided
    if let Some(ref polygon) = req.polygon {
        if polygon.len() < 3 {
            return Err(ApiError::from(AppError::Validation(
                "Polygon must have at least 3 points".to_string(),
            )));
        }
    }

    let polygon = req.polygon.map(|p| {
        p.into_iter()
            .map(|pt| PolygonPoint { lat: pt.lat, lng: pt.lng })
            .collect()
    });

    let updated = ServiceAreaRepository::update(
        &tenant.pool,
        tenant.org_id,
        area_uuid,
        UpdateServiceArea {
            name: req.name,
            color: req.color,
            polygon,
            is_active: req.is_active,
            priority: req.priority,
            price_adjustment_percent: req.price_adjustment_percent,
            notes: req.notes,
        },
    )
    .await?
    .ok_or_else(|| ApiError::from(AppError::NotFound("Service area not found".to_string())))?;

    Ok(Json(ServiceAreaResponse::from(updated)))
}

/// Delete a service area for current walker
pub async fn delete_my_service_area(
    State(_state): State<AppState>,
    tenant: TenantContext,
    auth: AuthUser,
    Path(area_id): Path<String>,
) -> ApiResult<Json<DeleteResponse>> {
    // Verify user is a walker
    let user = UserRepository::find_by_id(&tenant.pool, tenant.org_id, auth.user_id)
        .await?
        .ok_or_else(|| ApiError::from(AppError::Forbidden))?;

    if !user.is_walker() {
        return Err(ApiError::from(AppError::Forbidden));
    }

    let area_uuid = area_id
        .parse()
        .map_err(|_| ApiError::from(AppError::Validation("Invalid area ID".to_string())))?;

    // Verify area belongs to this walker
    let existing = ServiceAreaRepository::find_by_id(&tenant.pool, tenant.org_id, area_uuid)
        .await?
        .ok_or_else(|| ApiError::from(AppError::NotFound("Service area not found".to_string())))?;

    if existing.walker_id != *auth.user_id.as_uuid() {
        return Err(ApiError::from(AppError::Forbidden));
    }

    ServiceAreaRepository::delete(&tenant.pool, tenant.org_id, area_uuid).await?;

    Ok(Json(DeleteResponse { success: true }))
}

// MARK: - Admin Endpoints

/// Get all service areas for an organization (admin only)
pub async fn list_all_service_areas(
    State(_state): State<AppState>,
    tenant: TenantContext,
    auth: AuthUser,
) -> ApiResult<Json<Vec<ServiceAreaResponse>>> {
    // Verify user is admin
    let user = UserRepository::find_by_id(&tenant.pool, tenant.org_id, auth.user_id)
        .await?
        .ok_or_else(|| ApiError::from(AppError::Forbidden))?;

    if !user.is_admin() {
        return Err(ApiError::from(AppError::Forbidden));
    }

    let areas = ServiceAreaRepository::list_all(&tenant.pool, tenant.org_id).await?;

    let responses: Vec<ServiceAreaResponse> = areas.into_iter().map(ServiceAreaResponse::from).collect();

    Ok(Json(responses))
}

/// Get service areas for a specific walker (admin only)
pub async fn get_walker_service_areas(
    State(_state): State<AppState>,
    tenant: TenantContext,
    auth: AuthUser,
    Path(walker_id): Path<String>,
) -> ApiResult<Json<Vec<ServiceAreaResponse>>> {
    // Verify user is admin
    let user = UserRepository::find_by_id(&tenant.pool, tenant.org_id, auth.user_id)
        .await?
        .ok_or_else(|| ApiError::from(AppError::Forbidden))?;

    if !user.is_admin() {
        return Err(ApiError::from(AppError::Forbidden));
    }

    let walker_uuid = walker_id
        .parse()
        .map_err(|_| ApiError::from(AppError::Validation("Invalid walker ID".to_string())))?;

    let walker_user_id = shared::types::UserId::from(walker_uuid);

    let areas = ServiceAreaRepository::find_by_walker(&tenant.pool, tenant.org_id, walker_user_id).await?;

    let responses: Vec<ServiceAreaResponse> = areas.into_iter().map(ServiceAreaResponse::from).collect();

    Ok(Json(responses))
}

/// Create a service area for a walker (admin only)
pub async fn create_walker_service_area(
    State(_state): State<AppState>,
    tenant: TenantContext,
    auth: AuthUser,
    Path(walker_id): Path<String>,
    Json(req): Json<CreateServiceAreaRequest>,
) -> ApiResult<Json<ServiceAreaResponse>> {
    // Verify user is admin
    let user = UserRepository::find_by_id(&tenant.pool, tenant.org_id, auth.user_id)
        .await?
        .ok_or_else(|| ApiError::from(AppError::Forbidden))?;

    if !user.is_admin() {
        return Err(ApiError::from(AppError::Forbidden));
    }

    let walker_uuid = walker_id
        .parse()
        .map_err(|_| ApiError::from(AppError::Validation("Invalid walker ID".to_string())))?;

    let walker_user_id = shared::types::UserId::from(walker_uuid);

    // Verify walker exists
    let walker = UserRepository::find_by_id(&tenant.pool, tenant.org_id, walker_user_id)
        .await?
        .ok_or_else(|| ApiError::from(AppError::NotFound("Walker not found".to_string())))?;

    if !walker.is_walker() {
        return Err(ApiError::from(AppError::Validation("User is not a walker".to_string())));
    }

    // Validate polygon
    if req.polygon.len() < 3 {
        return Err(ApiError::from(AppError::Validation(
            "Polygon must have at least 3 points".to_string(),
        )));
    }

    let polygon: Vec<PolygonPoint> = req
        .polygon
        .into_iter()
        .map(|p| PolygonPoint { lat: p.lat, lng: p.lng })
        .collect();

    let area = ServiceAreaRepository::create(
        &tenant.pool,
        CreateServiceArea {
            organization_id: tenant.org_id,
            walker_id: walker_user_id,
            name: req.name,
            color: req.color,
            polygon,
            is_active: req.is_active.unwrap_or(true),
            priority: req.priority,
            price_adjustment_percent: req.price_adjustment_percent,
            notes: req.notes,
        },
    )
    .await?;

    Ok(Json(ServiceAreaResponse::from(area)))
}

/// Update any service area (admin only)
pub async fn update_service_area(
    State(_state): State<AppState>,
    tenant: TenantContext,
    auth: AuthUser,
    Path(area_id): Path<String>,
    Json(req): Json<UpdateServiceAreaRequest>,
) -> ApiResult<Json<ServiceAreaResponse>> {
    // Verify user is admin
    let user = UserRepository::find_by_id(&tenant.pool, tenant.org_id, auth.user_id)
        .await?
        .ok_or_else(|| ApiError::from(AppError::Forbidden))?;

    if !user.is_admin() {
        return Err(ApiError::from(AppError::Forbidden));
    }

    let area_uuid = area_id
        .parse()
        .map_err(|_| ApiError::from(AppError::Validation("Invalid area ID".to_string())))?;

    // Validate polygon if provided
    if let Some(ref polygon) = req.polygon {
        if polygon.len() < 3 {
            return Err(ApiError::from(AppError::Validation(
                "Polygon must have at least 3 points".to_string(),
            )));
        }
    }

    let polygon = req.polygon.map(|p| {
        p.into_iter()
            .map(|pt| PolygonPoint { lat: pt.lat, lng: pt.lng })
            .collect()
    });

    let updated = ServiceAreaRepository::update(
        &tenant.pool,
        tenant.org_id,
        area_uuid,
        UpdateServiceArea {
            name: req.name,
            color: req.color,
            polygon,
            is_active: req.is_active,
            priority: req.priority,
            price_adjustment_percent: req.price_adjustment_percent,
            notes: req.notes,
        },
    )
    .await?
    .ok_or_else(|| ApiError::from(AppError::NotFound("Service area not found".to_string())))?;

    Ok(Json(ServiceAreaResponse::from(updated)))
}

/// Delete any service area (admin only)
pub async fn delete_service_area(
    State(_state): State<AppState>,
    tenant: TenantContext,
    auth: AuthUser,
    Path(area_id): Path<String>,
) -> ApiResult<Json<DeleteResponse>> {
    // Verify user is admin
    let user = UserRepository::find_by_id(&tenant.pool, tenant.org_id, auth.user_id)
        .await?
        .ok_or_else(|| ApiError::from(AppError::Forbidden))?;

    if !user.is_admin() {
        return Err(ApiError::from(AppError::Forbidden));
    }

    let area_uuid = area_id
        .parse()
        .map_err(|_| ApiError::from(AppError::Validation("Invalid area ID".to_string())))?;

    let deleted = ServiceAreaRepository::delete(&tenant.pool, tenant.org_id, area_uuid).await?;

    if !deleted {
        return Err(ApiError::from(AppError::NotFound("Service area not found".to_string())));
    }

    Ok(Json(DeleteResponse { success: true }))
}

#[derive(Debug, Serialize)]
pub struct DeleteResponse {
    pub success: bool,
}
