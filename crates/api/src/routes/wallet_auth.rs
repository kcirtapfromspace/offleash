use axum::{extract::State, Json};
use chrono::{Duration, Utc};
use db::models::{AuthProvider, CreateUser, CreateUserIdentity, CreateWalletChallenge, UserRole};
use db::{OrganizationRepository, UserIdentityRepository, UserRepository, WalletChallengeRepository};
use ethers_core::types::{RecoveryMessage, Signature};
use rand::Rng;
use serde::{Deserialize, Serialize};
use shared::types::UserId;
use siwe::Message;
use std::str::FromStr;

use crate::{
    auth::create_token,
    error::{ApiError, ApiResult},
    routes::auth::{AuthResponse, UserResponse},
    state::AppState,
};

const CHALLENGE_EXPIRY_MINUTES: i64 = 10;

#[derive(Debug, Deserialize)]
pub struct GetChallengeRequest {
    pub org_slug: String,
    pub wallet_address: String,
}

#[derive(Debug, Serialize)]
pub struct ChallengeResponse {
    pub message: String,
    pub nonce: String,
}

#[derive(Debug, Deserialize)]
pub struct VerifySignatureRequest {
    pub org_slug: String,
    pub wallet_address: String,
    pub message: String,
    pub signature: String,
}

/// Validate Ethereum address format
fn validate_wallet_address(address: &str) -> Result<String, ApiError> {
    // Ethereum addresses are 42 characters (0x + 40 hex chars)
    let address = address.to_lowercase();

    if !address.starts_with("0x") {
        return Err(ApiError::from(shared::DomainError::ValidationError(
            "Wallet address must start with 0x".to_string(),
        )));
    }

    if address.len() != 42 {
        return Err(ApiError::from(shared::DomainError::ValidationError(
            "Invalid wallet address length".to_string(),
        )));
    }

    // Check if remaining characters are valid hex
    if !address[2..].chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(ApiError::from(shared::DomainError::ValidationError(
            "Invalid wallet address format".to_string(),
        )));
    }

    Ok(address)
}

/// Generate a random nonce for the challenge
fn generate_nonce() -> String {
    let mut rng = rand::thread_rng();
    let nonce: [u8; 16] = rng.gen();
    hex::encode(nonce)
}

/// Generate SIWE (Sign-In with Ethereum) message
fn generate_siwe_message(
    wallet_address: &str,
    nonce: &str,
    org_slug: &str,
    domain: &str,
    issued_at: chrono::DateTime<Utc>,
    expiration: chrono::DateTime<Utc>,
) -> String {
    // Format as a standard SIWE message
    format!(
        "{domain} wants you to sign in with your Ethereum account:\n\
        {wallet_address}\n\n\
        Sign in to {org_slug} on OFFLEASH\n\n\
        URI: https://{domain}\n\
        Version: 1\n\
        Chain ID: 1\n\
        Nonce: {nonce}\n\
        Issued At: {issued_at}\n\
        Expiration Time: {expiration}",
        domain = domain,
        wallet_address = wallet_address,
        org_slug = org_slug,
        nonce = nonce,
        issued_at = issued_at.format("%Y-%m-%dT%H:%M:%S%.3fZ"),
        expiration = expiration.format("%Y-%m-%dT%H:%M:%S%.3fZ"),
    )
}

/// Get a challenge message for wallet authentication
pub async fn get_challenge(
    State(state): State<AppState>,
    Json(req): Json<GetChallengeRequest>,
) -> ApiResult<Json<ChallengeResponse>> {
    // Validate wallet address
    let wallet_address = validate_wallet_address(&req.wallet_address)?;

    // Verify organization exists
    let _organization = OrganizationRepository::find_by_slug(&state.pool, &req.org_slug)
        .await?
        .ok_or_else(|| ApiError::from(shared::DomainError::OrganizationNotFound(req.org_slug.clone())))?;

    // Generate nonce
    let nonce = generate_nonce();
    let expires_at = Utc::now() + Duration::minutes(CHALLENGE_EXPIRY_MINUTES);

    // Store challenge
    WalletChallengeRepository::create(
        &state.pool,
        CreateWalletChallenge {
            wallet_address: wallet_address.clone(),
            nonce: nonce.clone(),
            expires_at,
        },
    ).await?;

    // Generate SIWE message
    let domain = std::env::var("APP_DOMAIN").unwrap_or_else(|_| "offleash.app".to_string());
    let issued_at = Utc::now();
    let message = generate_siwe_message(&wallet_address, &nonce, &req.org_slug, &domain, issued_at, expires_at);

    Ok(Json(ChallengeResponse { message, nonce }))
}

