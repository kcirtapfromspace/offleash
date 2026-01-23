use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{extract::State, Json};
use chrono::{Duration, Utc};
use db::models::{AuthProvider, CreatePhoneVerification, CreateUser, CreateUserIdentity, UserRole};
use db::{OrganizationRepository, PhoneVerificationRepository, UserIdentityRepository, UserRepository};
use rand::Rng;
use serde::{Deserialize, Serialize};
use shared::types::UserId;

use crate::{
    auth::create_token,
    error::{ApiError, ApiResult},
    routes::auth::{AuthResponse, UserResponse},
    state::AppState,
};

const MAX_CODES_PER_HOUR: i64 = 3;
const MAX_VERIFICATION_ATTEMPTS: i32 = 5;
const CODE_EXPIRY_MINUTES: i64 = 10;

#[derive(Debug, Deserialize)]
pub struct SendCodeRequest {
    pub org_slug: String,
    pub phone_number: String,
}

#[derive(Debug, Serialize)]
pub struct SendCodeResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct VerifyCodeRequest {
    pub org_slug: String,
    pub phone_number: String,
    pub code: String,
}

/// Validate phone number is in E.164 format
fn validate_phone_number(phone: &str) -> Result<String, ApiError> {
    // Basic E.164 validation: starts with +, followed by 10-15 digits
    let cleaned: String = phone.chars().filter(|c| c.is_ascii_digit() || *c == '+').collect();

    if !cleaned.starts_with('+') {
        return Err(ApiError::from(shared::DomainError::ValidationError(
            "Phone number must start with country code (e.g., +1)".to_string(),
        )));
    }

    let digits: String = cleaned.chars().skip(1).collect();
    if digits.len() < 10 || digits.len() > 15 {
        return Err(ApiError::from(shared::DomainError::ValidationError(
            "Invalid phone number format".to_string(),
        )));
    }

    Ok(cleaned)
}

/// Generate a 6-digit OTP code
fn generate_otp() -> String {
    let mut rng = rand::thread_rng();
    format!("{:06}", rng.gen_range(0..1000000))
}

/// Hash the OTP code for storage
fn hash_code(code: &str) -> Result<String, ApiError> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(code.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|_| ApiError::from(shared::AppError::Internal("Failed to hash code".to_string())))
}

/// Verify the OTP code against stored hash
fn verify_code(code: &str, hash: &str) -> bool {
    PasswordHash::new(hash)
        .ok()
        .map(|parsed| Argon2::default().verify_password(code.as_bytes(), &parsed).is_ok())
        .unwrap_or(false)
}

/// Send verification code to phone number
pub async fn send_code(
    State(state): State<AppState>,
    Json(req): Json<SendCodeRequest>,
) -> ApiResult<Json<SendCodeResponse>> {
    // Validate phone number format
    let phone_number = validate_phone_number(&req.phone_number)?;

    // Verify organization exists
    let _organization = OrganizationRepository::find_by_slug(&state.pool, &req.org_slug)
        .await?
        .ok_or_else(|| ApiError::from(shared::DomainError::OrganizationNotFound(req.org_slug.clone())))?;

    // Rate limit: check how many codes sent in last hour
    let recent_count = PhoneVerificationRepository::count_recent(&state.pool, &phone_number).await?;
    if recent_count >= MAX_CODES_PER_HOUR {
        // Don't reveal rate limit to prevent enumeration
        return Ok(Json(SendCodeResponse {
            success: true,
            message: "If this phone number is valid, you will receive a code shortly.".to_string(),
        }));
    }

    // Generate OTP
    let code = generate_otp();
    let code_hash = hash_code(&code)?;
    let expires_at = Utc::now() + Duration::minutes(CODE_EXPIRY_MINUTES);

    // Store verification record
    PhoneVerificationRepository::create(
        &state.pool,
        CreatePhoneVerification {
            phone_number: phone_number.clone(),
            code_hash,
            expires_at,
        },
    ).await?;

    // Send SMS via Twilio (if configured)
    if let Err(e) = send_sms(&phone_number, &code).await {
        tracing::error!("Failed to send SMS to {}: {:?}", phone_number, e);
        // Still return success to not reveal if phone is valid
    } else {
        tracing::info!("Sent verification code to {}", phone_number);
    }

    Ok(Json(SendCodeResponse {
        success: true,
        message: "If this phone number is valid, you will receive a code shortly.".to_string(),
    }))
}

