//! Supabase Auth integration for OAuth and Passkey authentication
//!
//! This module handles verification of Supabase JWTs and user provisioning
//! for users who sign in via OAuth (Google, Apple) or Passkeys.

use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use shared::types::{OrganizationId, UserId};

/// Supabase JWT claims structure
/// Reference: https://supabase.com/docs/guides/auth/jwts
#[derive(Debug, Serialize, Deserialize)]
pub struct SupabaseClaims {
    /// Supabase user ID (UUID)
    pub sub: String,
    /// Audience - should be "authenticated" for logged-in users
    pub aud: String,
    /// User's email address
    pub email: Option<String>,
    /// Email verification status
    pub email_confirmed_at: Option<String>,
    /// Phone number (if provided)
    pub phone: Option<String>,
    /// App metadata (custom claims set by the app)
    #[serde(default)]
    pub app_metadata: AppMetadata,
    /// User metadata (profile data from OAuth provider)
    #[serde(default)]
    pub user_metadata: UserMetadata,
    /// Role (typically "authenticated")
    pub role: Option<String>,
    /// Expiration time
    pub exp: usize,
    /// Issued at
    pub iat: usize,
}

/// App metadata - custom claims we can set for the user
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AppMetadata {
    /// OAuth provider used (google, apple, etc.)
    pub provider: Option<String>,
    /// OFFLEASH organization ID (set after first login)
    pub offleash_org_id: Option<String>,
    /// OFFLEASH user ID (set after provisioning)
    pub offleash_user_id: Option<String>,
}

/// User metadata from OAuth provider
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct UserMetadata {
    /// User's full name
    pub full_name: Option<String>,
    /// User's first name
    pub name: Option<String>,
    /// Avatar URL
    pub avatar_url: Option<String>,
    /// Picture URL (Google)
    pub picture: Option<String>,
    /// Email from OAuth provider
    pub email: Option<String>,
    /// Email verified status
    pub email_verified: Option<bool>,
}

impl SupabaseClaims {
    /// Get the Supabase user ID
    pub fn supabase_user_id(&self) -> &str {
        &self.sub
    }

    /// Get the user's email
    pub fn email(&self) -> Option<&str> {
        self.email.as_deref()
            .or(self.user_metadata.email.as_deref())
    }

    /// Get the user's display name
    pub fn display_name(&self) -> Option<&str> {
        self.user_metadata.full_name.as_deref()
            .or(self.user_metadata.name.as_deref())
    }

    /// Get avatar URL
    pub fn avatar_url(&self) -> Option<&str> {
        self.user_metadata.avatar_url.as_deref()
            .or(self.user_metadata.picture.as_deref())
    }

    /// Get linked OFFLEASH user ID if provisioned
    pub fn offleash_user_id(&self) -> Option<UserId> {
        self.app_metadata.offleash_user_id
            .as_ref()
            .and_then(|id| id.parse().ok())
    }

    /// Get linked OFFLEASH organization ID
    pub fn offleash_org_id(&self) -> Option<OrganizationId> {
        self.app_metadata.offleash_org_id
            .as_ref()
            .and_then(|id| id.parse().ok())
    }

    /// Check if this is an authenticated user
    pub fn is_authenticated(&self) -> bool {
        self.aud == "authenticated"
    }
}

/// Verify a Supabase JWT token
///
/// # Arguments
/// * `token` - The JWT token to verify
/// * `jwt_secret` - The Supabase JWT secret (from project settings)
///
/// # Returns
/// The decoded claims if valid, or an error
pub fn verify_supabase_token(
    token: &str,
    jwt_secret: &str,
) -> Result<SupabaseClaims, jsonwebtoken::errors::Error> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.set_audience(&["authenticated"]);
    // Supabase tokens don't have a standard issuer claim we need to validate
    validation.validate_exp = true;

    let token_data = decode::<SupabaseClaims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_bytes()),
        &validation,
    )?;

    Ok(token_data.claims)
}

/// Try to verify a token as Supabase, returns None if verification fails
/// This is used to detect whether a token is a Supabase token vs our own JWT
pub fn try_verify_supabase(
    token: &str,
    jwt_secret: &str,
) -> Option<SupabaseClaims> {
    verify_supabase_token(token, jwt_secret).ok()
}
