//! Calendar repository for calendar events, connections, and sync operations

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{
    CalendarConnection, CalendarEvent, CalendarEventType, CalendarEventWithDetails,
    CalendarSyncLog, CompleteSyncLog, CreateCalendarConnection, CreateCalendarEvent,
    CreateSyncLog, UpdateCalendarEvent,
};
use shared::types::{OrganizationId, UserId};

pub struct CalendarRepository;

impl CalendarRepository {
    // ============ Calendar Events ============

    /// Create a new calendar event
    pub async fn create_event(
        pool: &PgPool,
        input: CreateCalendarEvent,
    ) -> Result<CalendarEvent, sqlx::Error> {
        sqlx::query_as::<_, CalendarEvent>(
            r#"
            INSERT INTO calendar_events (
                organization_id, user_id, title, description, start_time, end_time,
                all_day, event_type, calendar_connection_id, external_event_id,
                recurrence_rule, recurrence_parent_id, color, is_blocking
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            RETURNING *
            "#,
        )
        .bind(input.organization_id.as_uuid())
        .bind(input.user_id.as_uuid())
        .bind(&input.title)
        .bind(&input.description)
        .bind(input.start_time)
        .bind(input.end_time)
        .bind(input.all_day)
        .bind(input.event_type)
        .bind(input.calendar_connection_id)
        .bind(&input.external_event_id)
        .bind(&input.recurrence_rule)
        .bind(input.recurrence_parent_id)
        .bind(&input.color)
        .bind(input.is_blocking)
        .fetch_one(pool)
        .await
    }

    /// Find event by ID
    pub async fn find_event_by_id(
        pool: &PgPool,
        org_id: OrganizationId,
        id: Uuid,
    ) -> Result<Option<CalendarEvent>, sqlx::Error> {
        sqlx::query_as::<_, CalendarEvent>(
            r#"
            SELECT * FROM calendar_events
            WHERE id = $1 AND organization_id = $2
            "#,
        )
        .bind(id)
        .bind(org_id.as_uuid())
        .fetch_optional(pool)
        .await
    }

    /// Find events for a user in a date range
    pub async fn find_events_in_range(
        pool: &PgPool,
        org_id: OrganizationId,
        user_id: UserId,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<CalendarEvent>, sqlx::Error> {
        sqlx::query_as::<_, CalendarEvent>(
            r#"
            SELECT * FROM calendar_events
            WHERE organization_id = $1
              AND user_id = $2
              AND start_time < $4
              AND end_time > $3
            ORDER BY start_time
            "#,
        )
        .bind(org_id.as_uuid())
        .bind(user_id.as_uuid())
        .bind(start)
        .bind(end)
        .fetch_all(pool)
        .await
    }

    /// Find events with connection details (for display)
    pub async fn find_events_with_details(
        pool: &PgPool,
        org_id: OrganizationId,
        user_id: UserId,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<CalendarEventWithDetails>, sqlx::Error> {
        sqlx::query_as::<_, CalendarEventWithDetails>(
            r#"
            SELECT
                ce.*,
                cc.calendar_name as connection_name,
                cc.provider as provider
            FROM calendar_events ce
            LEFT JOIN calendar_connections cc ON ce.calendar_connection_id = cc.id
            WHERE ce.organization_id = $1
              AND ce.user_id = $2
              AND ce.start_time < $4
              AND ce.end_time > $3
            ORDER BY ce.start_time
            "#,
        )
        .bind(org_id.as_uuid())
        .bind(user_id.as_uuid())
        .bind(start)
        .bind(end)
        .fetch_all(pool)
        .await
    }

    /// Find blocking events for availability checking
    pub async fn find_blocking_events_in_range(
        pool: &PgPool,
        org_id: OrganizationId,
        user_id: UserId,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<CalendarEvent>, sqlx::Error> {
        sqlx::query_as::<_, CalendarEvent>(
            r#"
            SELECT * FROM calendar_events
            WHERE organization_id = $1
              AND user_id = $2
              AND is_blocking = true
              AND start_time < $4
              AND end_time > $3
            ORDER BY start_time
            "#,
        )
        .bind(org_id.as_uuid())
        .bind(user_id.as_uuid())
        .bind(start)
        .bind(end)
        .fetch_all(pool)
        .await
    }

    /// Find events by type
    pub async fn find_events_by_type(
        pool: &PgPool,
        org_id: OrganizationId,
        user_id: UserId,
        event_type: CalendarEventType,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<CalendarEvent>, sqlx::Error> {
        sqlx::query_as::<_, CalendarEvent>(
            r#"
            SELECT * FROM calendar_events
            WHERE organization_id = $1
              AND user_id = $2
              AND event_type = $3
              AND start_time < $5
              AND end_time > $4
            ORDER BY start_time
            "#,
        )
        .bind(org_id.as_uuid())
        .bind(user_id.as_uuid())
        .bind(event_type)
        .bind(start)
        .bind(end)
        .fetch_all(pool)
        .await
    }

    /// Find event by external ID (for sync)
    pub async fn find_event_by_external_id(
        pool: &PgPool,
        connection_id: Uuid,
        external_id: &str,
    ) -> Result<Option<CalendarEvent>, sqlx::Error> {
        sqlx::query_as::<_, CalendarEvent>(
            r#"
            SELECT * FROM calendar_events
            WHERE calendar_connection_id = $1 AND external_event_id = $2
            "#,
        )
        .bind(connection_id)
        .bind(external_id)
        .fetch_optional(pool)
        .await
    }

    /// Update a calendar event
    pub async fn update_event(
        pool: &PgPool,
        org_id: OrganizationId,
        id: Uuid,
        input: UpdateCalendarEvent,
    ) -> Result<Option<CalendarEvent>, sqlx::Error> {
        // Build dynamic update query based on what's provided
        let mut query = String::from("UPDATE calendar_events SET updated_at = NOW()");
        let mut param_count = 2; // Starting after id and org_id

        if input.title.is_some() {
            param_count += 1;
            query.push_str(&format!(", title = ${}", param_count));
        }
        if input.description.is_some() {
            param_count += 1;
            query.push_str(&format!(", description = ${}", param_count));
        }
        if input.start_time.is_some() {
            param_count += 1;
            query.push_str(&format!(", start_time = ${}", param_count));
        }
        if input.end_time.is_some() {
            param_count += 1;
            query.push_str(&format!(", end_time = ${}", param_count));
        }
        if input.all_day.is_some() {
            param_count += 1;
            query.push_str(&format!(", all_day = ${}", param_count));
        }
        if input.color.is_some() {
            param_count += 1;
            query.push_str(&format!(", color = ${}", param_count));
        }
        if input.is_blocking.is_some() {
            param_count += 1;
            query.push_str(&format!(", is_blocking = ${}", param_count));
        }
        if input.sync_status.is_some() {
            param_count += 1;
            query.push_str(&format!(", sync_status = ${}", param_count));
        }
        if input.last_synced_at.is_some() {
            param_count += 1;
            query.push_str(&format!(", last_synced_at = ${}", param_count));
        }

        query.push_str(" WHERE id = $1 AND organization_id = $2 RETURNING *");

        let mut q = sqlx::query_as::<_, CalendarEvent>(&query)
            .bind(id)
            .bind(org_id.as_uuid());

        if let Some(title) = input.title {
            q = q.bind(title);
        }
        if let Some(description) = input.description {
            q = q.bind(description);
        }
        if let Some(start_time) = input.start_time {
            q = q.bind(start_time);
        }
        if let Some(end_time) = input.end_time {
            q = q.bind(end_time);
        }
        if let Some(all_day) = input.all_day {
            q = q.bind(all_day);
        }
        if let Some(color) = input.color {
            q = q.bind(color);
        }
        if let Some(is_blocking) = input.is_blocking {
            q = q.bind(is_blocking);
        }
        if let Some(sync_status) = input.sync_status {
            q = q.bind(sync_status);
        }
        if let Some(last_synced_at) = input.last_synced_at {
            q = q.bind(last_synced_at);
        }

        q.fetch_optional(pool).await
    }

    /// Delete a calendar event
    pub async fn delete_event(
        pool: &PgPool,
        org_id: OrganizationId,
        id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result =
            sqlx::query("DELETE FROM calendar_events WHERE id = $1 AND organization_id = $2")
                .bind(id)
                .bind(org_id.as_uuid())
                .execute(pool)
                .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Delete all events for a connection (when disconnecting)
    pub async fn delete_events_by_connection(
        pool: &PgPool,
        connection_id: Uuid,
    ) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM calendar_events WHERE calendar_connection_id = $1")
            .bind(connection_id)
            .execute(pool)
            .await?;

        Ok(result.rows_affected())
    }

    // ============ Calendar Connections ============

    /// Create a new calendar connection
    pub async fn create_connection(
        pool: &PgPool,
        input: CreateCalendarConnection,
    ) -> Result<CalendarConnection, sqlx::Error> {
        sqlx::query_as::<_, CalendarConnection>(
            r#"
            INSERT INTO calendar_connections (
                user_id, provider, access_token, refresh_token, token_expires_at,
                server_url, username, password_encrypted, calendar_id, calendar_name,
                calendar_color, sync_direction, push_bookings, push_blocks
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            RETURNING *
            "#,
        )
        .bind(input.user_id.as_uuid())
        .bind(input.provider)
        .bind(&input.access_token)
        .bind(&input.refresh_token)
        .bind(input.token_expires_at)
        .bind(&input.server_url)
        .bind(&input.username)
        .bind(&input.password_encrypted)
        .bind(&input.calendar_id)
        .bind(&input.calendar_name)
        .bind(&input.calendar_color)
        .bind(input.sync_direction)
        .bind(input.push_bookings)
        .bind(input.push_blocks)
        .fetch_one(pool)
        .await
    }

    /// Find connection by ID
    pub async fn find_connection_by_id(
        pool: &PgPool,
        id: Uuid,
    ) -> Result<Option<CalendarConnection>, sqlx::Error> {
        sqlx::query_as::<_, CalendarConnection>("SELECT * FROM calendar_connections WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await
    }

    /// Find all connections for a user
    pub async fn find_connections_by_user(
        pool: &PgPool,
        user_id: UserId,
    ) -> Result<Vec<CalendarConnection>, sqlx::Error> {
        sqlx::query_as::<_, CalendarConnection>(
            r#"
            SELECT * FROM calendar_connections
            WHERE user_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id.as_uuid())
        .fetch_all(pool)
        .await
    }

    /// Find enabled connections that need sync
    pub async fn find_connections_needing_sync(
        pool: &PgPool,
        older_than: DateTime<Utc>,
    ) -> Result<Vec<CalendarConnection>, sqlx::Error> {
        sqlx::query_as::<_, CalendarConnection>(
            r#"
            SELECT * FROM calendar_connections
            WHERE sync_enabled = true
              AND (last_sync_at IS NULL OR last_sync_at < $1)
            ORDER BY last_sync_at NULLS FIRST
            "#,
        )
        .bind(older_than)
        .fetch_all(pool)
        .await
    }

    /// Find connections with expiring tokens (within 5 minutes)
    pub async fn find_connections_with_expiring_tokens(
        pool: &PgPool,
    ) -> Result<Vec<CalendarConnection>, sqlx::Error> {
        sqlx::query_as::<_, CalendarConnection>(
            r#"
            SELECT * FROM calendar_connections
            WHERE token_expires_at IS NOT NULL
              AND token_expires_at <= NOW() + INTERVAL '5 minutes'
              AND sync_enabled = true
            "#,
        )
        .fetch_all(pool)
        .await
    }

    /// Update connection tokens (after OAuth refresh)
    pub async fn update_connection_tokens(
        pool: &PgPool,
        id: Uuid,
        access_token: &str,
        refresh_token: Option<&str>,
        expires_at: DateTime<Utc>,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"
            UPDATE calendar_connections
            SET access_token = $2, refresh_token = COALESCE($3, refresh_token),
                token_expires_at = $4, updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(access_token)
        .bind(refresh_token)
        .bind(expires_at)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Update last sync time
    pub async fn update_connection_last_sync(
        pool: &PgPool,
        id: Uuid,
        sync_token: Option<&str>,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"
            UPDATE calendar_connections
            SET last_sync_at = NOW(), sync_token = COALESCE($2, sync_token), updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(sync_token)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Toggle sync enabled
    pub async fn update_connection_sync_enabled(
        pool: &PgPool,
        id: Uuid,
        user_id: UserId,
        enabled: bool,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"
            UPDATE calendar_connections
            SET sync_enabled = $3, updated_at = NOW()
            WHERE id = $1 AND user_id = $2
            "#,
        )
        .bind(id)
        .bind(user_id.as_uuid())
        .bind(enabled)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Delete a calendar connection
    pub async fn delete_connection(
        pool: &PgPool,
        id: Uuid,
        user_id: UserId,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM calendar_connections WHERE id = $1 AND user_id = $2")
            .bind(id)
            .bind(user_id.as_uuid())
            .execute(pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    // ============ Sync Logs ============

    /// Create a new sync log entry
    pub async fn create_sync_log(
        pool: &PgPool,
        input: CreateSyncLog,
    ) -> Result<CalendarSyncLog, sqlx::Error> {
        sqlx::query_as::<_, CalendarSyncLog>(
            r#"
            INSERT INTO calendar_sync_logs (connection_id, direction, status)
            VALUES ($1, $2, 'pending')
            RETURNING *
            "#,
        )
        .bind(input.connection_id)
        .bind(input.direction)
        .fetch_one(pool)
        .await
    }

    /// Complete a sync log with results
    pub async fn complete_sync_log(
        pool: &PgPool,
        input: CompleteSyncLog<'_>,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"
            UPDATE calendar_sync_logs
            SET status = $2, events_created = $3, events_updated = $4,
                events_deleted = $5, conflicts_detected = $6, error_message = $7,
                completed_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(input.id)
        .bind(input.status)
        .bind(input.events_created)
        .bind(input.events_updated)
        .bind(input.events_deleted)
        .bind(input.conflicts_detected)
        .bind(input.error_message)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Get recent sync logs for a connection
    pub async fn find_sync_logs_by_connection(
        pool: &PgPool,
        connection_id: Uuid,
        limit: i64,
    ) -> Result<Vec<CalendarSyncLog>, sqlx::Error> {
        sqlx::query_as::<_, CalendarSyncLog>(
            r#"
            SELECT * FROM calendar_sync_logs
            WHERE connection_id = $1
            ORDER BY created_at DESC
            LIMIT $2
            "#,
        )
        .bind(connection_id)
        .bind(limit)
        .fetch_all(pool)
        .await
    }
}
