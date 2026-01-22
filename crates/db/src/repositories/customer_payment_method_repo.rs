use shared::types::{OrganizationId, UserId};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{CreateCustomerPaymentMethod, CustomerPaymentMethod};

pub struct CustomerPaymentMethodRepository;

impl CustomerPaymentMethodRepository {
    /// Create a new payment method for a customer
    pub async fn create(
        pool: &PgPool,
        org_id: OrganizationId,
        customer_id: UserId,
        input: CreateCustomerPaymentMethod,
    ) -> Result<CustomerPaymentMethod, sqlx::Error> {
        let id = Uuid::new_v4();

        // If this is marked as default, unset other defaults first
        if input.is_default {
            Self::unset_defaults(pool, org_id, customer_id).await?;
        }

        sqlx::query_as::<_, CustomerPaymentMethod>(
            r#"
            INSERT INTO customer_payment_methods (
                id, organization_id, customer_id, method_type,
                card_last_four, card_brand, card_exp_month, card_exp_year,
                square_card_id, nickname, is_default
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(org_id.as_uuid())
        .bind(customer_id.as_uuid())
        .bind(input.method_type)
        .bind(input.card_last_four)
        .bind(input.card_brand)
        .bind(input.card_exp_month)
        .bind(input.card_exp_year)
        .bind(input.square_card_id)
        .bind(input.nickname)
        .bind(input.is_default)
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
    pub async fn list_for_customer(
        pool: &PgPool,
        org_id: OrganizationId,
        customer_id: UserId,
    ) -> Result<Vec<CustomerPaymentMethod>, sqlx::Error> {
        sqlx::query_as::<_, CustomerPaymentMethod>(
            r#"
            SELECT * FROM customer_payment_methods
            WHERE customer_id = $1 AND organization_id = $2 AND is_active = true
            ORDER BY is_default DESC, created_at DESC
            "#,
        )
        .bind(customer_id.as_uuid())
        .bind(org_id.as_uuid())
        .fetch_all(pool)
        .await
    }

    /// Get the default payment method for a customer
    pub async fn get_default(
        pool: &PgPool,
        org_id: OrganizationId,
        customer_id: UserId,
    ) -> Result<Option<CustomerPaymentMethod>, sqlx::Error> {
        sqlx::query_as::<_, CustomerPaymentMethod>(
            r#"
            SELECT * FROM customer_payment_methods
            WHERE customer_id = $1 AND organization_id = $2 AND is_default = true AND is_active = true
            "#,
        )
        .bind(customer_id.as_uuid())
        .bind(org_id.as_uuid())
        .fetch_optional(pool)
        .await
    }

    /// Set a payment method as the default
    pub async fn set_default(
        pool: &PgPool,
        org_id: OrganizationId,
        customer_id: UserId,
        id: Uuid,
    ) -> Result<Option<CustomerPaymentMethod>, sqlx::Error> {
        // Unset existing defaults
        Self::unset_defaults(pool, org_id, customer_id).await?;

        // Set the new default
        sqlx::query_as::<_, CustomerPaymentMethod>(
            r#"
            UPDATE customer_payment_methods
            SET is_default = true, updated_at = NOW()
            WHERE id = $1 AND organization_id = $2 AND customer_id = $3 AND is_active = true
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(org_id.as_uuid())
        .bind(customer_id.as_uuid())
        .fetch_optional(pool)
        .await
    }

    /// Unset all defaults for a customer
    async fn unset_defaults(
        pool: &PgPool,
        org_id: OrganizationId,
        customer_id: UserId,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE customer_payment_methods
            SET is_default = false, updated_at = NOW()
            WHERE customer_id = $1 AND organization_id = $2 AND is_default = true
            "#,
        )
        .bind(customer_id.as_uuid())
        .bind(org_id.as_uuid())
        .execute(pool)
        .await?;
        Ok(())
    }

    /// Delete (soft delete) a payment method
    pub async fn delete(
        pool: &PgPool,
        org_id: OrganizationId,
        customer_id: UserId,
        id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"
            UPDATE customer_payment_methods
            SET is_active = false, is_default = false, updated_at = NOW()
            WHERE id = $1 AND organization_id = $2 AND customer_id = $3
            "#,
        )
        .bind(id)
        .bind(org_id.as_uuid())
        .bind(customer_id.as_uuid())
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Update nickname for a payment method
    pub async fn update_nickname(
        pool: &PgPool,
        org_id: OrganizationId,
        customer_id: UserId,
        id: Uuid,
        nickname: Option<&str>,
    ) -> Result<Option<CustomerPaymentMethod>, sqlx::Error> {
        sqlx::query_as::<_, CustomerPaymentMethod>(
            r#"
            UPDATE customer_payment_methods
            SET nickname = $4, updated_at = NOW()
            WHERE id = $1 AND organization_id = $2 AND customer_id = $3 AND is_active = true
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(org_id.as_uuid())
        .bind(customer_id.as_uuid())
        .bind(nickname)
        .fetch_optional(pool)
        .await
    }
}
