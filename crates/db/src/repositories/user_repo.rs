use shared::types::{OrganizationId, UserId};
use sqlx::PgPool;

use crate::models::{CreateUser, UpdateUser, User, UserRole};

pub struct UserRepository;

impl UserRepository {
    pub async fn create(pool: &PgPool, input: CreateUser) -> Result<User, sqlx::Error> {
        let id = UserId::new();
        let timezone = input
            .timezone
            .unwrap_or_else(|| "America/Denver".to_string());

        sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (id, organization_id, email, password_hash, role, first_name, last_name, phone, timezone)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id, organization_id, email, password_hash, role, first_name, last_name, phone, timezone, created_at, updated_at
            "#,
        )
        .bind(id.as_uuid())
        .bind(input.organization_id.as_uuid())
        .bind(&input.email)
        .bind(&input.password_hash)
        .bind(input.role)
        .bind(&input.first_name)
        .bind(&input.last_name)
        .bind(&input.phone)
        .bind(&timezone)
        .fetch_one(pool)
        .await
    }

    pub async fn find_by_id(
        pool: &PgPool,
        org_id: OrganizationId,
        id: UserId,
    ) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            r#"
            SELECT id, organization_id, email, password_hash, role, first_name, last_name, phone, timezone, created_at, updated_at
            FROM users
            WHERE id = $1 AND organization_id = $2
            "#,
        )
        .bind(id.as_uuid())
        .bind(org_id.as_uuid())
        .fetch_optional(pool)
        .await
    }

    /// Find user by ID only (no org check) - for OAuth identity lookups
    pub async fn find_by_id_unchecked(
        pool: &PgPool,
        id: UserId,
    ) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            r#"
            SELECT id, organization_id, email, password_hash, role, first_name, last_name, phone, timezone, created_at, updated_at
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
        .fetch_optional(pool)
        .await
    }

    pub async fn find_by_email(
        pool: &PgPool,
        org_id: OrganizationId,
        email: &str,
    ) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            r#"
            SELECT id, organization_id, email, password_hash, role, first_name, last_name, phone, timezone, created_at, updated_at
            FROM users
            WHERE email = $1 AND organization_id = $2
            "#,
        )
        .bind(email)
        .bind(org_id.as_uuid())
        .fetch_optional(pool)
        .await
    }

    /// Find user by email globally (across all organizations)
    /// Used for universal login when user doesn't specify an organization
    pub async fn find_by_email_globally(
        pool: &PgPool,
        email: &str,
    ) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            r#"
            SELECT id, organization_id, email, password_hash, role, first_name, last_name, phone, timezone, created_at, updated_at
            FROM users
            WHERE LOWER(email) = LOWER($1)
            LIMIT 1
            "#,
        )
        .bind(email)
        .fetch_optional(pool)
        .await
    }

    pub async fn update(
        pool: &PgPool,
        org_id: OrganizationId,
        id: UserId,
        input: UpdateUser,
    ) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            r#"
            UPDATE users
            SET
                first_name = COALESCE($3, first_name),
                last_name = COALESCE($4, last_name),
                phone = COALESCE($5, phone),
                timezone = COALESCE($6, timezone),
                updated_at = NOW()
            WHERE id = $1 AND organization_id = $2
            RETURNING id, organization_id, email, password_hash, role, first_name, last_name, phone, timezone, created_at, updated_at
            "#,
        )
        .bind(id.as_uuid())
        .bind(org_id.as_uuid())
        .bind(&input.first_name)
        .bind(&input.last_name)
        .bind(&input.phone)
        .bind(&input.timezone)
        .fetch_optional(pool)
        .await
    }

    pub async fn delete(
        pool: &PgPool,
        org_id: OrganizationId,
        id: UserId,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM users WHERE id = $1 AND organization_id = $2")
            .bind(id.as_uuid())
            .bind(org_id.as_uuid())
            .execute(pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn list_all(
        pool: &PgPool,
        org_id: OrganizationId,
    ) -> Result<Vec<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            r#"
            SELECT id, organization_id, email, password_hash, role, first_name, last_name, phone, timezone, created_at, updated_at
            FROM users
            WHERE organization_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(org_id.as_uuid())
        .fetch_all(pool)
        .await
    }

    pub async fn list_by_role(
        pool: &PgPool,
        org_id: OrganizationId,
        role: UserRole,
    ) -> Result<Vec<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            r#"
            SELECT id, organization_id, email, password_hash, role, first_name, last_name, phone, timezone, created_at, updated_at
            FROM users
            WHERE organization_id = $1 AND role = $2
            ORDER BY first_name, last_name
            "#,
        )
        .bind(org_id.as_uuid())
        .bind(role)
        .fetch_all(pool)
        .await
    }
}
