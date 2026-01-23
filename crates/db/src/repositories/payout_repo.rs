use chrono::{DateTime, Utc};
use shared::types::OrganizationId;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{
    CreatePayout, CreatePayoutSettings, Payout, PayoutSettings, PayoutStatus, PayoutSummary,
    UpdatePayout, UpdatePayoutSettings,
};

pub struct PayoutRepository;

impl PayoutRepository {
    // ========== Payout Settings ==========

    /// Create payout settings
    pub async fn create_settings(
        pool: &PgPool,
        org_id: OrganizationId,
        input: CreatePayoutSettings,
    ) -> Result<PayoutSettings, sqlx::Error> {
        let id = Uuid::new_v4();

        sqlx::query_as::<_, PayoutSettings>(
            r#"
            INSERT INTO payout_settings (
                id, organization_id, payout_method, payout_schedule,
                payout_day_of_week, payout_day_of_month, minimum_payout_cents
            )
            VALUES ($1, $2, $3, $4, $5, $6, COALESCE($7, 100))
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(org_id.as_uuid())
        .bind(&input.payout_method)
        .bind(&input.payout_schedule)
        .bind(input.payout_day_of_week)
        .bind(input.payout_day_of_month)
        .bind(input.minimum_payout_cents)
        .fetch_one(pool)
        .await
    }

    /// Get payout settings for an organization
    pub async fn get_settings(
        pool: &PgPool,
        org_id: OrganizationId,
    ) -> Result<Option<PayoutSettings>, sqlx::Error> {
        sqlx::query_as::<_, PayoutSettings>(
            r#"
            SELECT * FROM payout_settings
            WHERE organization_id = $1
            "#,
        )
        .bind(org_id.as_uuid())
        .fetch_optional(pool)
        .await
    }

    /// Update payout settings
    pub async fn update_settings(
        pool: &PgPool,
        org_id: OrganizationId,
        input: UpdatePayoutSettings,
    ) -> Result<Option<PayoutSettings>, sqlx::Error> {
        sqlx::query_as::<_, PayoutSettings>(
            r#"
            UPDATE payout_settings
            SET
                payout_method = COALESCE($2, payout_method),
                stripe_bank_account_id = COALESCE($3, stripe_bank_account_id),
                square_bank_account_id = COALESCE($4, square_bank_account_id),
                bank_name = COALESCE($5, bank_name),
                bank_account_last_four = COALESCE($6, bank_account_last_four),
                bank_routing_last_four = COALESCE($7, bank_routing_last_four),
                payout_schedule = COALESCE($8, payout_schedule),
                payout_day_of_week = COALESCE($9, payout_day_of_week),
                payout_day_of_month = COALESCE($10, payout_day_of_month),
                minimum_payout_cents = COALESCE($11, minimum_payout_cents),
                is_verified = COALESCE($12, is_verified),
                verification_status = COALESCE($13, verification_status),
                updated_at = NOW()
            WHERE organization_id = $1
            RETURNING *
            "#,
        )
        .bind(org_id.as_uuid())
        .bind(input.payout_method)
        .bind(input.stripe_bank_account_id)
        .bind(input.square_bank_account_id)
        .bind(input.bank_name)
        .bind(input.bank_account_last_four)
        .bind(input.bank_routing_last_four)
        .bind(input.payout_schedule)
        .bind(input.payout_day_of_week)
        .bind(input.payout_day_of_month)
        .bind(input.minimum_payout_cents)
        .bind(input.is_verified)
        .bind(input.verification_status)
        .fetch_optional(pool)
        .await
    }

    // ========== Payouts ==========

    /// Create a payout
    pub async fn create(
        pool: &PgPool,
        org_id: OrganizationId,
        input: CreatePayout,
    ) -> Result<Payout, sqlx::Error> {
        let id = Uuid::new_v4();

        sqlx::query_as::<_, Payout>(
            r#"
            INSERT INTO payouts (
                id, organization_id, amount_cents, fee_cents, net_amount_cents,
                currency, period_start, period_end, transaction_count, transaction_ids
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(org_id.as_uuid())
        .bind(input.amount_cents)
        .bind(input.fee_cents)
        .bind(input.net_amount_cents)
        .bind(&input.currency)
        .bind(input.period_start)
        .bind(input.period_end)
        .bind(input.transaction_count)
        .bind(&input.transaction_ids)
        .fetch_one(pool)
        .await
    }

    /// Get payout by ID
    pub async fn get_by_id(
        pool: &PgPool,
        org_id: OrganizationId,
        id: Uuid,
    ) -> Result<Option<Payout>, sqlx::Error> {
        sqlx::query_as::<_, Payout>(
            r#"
            SELECT * FROM payouts
            WHERE id = $1 AND organization_id = $2
            "#,
        )
        .bind(id)
        .bind(org_id.as_uuid())
        .fetch_optional(pool)
        .await
    }

    /// Get payout by Stripe payout ID
    pub async fn get_by_stripe_payout(
        pool: &PgPool,
        stripe_payout_id: &str,
    ) -> Result<Option<Payout>, sqlx::Error> {
        sqlx::query_as::<_, Payout>(
            r#"
            SELECT * FROM payouts
            WHERE stripe_payout_id = $1
            "#,
        )
        .bind(stripe_payout_id)
        .fetch_optional(pool)
        .await
    }

    /// List payouts for an organization
    pub async fn list_for_org(
        pool: &PgPool,
        org_id: OrganizationId,
        status: Option<PayoutStatus>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Payout>, sqlx::Error> {
        sqlx::query_as::<_, Payout>(
            r#"
            SELECT * FROM payouts
            WHERE organization_id = $1
                AND ($2::payout_status IS NULL OR status = $2)
            ORDER BY created_at DESC
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(org_id.as_uuid())
        .bind(status)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
    }

    /// Update payout
    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        input: UpdatePayout,
    ) -> Result<Option<Payout>, sqlx::Error> {
        sqlx::query_as::<_, Payout>(
            r#"
            UPDATE payouts
            SET
                stripe_payout_id = COALESCE($2, stripe_payout_id),
                stripe_transfer_id = COALESCE($3, stripe_transfer_id),
                square_payout_id = COALESCE($4, square_payout_id),
                status = COALESCE($5, status),
                initiated_at = COALESCE($6, initiated_at),
                arrival_date = COALESCE($7, arrival_date),
                completed_at = COALESCE($8, completed_at),
                failure_code = COALESCE($9, failure_code),
                failure_message = COALESCE($10, failure_message),
                metadata = COALESCE($11, metadata),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(input.stripe_payout_id)
        .bind(input.stripe_transfer_id)
        .bind(input.square_payout_id)
        .bind(input.status)
        .bind(input.initiated_at)
        .bind(input.arrival_date)
        .bind(input.completed_at)
        .bind(input.failure_code)
        .bind(input.failure_message)
        .bind(input.metadata)
        .fetch_optional(pool)
        .await
    }

    /// Update payout status
    pub async fn update_status(
        pool: &PgPool,
        id: Uuid,
        status: PayoutStatus,
    ) -> Result<Option<Payout>, sqlx::Error> {
        let completed_at = if status == PayoutStatus::Paid {
            Some(Utc::now())
        } else {
            None
        };

        sqlx::query_as::<_, Payout>(
            r#"
            UPDATE payouts
            SET
                status = $2,
                completed_at = COALESCE($3, completed_at),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(status)
        .bind(completed_at)
        .fetch_optional(pool)
        .await
    }

    /// Get payout summary for an organization
    pub async fn get_summary(
        pool: &PgPool,
        org_id: OrganizationId,
    ) -> Result<PayoutSummary, sqlx::Error> {
        sqlx::query_as::<_, PayoutSummary>(
            r#"
            SELECT
                COUNT(*) as total_payouts,
                COALESCE(SUM(net_amount_cents), 0) as total_amount_cents,
                COALESCE(SUM(CASE WHEN status = 'pending' THEN net_amount_cents ELSE 0 END), 0) as pending_amount_cents,
                MAX(CASE WHEN status = 'paid' THEN completed_at END) as last_payout_date,
                MIN(CASE WHEN status = 'pending' THEN arrival_date END) as next_payout_date
            FROM payouts
            WHERE organization_id = $1
            "#,
        )
        .bind(org_id.as_uuid())
        .fetch_one(pool)
        .await
    }

    /// Get organizations due for payout
    pub async fn get_orgs_due_for_payout(
        pool: &PgPool,
        payout_schedule: &str,
        day_of_week: Option<i32>,
        day_of_month: Option<i32>,
    ) -> Result<Vec<OrganizationId>, sqlx::Error> {
        let orgs = sqlx::query_scalar::<_, Uuid>(
            r#"
            SELECT DISTINCT ps.organization_id
            FROM payout_settings ps
            WHERE ps.is_verified = true
                AND ps.payout_schedule = $1
                AND ($2::integer IS NULL OR ps.payout_day_of_week = $2)
                AND ($3::integer IS NULL OR ps.payout_day_of_month = $3)
            "#,
        )
        .bind(payout_schedule)
        .bind(day_of_week)
        .bind(day_of_month)
        .fetch_all(pool)
        .await?;

        Ok(orgs.into_iter().map(OrganizationId::from_uuid).collect())
    }
}
