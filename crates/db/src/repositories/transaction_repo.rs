use chrono::{DateTime, Utc};
use shared::types::{BookingId, OrganizationId, UserId};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{CreateTransaction, Transaction, TransactionStatus, UpdateTransaction};

pub struct TransactionRepository;

impl TransactionRepository {
    /// Create a new transaction
    pub async fn create(
        pool: &PgPool,
        org_id: OrganizationId,
        input: CreateTransaction,
    ) -> Result<Transaction, sqlx::Error> {
        let id = Uuid::new_v4();

        sqlx::query_as::<_, Transaction>(
            r#"
            INSERT INTO transactions (
                id, organization_id, booking_id, customer_user_id, provider_user_id,
                payment_method_id, provider_id,
                subtotal_cents, tip_cents, customer_fee_cents, provider_fee_cents,
                platform_fee_cents, tax_cents, processing_fee_cents, total_cents,
                provider_payout_cents, currency, tax_rate_percent, tax_jurisdiction,
                description, metadata
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21)
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(org_id.as_uuid())
        .bind(input.booking_id.map(|b| b.into_uuid()))
        .bind(input.customer_user_id.as_uuid())
        .bind(input.provider_user_id.as_uuid())
        .bind(input.payment_method_id)
        .bind(input.provider_id)
        .bind(input.subtotal_cents)
        .bind(input.tip_cents)
        .bind(input.customer_fee_cents)
        .bind(input.provider_fee_cents)
        .bind(input.platform_fee_cents)
        .bind(input.tax_cents)
        .bind(input.processing_fee_cents)
        .bind(input.total_cents)
        .bind(input.provider_payout_cents)
        .bind(&input.currency)
        .bind(input.tax_rate_percent)
        .bind(input.tax_jurisdiction)
        .bind(input.description)
        .bind(input.metadata)
        .fetch_one(pool)
        .await
    }

    /// Get transaction by ID
    pub async fn get_by_id(
        pool: &PgPool,
        org_id: OrganizationId,
        id: Uuid,
    ) -> Result<Option<Transaction>, sqlx::Error> {
        sqlx::query_as::<_, Transaction>(
            r#"
            SELECT * FROM transactions
            WHERE id = $1 AND organization_id = $2
            "#,
        )
        .bind(id)
        .bind(org_id.as_uuid())
        .fetch_optional(pool)
        .await
    }

    /// Get transaction by booking ID
    pub async fn get_by_booking(
        pool: &PgPool,
        org_id: OrganizationId,
        booking_id: BookingId,
    ) -> Result<Option<Transaction>, sqlx::Error> {
        sqlx::query_as::<_, Transaction>(
            r#"
            SELECT * FROM transactions
            WHERE booking_id = $1 AND organization_id = $2
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .bind(booking_id.as_uuid())
        .bind(org_id.as_uuid())
        .fetch_optional(pool)
        .await
    }

    /// Get transaction by Stripe payment intent ID
    pub async fn get_by_stripe_payment_intent(
        pool: &PgPool,
        payment_intent_id: &str,
    ) -> Result<Option<Transaction>, sqlx::Error> {
        sqlx::query_as::<_, Transaction>(
            r#"
            SELECT * FROM transactions
            WHERE stripe_payment_intent_id = $1
            "#,
        )
        .bind(payment_intent_id)
        .fetch_optional(pool)
        .await
    }

    /// Get transaction by Square payment ID
    pub async fn get_by_square_payment(
        pool: &PgPool,
        square_payment_id: &str,
    ) -> Result<Option<Transaction>, sqlx::Error> {
        sqlx::query_as::<_, Transaction>(
            r#"
            SELECT * FROM transactions
            WHERE square_payment_id = $1
            "#,
        )
        .bind(square_payment_id)
        .fetch_optional(pool)
        .await
    }

    /// List transactions for a user (either as customer or provider)
    pub async fn list_for_user(
        pool: &PgPool,
        org_id: OrganizationId,
        user_id: UserId,
    ) -> Result<Vec<Transaction>, sqlx::Error> {
        sqlx::query_as::<_, Transaction>(
            r#"
            SELECT * FROM transactions
            WHERE (customer_user_id = $1 OR provider_user_id = $1) AND organization_id = $2
            ORDER BY created_at DESC
            LIMIT 100
            "#,
        )
        .bind(user_id.as_uuid())
        .bind(org_id.as_uuid())
        .fetch_all(pool)
        .await
    }

    /// Get transaction by external payment ID
    pub async fn get_by_external_id(
        pool: &PgPool,
        org_id: OrganizationId,
        external_id: &str,
    ) -> Result<Option<Transaction>, sqlx::Error> {
        sqlx::query_as::<_, Transaction>(
            r#"
            SELECT * FROM transactions
            WHERE organization_id = $1 AND (
                external_payment_id = $2 OR
                stripe_payment_intent_id = $2 OR
                square_payment_id = $2
            )
            "#,
        )
        .bind(org_id.as_uuid())
        .bind(external_id)
        .fetch_optional(pool)
        .await
    }

    /// Update external payment ID
    pub async fn update_external_id(
        pool: &PgPool,
        id: Uuid,
        external_id: &str,
    ) -> Result<Option<Transaction>, sqlx::Error> {
        sqlx::query_as::<_, Transaction>(
            r#"
            UPDATE transactions
            SET external_payment_id = $2, updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(external_id)
        .fetch_optional(pool)
        .await
    }

    /// List transactions for an organization (tenant dashboard)
    pub async fn list_for_org(
        pool: &PgPool,
        org_id: OrganizationId,
        status: Option<TransactionStatus>,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Transaction>, sqlx::Error> {
        sqlx::query_as::<_, Transaction>(
            r#"
            SELECT * FROM transactions
            WHERE organization_id = $1
                AND ($2::transaction_status IS NULL OR status = $2)
                AND ($3::timestamptz IS NULL OR created_at >= $3)
                AND ($4::timestamptz IS NULL OR created_at <= $4)
            ORDER BY created_at DESC
            LIMIT $5 OFFSET $6
            "#,
        )
        .bind(org_id.as_uuid())
        .bind(status)
        .bind(start_date)
        .bind(end_date)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
    }

    /// Update transaction
    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        input: UpdateTransaction,
    ) -> Result<Option<Transaction>, sqlx::Error> {
        sqlx::query_as::<_, Transaction>(
            r#"
            UPDATE transactions
            SET
                status = COALESCE($2, status),
                stripe_payment_intent_id = COALESCE($3, stripe_payment_intent_id),
                stripe_charge_id = COALESCE($4, stripe_charge_id),
                stripe_transfer_id = COALESCE($5, stripe_transfer_id),
                square_payment_id = COALESCE($6, square_payment_id),
                square_order_id = COALESCE($7, square_order_id),
                tax_calculation_id = COALESCE($8, tax_calculation_id),
                refunded_amount_cents = COALESCE($9, refunded_amount_cents),
                failure_code = COALESCE($10, failure_code),
                failure_message = COALESCE($11, failure_message),
                metadata = COALESCE($12, metadata),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(input.status)
        .bind(input.stripe_payment_intent_id)
        .bind(input.stripe_charge_id)
        .bind(input.stripe_transfer_id)
        .bind(input.square_payment_id)
        .bind(input.square_order_id)
        .bind(input.tax_calculation_id)
        .bind(input.refunded_amount_cents)
        .bind(input.failure_code)
        .bind(input.failure_message)
        .bind(input.metadata)
        .fetch_optional(pool)
        .await
    }

    /// Update transaction status
    pub async fn update_status(
        pool: &PgPool,
        id: Uuid,
        status: TransactionStatus,
    ) -> Result<Option<Transaction>, sqlx::Error> {
        sqlx::query_as::<_, Transaction>(
            r#"
            UPDATE transactions
            SET status = $2, updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(status)
        .fetch_optional(pool)
        .await
    }

    /// Record refund on transaction
    pub async fn record_refund(
        pool: &PgPool,
        id: Uuid,
        refund_amount_cents: i32,
    ) -> Result<Option<Transaction>, sqlx::Error> {
        sqlx::query_as::<_, Transaction>(
            r#"
            UPDATE transactions
            SET
                refunded_amount_cents = refunded_amount_cents + $2,
                status = CASE
                    WHEN refunded_amount_cents + $2 >= total_cents THEN 'refunded'::transaction_status
                    ELSE 'partially_refunded'::transaction_status
                END,
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(refund_amount_cents)
        .fetch_optional(pool)
        .await
    }

    /// Get transaction summary for an organization
    pub async fn get_summary(
        pool: &PgPool,
        org_id: OrganizationId,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<TransactionSummary, sqlx::Error> {
        sqlx::query_as::<_, TransactionSummary>(
            r#"
            SELECT
                COUNT(*) as transaction_count,
                COALESCE(SUM(total_cents), 0) as total_volume_cents,
                COALESCE(SUM(provider_fee_cents), 0) as total_fees_cents,
                COALESCE(SUM(net_amount_cents), 0) as net_earnings_cents,
                COALESCE(SUM(CASE WHEN status = 'succeeded' THEN 1 ELSE 0 END), 0) as successful_count,
                COALESCE(SUM(CASE WHEN status = 'failed' THEN 1 ELSE 0 END), 0) as failed_count,
                COALESCE(SUM(refunded_amount_cents), 0) as refunded_cents
            FROM transactions
            WHERE organization_id = $1
                AND created_at >= $2
                AND created_at <= $3
            "#,
        )
        .bind(org_id.as_uuid())
        .bind(start_date)
        .bind(end_date)
        .fetch_one(pool)
        .await
    }

    /// Get pending transactions for payout calculation
    pub async fn get_pending_for_payout(
        pool: &PgPool,
        org_id: OrganizationId,
        before_date: DateTime<Utc>,
    ) -> Result<Vec<Transaction>, sqlx::Error> {
        sqlx::query_as::<_, Transaction>(
            r#"
            SELECT * FROM transactions
            WHERE organization_id = $1
                AND status = 'succeeded'
                AND created_at < $2
                AND id NOT IN (
                    SELECT UNNEST(transaction_ids) FROM payouts WHERE organization_id = $1
                )
            ORDER BY created_at ASC
            "#,
        )
        .bind(org_id.as_uuid())
        .bind(before_date)
        .fetch_all(pool)
        .await
    }
}

/// Transaction summary for dashboard
#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct TransactionSummary {
    pub transaction_count: i64,
    pub total_volume_cents: i64,
    pub total_fees_cents: i64,
    pub net_earnings_cents: i64,
    pub successful_count: i64,
    pub failed_count: i64,
    pub refunded_cents: i64,
}

impl TransactionSummary {
    pub fn total_volume_dollars(&self) -> f64 {
        self.total_volume_cents as f64 / 100.0
    }

    pub fn net_earnings_dollars(&self) -> f64 {
        self.net_earnings_cents as f64 / 100.0
    }

    pub fn success_rate(&self) -> f64 {
        if self.transaction_count == 0 {
            return 0.0;
        }
        (self.successful_count as f64 / self.transaction_count as f64) * 100.0
    }
}
