use axum::{extract::State, Json};
use db::PlatformAdminRepository;
use serde::{Deserialize, Serialize};
use shared::DomainError;

use crate::{
    auth::create_platform_admin_token,
    error::{ApiError, ApiResult},
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct PlatformLoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct PlatformAuthResponse {
    pub token: String,
    pub admin: PlatformAdminResponse,
}

#[derive(Debug, Serialize)]
pub struct PlatformAdminResponse {
    pub id: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
}

pub async fn platform_login(
    State(state): State<AppState>,
    Json(req): Json<PlatformLoginRequest>,
) -> ApiResult<Json<PlatformAuthResponse>> {
    // Find admin by email
    let admin = PlatformAdminRepository::find_by_email(&state.pool, &req.email)
        .await?
        .ok_or_else(|| ApiError::from(DomainError::InvalidCredentials))?;

    // Verify password
    if !PlatformAdminRepository::verify_password(&admin, &req.password) {
        return Err(ApiError::from(DomainError::InvalidCredentials));
    }

    // Create token with platform_admin claim (no org_id)
    let token = create_platform_admin_token(admin.id, &state.jwt_secret).map_err(|_| {
        ApiError::from(shared::AppError::Internal(
            "Token creation failed".to_string(),
        ))
    })?;

    Ok(Json(PlatformAuthResponse {
        token,
        admin: PlatformAdminResponse {
            id: admin.id.to_string(),
            email: admin.email,
            first_name: admin.first_name,
            last_name: admin.last_name,
        },
    }))
}
