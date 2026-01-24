use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use shared::types::{OrganizationId, UserId};
use sqlx::FromRow;
use uuid::Uuid;

/// Pet database model (dogs and other animals)
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Pet {
    pub id: Uuid,
    pub organization_id: OrganizationId,
    pub owner_id: UserId,
    pub name: String,
    pub species: String,
    pub breed: Option<String>,
    pub date_of_birth: Option<NaiveDate>,
    pub weight_lbs: Option<Decimal>,
    pub gender: Option<String>,
    pub color: Option<String>,
    pub microchip_id: Option<String>,
    pub is_spayed_neutered: Option<bool>,
    pub vaccination_status: Option<String>,
    pub temperament: Option<String>,
    pub special_needs: Option<String>,
    pub emergency_contact_name: Option<String>,
    pub emergency_contact_phone: Option<String>,
    pub vet_name: Option<String>,
    pub vet_phone: Option<String>,
    pub photo_url: Option<String>,
    pub notes: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Pet {
    /// Calculate approximate age in years
    pub fn age_years(&self) -> Option<i32> {
        self.date_of_birth.map(|dob| {
            let today = Utc::now().date_naive();
            today.years_since(dob).unwrap_or(0) as i32
        })
    }
}

/// Input for creating a new pet
#[derive(Debug, Clone, Deserialize)]
pub struct CreatePet {
    pub name: String,
    pub species: Option<String>,
    pub breed: Option<String>,
    pub date_of_birth: Option<NaiveDate>,
    pub weight_lbs: Option<Decimal>,
    pub gender: Option<String>,
    pub color: Option<String>,
    pub microchip_id: Option<String>,
    pub is_spayed_neutered: Option<bool>,
    pub vaccination_status: Option<String>,
    pub temperament: Option<String>,
    pub special_needs: Option<String>,
    pub emergency_contact_name: Option<String>,
    pub emergency_contact_phone: Option<String>,
    pub vet_name: Option<String>,
    pub vet_phone: Option<String>,
    pub photo_url: Option<String>,
    pub notes: Option<String>,
}

/// Input for updating a pet
#[derive(Debug, Clone, Deserialize)]
pub struct UpdatePet {
    pub name: Option<String>,
    pub species: Option<String>,
    pub breed: Option<String>,
    pub date_of_birth: Option<NaiveDate>,
    pub weight_lbs: Option<Decimal>,
    pub gender: Option<String>,
    pub color: Option<String>,
    pub microchip_id: Option<String>,
    pub is_spayed_neutered: Option<bool>,
    pub vaccination_status: Option<String>,
    pub temperament: Option<String>,
    pub special_needs: Option<String>,
    pub emergency_contact_name: Option<String>,
    pub emergency_contact_phone: Option<String>,
    pub vet_name: Option<String>,
    pub vet_phone: Option<String>,
    pub photo_url: Option<String>,
    pub notes: Option<String>,
    pub is_active: Option<bool>,
}
