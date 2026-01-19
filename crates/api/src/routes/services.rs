use axum::{
    extract::{Path, State},
    Json,
};
use db::ServiceRepository;
use serde::{Deserialize, Serialize};
use shared::{AppError, DomainError};

use crate::{
    auth::TenantContext,
    error::{ApiError, ApiResult},
    state::AppState,
};

#[derive(Debug, Serialize)]
pub struct ServiceResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub duration_minutes: i32,
    pub price_cents: i64,
    pub price_display: String,
    pub is_active: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateServiceRequest {
    pub name: String,
    pub description: Option<String>,
    pub duration_minutes: i32,
    pub base_price_cents: i64,
}

#[derive(Debug, Deserialize)]
pub struct UpdateServiceRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub duration_minutes: Option<i32>,
    pub base_price_cents: Option<i64>,
    pub is_active: Option<bool>,
}

pub async fn list_services(
    State(_state): State<AppState>,
    tenant: TenantContext,
) -> ApiResult<Json<Vec<ServiceResponse>>> {
    let services = ServiceRepository::list_all(&tenant.pool, tenant.org_id).await?;

    let response: Vec<ServiceResponse> = services
        .into_iter()
        .map(|s| {
            let price_display = format!("${:.2}", s.price_dollars());
            ServiceResponse {
                id: s.id.to_string(),
                name: s.name,
                description: s.description,
                duration_minutes: s.duration_minutes,
                price_cents: s.base_price_cents,
                price_display,
                is_active: s.is_active,
            }
        })
        .collect();

    Ok(Json(response))
}

pub async fn get_service(
    State(_state): State<AppState>,
    tenant: TenantContext,
    Path(id): Path<String>,
) -> ApiResult<Json<ServiceResponse>> {
    let service_id = id
        .parse()
        .map_err(|_| ApiError::from(AppError::Validation("Invalid service ID".to_string())))?;

    let service = ServiceRepository::find_by_id(&tenant.pool, tenant.org_id, service_id)
        .await?
        .ok_or_else(|| ApiError::from(DomainError::ServiceNotFound(id)))?;

    let price_display = format!("${:.2}", service.price_dollars());
    Ok(Json(ServiceResponse {
        id: service.id.to_string(),
        name: service.name,
        description: service.description,
        duration_minutes: service.duration_minutes,
        price_cents: service.base_price_cents,
        price_display,
        is_active: service.is_active,
    }))
}

pub async fn create_service(
    State(_state): State<AppState>,
    tenant: TenantContext,
    Json(req): Json<CreateServiceRequest>,
) -> ApiResult<Json<ServiceResponse>> {
    let input = db::models::CreateService {
        organization_id: tenant.org_id,
        name: req.name,
        description: req.description,
        duration_minutes: req.duration_minutes,
        base_price_cents: req.base_price_cents,
    };

    let service = ServiceRepository::create(&tenant.pool, input).await?;

    let price_display = format!("${:.2}", service.price_dollars());
    Ok(Json(ServiceResponse {
        id: service.id.to_string(),
        name: service.name,
        description: service.description,
        duration_minutes: service.duration_minutes,
        price_cents: service.base_price_cents,
        price_display,
        is_active: service.is_active,
    }))
}

pub async fn update_service(
    State(_state): State<AppState>,
    tenant: TenantContext,
    Path(id): Path<String>,
    Json(req): Json<UpdateServiceRequest>,
) -> ApiResult<Json<ServiceResponse>> {
    let service_id = id
        .parse()
        .map_err(|_| ApiError::from(AppError::Validation("Invalid service ID".to_string())))?;

    let input = db::models::UpdateService {
        name: req.name,
        description: req.description,
        duration_minutes: req.duration_minutes,
        base_price_cents: req.base_price_cents,
        is_active: req.is_active,
    };

    let service = ServiceRepository::update(&tenant.pool, tenant.org_id, service_id, input)
        .await?
        .ok_or_else(|| ApiError::from(DomainError::ServiceNotFound(id)))?;

    let price_display = format!("${:.2}", service.price_dollars());
    Ok(Json(ServiceResponse {
        id: service.id.to_string(),
        name: service.name,
        description: service.description,
        duration_minutes: service.duration_minutes,
        price_cents: service.base_price_cents,
        price_display,
        is_active: service.is_active,
    }))
}
