use shared::types::OrganizationId;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{
    CreatePaymentProvider, PaymentProvider, PaymentProviderType, UpdatePaymentProvider,
};

pub struct PaymentProviderRepository;

impl PaymentProviderRepository {
    /// Create a new payment provider for an organization
    pub async fn create(
        pool: &PgPool,
        org_id: OrganizationId,
        input: CreatePaymentProvider,
    ) -> Result<PaymentProvider, sqlx::Error> {
        let id = Uuid::new_v4();

        sqlx::query_as::<_, PaymentProvider>(
            r#"
            INSERT INTO payment_providers (
                id, organization_id, provider_type,
                stripe_account_id, stripe_account_type,
                square_merchant_id,
                access_token_encrypted, refresh_token_encrypted, token_expires_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(org_id.as_uuid())
        .bind(input.provider_type)
        .bind(input.stripe_account_id)
        .bind(input.stripe_account_type)
        .bind(input.square_merchant_id)
        .bind(input.access_token_encrypted)
        .bind(input.refresh_token_encrypted)
        .bind(input.token_expires_at)
        .fetch_one(pool)
        .await
    }

    /// Get payment provider by ID
    pub async fn get_by_id(
        pool: &PgPool,
        org_id: OrganizationId,
        id: Uuid,
    ) -> Result<Option<PaymentProvider>, sqlx::Error> {
        sqlx::query_as::<_, PaymentProvider>(
            r#"
            SELECT * FROM payment_providers
            WHERE id = $1 AND organization_id = $2
            "#,
        )
        .bind(id)
        .bind(org_id.as_uuid())
        .fetch_optional(pool)
        .await
    }

    /// Get active payment provider for an organization by type
    pub async fn get_active_by_type(
        pool: &PgPool,
        org_id: OrganizationId,
        provider_type: PaymentProviderType,
    ) -> Result<Option<PaymentProvider>, sqlx::Error> {
        sqlx::query_as::<_, PaymentProvider>(
            r#"
            SELECT * FROM payment_providers
            WHERE organization_id = $1 AND provider_type = $2 AND is_active = true
            "#,
        )
        .bind(org_id.as_uuid())
        .bind(provider_type)
        .fetch_optional(pool)
        .await
    }

    /// Get payment provider by type (for webhooks)
    pub async fn get_by_type(
        pool: &PgPool,
        org_id: OrganizationId,
        provider_type: PaymentProviderType,
    ) -> Result<Option<PaymentProvider>, sqlx::Error> {
        sqlx::query_as::<_, PaymentProvider>(
            r#"
            SELECT * FROM payment_providers
            WHERE organization_id = $1 AND provider_type = $2
            LIMIT 1
            "#,
        )
        .bind(org_id.as_uuid())
        .bind(provider_type)
        .fetch_optional(pool)
        .await
    }

    /// Get the primary active payment provider for an organization
    /// Returns the first active provider (prefers Stripe/Square over Platform)
    pub async fn get_primary(
        pool: &PgPool,
        org_id: OrganizationId,
    ) -> Result<Option<PaymentProvider>, sqlx::Error> {
        sqlx::query_as::<_, PaymentProvider>(
            r#"
            SELECT * FROM payment_providers
            WHERE organization_id = $1 AND is_active = true
            ORDER BY
                CASE provider_type
                    WHEN 'stripe' THEN 1
                    WHEN 'square' THEN 2
                    WHEN 'platform' THEN 3
                END
            LIMIT 1
            "#,
        )
        .bind(org_id.as_uuid())
        .fetch_optional(pool)
        .await
    }

    /// List all payment providers for an organization
    pub async fn list_for_org(
        pool: &PgPool,
        org_id: OrganizationId,
    ) -> Result<Vec<PaymentProvider>, sqlx::Error> {
        sqlx::query_as::<_, PaymentProvider>(
            r#"
            SELECT * FROM payment_providers
            WHERE organization_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(org_id.as_uuid())
        .fetch_all(pool)
        .await
    }

    /// Update payment provider
    pub async fn update(
        pool: &PgPool,
        org_id: OrganizationId,
        id: Uuid,
        input: UpdatePaymentProvider,
    ) -> Result<Option<PaymentProvider>, sqlx::Error> {
        sqlx::query_as::<_, PaymentProvider>(
            r#"
            UPDATE payment_providers
            SET
                access_token_encrypted = COALESCE($3, access_token_encrypted),
                refresh_token_encrypted = COALESCE($4, refresh_token_encrypted),
                token_expires_at = COALESCE($5, token_expires_at),
                is_active = COALESCE($6, is_active),
                is_verified = COALESCE($7, is_verified),
                verification_status = COALESCE($8, verification_status),
                charges_enabled = COALESCE($9, charges_enabled),
                payouts_enabled = COALESCE($10, payouts_enabled),
                metadata = COALESCE($11, metadata),
                updated_at = NOW()
            WHERE id = $1 AND organization_id = $2
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(org_id.as_uuid())
        .bind(input.access_token_encrypted)
        .bind(input.refresh_token_encrypted)
        .bind(input.token_expires_at)
        .bind(input.is_active)
        .bind(input.is_verified)
        .bind(input.verification_status)
        .bind(input.charges_enabled)
        .bind(input.payouts_enabled)
        .bind(input.metadata)
        .fetch_optional(pool)
        .await
    }

    /// Deactivate a payment provider
    pub async fn deactivate(
        pool: &PgPool,
        org_id: OrganizationId,
        id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"
            UPDATE payment_providers
            SET is_active = false, updated_at = NOW()
            WHERE id = $1 AND organization_id = $2
            "#,
        )
        .bind(id)
        .bind(org_id.as_uuid())
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Delete a payment provider (hard delete - use with caution)
    pub async fn delete(
        pool: &PgPool,
        org_id: OrganizationId,
        id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"
            DELETE FROM payment_providers
            WHERE id = $1 AND organization_id = $2
            "#,
        )
        .bind(id)
        .bind(org_id.as_uuid())
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Create or update platform default provider for an organization
    pub async fn upsert_platform_default(
        pool: &PgPool,
        org_id: OrganizationId,
    ) -> Result<PaymentProvider, sqlx::Error> {
        sqlx::query_as::<_, PaymentProvider>(
            r#"
            INSERT INTO payment_providers (
                id, organization_id, provider_type, is_active, is_verified, charges_enabled
            )
            VALUES ($1, $2, 'platform', true, true, true)
            ON CONFLICT (organization_id, provider_type)
            DO UPDATE SET is_active = true, updated_at = NOW()
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(org_id.as_uuid())
        .fetch_one(pool)
        .await
    }

    /// Update Stripe Connect account status after webhook
    pub async fn update_stripe_status(
        pool: &PgPool,
        stripe_account_id: &str,
        charges_enabled: bool,
        payouts_enabled: bool,
        verification_status: Option<&str>,
    ) -> Result<Option<PaymentProvider>, sqlx::Error> {
        sqlx::query_as::<_, PaymentProvider>(
            r#"
            UPDATE payment_providers
            SET
                charges_enabled = $2,
                payouts_enabled = $3,
                is_verified = $2,
                verification_status = $4,
                updated_at = NOW()
            WHERE stripe_account_id = $1
            RETURNING *
            "#,
        )
        .bind(stripe_account_id)
        .bind(charges_enabled)
        .bind(payouts_enabled)
        .bind(verification_status)
        .fetch_optional(pool)
        .await
    }
}
