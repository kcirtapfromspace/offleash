use axum::{extract::State, Json};
use db::models::{AuthProvider, CreateUser, CreateUserIdentity, UserRole};
use db::{OrganizationRepository, UserIdentityRepository, UserRepository};
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use once_cell::sync::Lazy;
use reqwest::Client;
use serde::Deserialize;
use shared::types::UserId;
use std::sync::RwLock;
use std::time::{Duration, Instant};

use crate::{
    auth::create_token,
    error::{ApiError, ApiResult},
    routes::auth::{AuthResponse, UserResponse},
    state::AppState,
};

// Cache for Google's public keys (JWKs)
static GOOGLE_KEYS_CACHE: Lazy<RwLock<Option<(Instant, GoogleKeys)>>> =
    Lazy::new(|| RwLock::new(None));

// Cache for Apple's public keys
static APPLE_KEYS_CACHE: Lazy<RwLock<Option<(Instant, AppleKeys)>>> =
    Lazy::new(|| RwLock::new(None));

const KEYS_CACHE_DURATION: Duration = Duration::from_secs(3600); // 1 hour

#[derive(Debug, Deserialize)]
pub struct GoogleAuthRequest {
    pub org_slug: String,
    pub id_token: String,
}

#[derive(Debug, Deserialize)]
pub struct AppleAuthRequest {
    pub org_slug: String,
    pub id_token: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

// Google JWT claims
#[derive(Debug, Deserialize)]
struct GoogleClaims {
    sub: String,           // Google user ID
    email: String,
    email_verified: bool,
    name: Option<String>,
    given_name: Option<String>,
    family_name: Option<String>,
    picture: Option<String>,
}

// Apple JWT claims
#[derive(Debug, Deserialize)]
struct AppleClaims {
    sub: String,           // Apple user ID
    email: Option<String>,
}

// Google JWK response
#[derive(Debug, Deserialize, Clone)]
struct GoogleKeys {
    keys: Vec<JwkKey>,
}

// Apple JWK response
#[derive(Debug, Deserialize, Clone)]
struct AppleKeys {
    keys: Vec<JwkKey>,
}

#[derive(Debug, Deserialize, Clone)]
struct JwkKey {
    kid: String,
    n: String,   // RSA modulus
    e: String,   // RSA exponent
}

/// Google OAuth sign-in
pub async fn google_auth(
    State(state): State<AppState>,
    Json(req): Json<GoogleAuthRequest>,
) -> ApiResult<Json<AuthResponse>> {
    // Get Google client ID from environment
    let google_client_id = std::env::var("GOOGLE_CLIENT_ID")
        .map_err(|_| ApiError::from(shared::AppError::Internal(
            "Google OAuth not configured".to_string()
        )))?;

    // Verify the ID token
    let claims = verify_google_token(&req.id_token, &google_client_id).await?;

    // Look up organization
    let organization = OrganizationRepository::find_by_slug(&state.pool, &req.org_slug)
        .await?
        .ok_or_else(|| ApiError::from(shared::DomainError::OrganizationNotFound(req.org_slug.clone())))?;

    // Try to find existing identity
    if let Some(identity) = UserIdentityRepository::find_by_provider(
        &state.pool,
        AuthProvider::Google,
        &claims.sub,
    ).await? {
        // Existing user - fetch and return token
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
        }));
    }

    // Try to find existing user by email to link accounts
    if let Some(existing_user) = UserRepository::find_by_email(&state.pool, organization.id, &claims.email).await? {
        // Link Google identity to existing user
        UserIdentityRepository::create(&state.pool, CreateUserIdentity {
            user_id: existing_user.id.into_uuid(),
            provider: AuthProvider::Google,
            provider_user_id: claims.sub,
            provider_email: Some(claims.email.clone()),
            provider_data: Some(serde_json::json!({
                "name": claims.name,
                "picture": claims.picture,
            })),
        }).await?;

        let token = create_token(existing_user.id, Some(existing_user.organization_id), &state.jwt_secret)
            .map_err(|_| ApiError::from(shared::AppError::Internal("Token creation failed".to_string())))?;

        return Ok(Json(AuthResponse {
            token,
            user: UserResponse {
                id: existing_user.id.to_string(),
                email: existing_user.email,
                first_name: existing_user.first_name,
                last_name: existing_user.last_name,
                role: existing_user.role.to_string(),
            },
        }));
    }

    // Create new user
    let first_name = claims.given_name.unwrap_or_else(||
        claims.name.clone().unwrap_or_else(|| "User".to_string())
    );
    let last_name = claims.family_name.unwrap_or_default();

    let user = UserRepository::create(
        &state.pool,
        CreateUser {
            organization_id: organization.id,
            email: claims.email.clone(),
            password_hash: "".to_string(), // No password for OAuth users
            role: UserRole::Customer,
            first_name: first_name.clone(),
            last_name: last_name.clone(),
            phone: None,
            timezone: None,
        },
    ).await?;

    // Create identity
    UserIdentityRepository::create(&state.pool, CreateUserIdentity {
        user_id: user.id.into_uuid(),
        provider: AuthProvider::Google,
        provider_user_id: claims.sub,
        provider_email: Some(claims.email.clone()),
        provider_data: Some(serde_json::json!({
            "name": claims.name,
            "picture": claims.picture,
        })),
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
    }))
}

