use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{
    AuthProvider, CreatePhoneVerification, CreateUserIdentity, CreateWalletChallenge,
    PhoneVerification, UserIdentity, WalletChallenge,
};

pub struct UserIdentityRepository;

impl UserIdentityRepository {
    /// Find identity by provider and provider user ID
    pub async fn find_by_provider(
        pool: &PgPool,
        provider: AuthProvider,
        provider_user_id: &str,
    ) -> Result<Option<UserIdentity>, sqlx::Error> {
        sqlx::query_as::<_, UserIdentity>(
            r#"
            SELECT id, user_id, provider, provider_user_id, provider_email, provider_data, created_at
            FROM user_identities
            WHERE provider = $1 AND provider_user_id = $2
            "#,
        )
        .bind(provider)
        .bind(provider_user_id)
        .fetch_optional(pool)
        .await
    }

    /// Find all identities for a user
    pub async fn find_by_user(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Vec<UserIdentity>, sqlx::Error> {
        sqlx::query_as::<_, UserIdentity>(
            r#"
            SELECT id, user_id, provider, provider_user_id, provider_email, provider_data, created_at
            FROM user_identities
            WHERE user_id = $1
            ORDER BY created_at ASC
            "#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
    }

    /// Find identity by provider email (for account linking)
    pub async fn find_by_provider_email(
        pool: &PgPool,
        provider: AuthProvider,
        email: &str,
    ) -> Result<Option<UserIdentity>, sqlx::Error> {
        sqlx::query_as::<_, UserIdentity>(
            r#"
            SELECT id, user_id, provider, provider_user_id, provider_email, provider_data, created_at
            FROM user_identities
            WHERE provider = $1 AND provider_email = $2
            "#,
        )
        .bind(provider)
        .bind(email)
        .fetch_optional(pool)
        .await
    }

    /// Create a new user identity
    pub async fn create(
        pool: &PgPool,
        identity: CreateUserIdentity,
    ) -> Result<UserIdentity, sqlx::Error> {
        sqlx::query_as::<_, UserIdentity>(
            r#"
            INSERT INTO user_identities (user_id, provider, provider_user_id, provider_email, provider_data)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, user_id, provider, provider_user_id, provider_email, provider_data, created_at
            "#,
        )
        .bind(identity.user_id)
        .bind(identity.provider)
        .bind(identity.provider_user_id)
        .bind(identity.provider_email)
        .bind(identity.provider_data.unwrap_or(serde_json::json!({})))
        .fetch_one(pool)
        .await
    }

    /// Delete an identity
    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM user_identities WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    /// Count identities for a user (to prevent deleting the last one)
    pub async fn count_for_user(pool: &PgPool, user_id: Uuid) -> Result<i64, sqlx::Error> {
        let result: (i64,) =
            sqlx::query_as("SELECT COUNT(*)::bigint FROM user_identities WHERE user_id = $1")
                .bind(user_id)
                .fetch_one(pool)
                .await?;
        Ok(result.0)
    }
}

pub struct PhoneVerificationRepository;

impl PhoneVerificationRepository {
    /// Find active verification for phone number
    pub async fn find_active(
        pool: &PgPool,
        phone_number: &str,
    ) -> Result<Option<PhoneVerification>, sqlx::Error> {
        sqlx::query_as::<_, PhoneVerification>(
            r#"
            SELECT id, phone_number, code_hash, expires_at, attempts, created_at
            FROM phone_verifications
            WHERE phone_number = $1 AND expires_at > NOW()
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .bind(phone_number)
        .fetch_optional(pool)
        .await
    }

    /// Create a new verification
    pub async fn create(
        pool: &PgPool,
        verification: CreatePhoneVerification,
    ) -> Result<PhoneVerification, sqlx::Error> {
        // Delete any existing verifications for this phone first
        sqlx::query("DELETE FROM phone_verifications WHERE phone_number = $1")
            .bind(&verification.phone_number)
            .execute(pool)
            .await?;

        sqlx::query_as::<_, PhoneVerification>(
            r#"
            INSERT INTO phone_verifications (phone_number, code_hash, expires_at)
            VALUES ($1, $2, $3)
            RETURNING id, phone_number, code_hash, expires_at, attempts, created_at
            "#,
        )
        .bind(verification.phone_number)
        .bind(verification.code_hash)
        .bind(verification.expires_at)
        .fetch_one(pool)
        .await
    }

    /// Increment attempts counter
    pub async fn increment_attempts(pool: &PgPool, id: Uuid) -> Result<i32, sqlx::Error> {
        let result: (i32,) = sqlx::query_as(
            r#"
            UPDATE phone_verifications
            SET attempts = attempts + 1
            WHERE id = $1
            RETURNING attempts
            "#,
        )
        .bind(id)
        .fetch_one(pool)
        .await?;
        Ok(result.0)
    }

    /// Delete verification
    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM phone_verifications WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    /// Count verifications in last hour for rate limiting
    pub async fn count_recent(pool: &PgPool, phone_number: &str) -> Result<i64, sqlx::Error> {
        let result: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*)::bigint FROM phone_verifications
            WHERE phone_number = $1 AND created_at > NOW() - INTERVAL '1 hour'
            "#,
        )
        .bind(phone_number)
        .fetch_one(pool)
        .await?;
        Ok(result.0)
    }

    /// Cleanup expired verifications
    pub async fn cleanup_expired(pool: &PgPool) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM phone_verifications WHERE expires_at < NOW()")
            .execute(pool)
            .await?;
        Ok(result.rows_affected())
    }
}

pub struct WalletChallengeRepository;

impl WalletChallengeRepository {
    /// Find active challenge for wallet address
    pub async fn find_active(
        pool: &PgPool,
        wallet_address: &str,
    ) -> Result<Option<WalletChallenge>, sqlx::Error> {
        sqlx::query_as::<_, WalletChallenge>(
            r#"
            SELECT id, wallet_address, nonce, expires_at, created_at
            FROM wallet_challenges
            WHERE wallet_address = $1 AND expires_at > NOW()
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .bind(wallet_address)
        .fetch_optional(pool)
        .await
    }

    /// Create a new challenge
    pub async fn create(
        pool: &PgPool,
        challenge: CreateWalletChallenge,
    ) -> Result<WalletChallenge, sqlx::Error> {
        // Delete any existing challenges for this wallet first
        sqlx::query("DELETE FROM wallet_challenges WHERE wallet_address = $1")
            .bind(&challenge.wallet_address)
            .execute(pool)
            .await?;

        sqlx::query_as::<_, WalletChallenge>(
            r#"
            INSERT INTO wallet_challenges (wallet_address, nonce, expires_at)
            VALUES ($1, $2, $3)
            RETURNING id, wallet_address, nonce, expires_at, created_at
            "#,
        )
        .bind(challenge.wallet_address)
        .bind(challenge.nonce)
        .bind(challenge.expires_at)
        .fetch_one(pool)
        .await
    }

    /// Delete challenge
    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM wallet_challenges WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    /// Cleanup expired challenges
    pub async fn cleanup_expired(pool: &PgPool) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM wallet_challenges WHERE expires_at < NOW()")
            .execute(pool)
            .await?;
        Ok(result.rows_affected())
    }
}
