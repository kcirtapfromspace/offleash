//! Calendar models for scheduling and external calendar integration

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use shared::types::{OrganizationId, UserId};

/// Calendar event type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "calendar_event_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum CalendarEventType {
    Booking,
    Block,
    Personal,
    Synced,
}

impl std::fmt::Display for CalendarEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CalendarEventType::Booking => write!(f, "booking"),
            CalendarEventType::Block => write!(f, "block"),
            CalendarEventType::Personal => write!(f, "personal"),
            CalendarEventType::Synced => write!(f, "synced"),
        }
    }
}

/// Calendar provider for external integrations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "calendar_provider", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum CalendarProvider {
    Google,
    Apple,
    Caldav,
}

impl std::fmt::Display for CalendarProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CalendarProvider::Google => write!(f, "google"),
            CalendarProvider::Apple => write!(f, "apple"),
            CalendarProvider::Caldav => write!(f, "caldav"),
        }
    }
}

/// Sync direction for calendar connections
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "sync_direction", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum SyncDirection {
    Push,
    Pull,
    Bidirectional,
}

/// Sync status for events and sync operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "sync_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum SyncStatus {
    Pending,
    Synced,
    Failed,
    Conflict,
}

impl std::fmt::Display for SyncStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyncStatus::Pending => write!(f, "pending"),
            SyncStatus::Synced => write!(f, "synced"),
            SyncStatus::Failed => write!(f, "failed"),
            SyncStatus::Conflict => write!(f, "conflict"),
        }
    }
}

/// Calendar event (blocks, synced events, etc.)
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct CalendarEvent {
    pub id: Uuid,
    pub organization_id: OrganizationId,
    pub user_id: UserId,
    pub title: Option<String>,
    pub description: Option<String>,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub all_day: bool,
    pub event_type: CalendarEventType,
    pub calendar_connection_id: Option<Uuid>,
    pub external_event_id: Option<String>,
    pub sync_status: SyncStatus,
    pub last_synced_at: Option<DateTime<Utc>>,
    pub recurrence_rule: Option<String>,
    pub recurrence_parent_id: Option<Uuid>,
    pub color: Option<String>,
    pub is_blocking: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl CalendarEvent {
    /// Get duration in minutes
    pub fn duration_minutes(&self) -> i64 {
        (self.end_time - self.start_time).num_minutes()
    }

    /// Check if event overlaps with a time range
    pub fn overlaps(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> bool {
        self.start_time < end && self.end_time > start
    }

    /// Check if this is a recurring event instance
    pub fn is_recurrence_instance(&self) -> bool {
        self.recurrence_parent_id.is_some()
    }

    /// Check if this is a recurring event parent
    pub fn is_recurrence_parent(&self) -> bool {
        self.recurrence_rule.is_some() && self.recurrence_parent_id.is_none()
    }
}

/// Input for creating a calendar event
#[derive(Debug, Clone)]
pub struct CreateCalendarEvent {
    pub organization_id: OrganizationId,
    pub user_id: UserId,
    pub title: Option<String>,
    pub description: Option<String>,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub all_day: bool,
    pub event_type: CalendarEventType,
    pub calendar_connection_id: Option<Uuid>,
    pub external_event_id: Option<String>,
    pub recurrence_rule: Option<String>,
    pub recurrence_parent_id: Option<Uuid>,
    pub color: Option<String>,
    pub is_blocking: bool,
}

/// Input for updating a calendar event
#[derive(Debug, Clone, Default)]
pub struct UpdateCalendarEvent {
    pub title: Option<Option<String>>,
    pub description: Option<Option<String>>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub all_day: Option<bool>,
    pub color: Option<Option<String>>,
    pub is_blocking: Option<bool>,
    pub sync_status: Option<SyncStatus>,
    pub last_synced_at: Option<Option<DateTime<Utc>>>,
}

/// Calendar connection for external calendar integration
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct CalendarConnection {
    pub id: Uuid,
    pub user_id: UserId,
    pub provider: CalendarProvider,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub token_expires_at: Option<DateTime<Utc>>,
    pub server_url: Option<String>,
    pub username: Option<String>,
    pub password_encrypted: Option<String>,
    pub calendar_id: String,
    pub calendar_name: Option<String>,
    pub calendar_color: Option<String>,
    pub sync_enabled: bool,
    pub sync_direction: SyncDirection,
    pub push_bookings: bool,
    pub push_blocks: bool,
    pub last_sync_at: Option<DateTime<Utc>>,
    pub sync_token: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl CalendarConnection {
    /// Check if tokens need refresh (within 5 minutes of expiry)
    pub fn needs_token_refresh(&self) -> bool {
        if let Some(expires_at) = self.token_expires_at {
            let buffer = chrono::Duration::minutes(5);
            Utc::now() + buffer >= expires_at
        } else {
            false
        }
    }

    /// Check if connection is OAuth-based
    pub fn is_oauth(&self) -> bool {
        matches!(self.provider, CalendarProvider::Google)
    }

    /// Check if connection is CalDAV-based
    pub fn is_caldav(&self) -> bool {
        matches!(
            self.provider,
            CalendarProvider::Apple | CalendarProvider::Caldav
        )
    }
}

/// Input for creating a calendar connection
#[derive(Debug, Clone)]
pub struct CreateCalendarConnection {
    pub user_id: UserId,
    pub provider: CalendarProvider,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub token_expires_at: Option<DateTime<Utc>>,
    pub server_url: Option<String>,
    pub username: Option<String>,
    pub password_encrypted: Option<String>,
    pub calendar_id: String,
    pub calendar_name: Option<String>,
    pub calendar_color: Option<String>,
    pub sync_direction: SyncDirection,
    pub push_bookings: bool,
    pub push_blocks: bool,
}

/// Calendar sync log entry
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct CalendarSyncLog {
    pub id: Uuid,
    pub connection_id: Uuid,
    pub direction: SyncDirection,
    pub status: SyncStatus,
    pub events_created: i32,
    pub events_updated: i32,
    pub events_deleted: i32,
    pub conflicts_detected: i32,
    pub error_message: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Input for creating a sync log entry
#[derive(Debug, Clone)]
pub struct CreateSyncLog {
    pub connection_id: Uuid,
    pub direction: SyncDirection,
}

/// Input for completing a sync log with results
#[derive(Debug, Clone)]
pub struct CompleteSyncLog<'a> {
    pub id: Uuid,
    pub status: SyncStatus,
    pub events_created: i32,
    pub events_updated: i32,
    pub events_deleted: i32,
    pub conflicts_detected: i32,
    pub error_message: Option<&'a str>,
}

/// Calendar event with additional display info (flattened for sqlx queries)
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct CalendarEventWithDetails {
    // CalendarEvent fields
    pub id: Uuid,
    pub organization_id: OrganizationId,
    pub user_id: UserId,
    pub title: Option<String>,
    pub description: Option<String>,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub all_day: bool,
    pub event_type: CalendarEventType,
    pub calendar_connection_id: Option<Uuid>,
    pub external_event_id: Option<String>,
    pub sync_status: SyncStatus,
    pub last_synced_at: Option<DateTime<Utc>>,
    pub recurrence_rule: Option<String>,
    pub recurrence_parent_id: Option<Uuid>,
    pub color: Option<String>,
    pub is_blocking: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    // Join fields
    pub connection_name: Option<String>,
    pub provider: Option<CalendarProvider>,
}

/// Time slot for availability checking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSlot {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub available: bool,
    pub conflict_reason: Option<String>,
}