/// Apple Sign-In
pub async fn apple_auth(
    State(state): State<AppState>,
    Json(req): Json<AppleAuthRequest>,
) -> ApiResult<Json<AuthResponse>> {
    // Get Apple client ID (Service ID) from environment
    let apple_client_id = std::env::var("APPLE_SERVICE_ID")
        .map_err(|_| ApiError::from(shared::AppError::Internal(
            "Apple Sign-In not configured".to_string()
        )))?;

    // Verify the ID token
    let claims = verify_apple_token(&req.id_token, &apple_client_id).await?;

    // Look up organization
    let organization = OrganizationRepository::find_by_slug(&state.pool, &req.org_slug)
        .await?
        .ok_or_else(|| ApiError::from(shared::DomainError::OrganizationNotFound(req.org_slug.clone())))?;

    // Try to find existing identity
    if let Some(identity) = UserIdentityRepository::find_by_provider(
        &state.pool,
        AuthProvider::Apple,
        &claims.sub,
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
        }));
    }

    // Apple may or may not provide email (privacy relay or hidden)
    let email = claims.email.unwrap_or_else(|| format!("{}@privaterelay.appleid.com", claims.sub));

    // Try to find existing user by email
    if let Some(existing_user) = UserRepository::find_by_email(&state.pool, organization.id, &email).await? {
        // Link Apple identity
        UserIdentityRepository::create(&state.pool, CreateUserIdentity {
            user_id: existing_user.id.into_uuid(),
            provider: AuthProvider::Apple,
            provider_user_id: claims.sub,
            provider_email: Some(email.clone()),
            provider_data: None,
        }).await?;

        let token = create_token(existing_user.id, Some(existing_user.organization_id), &state.jwt_secret)
            .map_err(|_| ApiError::from(shared::AppError::Internal("Token creation failed".to_string())))?;

        return Ok(Json(AuthResponse {
            token,
            user: UserResponse {
                id: existing_user.id.to_string(),
                email: existing_user.email,
                first_name: existing_user.first_name,
                last_name: existing_user.last_name,
                role: existing_user.role.to_string(),
            },
        }));
    }

    // Create new user (Apple only sends name on first sign-in)
    let first_name = req.first_name.unwrap_or_else(|| "Apple".to_string());
    let last_name = req.last_name.unwrap_or_else(|| "User".to_string());

    let user = UserRepository::create(
        &state.pool,
        CreateUser {
            organization_id: organization.id,
            email: email.clone(),
            password_hash: "".to_string(),
            role: UserRole::Customer,
            first_name: first_name.clone(),
            last_name: last_name.clone(),
            phone: None,
            timezone: None,
        },
    ).await?;

    // Create identity
    UserIdentityRepository::create(&state.pool, CreateUserIdentity {
        user_id: user.id.into_uuid(),
        provider: AuthProvider::Apple,
        provider_user_id: claims.sub,
        provider_email: Some(email),
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
    }))
}

