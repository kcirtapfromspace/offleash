use chrono::{DateTime, Utc};
use shared::types::{BlockId, UserId};
use sqlx::PgPool;

use crate::models::{Block, CreateBlock};

pub struct BlockRepository;

impl BlockRepository {
    pub async fn create(pool: &PgPool, input: CreateBlock) -> Result<Block, sqlx::Error> {
        let id = BlockId::new();

        sqlx::query_as::<_, Block>(
            r#"
            INSERT INTO blocks (id, walker_id, reason, start_time, end_time, is_recurring, recurrence_rule)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, walker_id, reason, start_time, end_time, is_recurring, recurrence_rule, created_at, updated_at
            "#,
        )
        .bind(id.as_uuid())
        .bind(input.walker_id.as_uuid())
        .bind(&input.reason)
        .bind(input.start_time)
        .bind(input.end_time)
        .bind(input.is_recurring)
        .bind(&input.recurrence_rule)
        .fetch_one(pool)
        .await
    }

    pub async fn find_by_id(pool: &PgPool, id: BlockId) -> Result<Option<Block>, sqlx::Error> {
        sqlx::query_as::<_, Block>(
            r#"
            SELECT id, walker_id, reason, start_time, end_time, is_recurring, recurrence_rule, created_at, updated_at
            FROM blocks
            WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
        .fetch_optional(pool)
        .await
    }

    pub async fn find_by_walker_in_range(
        pool: &PgPool,
        walker_id: UserId,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<Block>, sqlx::Error> {
        sqlx::query_as::<_, Block>(
            r#"
            SELECT id, walker_id, reason, start_time, end_time, is_recurring, recurrence_rule, created_at, updated_at
            FROM blocks
            WHERE walker_id = $1
              AND start_time < $3
              AND end_time > $2
            ORDER BY start_time
            "#,
        )
        .bind(walker_id.as_uuid())
        .bind(start)
        .bind(end)
        .fetch_all(pool)
        .await
    }

    pub async fn delete(pool: &PgPool, id: BlockId) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM blocks WHERE id = $1")
            .bind(id.as_uuid())
            .execute(pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }
}
