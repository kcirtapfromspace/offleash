use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Auth provider types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "auth_provider", rename_all = "lowercase")]
pub enum AuthProvider {
    Email,
    Phone,
    Google,
    Apple,
    Wallet,
}

impl std::fmt::Display for AuthProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthProvider::Email => write!(f, "email"),
            AuthProvider::Phone => write!(f, "phone"),
            AuthProvider::Google => write!(f, "google"),
            AuthProvider::Apple => write!(f, "apple"),
            AuthProvider::Wallet => write!(f, "wallet"),
        }
    }
}

/// User identity - links an auth provider to a user
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserIdentity {
    pub id: Uuid,
    pub user_id: Uuid,
    pub provider: AuthProvider,
    pub provider_user_id: String,
    pub provider_email: Option<String>,
    pub provider_data: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

/// Create a new user identity
#[derive(Debug)]
pub struct CreateUserIdentity {
    pub user_id: Uuid,
    pub provider: AuthProvider,
    pub provider_user_id: String,
    pub provider_email: Option<String>,
    pub provider_data: Option<serde_json::Value>,
}

/// Phone verification record
#[derive(Debug, Clone, FromRow)]
pub struct PhoneVerification {
    pub id: Uuid,
    pub phone_number: String,
    pub code_hash: String,
    pub expires_at: DateTime<Utc>,
    pub attempts: i32,
    pub created_at: DateTime<Utc>,
}

/// Create phone verification
#[derive(Debug)]
pub struct CreatePhoneVerification {
    pub phone_number: String,
    pub code_hash: String,
    pub expires_at: DateTime<Utc>,
}

/// Wallet authentication challenge
#[derive(Debug, Clone, FromRow)]
pub struct WalletChallenge {
    pub id: Uuid,
    pub wallet_address: String,
    pub nonce: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

/// Create wallet challenge
#[derive(Debug)]
pub struct CreateWalletChallenge {
    pub wallet_address: String,
    pub nonce: String,
    pub expires_at: DateTime<Utc>,
}
