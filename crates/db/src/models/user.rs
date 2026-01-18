use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared::types::UserId;
use sqlx::FromRow;

/// User role enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
pub enum UserRole {
    Admin,
    Walker,
    Customer,
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserRole::Admin => write!(f, "admin"),
            UserRole::Walker => write!(f, "walker"),
            UserRole::Customer => write!(f, "customer"),
        }
    }
}

/// User database model
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub role: UserRole,
    pub first_name: String,
    pub last_name: String,
    pub phone: Option<String>,
    pub timezone: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }

    pub fn is_walker(&self) -> bool {
        self.role == UserRole::Walker
    }

    pub fn is_customer(&self) -> bool {
        self.role == UserRole::Customer
    }

    pub fn is_admin(&self) -> bool {
        self.role == UserRole::Admin
    }
}

/// Input for creating a new user
#[derive(Debug, Clone, Deserialize)]
pub struct CreateUser {
    pub email: String,
    pub password_hash: String,
    pub role: UserRole,
    pub first_name: String,
    pub last_name: String,
    pub phone: Option<String>,
    pub timezone: Option<String>,
}

/// Input for updating a user
#[derive(Debug, Clone, Deserialize, Default)]
pub struct UpdateUser {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub phone: Option<String>,
    pub timezone: Option<String>,
}
