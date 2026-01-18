use axum::{
    extract::{Path, State},
    Json,
};
use db::ServiceRepository;
use serde::Serialize;
use shared::{AppError, DomainError};

use crate::{
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
}

pub async fn list_services(State(state): State<AppState>) -> ApiResult<Json<Vec<ServiceResponse>>> {
    let services = ServiceRepository::list_active(&state.pool).await?;

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
            }
        })
        .collect();

    Ok(Json(response))
}

pub async fn get_service(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<Json<ServiceResponse>> {
    let service_id = id
        .parse()
        .map_err(|_| ApiError::from(AppError::Validation("Invalid service ID".to_string())))?;

    let service = ServiceRepository::find_by_id(&state.pool, service_id)
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
    }))
}
