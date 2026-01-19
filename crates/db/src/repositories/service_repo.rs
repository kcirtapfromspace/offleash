use shared::types::{OrganizationId, ServiceId};
use sqlx::PgPool;

use crate::models::{CreateService, Service, UpdateService};

pub struct ServiceRepository;

impl ServiceRepository {
    pub async fn create(pool: &PgPool, input: CreateService) -> Result<Service, sqlx::Error> {
        let id = ServiceId::new();

        sqlx::query_as::<_, Service>(
            r#"
            INSERT INTO services (id, organization_id, name, description, duration_minutes, base_price_cents)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, organization_id, name, description, duration_minutes, base_price_cents, is_active, created_at, updated_at
            "#,
        )
        .bind(id.as_uuid())
        .bind(input.organization_id.as_uuid())
        .bind(&input.name)
        .bind(&input.description)
        .bind(input.duration_minutes)
        .bind(input.base_price_cents)
        .fetch_one(pool)
        .await
    }

    pub async fn find_by_id(
        pool: &PgPool,
        org_id: OrganizationId,
        id: ServiceId,
    ) -> Result<Option<Service>, sqlx::Error> {
        sqlx::query_as::<_, Service>(
            r#"
            SELECT id, organization_id, name, description, duration_minutes, base_price_cents, is_active, created_at, updated_at
            FROM services
            WHERE id = $1 AND organization_id = $2
            "#,
        )
        .bind(id.as_uuid())
        .bind(org_id.as_uuid())
        .fetch_optional(pool)
        .await
    }

    pub async fn list_active(
        pool: &PgPool,
        org_id: OrganizationId,
    ) -> Result<Vec<Service>, sqlx::Error> {
        sqlx::query_as::<_, Service>(
            r#"
            SELECT id, organization_id, name, description, duration_minutes, base_price_cents, is_active, created_at, updated_at
            FROM services
            WHERE organization_id = $1 AND is_active = true
            ORDER BY name
            "#,
        )
        .bind(org_id.as_uuid())
        .fetch_all(pool)
        .await
    }

    pub async fn list_all(
        pool: &PgPool,
        org_id: OrganizationId,
    ) -> Result<Vec<Service>, sqlx::Error> {
        sqlx::query_as::<_, Service>(
            r#"
            SELECT id, organization_id, name, description, duration_minutes, base_price_cents, is_active, created_at, updated_at
            FROM services
            WHERE organization_id = $1
            ORDER BY name
            "#,
        )
        .bind(org_id.as_uuid())
        .fetch_all(pool)
        .await
    }

    pub async fn update(
        pool: &PgPool,
        org_id: OrganizationId,
        id: ServiceId,
        input: UpdateService,
    ) -> Result<Option<Service>, sqlx::Error> {
        sqlx::query_as::<_, Service>(
            r#"
            UPDATE services
            SET
                name = COALESCE($3, name),
                description = COALESCE($4, description),
                duration_minutes = COALESCE($5, duration_minutes),
                base_price_cents = COALESCE($6, base_price_cents),
                is_active = COALESCE($7, is_active),
                updated_at = NOW()
            WHERE id = $1 AND organization_id = $2
            RETURNING id, organization_id, name, description, duration_minutes, base_price_cents, is_active, created_at, updated_at
            "#,
        )
        .bind(id.as_uuid())
        .bind(org_id.as_uuid())
        .bind(&input.name)
        .bind(&input.description)
        .bind(input.duration_minutes)
        .bind(input.base_price_cents)
        .bind(input.is_active)
        .fetch_optional(pool)
        .await
    }
}
