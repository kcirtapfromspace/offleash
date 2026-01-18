use shared::types::UserId;
use sqlx::PgPool;

use crate::models::{CreateUser, UpdateUser, User};

pub struct UserRepository;

impl UserRepository {
    pub async fn create(pool: &PgPool, input: CreateUser) -> Result<User, sqlx::Error> {
        let id = UserId::new();
        let timezone = input.timezone.unwrap_or_else(|| "America/Denver".to_string());

        sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (id, email, password_hash, role, first_name, last_name, phone, timezone)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, email, password_hash, role, first_name, last_name, phone, timezone, created_at, updated_at
            "#,
        )
        .bind(id.as_uuid())
        .bind(&input.email)
        .bind(&input.password_hash)
        .bind(&input.role)
        .bind(&input.first_name)
        .bind(&input.last_name)
        .bind(&input.phone)
        .bind(&timezone)
        .fetch_one(pool)
        .await
    }

    pub async fn find_by_id(pool: &PgPool, id: UserId) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            r#"
            SELECT id, email, password_hash, role, first_name, last_name, phone, timezone, created_at, updated_at
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
        .fetch_optional(pool)
        .await
    }

    pub async fn find_by_email(pool: &PgPool, email: &str) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            r#"
            SELECT id, email, password_hash, role, first_name, last_name, phone, timezone, created_at, updated_at
            FROM users
            WHERE email = $1
            "#,
        )
        .bind(email)
        .fetch_optional(pool)
        .await
    }

    pub async fn update(
        pool: &PgPool,
        id: UserId,
        input: UpdateUser,
    ) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            r#"
            UPDATE users
            SET
                first_name = COALESCE($2, first_name),
                last_name = COALESCE($3, last_name),
                phone = COALESCE($4, phone),
                timezone = COALESCE($5, timezone),
                updated_at = NOW()
            WHERE id = $1
            RETURNING id, email, password_hash, role, first_name, last_name, phone, timezone, created_at, updated_at
            "#,
        )
        .bind(id.as_uuid())
        .bind(&input.first_name)
        .bind(&input.last_name)
        .bind(&input.phone)
        .bind(&input.timezone)
        .fetch_optional(pool)
        .await
    }

    pub async fn delete(pool: &PgPool, id: UserId) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(id.as_uuid())
            .execute(pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }
}
