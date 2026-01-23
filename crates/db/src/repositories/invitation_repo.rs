use shared::types::{OrganizationId, UserId};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{CreateInvitation, Invitation, InvitationType};

pub struct InvitationRepository;

impl InvitationRepository {
    /// Create a new invitation
    pub async fn create(pool: &PgPool, invitation: CreateInvitation) -> Result<Invitation, sqlx::Error> {
        sqlx::query_as::<_, Invitation>(
            r#"
            INSERT INTO invitations (
                organization_id, invited_by, invitation_type, email, phone,
                token, token_hash, expires_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
        )
        .bind(invitation.organization_id)
        .bind(invitation.invited_by)
        .bind(invitation.invitation_type)
        .bind(invitation.email)
        .bind(invitation.phone)
        .bind(invitation.token)
        .bind(invitation.token_hash)
        .bind(invitation.expires_at)
        .fetch_one(pool)
        .await
    }

    /// Find an invitation by token (for accepting)
    pub async fn find_by_token(pool: &PgPool, token: &str) -> Result<Option<Invitation>, sqlx::Error> {
        sqlx::query_as::<_, Invitation>(
            r#"
            SELECT *
            FROM invitations
            WHERE token = $1
            "#,
        )
        .bind(token)
        .fetch_optional(pool)
        .await
    }

    /// Find a valid (pending and not expired) invitation by token
    pub async fn find_valid_by_token(pool: &PgPool, token: &str) -> Result<Option<Invitation>, sqlx::Error> {
        sqlx::query_as::<_, Invitation>(
            r#"
            SELECT *
            FROM invitations
            WHERE token = $1
              AND status = 'pending'
              AND expires_at > NOW()
            "#,
        )
        .bind(token)
        .fetch_optional(pool)
        .await
    }

    /// Find invitation by ID
    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Invitation>, sqlx::Error> {
        sqlx::query_as::<_, Invitation>(
            r#"
            SELECT *
            FROM invitations
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    /// List all invitations for an organization
    pub async fn list_by_organization(
        pool: &PgPool,
        organization_id: OrganizationId,
    ) -> Result<Vec<Invitation>, sqlx::Error> {
        sqlx::query_as::<_, Invitation>(
            r#"
            SELECT *
            FROM invitations
            WHERE organization_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(organization_id)
        .fetch_all(pool)
        .await
    }

    /// List pending invitations for an organization
    pub async fn list_pending_by_organization(
        pool: &PgPool,
        organization_id: OrganizationId,
        invitation_type: Option<InvitationType>,
    ) -> Result<Vec<Invitation>, sqlx::Error> {
        if let Some(inv_type) = invitation_type {
            sqlx::query_as::<_, Invitation>(
                r#"
                SELECT *
                FROM invitations
                WHERE organization_id = $1
                  AND status = 'pending'
                  AND expires_at > NOW()
                  AND invitation_type = $2
                ORDER BY created_at DESC
                "#,
            )
            .bind(organization_id)
            .bind(inv_type)
            .fetch_all(pool)
            .await
        } else {
            sqlx::query_as::<_, Invitation>(
                r#"
                SELECT *
                FROM invitations
                WHERE organization_id = $1
                  AND status = 'pending'
                  AND expires_at > NOW()
                ORDER BY created_at DESC
                "#,
            )
            .bind(organization_id)
            .fetch_all(pool)
            .await
        }
    }

    /// Check if a pending invitation already exists for email/phone in organization
    pub async fn find_existing_pending(
        pool: &PgPool,
        organization_id: OrganizationId,
        email: Option<&str>,
        phone: Option<&str>,
    ) -> Result<Option<Invitation>, sqlx::Error> {
        if let Some(email) = email {
            sqlx::query_as::<_, Invitation>(
                r#"
                SELECT *
                FROM invitations
                WHERE organization_id = $1
                  AND email = $2
                  AND status = 'pending'
                  AND expires_at > NOW()
                LIMIT 1
                "#,
            )
            .bind(organization_id)
            .bind(email)
            .fetch_optional(pool)
            .await
        } else if let Some(phone) = phone {
            sqlx::query_as::<_, Invitation>(
                r#"
                SELECT *
                FROM invitations
                WHERE organization_id = $1
                  AND phone = $2
                  AND status = 'pending'
                  AND expires_at > NOW()
                LIMIT 1
                "#,
            )
            .bind(organization_id)
            .bind(phone)
            .fetch_optional(pool)
            .await
        } else {
            Ok(None)
        }
    }

    /// Mark invitation as accepted
    pub async fn accept(
        pool: &PgPool,
        invitation_id: Uuid,
        accepted_by: UserId,
    ) -> Result<Invitation, sqlx::Error> {
        sqlx::query_as::<_, Invitation>(
            r#"
            UPDATE invitations
            SET status = 'accepted',
                accepted_at = NOW(),
                accepted_by = $2,
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(invitation_id)
        .bind(accepted_by)
        .fetch_one(pool)
        .await
    }

    /// Revoke an invitation
    pub async fn revoke(pool: &PgPool, invitation_id: Uuid) -> Result<Invitation, sqlx::Error> {
        sqlx::query_as::<_, Invitation>(
            r#"
            UPDATE invitations
            SET status = 'revoked',
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(invitation_id)
        .fetch_one(pool)
        .await
    }

    /// Mark expired invitations as expired
    pub async fn expire_old(pool: &PgPool) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            r#"
            UPDATE invitations
            SET status = 'expired',
                updated_at = NOW()
            WHERE status = 'pending'
              AND expires_at < NOW()
            "#,
        )
        .execute(pool)
        .await?;
        Ok(result.rows_affected())
    }

    /// Count invitations sent in last hour for rate limiting
    pub async fn count_recent_by_inviter(
        pool: &PgPool,
        invited_by: UserId,
    ) -> Result<i64, sqlx::Error> {
        let result: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*)::bigint
            FROM invitations
            WHERE invited_by = $1
              AND created_at > NOW() - INTERVAL '1 hour'
            "#,
        )
        .bind(invited_by)
        .fetch_one(pool)
        .await?;
        Ok(result.0)
    }

    /// Delete an invitation
    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM invitations WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }
}
