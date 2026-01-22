use axum::{
    extract::{Path, Query, State},
    Json,
};
use db::{models::{UpdateUser, UserRole}, UserRepository};
use serde::{Deserialize, Serialize};
use shared::{AppError, DomainError};

use crate::{
    auth::{AuthUser, TenantContext},
    error::{ApiError, ApiResult},
    state::AppState,
};

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: String,
    pub email: String,
    pub role: String,
    pub first_name: String,
    pub last_name: String,
    pub full_name: String,
    pub phone: Option<String>,
    pub timezone: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct ListUsersQuery {
    pub role: Option<String>,
}

pub async fn list_users(
    State(_state): State<AppState>,
    tenant: TenantContext,
    Query(query): Query<ListUsersQuery>,
) -> ApiResult<Json<Vec<UserResponse>>> {
    let users = if let Some(role_str) = query.role {
        let role = match role_str.to_lowercase().as_str() {
            "walker" => UserRole::Walker,
            "customer" => UserRole::Customer,
            "admin" => UserRole::Admin,
            _ => {
                return Err(ApiError::from(AppError::Validation(
                    "Invalid role. Must be 'walker', 'customer', or 'admin'".to_string(),
                )))
            }
        };
        UserRepository::list_by_role(&tenant.pool, tenant.org_id, role).await?
    } else {
        UserRepository::list_all(&tenant.pool, tenant.org_id).await?
    };

    let response: Vec<UserResponse> = users
        .into_iter()
        .map(|u| UserResponse {
            id: u.id.to_string(),
            email: u.email,
            role: u.role.to_string(),
            first_name: u.first_name.clone(),
            last_name: u.last_name.clone(),
            full_name: format!("{} {}", u.first_name, u.last_name),
            phone: u.phone,
            timezone: u.timezone,
            created_at: u.created_at.to_rfc3339(),
        })
        .collect();

    Ok(Json(response))
}

pub async fn get_user(
    State(_state): State<AppState>,
    tenant: TenantContext,
    Path(id): Path<String>,
) -> ApiResult<Json<UserResponse>> {
    let user_id = id
        .parse()
        .map_err(|_| ApiError::from(AppError::Validation("Invalid user ID".to_string())))?;

    let user = UserRepository::find_by_id(&tenant.pool, tenant.org_id, user_id)
        .await?
        .ok_or_else(|| ApiError::from(DomainError::UserNotFound(id)))?;

    Ok(Json(UserResponse {
        id: user.id.to_string(),
        email: user.email,
        role: user.role.to_string(),
        first_name: user.first_name.clone(),
        last_name: user.last_name.clone(),
        full_name: format!("{} {}", user.first_name, user.last_name),
        phone: user.phone,
        timezone: user.timezone,
        created_at: user.created_at.to_rfc3339(),
    }))
}

/// Get the currently authenticated user's info
pub async fn get_me(
    State(_state): State<AppState>,
    auth_user: AuthUser,
    tenant: TenantContext,
) -> ApiResult<Json<UserResponse>> {
    let user = UserRepository::find_by_id(&tenant.pool, tenant.org_id, auth_user.user_id)
        .await?
        .ok_or_else(|| ApiError::from(DomainError::UserNotFound(auth_user.user_id.to_string())))?;

    Ok(Json(UserResponse {
        id: user.id.to_string(),
        email: user.email,
        role: user.role.to_string(),
        first_name: user.first_name.clone(),
        last_name: user.last_name.clone(),
        full_name: format!("{} {}", user.first_name, user.last_name),
        phone: user.phone,
        timezone: user.timezone,
        created_at: user.created_at.to_rfc3339(),
    }))
}

#[derive(Debug, Deserialize)]
pub struct UpdateMeRequest {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub phone: Option<String>,
    pub timezone: Option<String>,
}

/// Update the currently authenticated user's profile
pub async fn update_me(
    State(_state): State<AppState>,
    auth_user: AuthUser,
    tenant: TenantContext,
    Json(req): Json<UpdateMeRequest>,
) -> ApiResult<Json<UserResponse>> {
    let updated_user = UserRepository::update(
        &tenant.pool,
        tenant.org_id,
        auth_user.user_id,
        UpdateUser {
            first_name: req.first_name,
            last_name: req.last_name,
            phone: req.phone,
            timezone: req.timezone,
        },
    )
    .await?
    .ok_or_else(|| ApiError::from(DomainError::UserNotFound(auth_user.user_id.to_string())))?;

    Ok(Json(UserResponse {
        id: updated_user.id.to_string(),
        email: updated_user.email,
        role: updated_user.role.to_string(),
        first_name: updated_user.first_name.clone(),
        last_name: updated_user.last_name.clone(),
        full_name: format!("{} {}", updated_user.first_name, updated_user.last_name),
        phone: updated_user.phone,
        timezone: updated_user.timezone,
        created_at: updated_user.created_at.to_rfc3339(),
    }))
}
