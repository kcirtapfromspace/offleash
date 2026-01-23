use shared::types::{OrganizationId, UserId};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{
    CreateMembership, Membership, MembershipRole, MembershipStatus, MembershipWithOrg,
    UpdateMembership,
};

pub struct MembershipRepository;

impl MembershipRepository {
    /// Create a new membership
    pub async fn create(pool: &PgPool, input: CreateMembership) -> Result<Membership, sqlx::Error> {
        let status = input.status.unwrap_or(MembershipStatus::Active);

        sqlx::query_as::<_, Membership>(
            r#"
            INSERT INTO memberships (user_id, organization_id, role, status, title)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
        )
        .bind(input.user_id)
        .bind(input.organization_id)
        .bind(input.role)
        .bind(status)
        .bind(input.title)
        .fetch_one(pool)
        .await
    }

    /// Find membership by ID
    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Membership>, sqlx::Error> {
        sqlx::query_as::<_, Membership>(
            r#"
            SELECT *
            FROM memberships
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    /// Find all memberships for a user
    pub async fn find_by_user(
        pool: &PgPool,
        user_id: UserId,
    ) -> Result<Vec<Membership>, sqlx::Error> {
        sqlx::query_as::<_, Membership>(
            r#"
            SELECT *
            FROM memberships
            WHERE user_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
    }

    /// Find active memberships for a user (most common query)
    pub async fn find_active_by_user(
        pool: &PgPool,
        user_id: UserId,
    ) -> Result<Vec<Membership>, sqlx::Error> {
        sqlx::query_as::<_, Membership>(
            r#"
            SELECT *
            FROM memberships
            WHERE user_id = $1 AND status = 'active'
            ORDER BY joined_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
    }

    /// Find memberships with organization details (for UI display)
    pub async fn find_with_org_by_user(
        pool: &PgPool,
        user_id: UserId,
    ) -> Result<Vec<MembershipWithOrg>, sqlx::Error> {
        sqlx::query_as::<_, MembershipWithOrg>(
            r#"
            SELECT
                m.id,
                m.user_id,
                m.organization_id,
                o.name as organization_name,
                o.slug as organization_slug,
                m.role,
                m.status,
                m.title,
                m.joined_at
            FROM memberships m
            JOIN organizations o ON o.id = m.organization_id
            WHERE m.user_id = $1 AND m.status = 'active'
            ORDER BY m.joined_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
    }

    /// Find a specific membership for user in organization
    pub async fn find_by_user_and_org(
        pool: &PgPool,
        user_id: UserId,
        organization_id: OrganizationId,
    ) -> Result<Vec<Membership>, sqlx::Error> {
        sqlx::query_as::<_, Membership>(
            r#"
            SELECT *
            FROM memberships
            WHERE user_id = $1 AND organization_id = $2
            ORDER BY role ASC
            "#,
        )
        .bind(user_id)
        .bind(organization_id)
        .fetch_all(pool)
        .await
    }

    /// Find a specific membership for user in organization with specific role
    pub async fn find_by_user_org_role(
        pool: &PgPool,
        user_id: UserId,
        organization_id: OrganizationId,
        role: MembershipRole,
    ) -> Result<Option<Membership>, sqlx::Error> {
        sqlx::query_as::<_, Membership>(
            r#"
            SELECT *
            FROM memberships
            WHERE user_id = $1 AND organization_id = $2 AND role = $3
            "#,
        )
        .bind(user_id)
        .bind(organization_id)
        .bind(role)
        .fetch_optional(pool)
        .await
    }

    /// Check if user has any active membership in organization
    pub async fn has_active_membership(
        pool: &PgPool,
        user_id: UserId,
        organization_id: OrganizationId,
    ) -> Result<bool, sqlx::Error> {
        let result: (bool,) = sqlx::query_as(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM memberships
                WHERE user_id = $1 AND organization_id = $2 AND status = 'active'
            )
            "#,
        )
        .bind(user_id)
        .bind(organization_id)
        .fetch_one(pool)
        .await?;
        Ok(result.0)
    }

    /// Get the highest role a user has in an organization
    pub async fn get_highest_role(
        pool: &PgPool,
        user_id: UserId,
        organization_id: OrganizationId,
    ) -> Result<Option<MembershipRole>, sqlx::Error> {
        // Role priority: owner > admin > walker > customer
        let result: Option<(MembershipRole,)> = sqlx::query_as(
            r#"
            SELECT role
            FROM memberships
            WHERE user_id = $1 AND organization_id = $2 AND status = 'active'
            ORDER BY
                CASE role
                    WHEN 'owner' THEN 1
                    WHEN 'admin' THEN 2
                    WHEN 'walker' THEN 3
                    WHEN 'customer' THEN 4
                END
            LIMIT 1
            "#,
        )
        .bind(user_id)
        .bind(organization_id)
        .fetch_optional(pool)
        .await?;
        Ok(result.map(|r| r.0))
    }

    /// Find all members of an organization
    pub async fn find_by_organization(
        pool: &PgPool,
        organization_id: OrganizationId,
    ) -> Result<Vec<Membership>, sqlx::Error> {
        sqlx::query_as::<_, Membership>(
            r#"
            SELECT *
            FROM memberships
            WHERE organization_id = $1
            ORDER BY role ASC, joined_at ASC
            "#,
        )
        .bind(organization_id)
        .fetch_all(pool)
        .await
    }

    /// Find all active members with a specific role in organization
    pub async fn find_by_organization_and_role(
        pool: &PgPool,
        organization_id: OrganizationId,
        role: MembershipRole,
    ) -> Result<Vec<Membership>, sqlx::Error> {
        sqlx::query_as::<_, Membership>(
            r#"
            SELECT *
            FROM memberships
            WHERE organization_id = $1 AND role = $2 AND status = 'active'
            ORDER BY joined_at ASC
            "#,
        )
        .bind(organization_id)
        .bind(role)
        .fetch_all(pool)
        .await
    }

    /// Update a membership
    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        input: UpdateMembership,
    ) -> Result<Option<Membership>, sqlx::Error> {
        sqlx::query_as::<_, Membership>(
            r#"
            UPDATE memberships
            SET
                role = COALESCE($2, role),
                status = COALESCE($3, status),
                title = COALESCE($4, title),
                permissions = COALESCE($5, permissions),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(input.role)
        .bind(input.status)
        .bind(input.title)
        .bind(input.permissions)
        .fetch_optional(pool)
        .await
    }

    /// Update membership status
    pub async fn update_status(
        pool: &PgPool,
        id: Uuid,
        status: MembershipStatus,
    ) -> Result<Option<Membership>, sqlx::Error> {
        sqlx::query_as::<_, Membership>(
            r#"
            UPDATE memberships
            SET status = $2, updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(status)
        .fetch_one(pool)
        .await
        .map(Some)
    }

    /// Delete a membership
    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM memberships WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    /// Count members in organization by role
    pub async fn count_by_organization_role(
        pool: &PgPool,
        organization_id: OrganizationId,
        role: MembershipRole,
    ) -> Result<i64, sqlx::Error> {
        let result: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*)::bigint
            FROM memberships
            WHERE organization_id = $1 AND role = $2 AND status = 'active'
            "#,
        )
        .bind(organization_id)
        .bind(role)
        .fetch_one(pool)
        .await?;
        Ok(result.0)
    }

    /// Check if organization has any owners (for preventing last owner deletion)
    pub async fn has_other_owners(
        pool: &PgPool,
        organization_id: OrganizationId,
        exclude_user_id: UserId,
    ) -> Result<bool, sqlx::Error> {
        let result: (bool,) = sqlx::query_as(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM memberships
                WHERE organization_id = $1
                  AND user_id != $2
                  AND role = 'owner'
                  AND status = 'active'
            )
            "#,
        )
        .bind(organization_id)
        .bind(exclude_user_id)
        .fetch_one(pool)
        .await?;
        Ok(result.0)
    }
}
