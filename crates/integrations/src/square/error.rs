use thiserror::Error;

pub type SquareResult<T> = Result<T, SquareError>;

#[derive(Debug, Error)]
pub enum SquareError {
    #[error("Square API error: {message}")]
    ApiError {
        category: String,
        code: String,
        message: String,
        detail: Option<String>,
    },

    #[error("Card declined: {0}")]
    CardDeclined(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Authentication failed")]
    AuthenticationError,

    #[error("Rate limited")]
    RateLimited,

    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Missing required field: {0}")]
    MissingField(String),
}

impl SquareError {
    pub fn is_retryable(&self) -> bool {
        matches!(self, SquareError::RateLimited | SquareError::HttpError(_))
    }

    pub fn user_message(&self) -> &str {
        match self {
            SquareError::CardDeclined(_) => {
                "Your card was declined. Please try a different payment method."
            }
            SquareError::InvalidRequest(_) => "There was an issue with your payment information.",
            SquareError::RateLimited => "Too many requests. Please try again in a moment.",
            _ => "An error occurred processing your payment. Please try again.",
        }
    }
}
