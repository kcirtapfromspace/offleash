use shared::types::ServiceId;
use sqlx::PgPool;

use crate::models::{CreateService, Service, UpdateService};

pub struct ServiceRepository;

impl ServiceRepository {
    pub async fn create(pool: &PgPool, input: CreateService) -> Result<Service, sqlx::Error> {
        let id = ServiceId::new();

        sqlx::query_as::<_, Service>(
            r#"
            INSERT INTO services (id, name, description, duration_minutes, base_price_cents)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, name, description, duration_minutes, base_price_cents, is_active, created_at, updated_at
            "#,
        )
        .bind(id.as_uuid())
        .bind(&input.name)
        .bind(&input.description)
        .bind(input.duration_minutes)
        .bind(input.base_price_cents)
        .fetch_one(pool)
        .await
    }

    pub async fn find_by_id(pool: &PgPool, id: ServiceId) -> Result<Option<Service>, sqlx::Error> {
        sqlx::query_as::<_, Service>(
            r#"
            SELECT id, name, description, duration_minutes, base_price_cents, is_active, created_at, updated_at
            FROM services
            WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
        .fetch_optional(pool)
        .await
    }

    pub async fn list_active(pool: &PgPool) -> Result<Vec<Service>, sqlx::Error> {
        sqlx::query_as::<_, Service>(
            r#"
            SELECT id, name, description, duration_minutes, base_price_cents, is_active, created_at, updated_at
            FROM services
            WHERE is_active = true
            ORDER BY name
            "#,
        )
        .fetch_all(pool)
        .await
    }

    pub async fn update(
        pool: &PgPool,
        id: ServiceId,
        input: UpdateService,
    ) -> Result<Option<Service>, sqlx::Error> {
        sqlx::query_as::<_, Service>(
            r#"
            UPDATE services
            SET
                name = COALESCE($2, name),
                description = COALESCE($3, description),
                duration_minutes = COALESCE($4, duration_minutes),
                base_price_cents = COALESCE($5, base_price_cents),
                is_active = COALESCE($6, is_active),
                updated_at = NOW()
            WHERE id = $1
            RETURNING id, name, description, duration_minutes, base_price_cents, is_active, created_at, updated_at
            "#,
        )
        .bind(id.as_uuid())
        .bind(&input.name)
        .bind(&input.description)
        .bind(input.duration_minutes)
        .bind(input.base_price_cents)
        .bind(input.is_active)
        .fetch_optional(pool)
        .await
    }
}
