use chrono::NaiveTime;
use shared::types::{UserId, WorkingHoursId};
use sqlx::PgPool;

use crate::models::{CreateWorkingHours, UpdateWorkingHours, WorkingHours};

pub struct WorkingHoursRepository;

impl WorkingHoursRepository {
    pub async fn create(pool: &PgPool, input: CreateWorkingHours) -> Result<WorkingHours, sqlx::Error> {
        let id = WorkingHoursId::new();

        sqlx::query_as::<_, WorkingHours>(
            r#"
            INSERT INTO working_hours (id, walker_id, day_of_week, start_time, end_time)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, walker_id, day_of_week, start_time, end_time, is_active, created_at, updated_at
            "#,
        )
        .bind(id.as_uuid())
        .bind(input.walker_id.as_uuid())
        .bind(input.day_of_week)
        .bind(input.start_time)
        .bind(input.end_time)
        .fetch_one(pool)
        .await
    }

    pub async fn find_by_id(
        pool: &PgPool,
        id: WorkingHoursId,
    ) -> Result<Option<WorkingHours>, sqlx::Error> {
        sqlx::query_as::<_, WorkingHours>(
            r#"
            SELECT id, walker_id, day_of_week, start_time, end_time, is_active, created_at, updated_at
            FROM working_hours
            WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
        .fetch_optional(pool)
        .await
    }

    pub async fn find_by_walker(
        pool: &PgPool,
        walker_id: UserId,
    ) -> Result<Vec<WorkingHours>, sqlx::Error> {
        sqlx::query_as::<_, WorkingHours>(
            r#"
            SELECT id, walker_id, day_of_week, start_time, end_time, is_active, created_at, updated_at
            FROM working_hours
            WHERE walker_id = $1
            ORDER BY day_of_week
            "#,
        )
        .bind(walker_id.as_uuid())
        .fetch_all(pool)
        .await
    }

    pub async fn find_all(pool: &PgPool) -> Result<Vec<WorkingHours>, sqlx::Error> {
        sqlx::query_as::<_, WorkingHours>(
            r#"
            SELECT id, walker_id, day_of_week, start_time, end_time, is_active, created_at, updated_at
            FROM working_hours
            ORDER BY walker_id, day_of_week
            "#,
        )
        .fetch_all(pool)
        .await
    }

    pub async fn update(
        pool: &PgPool,
        id: WorkingHoursId,
        input: UpdateWorkingHours,
    ) -> Result<Option<WorkingHours>, sqlx::Error> {
        sqlx::query_as::<_, WorkingHours>(
            r#"
            UPDATE working_hours
            SET
                start_time = COALESCE($2, start_time),
                end_time = COALESCE($3, end_time),
                is_active = COALESCE($4, is_active),
                updated_at = NOW()
            WHERE id = $1
            RETURNING id, walker_id, day_of_week, start_time, end_time, is_active, created_at, updated_at
            "#,
        )
        .bind(id.as_uuid())
        .bind(input.start_time)
        .bind(input.end_time)
        .bind(input.is_active)
        .fetch_optional(pool)
        .await
    }

    pub async fn delete(pool: &PgPool, id: WorkingHoursId) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM working_hours WHERE id = $1")
            .bind(id.as_uuid())
            .execute(pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn delete_by_walker(pool: &PgPool, walker_id: UserId) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM working_hours WHERE walker_id = $1")
            .bind(walker_id.as_uuid())
            .execute(pool)
            .await?;

        Ok(result.rows_affected())
    }

    /// Upsert working hours for a specific day
    pub async fn upsert(
        pool: &PgPool,
        walker_id: UserId,
        day_of_week: i16,
        start_time: NaiveTime,
        end_time: NaiveTime,
        is_active: bool,
    ) -> Result<WorkingHours, sqlx::Error> {
        let id = WorkingHoursId::new();

        sqlx::query_as::<_, WorkingHours>(
            r#"
            INSERT INTO working_hours (id, walker_id, day_of_week, start_time, end_time, is_active)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (walker_id, day_of_week)
            DO UPDATE SET
                start_time = EXCLUDED.start_time,
                end_time = EXCLUDED.end_time,
                is_active = EXCLUDED.is_active,
                updated_at = NOW()
            RETURNING id, walker_id, day_of_week, start_time, end_time, is_active, created_at, updated_at
            "#,
        )
        .bind(id.as_uuid())
        .bind(walker_id.as_uuid())
        .bind(day_of_week)
        .bind(start_time)
        .bind(end_time)
        .bind(is_active)
        .fetch_one(pool)
        .await
    }
}
