use shared::types::{OrganizationId, UserId};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{
    CreateCustomerSubscription, CreateTenantSubscription, CustomerSubscription, PlanTier,
    PlatformFeeTier, TenantSubscription, UpdateCustomerSubscription, UpdateTenantSubscription,
};

pub struct SubscriptionRepository;

impl SubscriptionRepository {
    // ========== Platform Fee Tiers ==========

    /// Get fee tier by plan type
    pub async fn get_fee_tier(
        pool: &PgPool,
        plan_tier: PlanTier,
    ) -> Result<Option<PlatformFeeTier>, sqlx::Error> {
        sqlx::query_as::<_, PlatformFeeTier>(
            r#"
            SELECT * FROM platform_fee_tiers
            WHERE plan_tier = $1 AND is_active = true
            "#,
        )
        .bind(plan_tier)
        .fetch_optional(pool)
        .await
    }

    /// List all active fee tiers
    pub async fn list_fee_tiers(pool: &PgPool) -> Result<Vec<PlatformFeeTier>, sqlx::Error> {
        sqlx::query_as::<_, PlatformFeeTier>(
            r#"
            SELECT * FROM platform_fee_tiers
            WHERE is_active = true
            ORDER BY monthly_price_cents ASC
            "#,
        )
        .fetch_all(pool)
        .await
    }

    /// Get fee tier for an organization based on their subscription
    /// Returns the 'free' tier if no subscription exists
    pub async fn get_org_fee_tier(
        pool: &PgPool,
        org_id: OrganizationId,
    ) -> Result<PlatformFeeTier, sqlx::Error> {
        // Get the org's plan tier from tenant_subscriptions or default to free
        let tier = sqlx::query_as::<_, PlatformFeeTier>(
            r#"
            SELECT pft.*
            FROM platform_fee_tiers pft
            LEFT JOIN tenant_subscriptions ts ON ts.plan_tier = pft.plan_tier
            WHERE (ts.organization_id = $1 OR pft.plan_tier = 'free')
            ORDER BY
                CASE WHEN ts.organization_id = $1 THEN 0 ELSE 1 END,
                pft.monthly_price_cents DESC
            LIMIT 1
            "#,
        )
        .bind(org_id.as_uuid())
        .fetch_one(pool)
        .await?;

        Ok(tier)
    }

    // ========== Tenant Subscriptions ==========

