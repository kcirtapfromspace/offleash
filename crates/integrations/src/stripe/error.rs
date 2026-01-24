use thiserror::Error;

pub type StripeResult<T> = Result<T, StripeError>;

#[derive(Debug, Error)]
pub enum StripeError {
    #[error("Stripe API error: {message}")]
    ApiError {
        code: Option<String>,
        message: String,
        param: Option<String>,
        decline_code: Option<String>,
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

    #[error("Webhook signature verification failed")]
    WebhookSignatureError,

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Invalid state: {0}")]
    InvalidState(String),
}

impl StripeError {
    /// Check if this is a card decline error
    pub fn is_card_error(&self) -> bool {
        matches!(self, StripeError::CardDeclined(_))
    }

    /// Check if this error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(self, StripeError::RateLimited | StripeError::HttpError(_))
    }

    /// Get user-friendly error message
    pub fn user_message(&self) -> &str {
        match self {
            StripeError::CardDeclined(code) => match code.as_str() {
                "insufficient_funds" => "Your card has insufficient funds.",
                "lost_card" | "stolen_card" => {
                    "Your card has been declined. Please contact your bank."
                }
                "expired_card" => "Your card has expired.",
                "incorrect_cvc" => "Your card's security code is incorrect.",
                "processing_error" => "An error occurred processing your card. Please try again.",
                _ => "Your card was declined. Please try a different payment method.",
            },
            StripeError::InvalidRequest(_) => "There was an issue with your payment information.",
            StripeError::RateLimited => "Too many requests. Please try again in a moment.",
            _ => "An error occurred processing your payment. Please try again.",
        }
    }
}
