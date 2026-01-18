use shared::types::{OrganizationId, TenantDatabaseId};
use sqlx::PgPool;

use crate::models::{CreateTenantDatabase, TenantDatabase, TenantDbStatus};

pub struct TenantDatabaseRepository;

impl TenantDatabaseRepository {
    pub async fn create(
        pool: &PgPool,
        input: CreateTenantDatabase,
    ) -> Result<TenantDatabase, sqlx::Error> {
        let id = TenantDatabaseId::new();
        let status = input.status.unwrap_or(TenantDbStatus::Provisioning);

        sqlx::query_as::<_, TenantDatabase>(
            r#"
            INSERT INTO tenant_databases (id, organization_id, connection_string, status)
            VALUES ($1, $2, $3, $4)
            RETURNING id, organization_id, connection_string, status, created_at, updated_at
            "#,
        )
        .bind(id.as_uuid())
        .bind(input.organization_id.as_uuid())
        .bind(&input.connection_string)
        .bind(status)
        .fetch_one(pool)
        .await
    }

    pub async fn find_by_org_id(
        pool: &PgPool,
        organization_id: OrganizationId,
    ) -> Result<Option<TenantDatabase>, sqlx::Error> {
        sqlx::query_as::<_, TenantDatabase>(
            r#"
            SELECT id, organization_id, connection_string, status, created_at, updated_at
            FROM tenant_databases
            WHERE organization_id = $1
            "#,
        )
        .bind(organization_id.as_uuid())
        .fetch_optional(pool)
        .await
    }

    pub async fn update_status(
        pool: &PgPool,
        id: TenantDatabaseId,
        status: TenantDbStatus,
    ) -> Result<Option<TenantDatabase>, sqlx::Error> {
        sqlx::query_as::<_, TenantDatabase>(
            r#"
            UPDATE tenant_databases
            SET status = $2, updated_at = NOW()
            WHERE id = $1
            RETURNING id, organization_id, connection_string, status, created_at, updated_at
            "#,
        )
        .bind(id.as_uuid())
        .bind(status)
        .fetch_optional(pool)
        .await
    }
}
