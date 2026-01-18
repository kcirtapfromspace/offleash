use axum::{
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts, StatusCode},
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use shared::types::{OrganizationId, PlatformAdminId, UserId};
use sqlx::PgPool;
use std::future::Future;

use crate::state::AppState;

/// JWT claims
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,                  // User ID or Platform Admin ID
    pub org_id: Option<String>,       // Organization ID (None for platform admins)
    pub platform_admin: Option<bool>, // True if this is a platform admin token
    pub exp: usize,                   // Expiration time
    pub iat: usize,                   // Issued at
}

impl Claims {
    pub fn new(user_id: UserId, org_id: Option<OrganizationId>, expires_in_hours: i64) -> Self {
        let now = chrono::Utc::now();
        Self {
            sub: user_id.to_string(),
            org_id: org_id.map(|id| id.to_string()),
            platform_admin: None,
            exp: (now + chrono::Duration::hours(expires_in_hours)).timestamp() as usize,
            iat: now.timestamp() as usize,
        }
    }

    pub fn new_platform_admin(admin_id: PlatformAdminId, expires_in_hours: i64) -> Self {
        let now = chrono::Utc::now();
        Self {
            sub: admin_id.to_string(),
            org_id: None,
            platform_admin: Some(true),
            exp: (now + chrono::Duration::hours(expires_in_hours)).timestamp() as usize,
            iat: now.timestamp() as usize,
        }
    }

    pub fn user_id(&self) -> Option<UserId> {
        self.sub.parse().ok()
    }

    pub fn org_id(&self) -> Option<OrganizationId> {
        self.org_id.as_ref().and_then(|id| id.parse().ok())
    }

    pub fn is_platform_admin(&self) -> bool {
        self.platform_admin.unwrap_or(false)
    }

    pub fn platform_admin_id(&self) -> Option<PlatformAdminId> {
        if self.is_platform_admin() {
            self.sub.parse().ok()
        } else {
            None
        }
    }
}

/// Create a JWT token
pub fn create_token(
    user_id: UserId,
    org_id: Option<OrganizationId>,
    secret: &str,
) -> Result<String, jsonwebtoken::errors::Error> {
    let claims = Claims::new(user_id, org_id, 24); // 24 hour expiry
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

/// Create a JWT token for platform admin
pub fn create_platform_admin_token(
    admin_id: PlatformAdminId,
    secret: &str,
) -> Result<String, jsonwebtoken::errors::Error> {
    let claims = Claims::new_platform_admin(admin_id, 24); // 24 hour expiry
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

/// Verify and decode a JWT token
pub fn verify_token(token: &str, secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;
    Ok(token_data.claims)
}

/// Extractor for authenticated user
pub struct AuthUser {
    pub user_id: UserId,
    pub org_id: Option<OrganizationId>,
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = (StatusCode, &'static str);

    fn from_request_parts<'life0, 'life1, 'async_trait>(
        parts: &'life0 mut Parts,
        state: &'life1 AppState,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<Self, Self::Rejection>> + Send + 'async_trait>>
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        Self: 'async_trait,
    {
        let auth_result = (|| {
            let auth_header = parts
                .headers
                .get(AUTHORIZATION)
                .and_then(|h| h.to_str().ok())
                .ok_or((StatusCode::UNAUTHORIZED, "Missing authorization header"))?;

            let token = auth_header
                .strip_prefix("Bearer ")
                .ok_or((StatusCode::UNAUTHORIZED, "Invalid authorization header"))?;

            let claims = verify_token(token, &state.jwt_secret)
                .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token"))?;

            let user_id = claims
                .user_id()
                .ok_or((StatusCode::UNAUTHORIZED, "Invalid user ID in token"))?;

            let org_id = claims.org_id();

            Ok(AuthUser { user_id, org_id })
        })();

        Box::pin(std::future::ready(auth_result))
    }
}

/// Extractor for tenant context providing org_id and tenant database pool
pub struct TenantContext {
    pub org_id: OrganizationId,
    pub pool: PgPool,
}

impl FromRequestParts<AppState> for TenantContext {
    type Rejection = (StatusCode, &'static str);

    fn from_request_parts<'life0, 'life1, 'async_trait>(
        parts: &'life0 mut Parts,
        state: &'life1 AppState,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<Self, Self::Rejection>> + Send + 'async_trait>>
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        Self: 'async_trait,
    {
        let jwt_secret = state.jwt_secret.clone();
        let tenant_pool_manager = state.tenant_pool_manager.clone();

        Box::pin(async move {
            // Extract and validate JWT token
            let auth_header = parts
                .headers
                .get(AUTHORIZATION)
                .and_then(|h| h.to_str().ok())
                .ok_or((StatusCode::UNAUTHORIZED, "Missing authorization header"))?;

            let token = auth_header
                .strip_prefix("Bearer ")
                .ok_or((StatusCode::UNAUTHORIZED, "Invalid authorization header"))?;

            let claims = verify_token(token, &jwt_secret)
                .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token"))?;

            // Extract org_id from claims - return 401 if missing
            let org_id = claims.org_id().ok_or((
                StatusCode::UNAUTHORIZED,
                "Organization ID missing from token",
            ))?;

            // Get tenant pool from TenantPoolManager
            let pool = tenant_pool_manager
                .get_pool(org_id)
                .await
                .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid organization"))?;

            Ok(TenantContext { org_id, pool })
        })
    }
}

/// Extractor for platform admin authentication
pub struct PlatformAdminAuth {
    pub admin_id: PlatformAdminId,
}

impl FromRequestParts<AppState> for PlatformAdminAuth {
    type Rejection = (StatusCode, &'static str);

    fn from_request_parts<'life0, 'life1, 'async_trait>(
        parts: &'life0 mut Parts,
        state: &'life1 AppState,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<Self, Self::Rejection>> + Send + 'async_trait>>
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        Self: 'async_trait,
    {
        let auth_result = (|| {
            let auth_header = parts
                .headers
                .get(AUTHORIZATION)
                .and_then(|h| h.to_str().ok())
                .ok_or((StatusCode::UNAUTHORIZED, "Missing authorization header"))?;

            let token = auth_header
                .strip_prefix("Bearer ")
                .ok_or((StatusCode::UNAUTHORIZED, "Invalid authorization header"))?;

            let claims = verify_token(token, &state.jwt_secret)
                .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token"))?;

            // Verify this is a platform admin token
            if !claims.is_platform_admin() {
                return Err((
                    StatusCode::UNAUTHORIZED,
                    "Token is not a platform admin token",
                ));
            }

            let admin_id = claims
                .platform_admin_id()
                .ok_or((StatusCode::UNAUTHORIZED, "Invalid admin ID in token"))?;

            Ok(PlatformAdminAuth { admin_id })
        })();

        Box::pin(std::future::ready(auth_result))
    }
}
