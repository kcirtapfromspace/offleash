use shared::types::{LocationId, UserId};
use sqlx::PgPool;

use crate::models::{CreateLocation, Location, UpdateLocation};

pub struct LocationRepository;

impl LocationRepository {
    pub async fn create(pool: &PgPool, input: CreateLocation) -> Result<Location, sqlx::Error> {
        let id = LocationId::new();

        sqlx::query_as::<_, Location>(
            r#"
            INSERT INTO locations (id, user_id, name, address, city, state, zip_code, latitude, longitude, notes, is_default)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING id, user_id, name, address, city, state, zip_code, latitude, longitude, notes, is_default, created_at, updated_at
            "#,
        )
        .bind(id.as_uuid())
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

    pub async fn find_by_id(pool: &PgPool, id: LocationId) -> Result<Option<Location>, sqlx::Error> {
        sqlx::query_as::<_, Location>(
            r#"
            SELECT id, user_id, name, address, city, state, zip_code, latitude, longitude, notes, is_default, created_at, updated_at
            FROM locations
            WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
        .fetch_optional(pool)
        .await
    }

    pub async fn find_by_user(pool: &PgPool, user_id: UserId) -> Result<Vec<Location>, sqlx::Error> {
        sqlx::query_as::<_, Location>(
            r#"
            SELECT id, user_id, name, address, city, state, zip_code, latitude, longitude, notes, is_default, created_at, updated_at
            FROM locations
            WHERE user_id = $1
            ORDER BY is_default DESC, name
            "#,
        )
        .bind(user_id.as_uuid())
        .fetch_all(pool)
        .await
    }

    pub async fn update(
        pool: &PgPool,
        id: LocationId,
        input: UpdateLocation,
    ) -> Result<Option<Location>, sqlx::Error> {
        sqlx::query_as::<_, Location>(
            r#"
            UPDATE locations
            SET
                name = COALESCE($2, name),
                address = COALESCE($3, address),
                city = COALESCE($4, city),
                state = COALESCE($5, state),
                zip_code = COALESCE($6, zip_code),
                latitude = COALESCE($7, latitude),
                longitude = COALESCE($8, longitude),
                notes = COALESCE($9, notes),
                is_default = COALESCE($10, is_default),
                updated_at = NOW()
            WHERE id = $1
            RETURNING id, user_id, name, address, city, state, zip_code, latitude, longitude, notes, is_default, created_at, updated_at
            "#,
        )
        .bind(id.as_uuid())
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

    pub async fn delete(pool: &PgPool, id: LocationId) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM locations WHERE id = $1")
            .bind(id.as_uuid())
            .execute(pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }
}
