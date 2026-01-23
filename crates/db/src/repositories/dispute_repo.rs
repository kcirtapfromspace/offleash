use shared::types::OrganizationId;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{
    CreateDispute, CreateWebhookEvent, Dispute, DisputeStatus, PaymentWebhookEvent, UpdateDispute,
};

pub struct DisputeRepository;

impl DisputeRepository {
    /// Create a dispute
    pub async fn create(
        pool: &PgPool,
        org_id: OrganizationId,
        input: CreateDispute,
    ) -> Result<Dispute, sqlx::Error> {
        let id = Uuid::new_v4();

        sqlx::query_as::<_, Dispute>(
            r#"
            INSERT INTO disputes (
                id, organization_id, transaction_id, amount_cents, currency,
                stripe_dispute_id, square_dispute_id, reason, evidence_due_by
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(org_id.as_uuid())
        .bind(input.transaction_id)
        .bind(input.amount_cents)
        .bind(&input.currency)
        .bind(input.stripe_dispute_id)
        .bind(input.square_dispute_id)
        .bind(&input.reason)
        .bind(input.evidence_due_by)
        .fetch_one(pool)
        .await
    }

    /// Get dispute by ID
    pub async fn get_by_id(
        pool: &PgPool,
        org_id: OrganizationId,
        id: Uuid,
    ) -> Result<Option<Dispute>, sqlx::Error> {
        sqlx::query_as::<_, Dispute>(
            r#"
            SELECT * FROM disputes
            WHERE id = $1 AND organization_id = $2
            "#,
        )
        .bind(id)
        .bind(org_id.as_uuid())
        .fetch_optional(pool)
        .await
    }

    /// Get dispute by Stripe dispute ID
    pub async fn get_by_stripe_dispute(
        pool: &PgPool,
        stripe_dispute_id: &str,
    ) -> Result<Option<Dispute>, sqlx::Error> {
        sqlx::query_as::<_, Dispute>(
            r#"
            SELECT * FROM disputes
            WHERE stripe_dispute_id = $1
            "#,
        )
        .bind(stripe_dispute_id)
        .fetch_optional(pool)
        .await
    }

    /// Get dispute by transaction ID
    pub async fn get_by_transaction(
        pool: &PgPool,
        transaction_id: Uuid,
    ) -> Result<Option<Dispute>, sqlx::Error> {
        sqlx::query_as::<_, Dispute>(
            r#"
            SELECT * FROM disputes
            WHERE transaction_id = $1
            "#,
        )
        .bind(transaction_id)
        .fetch_optional(pool)
        .await
    }

    /// List disputes for an organization
    pub async fn list_for_org(
        pool: &PgPool,
        org_id: OrganizationId,
        status: Option<DisputeStatus>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Dispute>, sqlx::Error> {
        sqlx::query_as::<_, Dispute>(
            r#"
            SELECT * FROM disputes
            WHERE organization_id = $1
                AND ($2::dispute_status IS NULL OR status = $2)
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

    /// List disputes needing response
    pub async fn list_needing_response(
        pool: &PgPool,
        org_id: OrganizationId,
    ) -> Result<Vec<Dispute>, sqlx::Error> {
        sqlx::query_as::<_, Dispute>(
            r#"
            SELECT * FROM disputes
            WHERE organization_id = $1
                AND status = 'needs_response'
            ORDER BY evidence_due_by ASC NULLS LAST
            "#,
        )
        .bind(org_id.as_uuid())
        .fetch_all(pool)
        .await
    }

    /// Update dispute
    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        input: UpdateDispute,
    ) -> Result<Option<Dispute>, sqlx::Error> {
        sqlx::query_as::<_, Dispute>(
            r#"
            UPDATE disputes
            SET
                status = COALESCE($2, status),
                evidence_submitted = COALESCE($3, evidence_submitted),
                evidence_due_by = COALESCE($4, evidence_due_by),
                resolved_at = COALESCE($5, resolved_at),
                outcome = COALESCE($6, outcome),
                metadata = COALESCE($7, metadata),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(input.status)
        .bind(input.evidence_submitted)
        .bind(input.evidence_due_by)
        .bind(input.resolved_at)
        .bind(input.outcome)
        .bind(input.metadata)
        .fetch_optional(pool)
        .await
    }

    /// Mark evidence as submitted
    pub async fn mark_evidence_submitted(
        pool: &PgPool,
        id: Uuid,
    ) -> Result<Option<Dispute>, sqlx::Error> {
        sqlx::query_as::<_, Dispute>(
            r#"
            UPDATE disputes
            SET
                evidence_submitted = true,
                status = 'under_review',
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    /// Resolve dispute
    pub async fn resolve(
        pool: &PgPool,
        id: Uuid,
        outcome: &str,
    ) -> Result<Option<Dispute>, sqlx::Error> {
        let status = if outcome == "won" {
            DisputeStatus::Won
        } else {
            DisputeStatus::Lost
        };

        sqlx::query_as::<_, Dispute>(
            r#"
            UPDATE disputes
            SET
                status = $2,
                outcome = $3,
                resolved_at = NOW(),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(status)
        .bind(outcome)
        .fetch_optional(pool)
        .await
    }

    /// Count open disputes for organization
    pub async fn count_open(
        pool: &PgPool,
        org_id: OrganizationId,
    ) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM disputes
            WHERE organization_id = $1
                AND status IN ('needs_response', 'under_review')
            "#,
        )
        .bind(org_id.as_uuid())
        .fetch_one(pool)
        .await
    }
}

pub struct WebhookEventRepository;

impl WebhookEventRepository {
    /// Record a webhook event
    pub async fn create(
        pool: &PgPool,
        input: CreateWebhookEvent,
    ) -> Result<PaymentWebhookEvent, sqlx::Error> {
        let id = Uuid::new_v4();

        sqlx::query_as::<_, PaymentWebhookEvent>(
            r#"
            INSERT INTO payment_webhook_events (id, provider, event_id, event_type, payload)
            VALUES ($1, $2::payment_provider_type, $3, $4, $5)
            ON CONFLICT (provider, event_id) DO NOTHING
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(&input.provider)
        .bind(&input.event_id)
        .bind(&input.event_type)
        .bind(&input.payload)
        .fetch_one(pool)
        .await
    }

    /// Check if event was already processed (idempotency)
    pub async fn is_processed(
        pool: &PgPool,
        provider: &str,
        event_id: &str,
    ) -> Result<bool, sqlx::Error> {
        let result: Option<bool> = sqlx::query_scalar(
            r#"
            SELECT processed FROM payment_webhook_events
            WHERE provider = $1::payment_provider_type AND event_id = $2
            "#,
        )
        .bind(provider)
        .bind(event_id)
        .fetch_optional(pool)
        .await?;

        Ok(result.unwrap_or(false))
    }

    /// Mark event as processed
    pub async fn mark_processed(
        pool: &PgPool,
        provider: &str,
        event_id: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE payment_webhook_events
            SET processed = true, processed_at = NOW()
            WHERE provider = $1::payment_provider_type AND event_id = $2
            "#,
        )
        .bind(provider)
        .bind(event_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Mark event as failed
    pub async fn mark_failed(
        pool: &PgPool,
        provider: &str,
        event_id: &str,
        error_message: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE payment_webhook_events
            SET error_message = $3
            WHERE provider = $1::payment_provider_type AND event_id = $2
            "#,
        )
        .bind(provider)
        .bind(event_id)
        .bind(error_message)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Get unprocessed events for retry
    pub async fn get_unprocessed(
        pool: &PgPool,
        provider: &str,
        limit: i64,
    ) -> Result<Vec<PaymentWebhookEvent>, sqlx::Error> {
        sqlx::query_as::<_, PaymentWebhookEvent>(
            r#"
            SELECT * FROM payment_webhook_events
            WHERE provider = $1::payment_provider_type
                AND processed = false
                AND error_message IS NULL
            ORDER BY created_at ASC
            LIMIT $2
            "#,
        )
        .bind(provider)
        .bind(limit)
        .fetch_all(pool)
        .await
    }
}
