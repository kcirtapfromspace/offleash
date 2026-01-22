use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use shared::types::{OrganizationId, UserId};
use sqlx::FromRow;
use uuid::Uuid;

// MARK: - Walker Profile

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct WalkerProfile {
    pub id: Uuid,
    pub user_id: Uuid,
    pub organization_id: Uuid,
    pub bio: Option<String>,
    pub profile_photo_url: Option<String>,
    pub emergency_contact_name: Option<String>,
    pub emergency_contact_phone: Option<String>,
    pub emergency_contact_relationship: Option<String>,
    pub years_experience: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWalkerProfile {
    pub user_id: UserId,
    pub organization_id: OrganizationId,
    pub bio: Option<String>,
    pub profile_photo_url: Option<String>,
    pub emergency_contact_name: Option<String>,
    pub emergency_contact_phone: Option<String>,
    pub emergency_contact_relationship: Option<String>,
    pub years_experience: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateWalkerProfile {
    pub bio: Option<String>,
    pub profile_photo_url: Option<String>,
    pub emergency_contact_name: Option<String>,
    pub emergency_contact_phone: Option<String>,
    pub emergency_contact_relationship: Option<String>,
    pub years_experience: Option<i32>,
}

// MARK: - Walker Specialization

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "walker_specialization", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum WalkerSpecializationType {
    Puppies,
    SeniorDogs,
    LargeBreeds,
    SmallBreeds,
    AnxiousReactive,
    MultipleDogs,
    PetFirstAid,
    DogTraining,
    CatCare,
    MedicationAdministration,
}

impl WalkerSpecializationType {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Puppies => "Puppies",
            Self::SeniorDogs => "Senior Dogs",
            Self::LargeBreeds => "Large Breeds",
            Self::SmallBreeds => "Small Breeds",
            Self::AnxiousReactive => "Anxious/Reactive Dogs",
            Self::MultipleDogs => "Multiple Dogs",
            Self::PetFirstAid => "Pet First Aid Certified",
            Self::DogTraining => "Dog Training",
            Self::CatCare => "Cat Care",
            Self::MedicationAdministration => "Medication Administration",
        }
    }

    pub fn all() -> Vec<Self> {
        vec![
            Self::Puppies,
            Self::SeniorDogs,
            Self::LargeBreeds,
            Self::SmallBreeds,
            Self::AnxiousReactive,
            Self::MultipleDogs,
            Self::PetFirstAid,
            Self::DogTraining,
            Self::CatCare,
            Self::MedicationAdministration,
        ]
    }
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct WalkerSpecialization {
    pub id: Uuid,
    pub walker_profile_id: Uuid,
    pub specialization: WalkerSpecializationType,
    pub certified: bool,
    pub certification_date: Option<NaiveDate>,
    pub certification_expiry: Option<NaiveDate>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWalkerSpecialization {
    pub walker_profile_id: Uuid,
    pub specialization: WalkerSpecializationType,
    pub certified: bool,
    pub certification_date: Option<NaiveDate>,
    pub certification_expiry: Option<NaiveDate>,
    pub notes: Option<String>,
}