// Helper: Verify Google ID token
async fn verify_google_token(token: &str, client_id: &str) -> ApiResult<GoogleClaims> {
    let keys = get_google_keys().await?;

    // Decode the header to get the key ID (kid)
    let header = decode_header(token)
        .map_err(|_| ApiError::from(shared::DomainError::InvalidCredentials))?;

    let kid = header.kid
        .ok_or_else(|| ApiError::from(shared::DomainError::InvalidCredentials))?;

    // Find the matching key
    let key = keys.keys.iter()
        .find(|k| k.kid == kid)
        .ok_or_else(|| ApiError::from(shared::DomainError::InvalidCredentials))?;

    // Create decoding key from RSA components
    let decoding_key = DecodingKey::from_rsa_components(&key.n, &key.e)
        .map_err(|_| ApiError::from(shared::DomainError::InvalidCredentials))?;

    // Set up validation
    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_audience(&[client_id]);
    validation.set_issuer(&["https://accounts.google.com", "accounts.google.com"]);

    // Decode and verify the token
    let token_data = decode::<GoogleClaims>(token, &decoding_key, &validation)
        .map_err(|e| {
            tracing::error!("Google token verification failed: {:?}", e);
            ApiError::from(shared::DomainError::InvalidCredentials)
        })?;

    // Verify email is verified
    if !token_data.claims.email_verified {
        return Err(ApiError::from(shared::DomainError::InvalidCredentials));
    }

    Ok(token_data.claims)
}

// Helper: Verify Apple ID token
async fn verify_apple_token(token: &str, client_id: &str) -> ApiResult<AppleClaims> {
    let keys = get_apple_keys().await?;

    let header = decode_header(token)
        .map_err(|_| ApiError::from(shared::DomainError::InvalidCredentials))?;

    let kid = header.kid
        .ok_or_else(|| ApiError::from(shared::DomainError::InvalidCredentials))?;

    let key = keys.keys.iter()
        .find(|k| k.kid == kid)
        .ok_or_else(|| ApiError::from(shared::DomainError::InvalidCredentials))?;

    let decoding_key = DecodingKey::from_rsa_components(&key.n, &key.e)
        .map_err(|_| ApiError::from(shared::DomainError::InvalidCredentials))?;

    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_audience(&[client_id]);
    validation.set_issuer(&["https://appleid.apple.com"]);

    let token_data = decode::<AppleClaims>(token, &decoding_key, &validation)
        .map_err(|e| {
            tracing::error!("Apple token verification failed: {:?}", e);
            ApiError::from(shared::DomainError::InvalidCredentials)
        })?;

    Ok(token_data.claims)
}

// Helper: Fetch Google's public keys with caching
async fn get_google_keys() -> ApiResult<GoogleKeys> {
    // Check cache first
    {
        let cache = GOOGLE_KEYS_CACHE.read().unwrap();
        if let Some((fetched_at, keys)) = cache.as_ref() {
            if fetched_at.elapsed() < KEYS_CACHE_DURATION {
                return Ok(keys.clone());
            }
        }
    }

    // Fetch fresh keys
    let client = Client::new();
    let response = client
        .get("https://www.googleapis.com/oauth2/v3/certs")
        .send()
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch Google keys: {:?}", e);
            ApiError::from(shared::AppError::Internal("Failed to verify token".to_string()))
        })?;

    let keys: GoogleKeys = response.json().await
        .map_err(|e| {
            tracing::error!("Failed to parse Google keys: {:?}", e);
            ApiError::from(shared::AppError::Internal("Failed to verify token".to_string()))
        })?;

    // Update cache
    {
        let mut cache = GOOGLE_KEYS_CACHE.write().unwrap();
        *cache = Some((Instant::now(), keys.clone()));
    }

    Ok(keys)
}

// Helper: Fetch Apple's public keys with caching
async fn get_apple_keys() -> ApiResult<AppleKeys> {
    // Check cache first
    {
        let cache = APPLE_KEYS_CACHE.read().unwrap();
        if let Some((fetched_at, keys)) = cache.as_ref() {
            if fetched_at.elapsed() < KEYS_CACHE_DURATION {
                return Ok(keys.clone());
            }
        }
    }

    // Fetch fresh keys
    let client = Client::new();
    let response = client
        .get("https://appleid.apple.com/auth/keys")
        .send()
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch Apple keys: {:?}", e);
            ApiError::from(shared::AppError::Internal("Failed to verify token".to_string()))
        })?;

    let keys: AppleKeys = response.json().await
        .map_err(|e| {
            tracing::error!("Failed to parse Apple keys: {:?}", e);
            ApiError::from(shared::AppError::Internal("Failed to verify token".to_string()))
        })?;

    // Update cache
    {
        let mut cache = APPLE_KEYS_CACHE.write().unwrap();
        *cache = Some((Instant::now(), keys.clone()));
    }

    Ok(keys)
}
