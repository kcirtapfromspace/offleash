use thiserror::Error;

/// Domain-level errors representing business logic violations
#[derive(Debug, Error)]
pub enum DomainError {
    #[error("Time slot is not available")]
    SlotNotAvailable,

    #[error("Insufficient travel time between appointments")]
    InsufficientTravelTime,

    #[error("Booking conflicts with existing appointment")]
    BookingConflict,

    #[error("Service not found: {0}")]
    ServiceNotFound(String),

    #[error("Walker not found: {0}")]
    WalkerNotFound(String),

    #[error("Location not found: {0}")]
    LocationNotFound(String),

    #[error("User not found: {0}")]
    UserNotFound(String),

    #[error("Booking not found: {0}")]
    BookingNotFound(String),

    #[error("Invalid booking time: {0}")]
    InvalidBookingTime(String),

    #[error("Booking cannot be modified in current state: {0}")]
    InvalidStateTransition(String),

    #[error("Payment required before booking can be confirmed")]
    PaymentRequired,

    #[error("Insufficient notice time for booking (minimum {min_hours} hours required)")]
    InsufficientNotice { min_hours: i32 },

    #[error("Booking is too far in advance (maximum {max_days} days)")]
    TooFarInAdvance { max_days: i32 },

    #[error("Working hours not configured for this day")]
    NoWorkingHours,

    #[error("Outside working hours")]
    OutsideWorkingHours,

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Email already exists")]
    EmailAlreadyExists,

    #[error("Organization not found: {0}")]
    OrganizationNotFound(String),

    #[error("Organization slug already exists: {0}")]
    SlugAlreadyExists(String),

    #[error("Invalid token")]
    InvalidToken,

    #[error("Token expired")]
    TokenExpired,
}
