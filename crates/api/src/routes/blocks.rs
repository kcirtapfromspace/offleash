use axum::{
    extract::{Path, State},
    Json,
};
use chrono::DateTime;
use db::models::CreateBlock;
use db::BlockRepository;
use serde::{Deserialize, Serialize};
use shared::{AppError, DomainError};

use crate::{
    auth::{AuthUser, TenantContext},
    error::{ApiError, ApiResult},
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct CreateBlockRequest {
    pub reason: String,
    pub start_time: String, // ISO 8601
    pub end_time: String,   // ISO 8601
    pub is_recurring: Option<bool>,
    pub recurrence_rule: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BlockResponse {
    pub id: String,
    pub walker_id: String,
    pub reason: String,
    pub start_time: String,
    pub end_time: String,
    pub is_recurring: bool,
    pub recurrence_rule: Option<String>,
}

/// List blocks for the authenticated walker
pub async fn list_blocks(
    State(_state): State<AppState>,
    tenant: TenantContext,
    auth: AuthUser,
) -> ApiResult<Json<Vec<BlockResponse>>> {
    let blocks = BlockRepository::find_by_walker(&tenant.pool, tenant.org_id, auth.user_id).await?;

    let response: Vec<BlockResponse> = blocks
        .into_iter()
        .map(|b| BlockResponse {
            id: b.id.to_string(),
            walker_id: b.walker_id.to_string(),
            reason: b.reason,
            start_time: b.start_time.to_rfc3339(),
            end_time: b.end_time.to_rfc3339(),
            is_recurring: b.is_recurring,
            recurrence_rule: b.recurrence_rule,
        })
        .collect();

    Ok(Json(response))
}

pub async fn create_block(
    State(_state): State<AppState>,
    tenant: TenantContext,
    auth: AuthUser,
    Json(req): Json<CreateBlockRequest>,
) -> ApiResult<Json<BlockResponse>> {
    // Parse times
    let start_time = DateTime::parse_from_rfc3339(&req.start_time)
        .map_err(|_| {
            ApiError::from(AppError::Validation(
                "Invalid start time format".to_string(),
            ))
        })?
        .with_timezone(&chrono::Utc);

    let end_time = DateTime::parse_from_rfc3339(&req.end_time)
        .map_err(|_| ApiError::from(AppError::Validation("Invalid end time format".to_string())))?
        .with_timezone(&chrono::Utc);

    if end_time <= start_time {
        return Err(ApiError::from(AppError::Validation(
            "End time must be after start time".to_string(),
        )));
    }

    let block = BlockRepository::create(
        &tenant.pool,
        CreateBlock {
            organization_id: tenant.org_id,
            walker_id: auth.user_id,
            reason: req.reason,
            start_time,
            end_time,
            is_recurring: req.is_recurring.unwrap_or(false),
            recurrence_rule: req.recurrence_rule,
        },
    )
    .await?;

    Ok(Json(BlockResponse {
        id: block.id.to_string(),
        walker_id: block.walker_id.to_string(),
        reason: block.reason,
        start_time: block.start_time.to_rfc3339(),
        end_time: block.end_time.to_rfc3339(),
        is_recurring: block.is_recurring,
        recurrence_rule: block.recurrence_rule,
    }))
}

pub async fn delete_block(
    State(_state): State<AppState>,
    tenant: TenantContext,
    auth: AuthUser,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let block_id = id
        .parse()
        .map_err(|_| ApiError::from(AppError::Validation("Invalid block ID".to_string())))?;

    // Verify block belongs to user
    let block = BlockRepository::find_by_id(&tenant.pool, tenant.org_id, block_id)
        .await?
        .ok_or_else(|| ApiError::from(DomainError::BookingNotFound(id.clone())))?;

    if block.walker_id != auth.user_id {
        return Err(ApiError::from(AppError::Forbidden));
    }

    BlockRepository::delete(&tenant.pool, tenant.org_id, block_id).await?;

    Ok(Json(serde_json::json!({ "deleted": true })))
}
