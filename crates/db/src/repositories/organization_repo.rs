use shared::types::OrganizationId;
use sqlx::PgPool;

use crate::models::{CreateOrganization, Organization, UpdateOrganization};

pub struct OrganizationRepository;

impl OrganizationRepository {
    pub async fn create(
        pool: &PgPool,
        input: CreateOrganization,
    ) -> Result<Organization, sqlx::Error> {
        let id = OrganizationId::new();
        let settings = input.settings.unwrap_or_default();
        let subdomain = input.subdomain.unwrap_or_else(|| input.slug.clone());

        sqlx::query_as::<_, Organization>(
            r#"
            INSERT INTO organizations (id, name, slug, subdomain, custom_domain, settings)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, name, slug, subdomain, custom_domain, settings, created_at, updated_at
            "#,
        )
        .bind(id.as_uuid())
        .bind(&input.name)
        .bind(&input.slug)
        .bind(&subdomain)
        .bind(&input.custom_domain)
        .bind(sqlx::types::Json(&settings))
        .fetch_one(pool)
        .await
    }

    pub async fn find_by_id(
        pool: &PgPool,
        id: OrganizationId,
    ) -> Result<Option<Organization>, sqlx::Error> {
        sqlx::query_as::<_, Organization>(
            r#"
            SELECT id, name, slug, subdomain, custom_domain, settings, created_at, updated_at
            FROM organizations
            WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
        .fetch_optional(pool)
        .await
    }

    pub async fn find_by_slug(
        pool: &PgPool,
        slug: &str,
    ) -> Result<Option<Organization>, sqlx::Error> {
        sqlx::query_as::<_, Organization>(
            r#"
            SELECT id, name, slug, subdomain, custom_domain, settings, created_at, updated_at
            FROM organizations
            WHERE slug = $1
            "#,
        )
        .bind(slug)
        .fetch_optional(pool)
        .await
    }

    pub async fn update(
        pool: &PgPool,
        id: OrganizationId,
        input: UpdateOrganization,
    ) -> Result<Option<Organization>, sqlx::Error> {
        // For settings, we need to handle the JSON update specially
        // If settings is provided, we replace the entire settings object
        let settings_json = input.settings.map(sqlx::types::Json);

        sqlx::query_as::<_, Organization>(
            r#"
            UPDATE organizations
            SET
                name = COALESCE($2, name),
                slug = COALESCE($3, slug),
                custom_domain = COALESCE($4, custom_domain),
                settings = COALESCE($5, settings),
                updated_at = NOW()
            WHERE id = $1
            RETURNING id, name, slug, subdomain, custom_domain, settings, created_at, updated_at
            "#,
        )
        .bind(id.as_uuid())
        .bind(&input.name)
        .bind(&input.slug)
        .bind(&input.custom_domain)
        .bind(settings_json)
        .fetch_optional(pool)
        .await
    }

    pub async fn list(
        pool: &PgPool,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Organization>, sqlx::Error> {
        sqlx::query_as::<_, Organization>(
            r#"
            SELECT id, name, slug, subdomain, custom_domain, settings, created_at, updated_at
            FROM organizations
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
    }

    /// Count total number of organizations
    pub async fn count(pool: &PgPool) -> Result<i64, sqlx::Error> {
        let result: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) as count
            FROM organizations
            "#,
        )
        .fetch_one(pool)
        .await?;
        Ok(result.0)
    }

    /// Find organization by subdomain
    pub async fn find_by_subdomain(
        pool: &PgPool,
        subdomain: &str,
    ) -> Result<Option<Organization>, sqlx::Error> {
        sqlx::query_as::<_, Organization>(
            r#"
            SELECT id, name, slug, subdomain, custom_domain, settings, created_at, updated_at
            FROM organizations
            WHERE subdomain = $1
            "#,
        )
        .bind(subdomain)
        .fetch_optional(pool)
        .await
    }

    /// Find organization by custom domain
    pub async fn find_by_custom_domain(
        pool: &PgPool,
        domain: &str,
    ) -> Result<Option<Organization>, sqlx::Error> {
        sqlx::query_as::<_, Organization>(
            r#"
            SELECT id, name, slug, subdomain, custom_domain, settings, created_at, updated_at
            FROM organizations
            WHERE custom_domain = $1
            "#,
        )
        .bind(domain)
        .fetch_optional(pool)
        .await
    }
}
