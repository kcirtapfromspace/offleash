use shared::types::{LocationId, OrganizationId, UserId};
use sqlx::PgPool;

use crate::models::{CreateLocation, Location, UpdateLocation};

pub struct LocationRepository;

impl LocationRepository {
    pub async fn create(pool: &PgPool, input: CreateLocation) -> Result<Location, sqlx::Error> {
        let id = LocationId::new();

        sqlx::query_as::<_, Location>(
            r#"
            INSERT INTO locations (id, organization_id, user_id, name, address, city, state, zip_code, latitude, longitude, notes, is_default)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING id, organization_id, user_id, name, address, city, state, zip_code, latitude, longitude, notes, is_default, created_at, updated_at
            "#,
        )
        .bind(id.as_uuid())
        .bind(input.organization_id.as_uuid())
        .bind(input.user_id.as_uuid())
        .bind(&input.name)
        .bind(&input.address)
        .bind(&input.city)
        .bind(&input.state)
        .bind(&input.zip_code)
        .bind(input.latitude)
        .bind(input.longitude)
        .bind(&input.notes)
        .bind(input.is_default)
        .fetch_one(pool)
        .await
    }

    pub async fn find_by_id(
        pool: &PgPool,
        org_id: OrganizationId,
        id: LocationId,
    ) -> Result<Option<Location>, sqlx::Error> {
        sqlx::query_as::<_, Location>(
            r#"
            SELECT id, organization_id, user_id, name, address, city, state, zip_code, latitude, longitude, notes, is_default, created_at, updated_at
            FROM locations
            WHERE id = $1 AND organization_id = $2
            "#,
        )
        .bind(id.as_uuid())
        .bind(org_id.as_uuid())
        .fetch_optional(pool)
        .await
    }

    pub async fn find_by_user(
        pool: &PgPool,
        org_id: OrganizationId,
        user_id: UserId,
    ) -> Result<Vec<Location>, sqlx::Error> {
        sqlx::query_as::<_, Location>(
            r#"
            SELECT id, organization_id, user_id, name, address, city, state, zip_code, latitude, longitude, notes, is_default, created_at, updated_at
            FROM locations
            WHERE user_id = $1 AND organization_id = $2
            ORDER BY is_default DESC, name
            "#,
        )
        .bind(user_id.as_uuid())
        .bind(org_id.as_uuid())
        .fetch_all(pool)
        .await
    }

    pub async fn update(
        pool: &PgPool,
        org_id: OrganizationId,
        id: LocationId,
        input: UpdateLocation,
    ) -> Result<Option<Location>, sqlx::Error> {
        sqlx::query_as::<_, Location>(
            r#"
            UPDATE locations
            SET
                name = COALESCE($3, name),
                address = COALESCE($4, address),
                city = COALESCE($5, city),
                state = COALESCE($6, state),
                zip_code = COALESCE($7, zip_code),
                latitude = COALESCE($8, latitude),
                longitude = COALESCE($9, longitude),
                notes = COALESCE($10, notes),
                is_default = COALESCE($11, is_default),
                updated_at = NOW()
            WHERE id = $1 AND organization_id = $2
            RETURNING id, organization_id, user_id, name, address, city, state, zip_code, latitude, longitude, notes, is_default, created_at, updated_at
            "#,
        )
        .bind(id.as_uuid())
        .bind(org_id.as_uuid())
        .bind(&input.name)
        .bind(&input.address)
        .bind(&input.city)
        .bind(&input.state)
        .bind(&input.zip_code)
        .bind(input.latitude)
        .bind(input.longitude)
        .bind(&input.notes)
        .bind(input.is_default)
        .fetch_optional(pool)
        .await
    }

    pub async fn delete(
        pool: &PgPool,
        org_id: OrganizationId,
        id: LocationId,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM locations WHERE id = $1 AND organization_id = $2")
            .bind(id.as_uuid())
            .bind(org_id.as_uuid())
            .execute(pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Unset is_default for all locations belonging to a user
    pub async fn unset_defaults_for_user(
        pool: &PgPool,
        user_id: UserId,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE locations SET is_default = false WHERE user_id = $1")
            .bind(user_id.as_uuid())
            .execute(pool)
            .await?;

        Ok(())
    }
}