/// Verify the code and authenticate
pub async fn verify_code_endpoint(
    State(state): State<AppState>,
    Json(req): Json<VerifyCodeRequest>,
) -> ApiResult<Json<AuthResponse>> {
    // Validate phone number format
    let phone_number = validate_phone_number(&req.phone_number)?;

    // Look up organization
    let organization = OrganizationRepository::find_by_slug(&state.pool, &req.org_slug)
        .await?
        .ok_or_else(|| ApiError::from(shared::DomainError::OrganizationNotFound(req.org_slug.clone())))?;

    // Find active verification
    let verification = PhoneVerificationRepository::find_active(&state.pool, &phone_number)
        .await?
        .ok_or_else(|| ApiError::from(shared::DomainError::InvalidCredentials))?;

    // Check if locked out
    if verification.attempts >= MAX_VERIFICATION_ATTEMPTS {
        return Err(ApiError::from(shared::DomainError::ValidationError(
            "Too many failed attempts. Please request a new code.".to_string(),
        )));
    }

    // Verify the code
    if !verify_code(&req.code, &verification.code_hash) {
        // Increment attempts
        PhoneVerificationRepository::increment_attempts(&state.pool, verification.id).await?;
        return Err(ApiError::from(shared::DomainError::InvalidCredentials));
    }

    // Code is valid - delete the verification record
    PhoneVerificationRepository::delete(&state.pool, verification.id).await?;

    // Try to find existing identity
    if let Some(identity) = UserIdentityRepository::find_by_provider(
        &state.pool,
        AuthProvider::Phone,
        &phone_number,
    ).await? {
        // Existing user
        let user_id = UserId::from_uuid(identity.user_id);
        let user = UserRepository::find_by_id_unchecked(&state.pool, user_id)
            .await?
            .ok_or_else(|| ApiError::from(shared::AppError::Internal("User not found".to_string())))?;

        let token = create_token(user.id, Some(user.organization_id), &state.jwt_secret)
            .map_err(|_| ApiError::from(shared::AppError::Internal("Token creation failed".to_string())))?;

        return Ok(Json(AuthResponse {
            token,
            user: UserResponse {
                id: user.id.to_string(),
                email: user.email,
                first_name: user.first_name,
                last_name: user.last_name,
                role: user.role.to_string(),
            },
            membership: None,
            memberships: None,
        }));
    }

    // No existing identity - create new user
    // Generate a placeholder email from phone number
    let email = format!("{}@phone.offleash.app", phone_number.replace('+', ""));

    let user = UserRepository::create(
        &state.pool,
        CreateUser {
            organization_id: organization.id,
            email: email.clone(),
            password_hash: "".to_string(), // No password for phone users
            role: UserRole::Customer,
            first_name: "Phone".to_string(),
            last_name: "User".to_string(),
            phone: Some(phone_number.clone()),
            timezone: None,
        },
    ).await?;

    // Create identity
    UserIdentityRepository::create(&state.pool, CreateUserIdentity {
        user_id: user.id.into_uuid(),
        provider: AuthProvider::Phone,
        provider_user_id: phone_number,
        provider_email: None,
        provider_data: None,
    }).await?;

    let token = create_token(user.id, Some(user.organization_id), &state.jwt_secret)
        .map_err(|_| ApiError::from(shared::AppError::Internal("Token creation failed".to_string())))?;

    Ok(Json(AuthResponse {
        token,
        user: UserResponse {
            id: user.id.to_string(),
            email: user.email,
            first_name: user.first_name,
            last_name: user.last_name,
            role: user.role.to_string(),
        },
        membership: None,
        memberships: None,
    }))
}

/// Send SMS via Twilio
async fn send_sms(phone_number: &str, code: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let account_sid = std::env::var("TWILIO_ACCOUNT_SID")?;
    let auth_token = std::env::var("TWILIO_AUTH_TOKEN")?;
    let from_number = std::env::var("TWILIO_PHONE_NUMBER")?;

    let client = reqwest::Client::new();
    let url = format!(
        "https://api.twilio.com/2010-04-01/Accounts/{}/Messages.json",
        account_sid
    );

    let message = format!("Your OFFLEASH code is: {}. Expires in 10 minutes.", code);

    let response = client
        .post(&url)
        .basic_auth(&account_sid, Some(&auth_token))
        .form(&[
            ("To", phone_number),
            ("From", &from_number),
            ("Body", &message),
        ])
        .send()
        .await?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        tracing::error!("Twilio error: {}", error_text);
        return Err(format!("Twilio error: {}", error_text).into());
    }

    Ok(())
}