/// Verify a signed message and authenticate
pub async fn verify_signature(
    State(state): State<AppState>,
    Json(req): Json<VerifySignatureRequest>,
) -> ApiResult<Json<AuthResponse>> {
    // Validate wallet address
    let wallet_address = validate_wallet_address(&req.wallet_address)?;

    // Look up organization
    let organization = OrganizationRepository::find_by_slug(&state.pool, &req.org_slug)
        .await?
        .ok_or_else(|| ApiError::from(shared::DomainError::OrganizationNotFound(req.org_slug.clone())))?;

    // Find active challenge
    let challenge = WalletChallengeRepository::find_active(&state.pool, &wallet_address)
        .await?
        .ok_or_else(|| ApiError::from(shared::DomainError::ValidationError(
            "No active challenge found. Please request a new challenge.".to_string(),
        )))?;

    // Parse the SIWE message to extract nonce
    let siwe_message = Message::from_str(&req.message).map_err(|e| {
        ApiError::from(shared::DomainError::ValidationError(format!(
            "Invalid SIWE message format: {}",
            e
        )))
    })?;

    // Verify nonce matches
    if siwe_message.nonce != challenge.nonce {
        return Err(ApiError::from(shared::DomainError::ValidationError(
            "Nonce mismatch".to_string(),
        )));
    }

    // Verify the signature
    let signature_bytes = hex::decode(req.signature.trim_start_matches("0x")).map_err(|_| {
        ApiError::from(shared::DomainError::ValidationError(
            "Invalid signature format".to_string(),
        ))
    })?;

    let signature = Signature::try_from(signature_bytes.as_slice()).map_err(|_| {
        ApiError::from(shared::DomainError::ValidationError(
            "Invalid signature".to_string(),
        ))
    })?;

    // Recover the signer address from the signature
    let message_hash = RecoveryMessage::Data(req.message.as_bytes().to_vec());
    let recovered_address = signature.recover(message_hash).map_err(|_| {
        ApiError::from(shared::DomainError::InvalidCredentials)
    })?;

    // Format the recovered address for comparison
    let recovered_address_str = format!("{:?}", recovered_address).to_lowercase();

    // Verify the recovered address matches the claimed address
    if recovered_address_str != wallet_address {
        tracing::warn!(
            "Signature verification failed: recovered {} but claimed {}",
            recovered_address_str,
            wallet_address
        );
        return Err(ApiError::from(shared::DomainError::InvalidCredentials));
    }

    // Signature verified! Delete the challenge
    WalletChallengeRepository::delete(&state.pool, challenge.id).await?;

    // Try to find existing identity
    if let Some(identity) = UserIdentityRepository::find_by_provider(
        &state.pool,
        AuthProvider::Wallet,
        &wallet_address,
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
    // Generate a placeholder email from wallet address
    let email = format!("{}@wallet.offleash.app", &wallet_address[2..10]);

    let user = UserRepository::create(
        &state.pool,
        CreateUser {
            organization_id: organization.id,
            email: email.clone(),
            password_hash: "".to_string(), // No password for wallet users
            role: UserRole::Customer,
            first_name: "Wallet".to_string(),
            last_name: "User".to_string(),
            phone: None,
            timezone: None,
        },
    ).await?;

    // Create identity
    UserIdentityRepository::create(&state.pool, CreateUserIdentity {
        user_id: user.id.into_uuid(),
        provider: AuthProvider::Wallet,
        provider_user_id: wallet_address,
        provider_email: None,
        provider_data: Some(serde_json::json!({
            "chain_id": 1,
            "verified_at": Utc::now().to_rfc3339(),
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
        membership: None,
        memberships: None,
    }))
}
