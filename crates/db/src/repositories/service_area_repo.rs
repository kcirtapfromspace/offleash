use shared::types::{OrganizationId, UserId};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{CreateServiceArea, ServiceArea, UpdateServiceArea};

pub struct ServiceAreaRepository;

impl ServiceAreaRepository {
    pub async fn find_by_id(
        pool: &PgPool,
        org_id: OrganizationId,
        id: Uuid,
    ) -> Result<Option<ServiceArea>, sqlx::Error> {
        sqlx::query_as::<_, ServiceArea>(
            r#"
            SELECT id, organization_id, walker_id, name, color, polygon,
                   min_latitude, max_latitude, min_longitude, max_longitude,
                   is_active, priority, price_adjustment_percent, notes,
                   created_at, updated_at
            FROM service_areas
            WHERE id = $1 AND organization_id = $2
            "#,
        )
        .bind(id)
        .bind(org_id.as_uuid())
        .fetch_optional(pool)
        .await
    }

    pub async fn find_by_walker(
        pool: &PgPool,
        org_id: OrganizationId,
        walker_id: UserId,
    ) -> Result<Vec<ServiceArea>, sqlx::Error> {
        sqlx::query_as::<_, ServiceArea>(
            r#"
            SELECT id, organization_id, walker_id, name, color, polygon,
                   min_latitude, max_latitude, min_longitude, max_longitude,
                   is_active, priority, price_adjustment_percent, notes,
                   created_at, updated_at
            FROM service_areas
            WHERE walker_id = $1 AND organization_id = $2
            ORDER BY priority ASC, name ASC
            "#,
        )
        .bind(walker_id.as_uuid())
        .bind(org_id.as_uuid())
        .fetch_all(pool)
        .await
    }

    pub async fn list_all(
        pool: &PgPool,
        org_id: OrganizationId,
    ) -> Result<Vec<ServiceArea>, sqlx::Error> {
        sqlx::query_as::<_, ServiceArea>(
            r#"
            SELECT id, organization_id, walker_id, name, color, polygon,
                   min_latitude, max_latitude, min_longitude, max_longitude,
                   is_active, priority, price_adjustment_percent, notes,
                   created_at, updated_at
            FROM service_areas
            WHERE organization_id = $1
            ORDER BY walker_id, priority ASC, name ASC
            "#,
        )
        .bind(org_id.as_uuid())
        .fetch_all(pool)
        .await
    }

    pub async fn create(
        pool: &PgPool,
        area: CreateServiceArea,
    ) -> Result<ServiceArea, sqlx::Error> {
        let polygon_json = sqlx::types::Json(&area.polygon);

        sqlx::query_as::<_, ServiceArea>(
            r#"
            INSERT INTO service_areas (organization_id, walker_id, name, color, polygon,
                is_active, priority, price_adjustment_percent, notes)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id, organization_id, walker_id, name, color, polygon,
                min_latitude, max_latitude, min_longitude, max_longitude,
                is_active, priority, price_adjustment_percent, notes,
                created_at, updated_at
            "#,
        )
        .bind(area.organization_id.as_uuid())
        .bind(area.walker_id.as_uuid())
        .bind(&area.name)
        .bind(&area.color)
        .bind(polygon_json)
        .bind(area.is_active)
        .bind(area.priority)
        .bind(area.price_adjustment_percent)
        .bind(&area.notes)
        .fetch_one(pool)
        .await
    }

    pub async fn update(
        pool: &PgPool,
        org_id: OrganizationId,
        id: Uuid,
        update: UpdateServiceArea,
    ) -> Result<Option<ServiceArea>, sqlx::Error> {
        let polygon_json: Option<sqlx::types::Json<Vec<crate::models::PolygonPoint>>> = update
            .polygon
            .as_ref()
            .map(|p| sqlx::types::Json(p.clone()));

        sqlx::query_as::<_, ServiceArea>(
            r#"
            UPDATE service_areas
            SET name = COALESCE($3, name),
                color = COALESCE($4, color),
                polygon = COALESCE($5, polygon),
                is_active = COALESCE($6, is_active),
                priority = COALESCE($7, priority),
                price_adjustment_percent = COALESCE($8, price_adjustment_percent),
                notes = COALESCE($9, notes),
                updated_at = NOW()
            WHERE id = $1 AND organization_id = $2
            RETURNING id, organization_id, walker_id, name, color, polygon,
                min_latitude, max_latitude, min_longitude, max_longitude,
                is_active, priority, price_adjustment_percent, notes,
                created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(org_id.as_uuid())
        .bind(&update.name)
        .bind(&update.color)
        .bind(polygon_json)
        .bind(update.is_active)
        .bind(update.priority)
        .bind(update.price_adjustment_percent)
        .bind(&update.notes)
        .fetch_optional(pool)
        .await
    }

    pub async fn delete(
        pool: &PgPool,
        org_id: OrganizationId,
        id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"
            DELETE FROM service_areas
            WHERE id = $1 AND organization_id = $2
            "#,
        )
        .bind(id)
        .bind(org_id.as_uuid())
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Find walkers who have service areas covering a given location
    pub async fn find_walkers_for_location(
        pool: &PgPool,
        org_id: OrganizationId,
        latitude: f64,
        longitude: f64,
    ) -> Result<Vec<(Uuid, Uuid, String)>, sqlx::Error> {
        // Uses bounding box for quick filtering
        // Note: For precise polygon containment, you'd need PostGIS or application-level check
        sqlx::query_as::<_, (Uuid, Uuid, String)>(
            r#"
            SELECT walker_id, id, name
            FROM service_areas
            WHERE organization_id = $1
              AND is_active = TRUE
              AND $2 >= min_latitude
              AND $2 <= max_latitude
              AND $3 >= min_longitude
              AND $3 <= max_longitude
            ORDER BY priority ASC
            "#,
        )
        .bind(org_id.as_uuid())
        .bind(latitude)
        .bind(longitude)
        .fetch_all(pool)
        .await
    }
}
