use axum::{extract::State, Json};
use db::models::{AuthProvider, CreateMembership, CreateUser, CreateUserIdentity, MembershipRole, MembershipStatus, UserRole};
use db::{MembershipRepository, OrganizationRepository, UserIdentityRepository, UserRepository};
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
    routes::auth::{AuthResponse, MembershipInfo, UserResponse},
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
    #[serde(default)]
    pub org_slug: Option<String>,
    pub id_token: String,
}

#[derive(Debug, Deserialize)]
pub struct AppleAuthRequest {
    #[serde(default)]
    pub org_slug: Option<String>,
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

/// Google OAuth sign-in (Universal - works with or without org_slug)
pub async fn google_auth(
    State(state): State<AppState>,
    Json(req): Json<GoogleAuthRequest>,
) -> ApiResult<Json<AuthResponse>> {
    // Get Google client ID from environment (supports both names for flexibility)
    let google_client_id = std::env::var("PUBLIC_GOOGLE_CLIENT_ID")
        .or_else(|_| std::env::var("GOOGLE_CLIENT_ID"))
        .map_err(|_| ApiError::from(shared::AppError::Internal(
            "Google OAuth not configured".to_string()
        )))?;

    // Verify the ID token
    let claims = verify_google_token(&req.id_token, &google_client_id).await?;

    // Try to find existing identity (existing OAuth user)
    if let Some(identity) = UserIdentityRepository::find_by_provider(
        &state.pool,
        AuthProvider::Google,
        &claims.sub,
    ).await? {
        // Existing user - return with all memberships (universal style)
        let user_id = UserId::from_uuid(identity.user_id);
        let user = UserRepository::find_by_id_unchecked(&state.pool, user_id)
            .await?
            .ok_or_else(|| ApiError::from(shared::AppError::Internal("User not found".to_string())))?;

        return build_oauth_response(&state, user).await;
    }

    // Try to find existing user by email globally and link identity
    if let Some(existing_user) = UserRepository::find_by_email_globally(&state.pool, &claims.email).await? {
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

        return build_oauth_response(&state, existing_user).await;
    }

    // New user - need to create account
    // Get or create default organization for new OAuth users
    let organization = if let Some(ref slug) = req.org_slug {
        OrganizationRepository::find_by_slug(&state.pool, slug)
            .await?
            .ok_or_else(|| ApiError::from(shared::DomainError::OrganizationNotFound(slug.clone())))?
    } else {
        // Use first/default organization or create one
        OrganizationRepository::find_default(&state.pool)
            .await?
            .ok_or_else(|| ApiError::from(shared::AppError::Internal(
                "No default organization configured".to_string()
            )))?
    };

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

    // Create membership for the user
    let membership = MembershipRepository::create(
        &state.pool,
        CreateMembership {
            user_id: user.id,
            organization_id: organization.id,
            role: MembershipRole::Customer,
            status: Some(MembershipStatus::Active),
            title: None,
        },
    ).await?;

    // Set as default membership
    sqlx::query("UPDATE users SET default_membership_id = $1 WHERE id = $2")
        .bind(membership.id)
        .bind(user.id)
        .execute(&state.pool)
        .await?;

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

    build_oauth_response(&state, user).await
}

/// Apple Sign-In (Universal - works with or without org_slug)
pub async fn apple_auth(
    State(state): State<AppState>,
    Json(req): Json<AppleAuthRequest>,
) -> ApiResult<Json<AuthResponse>> {
    // Get Apple client ID (Service ID) from environment (supports both names for flexibility)
    let apple_client_id = std::env::var("PUBLIC_APPLE_CLIENT_ID")
        .or_else(|_| std::env::var("APPLE_SERVICE_ID"))
        .map_err(|_| ApiError::from(shared::AppError::Internal(
            "Apple Sign-In not configured".to_string()
        )))?;

    // Verify the ID token
    let claims = verify_apple_token(&req.id_token, &apple_client_id).await?;

    // Try to find existing identity
    if let Some(identity) = UserIdentityRepository::find_by_provider(
        &state.pool,
        AuthProvider::Apple,
        &claims.sub,
    ).await? {
        // Existing user - return with all memberships
        let user_id = UserId::from_uuid(identity.user_id);
        let user = UserRepository::find_by_id_unchecked(&state.pool, user_id)
            .await?
            .ok_or_else(|| ApiError::from(shared::AppError::Internal("User not found".to_string())))?;

        return build_oauth_response(&state, user).await;
    }

    // Apple may or may not provide email (privacy relay or hidden)
    // If user chose "Hide My Email", Apple provides a privaterelay address
    let (email, is_private_relay) = match &claims.email {
        Some(e) if e.ends_with("@privaterelay.appleid.com") => (e.clone(), true),
        Some(e) => (e.clone(), false),
        None => (format!("{}@privaterelay.appleid.com", claims.sub), true),
    };

    // Only auto-link by email if user shared their real email (not hidden)
    // If they chose "Hide My Email", we respect that and create a new account
    // Users can explicitly link accounts later in settings if needed
    if !is_private_relay {
        if let Some(existing_user) = UserRepository::find_by_email_globally(&state.pool, &email).await? {
            // Link Apple identity to existing user
            UserIdentityRepository::create(&state.pool, CreateUserIdentity {
                user_id: existing_user.id.into_uuid(),
                provider: AuthProvider::Apple,
                provider_user_id: claims.sub,
                provider_email: Some(email.clone()),
                provider_data: None,
            }).await?;

            return build_oauth_response(&state, existing_user).await;
        }
    }

    // New user - need to create account
    let organization = if let Some(ref slug) = req.org_slug {
        OrganizationRepository::find_by_slug(&state.pool, slug)
            .await?
            .ok_or_else(|| ApiError::from(shared::DomainError::OrganizationNotFound(slug.clone())))?
    } else {
        OrganizationRepository::find_default(&state.pool)
            .await?
            .ok_or_else(|| ApiError::from(shared::AppError::Internal(
                "No default organization configured".to_string()
            )))?
    };

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

    // Create membership
    let membership = MembershipRepository::create(
        &state.pool,
        CreateMembership {
            user_id: user.id,
            organization_id: organization.id,
            role: MembershipRole::Customer,
            status: Some(MembershipStatus::Active),
            title: None,
        },
    ).await?;

    // Set as default membership
    sqlx::query("UPDATE users SET default_membership_id = $1 WHERE id = $2")
        .bind(membership.id)
        .bind(user.id)
        .execute(&state.pool)
        .await?;

    // Create identity
    UserIdentityRepository::create(&state.pool, CreateUserIdentity {
        user_id: user.id.into_uuid(),
        provider: AuthProvider::Apple,
        provider_user_id: claims.sub,
        provider_email: Some(email),
        provider_data: None,
    }).await?;

    build_oauth_response(&state, user).await
}

/// Helper: Build OAuth response with all memberships (like universal_login)
async fn build_oauth_response(
    state: &AppState,
    user: db::models::User,
) -> ApiResult<Json<AuthResponse>> {
    use shared::types::OrganizationId;

    // Get all user's memberships
    let all_memberships = MembershipRepository::find_with_org_by_user(&state.pool, user.id).await?;

    // Get default membership ID
    let default_membership_id: Option<uuid::Uuid> = sqlx::query_scalar(
        "SELECT default_membership_id FROM users WHERE id = $1",
    )
    .bind(user.id)
    .fetch_optional(&state.pool)
    .await?
    .flatten();

    let memberships_info: Vec<MembershipInfo> = all_memberships
        .into_iter()
        .map(|m| MembershipInfo {
            id: m.id.to_string(),
            organization_id: m.organization_id.to_string(),
            organization_name: m.organization_name,
            organization_slug: m.organization_slug,
            role: m.role.to_string(),
            is_default: default_membership_id.map(|d| d == m.id).unwrap_or(false),
        })
        .collect();

    // Create token with default org context if available
    let (token, default_membership) = if let Some(default_id) = default_membership_id {
        if let Some(default_mem) = memberships_info.iter().find(|m| m.id == default_id.to_string()) {
            let org_id: uuid::Uuid = default_mem.organization_id.parse().unwrap();
            let token = create_token(user.id, Some(OrganizationId::from_uuid(org_id)), &state.jwt_secret)
                .map_err(|_| ApiError::from(shared::AppError::Internal("Token creation failed".to_string())))?;
            (token, Some(default_mem.clone()))
        } else {
            let token = create_token(user.id, None, &state.jwt_secret)
                .map_err(|_| ApiError::from(shared::AppError::Internal("Token creation failed".to_string())))?;
            (token, None)
        }
    } else if let Some(first_membership) = memberships_info.first() {
        let org_id: uuid::Uuid = first_membership.organization_id.parse().unwrap();
        let token = create_token(user.id, Some(OrganizationId::from_uuid(org_id)), &state.jwt_secret)
            .map_err(|_| ApiError::from(shared::AppError::Internal("Token creation failed".to_string())))?;
        (token, Some(first_membership.clone()))
    } else {
        let token = create_token(user.id, None, &state.jwt_secret)
            .map_err(|_| ApiError::from(shared::AppError::Internal("Token creation failed".to_string())))?;
        (token, None)
    };

    let role = default_membership
        .as_ref()
        .map(|m| m.role.clone())
        .unwrap_or_else(|| user.role.to_string());

    Ok(Json(AuthResponse {
        token,
        user: UserResponse {
            id: user.id.to_string(),
            email: user.email,
            first_name: user.first_name,
            last_name: user.last_name,
            role,
        },
        membership: default_membership,
        memberships: Some(memberships_info),
    }))
}

/// Public token verification result for linking identities
#[derive(Debug)]
pub struct VerifiedGoogleIdentity {
    pub provider_user_id: String,
    pub email: String,
    pub name: Option<String>,
    pub picture: Option<String>,
}

#[derive(Debug)]
pub struct VerifiedAppleIdentity {
    pub provider_user_id: String,
    pub email: Option<String>,
}

/// Public function to verify Google token and extract identity info
pub async fn verify_google_token_public(token: &str) -> ApiResult<VerifiedGoogleIdentity> {
    let google_client_id = std::env::var("PUBLIC_GOOGLE_CLIENT_ID")
        .or_else(|_| std::env::var("GOOGLE_CLIENT_ID"))
        .map_err(|_| ApiError::from(shared::AppError::Internal(
            "Google OAuth not configured".to_string()
        )))?;

    let claims = verify_google_token(token, &google_client_id).await?;

    Ok(VerifiedGoogleIdentity {
        provider_user_id: claims.sub,
        email: claims.email,
        name: claims.name,
        picture: claims.picture,
    })
}

/// Public function to verify Apple token and extract identity info
pub async fn verify_apple_token_public(token: &str) -> ApiResult<VerifiedAppleIdentity> {
    let apple_client_id = std::env::var("PUBLIC_APPLE_CLIENT_ID")
        .or_else(|_| std::env::var("APPLE_SERVICE_ID"))
        .map_err(|_| ApiError::from(shared::AppError::Internal(
            "Apple Sign-In not configured".to_string()
        )))?;

    let claims = verify_apple_token(token, &apple_client_id).await?;

    Ok(VerifiedAppleIdentity {
        provider_user_id: claims.sub,
        email: claims.email,
    })
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
