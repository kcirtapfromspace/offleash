use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared::types::{BlockId, UserId};
use sqlx::FromRow;

/// Calendar block database model (for lunch, personal time, etc.)
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Block {
    pub id: BlockId,
    pub walker_id: UserId,
    pub reason: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub is_recurring: bool,
    pub recurrence_rule: Option<String>, // iCal RRULE format
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Block {
    pub fn duration_minutes(&self) -> i64 {
        (self.end_time - self.start_time).num_minutes()
    }
}

/// Input for creating a new block
#[derive(Debug, Clone, Deserialize)]
pub struct CreateBlock {
    pub walker_id: UserId,
    pub reason: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub is_recurring: bool,
    pub recurrence_rule: Option<String>,
}
