use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use shared::AppError;

/// API error wrapper that implements IntoResponse
pub struct ApiError(pub AppError);

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = StatusCode::from_u16(self.0.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        let error_code = self.0.error_code();
        let message = self.0.to_string();

        // Log internal errors
        if status.is_server_error() {
            tracing::error!("Internal error: {:?}", self.0);
        }

        let body = json!({
            "error": {
                "code": error_code,
                "message": message,
            }
        });

        (status, Json(body)).into_response()
    }
}

impl From<AppError> for ApiError {
    fn from(err: AppError) -> Self {
        Self(err)
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(err: sqlx::Error) -> Self {
        Self(AppError::Database(err.to_string()))
    }
}

impl From<shared::DomainError> for ApiError {
    fn from(err: shared::DomainError) -> Self {
        Self(AppError::Domain(err))
    }
}

/// Result type for API handlers
pub type ApiResult<T> = Result<T, ApiError>;
