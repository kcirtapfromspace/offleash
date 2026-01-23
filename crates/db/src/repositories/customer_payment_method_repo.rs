use shared::types::{OrganizationId, UserId};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{CreateCustomerPaymentMethod, CustomerPaymentMethod, UpdateCustomerPaymentMethod};

pub struct CustomerPaymentMethodRepository;

impl CustomerPaymentMethodRepository {
    /// Create a new payment method for a customer
    pub async fn create(
        pool: &PgPool,
        org_id: OrganizationId,
        user_id: UserId,
        input: CreateCustomerPaymentMethod,
    ) -> Result<CustomerPaymentMethod, sqlx::Error> {
        let id = Uuid::new_v4();

        // If this is marked as default, unset other defaults first
        if input.is_default {
            Self::unset_defaults(pool, org_id, user_id).await?;
        }

        sqlx::query_as::<_, CustomerPaymentMethod>(
            r#"
            INSERT INTO customer_payment_methods (
                id, organization_id, user_id, provider_type, method_type,
                stripe_payment_method_id, stripe_customer_id,
                square_card_id, square_customer_id,
                last_four, brand, exp_month, exp_year, cardholder_name,
                bank_name, account_last_four, wallet_type,
                is_default, billing_address
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19)
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(org_id.as_uuid())
        .bind(user_id.as_uuid())
        .bind(input.provider_type)
        .bind(input.method_type)
        .bind(input.stripe_payment_method_id)
        .bind(input.stripe_customer_id)
        .bind(input.square_card_id)
        .bind(input.square_customer_id)
        .bind(input.last_four)
        .bind(input.brand)
        .bind(input.exp_month)
        .bind(input.exp_year)
        .bind(input.cardholder_name)
        .bind(input.bank_name)
        .bind(input.account_last_four)
        .bind(input.wallet_type)
        .bind(input.is_default)
        .bind(input.billing_address)
        .fetch_one(pool)
        .await
    }

    /// Find a payment method by ID
    pub async fn find_by_id(
        pool: &PgPool,
        org_id: OrganizationId,
        id: Uuid,
    ) -> Result<Option<CustomerPaymentMethod>, sqlx::Error> {
        sqlx::query_as::<_, CustomerPaymentMethod>(
            r#"
            SELECT * FROM customer_payment_methods
            WHERE id = $1 AND organization_id = $2 AND is_active = true
            "#,
        )
        .bind(id)
        .bind(org_id.as_uuid())
        .fetch_optional(pool)
        .await
    }

    /// List all active payment methods for a customer
    pub async fn list_for_user(
        pool: &PgPool,
        org_id: OrganizationId,
        user_id: UserId,
    ) -> Result<Vec<CustomerPaymentMethod>, sqlx::Error> {
        sqlx::query_as::<_, CustomerPaymentMethod>(
            r#"
            SELECT * FROM customer_payment_methods
            WHERE user_id = $1 AND organization_id = $2 AND is_active = true
            ORDER BY is_default DESC, created_at DESC
            "#,
        )
        .bind(user_id.as_uuid())
        .bind(org_id.as_uuid())
        .fetch_all(pool)
        .await
    }

    /// Get the default payment method for a customer
    pub async fn get_default(
        pool: &PgPool,
        org_id: OrganizationId,
        user_id: UserId,
    ) -> Result<Option<CustomerPaymentMethod>, sqlx::Error> {
        sqlx::query_as::<_, CustomerPaymentMethod>(
            r#"
            SELECT * FROM customer_payment_methods
            WHERE user_id = $1 AND organization_id = $2 AND is_default = true AND is_active = true
            "#,
        )
        .bind(user_id.as_uuid())
        .bind(org_id.as_uuid())
        .fetch_optional(pool)
        .await
    }

    /// Set a payment method as the default
    pub async fn set_default(
        pool: &PgPool,
        org_id: OrganizationId,
        user_id: UserId,
        id: Uuid,
    ) -> Result<Option<CustomerPaymentMethod>, sqlx::Error> {
        // Unset existing defaults
        Self::unset_defaults(pool, org_id, user_id).await?;

        // Set the new default
        sqlx::query_as::<_, CustomerPaymentMethod>(
            r#"
            UPDATE customer_payment_methods
            SET is_default = true, updated_at = NOW()
            WHERE id = $1 AND organization_id = $2 AND user_id = $3 AND is_active = true
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(org_id.as_uuid())
        .bind(user_id.as_uuid())
        .fetch_optional(pool)
        .await
    }

    /// Unset all defaults for a customer
    async fn unset_defaults(
        pool: &PgPool,
        org_id: OrganizationId,
        user_id: UserId,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE customer_payment_methods
            SET is_default = false, updated_at = NOW()
            WHERE user_id = $1 AND organization_id = $2 AND is_default = true
            "#,
        )
        .bind(user_id.as_uuid())
        .bind(org_id.as_uuid())
        .execute(pool)
        .await?;
        Ok(())
    }

    /// Update a payment method
    pub async fn update(
        pool: &PgPool,
        org_id: OrganizationId,
        user_id: UserId,
        id: Uuid,
        input: UpdateCustomerPaymentMethod,
    ) -> Result<Option<CustomerPaymentMethod>, sqlx::Error> {
        // If setting as default, unset others first
        if input.is_default == Some(true) {
            Self::unset_defaults(pool, org_id, user_id).await?;
        }

        sqlx::query_as::<_, CustomerPaymentMethod>(
            r#"
            UPDATE customer_payment_methods
            SET
                is_default = COALESCE($4, is_default),
                is_active = COALESCE($5, is_active),
                billing_address = COALESCE($6, billing_address),
                metadata = COALESCE($7, metadata),
                updated_at = NOW()
            WHERE id = $1 AND organization_id = $2 AND user_id = $3
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(org_id.as_uuid())
        .bind(user_id.as_uuid())
        .bind(input.is_default)
        .bind(input.is_active)
        .bind(input.billing_address)
        .bind(input.metadata)
        .fetch_optional(pool)
        .await
    }

    /// Delete (soft delete) a payment method
    pub async fn delete(
        pool: &PgPool,
        org_id: OrganizationId,
        user_id: UserId,
        id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"
            UPDATE customer_payment_methods
            SET is_active = false, is_default = false, updated_at = NOW()
            WHERE id = $1 AND organization_id = $2 AND user_id = $3
            "#,
        )
        .bind(id)
        .bind(org_id.as_uuid())
        .bind(user_id.as_uuid())
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Get payment method by Stripe payment method ID
    pub async fn get_by_stripe_pm(
        pool: &PgPool,
        stripe_payment_method_id: &str,
    ) -> Result<Option<CustomerPaymentMethod>, sqlx::Error> {
        sqlx::query_as::<_, CustomerPaymentMethod>(
            r#"
            SELECT * FROM customer_payment_methods
            WHERE stripe_payment_method_id = $1 AND is_active = true
            "#,
        )
        .bind(stripe_payment_method_id)
        .fetch_optional(pool)
        .await
    }

    /// Get payment method by Square card ID
    pub async fn get_by_square_card(
        pool: &PgPool,
        square_card_id: &str,
    ) -> Result<Option<CustomerPaymentMethod>, sqlx::Error> {
        sqlx::query_as::<_, CustomerPaymentMethod>(
            r#"
            SELECT * FROM customer_payment_methods
            WHERE square_card_id = $1 AND is_active = true
            "#,
        )
        .bind(square_card_id)
        .fetch_optional(pool)
        .await
    }
}