    /// Create tenant subscription
    pub async fn create_tenant_subscription(
        pool: &PgPool,
        org_id: OrganizationId,
        input: CreateTenantSubscription,
    ) -> Result<TenantSubscription, sqlx::Error> {
        let id = Uuid::new_v4();

        sqlx::query_as::<_, TenantSubscription>(
            r#"
            INSERT INTO tenant_subscriptions (
                id, organization_id, plan_tier,
                stripe_subscription_id, stripe_customer_id,
                current_period_start, current_period_end, trial_end
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(org_id.as_uuid())
        .bind(input.plan_tier)
        .bind(input.stripe_subscription_id)
        .bind(input.stripe_customer_id)
        .bind(input.current_period_start)
        .bind(input.current_period_end)
        .bind(input.trial_end)
        .fetch_one(pool)
        .await
    }

    /// Get tenant subscription by organization
    pub async fn get_tenant_subscription(
        pool: &PgPool,
        org_id: OrganizationId,
    ) -> Result<Option<TenantSubscription>, sqlx::Error> {
        sqlx::query_as::<_, TenantSubscription>(
            r#"
            SELECT * FROM tenant_subscriptions
            WHERE organization_id = $1
            "#,
        )
        .bind(org_id.as_uuid())
        .fetch_optional(pool)
        .await
    }

    /// Get tenant subscription by Stripe subscription ID
    pub async fn get_tenant_by_stripe_subscription(
        pool: &PgPool,
        stripe_subscription_id: &str,
    ) -> Result<Option<TenantSubscription>, sqlx::Error> {
        sqlx::query_as::<_, TenantSubscription>(
            r#"
            SELECT * FROM tenant_subscriptions
            WHERE stripe_subscription_id = $1
            "#,
        )
        .bind(stripe_subscription_id)
        .fetch_optional(pool)
        .await
    }

    /// Update tenant subscription
    pub async fn update_tenant_subscription(
        pool: &PgPool,
        org_id: OrganizationId,
        input: UpdateTenantSubscription,
    ) -> Result<Option<TenantSubscription>, sqlx::Error> {
        sqlx::query_as::<_, TenantSubscription>(
            r#"
            UPDATE tenant_subscriptions
            SET
                plan_tier = COALESCE($2, plan_tier),
                stripe_subscription_id = COALESCE($3, stripe_subscription_id),
                status = COALESCE($4, status),
                current_period_start = COALESCE($5, current_period_start),
                current_period_end = COALESCE($6, current_period_end),
                cancel_at_period_end = COALESCE($7, cancel_at_period_end),
                canceled_at = COALESCE($8, canceled_at),
                metadata = COALESCE($9, metadata),
                updated_at = NOW()
            WHERE organization_id = $1
            RETURNING *
            "#,
        )
        .bind(org_id.as_uuid())
        .bind(input.plan_tier)
        .bind(input.stripe_subscription_id)
        .bind(input.status)
        .bind(input.current_period_start)
        .bind(input.current_period_end)
        .bind(input.cancel_at_period_end)
        .bind(input.canceled_at)
        .bind(input.metadata)
        .fetch_optional(pool)
        .await
    }

    /// Update organization plan tier (also updates organizations table)
    pub async fn update_org_plan_tier(
        pool: &PgPool,
        org_id: OrganizationId,
        plan_tier: PlanTier,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE organizations
            SET plan_tier = $2, updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(org_id.as_uuid())
        .bind(plan_tier)
        .execute(pool)
        .await?;

        Ok(())
    }

    // ========== Customer Subscriptions ==========

    /// Create customer subscription
    pub async fn create_customer_subscription(
        pool: &PgPool,
        org_id: OrganizationId,
        user_id: UserId,
        input: CreateCustomerSubscription,
    ) -> Result<CustomerSubscription, sqlx::Error> {
        let id = Uuid::new_v4();

        sqlx::query_as::<_, CustomerSubscription>(
            r#"
            INSERT INTO customer_subscriptions (
                id, organization_id, user_id, service_id,
                name, description, price_cents, interval, interval_count,
                auto_create_bookings, preferred_day_of_week, preferred_time
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(org_id.as_uuid())
        .bind(user_id.as_uuid())
        .bind(input.service_id)
        .bind(&input.name)
        .bind(input.description)
        .bind(input.price_cents)
        .bind(&input.interval)
        .bind(input.interval_count)
        .bind(input.auto_create_bookings)
        .bind(input.preferred_day_of_week)
        .bind(input.preferred_time)
        .fetch_one(pool)
        .await
    }

    /// Get customer subscription by ID
    pub async fn get_customer_subscription(
        pool: &PgPool,
        org_id: OrganizationId,
        id: Uuid,
    ) -> Result<Option<CustomerSubscription>, sqlx::Error> {
        sqlx::query_as::<_, CustomerSubscription>(
            r#"
            SELECT * FROM customer_subscriptions
            WHERE id = $1 AND organization_id = $2
            "#,
        )
        .bind(id)
        .bind(org_id.as_uuid())
        .fetch_optional(pool)
        .await
    }

    /// Get customer subscription by Stripe subscription ID
    pub async fn get_customer_by_stripe_subscription(
        pool: &PgPool,
        stripe_subscription_id: &str,
    ) -> Result<Option<CustomerSubscription>, sqlx::Error> {
        sqlx::query_as::<_, CustomerSubscription>(
            r#"
            SELECT * FROM customer_subscriptions
            WHERE stripe_subscription_id = $1
            "#,
        )
        .bind(stripe_subscription_id)
        .fetch_optional(pool)
        .await
    }

    /// List customer subscriptions for a user
    pub async fn list_customer_subscriptions(
        pool: &PgPool,
        org_id: OrganizationId,
        user_id: UserId,
    ) -> Result<Vec<CustomerSubscription>, sqlx::Error> {
        sqlx::query_as::<_, CustomerSubscription>(
            r#"
            SELECT * FROM customer_subscriptions
            WHERE user_id = $1 AND organization_id = $2
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id.as_uuid())
        .bind(org_id.as_uuid())
        .fetch_all(pool)
        .await
    }

    /// List active customer subscriptions for an organization
    pub async fn list_active_customer_subscriptions(
        pool: &PgPool,
        org_id: OrganizationId,
    ) -> Result<Vec<CustomerSubscription>, sqlx::Error> {
        sqlx::query_as::<_, CustomerSubscription>(
            r#"
            SELECT * FROM customer_subscriptions
            WHERE organization_id = $1 AND status = 'active'
            ORDER BY created_at DESC
            "#,
        )
        .bind(org_id.as_uuid())
        .fetch_all(pool)
        .await
    }

    /// Update customer subscription
    pub async fn update_customer_subscription(
        pool: &PgPool,
        org_id: OrganizationId,
        id: Uuid,
        input: UpdateCustomerSubscription,
    ) -> Result<Option<CustomerSubscription>, sqlx::Error> {
        sqlx::query_as::<_, CustomerSubscription>(
            r#"
            UPDATE customer_subscriptions
            SET
                name = COALESCE($3, name),
                description = COALESCE($4, description),
                price_cents = COALESCE($5, price_cents),
                status = COALESCE($6, status),
                stripe_subscription_id = COALESCE($7, stripe_subscription_id),
                stripe_price_id = COALESCE($8, stripe_price_id),
                square_subscription_id = COALESCE($9, square_subscription_id),
                current_period_start = COALESCE($10, current_period_start),
                current_period_end = COALESCE($11, current_period_end),
                cancel_at_period_end = COALESCE($12, cancel_at_period_end),
                canceled_at = COALESCE($13, canceled_at),
                auto_create_bookings = COALESCE($14, auto_create_bookings),
                preferred_day_of_week = COALESCE($15, preferred_day_of_week),
                preferred_time = COALESCE($16, preferred_time),
                metadata = COALESCE($17, metadata),
                updated_at = NOW()
            WHERE id = $1 AND organization_id = $2
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(org_id.as_uuid())
        .bind(input.name)
        .bind(input.description)
        .bind(input.price_cents)
        .bind(input.status)
        .bind(input.stripe_subscription_id)
        .bind(input.stripe_price_id)
        .bind(input.square_subscription_id)
        .bind(input.current_period_start)
        .bind(input.current_period_end)
        .bind(input.cancel_at_period_end)
        .bind(input.canceled_at)
        .bind(input.auto_create_bookings)
        .bind(input.preferred_day_of_week)
        .bind(input.preferred_time)
        .bind(input.metadata)
        .fetch_optional(pool)
        .await
    }

    /// Cancel customer subscription
    pub async fn cancel_customer_subscription(
        pool: &PgPool,
        org_id: OrganizationId,
        id: Uuid,
        cancel_immediately: bool,
    ) -> Result<Option<CustomerSubscription>, sqlx::Error> {
        if cancel_immediately {
            sqlx::query_as::<_, CustomerSubscription>(
                r#"
                UPDATE customer_subscriptions
                SET
                    status = 'canceled',
                    canceled_at = NOW(),
                    updated_at = NOW()
                WHERE id = $1 AND organization_id = $2
                RETURNING *
                "#,
            )
            .bind(id)
            .bind(org_id.as_uuid())
            .fetch_optional(pool)
            .await
        } else {
            sqlx::query_as::<_, CustomerSubscription>(
                r#"
                UPDATE customer_subscriptions
                SET
                    cancel_at_period_end = true,
                    updated_at = NOW()
                WHERE id = $1 AND organization_id = $2
                RETURNING *
                "#,
            )
            .bind(id)
            .bind(org_id.as_uuid())
            .fetch_optional(pool)
            .await
        }
    }

    /// Get subscriptions needing booking creation
    pub async fn get_subscriptions_for_booking_creation(
        pool: &PgPool,
    ) -> Result<Vec<CustomerSubscription>, sqlx::Error> {
        sqlx::query_as::<_, CustomerSubscription>(
            r#"
            SELECT * FROM customer_subscriptions
            WHERE status = 'active'
                AND auto_create_bookings = true
                AND current_period_end IS NOT NULL
                AND current_period_end > NOW()
            ORDER BY current_period_end ASC
            "#,
        )
        .fetch_all(pool)
        .await
    }
}
