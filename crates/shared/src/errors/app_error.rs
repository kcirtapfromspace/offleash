use super::DomainError;
use thiserror::Error;

/// Application-level errors (includes infrastructure)
#[derive(Debug, Error)]
pub enum AppError {
    #[error("Domain error: {0}")]
    Domain(#[from] DomainError),

    #[error("Database error: {0}")]
    Database(String),

    #[error("External API error: {0}")]
    ExternalApi(String),

    #[error("Authentication required")]
    Unauthorized,

    #[error("Permission denied")]
    Forbidden,

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("Request timeout")]
    Timeout,

    #[error("Rate limit exceeded")]
    RateLimited,
}

impl AppError {
    /// Get the HTTP status code for this error
    pub fn status_code(&self) -> u16 {
        match self {
            AppError::Domain(e) => match e {
                DomainError::SlotNotAvailable
                | DomainError::BookingConflict
                | DomainError::InsufficientTravelTime => 409, // Conflict
                DomainError::ServiceNotFound(_)
                | DomainError::WalkerNotFound(_)
                | DomainError::LocationNotFound(_)
                | DomainError::UserNotFound(_)
                | DomainError::BookingNotFound(_)
                | DomainError::OrganizationNotFound(_)
                | DomainError::TenantNotFound(_) => 404, // Not Found
                DomainError::InvalidCredentials | DomainError::InvalidToken => 401, // Unauthorized
                DomainError::TokenExpired => 401,
                DomainError::EmailAlreadyExists | DomainError::SlugAlreadyExists(_) => 409, // Conflict
                _ => 400, // Bad Request
            },
            AppError::Database(_) | AppError::Internal(_) => 500, // Internal Server Error
            AppError::ExternalApi(_) => 503,                      // Service Unavailable
            AppError::Unauthorized => 401,
            AppError::Forbidden => 403,
            AppError::NotFound(_) => 404,
            AppError::Validation(_) => 422, // Unprocessable Entity
            AppError::Timeout => 504,       // Gateway Timeout
            AppError::RateLimited => 429,   // Too Many Requests
        }
    }

    /// Get a machine-readable error code
    pub fn error_code(&self) -> &'static str {
        match self {
            AppError::Domain(e) => match e {
                DomainError::SlotNotAvailable => "SLOT_NOT_AVAILABLE",
                DomainError::BookingConflict => "BOOKING_CONFLICT",
                DomainError::InsufficientTravelTime => "INSUFFICIENT_TRAVEL_TIME",
                DomainError::ServiceNotFound(_) => "SERVICE_NOT_FOUND",
                DomainError::WalkerNotFound(_) => "WALKER_NOT_FOUND",
                DomainError::LocationNotFound(_) => "LOCATION_NOT_FOUND",
                DomainError::UserNotFound(_) => "USER_NOT_FOUND",
                DomainError::BookingNotFound(_) => "BOOKING_NOT_FOUND",
                DomainError::OrganizationNotFound(_) => "ORGANIZATION_NOT_FOUND",
                DomainError::TenantNotFound(_) => "TENANT_NOT_FOUND",
                DomainError::InvalidCredentials => "INVALID_CREDENTIALS",
                DomainError::InvalidToken => "INVALID_TOKEN",
                DomainError::TokenExpired => "TOKEN_EXPIRED",
                DomainError::EmailAlreadyExists => "EMAIL_EXISTS",
                DomainError::SlugAlreadyExists(_) => "SLUG_EXISTS",
                _ => "DOMAIN_ERROR",
            },
            AppError::Database(_) => "DATABASE_ERROR",
            AppError::ExternalApi(_) => "EXTERNAL_API_ERROR",
            AppError::Unauthorized => "UNAUTHORIZED",
            AppError::Forbidden => "FORBIDDEN",
            AppError::NotFound(_) => "NOT_FOUND",
            AppError::Validation(_) => "VALIDATION_ERROR",
            AppError::Internal(_) => "INTERNAL_ERROR",
            AppError::Timeout => "TIMEOUT",
            AppError::RateLimited => "RATE_LIMITED",
        }
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        // Log the actual error but don't expose details
        tracing::error!("Database error: {:?}", err);
        AppError::Database(err.to_string())
    }
}

/// Result type alias for application operations
#[allow(dead_code)]
pub type AppResult<T> = Result<T, AppError>;
