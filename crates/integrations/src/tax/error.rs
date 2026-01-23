use thiserror::Error;

pub type TaxResult<T> = Result<T, TaxError>;

#[derive(Debug, Error)]
pub enum TaxError {
    #[error("Tax API error: {message}")]
    ApiError {
        status: u16,
        message: String,
    },

    #[error("Invalid address: {0}")]
    InvalidAddress(String),

    #[error("Tax calculation failed: {0}")]
    CalculationError(String),

    #[error("Rate lookup failed: {0}")]
    RateLookupError(String),

    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Service unavailable")]
    ServiceUnavailable,
}

impl TaxError {
    pub fn is_retryable(&self) -> bool {
        matches!(self, TaxError::ServiceUnavailable | TaxError::HttpError(_))
    }
}
