use argon2::{
    password_hash::{PasswordHash, PasswordVerifier},
    Argon2,
};
use shared::types::PlatformAdminId;
use sqlx::PgPool;

use crate::models::{CreatePlatformAdmin, PlatformAdmin};

pub struct PlatformAdminRepository;

impl PlatformAdminRepository {
    pub async fn create(
        pool: &PgPool,
        input: CreatePlatformAdmin,
    ) -> Result<PlatformAdmin, sqlx::Error> {
        let id = PlatformAdminId::new();

        sqlx::query_as::<_, PlatformAdmin>(
            r#"
            INSERT INTO platform_admins (id, email, password_hash, first_name, last_name)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, email, password_hash, first_name, last_name, created_at, updated_at
            "#,
        )
        .bind(id.as_uuid())
        .bind(&input.email)
        .bind(&input.password_hash)
        .bind(&input.first_name)
        .bind(&input.last_name)
        .fetch_one(pool)
        .await
    }

    pub async fn find_by_email(
        pool: &PgPool,
        email: &str,
    ) -> Result<Option<PlatformAdmin>, sqlx::Error> {
        sqlx::query_as::<_, PlatformAdmin>(
            r#"
            SELECT id, email, password_hash, first_name, last_name, created_at, updated_at
            FROM platform_admins
            WHERE email = $1
            "#,
        )
        .bind(email)
        .fetch_optional(pool)
        .await
    }

    pub fn verify_password(admin: &PlatformAdmin, password: &str) -> bool {
        let Ok(parsed_hash) = PasswordHash::new(&admin.password_hash) else {
            return false;
        };

        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok()
    }
}
