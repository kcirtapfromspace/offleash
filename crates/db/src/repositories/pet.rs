use shared::{types::{OrganizationId, UserId}, AppError};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{CreatePet, Pet, UpdatePet};

pub struct PetRepository;

impl PetRepository {
    /// List all pets for a user in an organization
    pub async fn list_for_owner(
        pool: &PgPool,
        organization_id: OrganizationId,
        owner_id: UserId,
    ) -> Result<Vec<Pet>, AppError> {
        let pets = sqlx::query_as::<_, Pet>(
            r#"
            SELECT * FROM pets
            WHERE organization_id = $1 AND owner_id = $2 AND is_active = true
            ORDER BY name ASC
            "#,
        )
        .bind(organization_id)
        .bind(owner_id)
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(pets)
    }

    /// Get a specific pet by ID
    pub async fn get_by_id(
        pool: &PgPool,
        organization_id: OrganizationId,
        owner_id: UserId,
        pet_id: Uuid,
    ) -> Result<Option<Pet>, AppError> {
        let pet = sqlx::query_as::<_, Pet>(
            r#"
            SELECT * FROM pets
            WHERE id = $1 AND organization_id = $2 AND owner_id = $3
            "#,
        )
        .bind(pet_id)
        .bind(organization_id)
        .bind(owner_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(pet)
    }

    /// Create a new pet
    pub async fn create(
        pool: &PgPool,
        organization_id: OrganizationId,
        owner_id: UserId,
        input: CreatePet,
    ) -> Result<Pet, AppError> {
        let species = input.species.unwrap_or_else(|| "dog".to_string());

        let pet = sqlx::query_as::<_, Pet>(
            r#"
            INSERT INTO pets (
                organization_id, owner_id, name, species, breed,
                date_of_birth, weight_lbs, gender, color, microchip_id,
                is_spayed_neutered, vaccination_status, temperament, special_needs,
                emergency_contact_name, emergency_contact_phone, vet_name, vet_phone,
                photo_url, notes
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20
            )
            RETURNING *
            "#,
        )
        .bind(organization_id)
        .bind(owner_id)
        .bind(&input.name)
        .bind(&species)
        .bind(&input.breed)
        .bind(&input.date_of_birth)
        .bind(&input.weight_lbs)
        .bind(&input.gender)
        .bind(&input.color)
        .bind(&input.microchip_id)
        .bind(input.is_spayed_neutered)
        .bind(&input.vaccination_status)
        .bind(&input.temperament)
        .bind(&input.special_needs)
        .bind(&input.emergency_contact_name)
        .bind(&input.emergency_contact_phone)
        .bind(&input.vet_name)
        .bind(&input.vet_phone)
        .bind(&input.photo_url)
        .bind(&input.notes)
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(pet)
    }

    /// Update a pet
    pub async fn update(
        pool: &PgPool,
        organization_id: OrganizationId,
        owner_id: UserId,
        pet_id: Uuid,
        input: UpdatePet,
    ) -> Result<Option<Pet>, AppError> {
        // Build dynamic update query
        let pet = sqlx::query_as::<_, Pet>(
            r#"
            UPDATE pets SET
                name = COALESCE($4, name),
                species = COALESCE($5, species),
                breed = COALESCE($6, breed),
                date_of_birth = COALESCE($7, date_of_birth),
                weight_lbs = COALESCE($8, weight_lbs),
                gender = COALESCE($9, gender),
                color = COALESCE($10, color),
                microchip_id = COALESCE($11, microchip_id),
                is_spayed_neutered = COALESCE($12, is_spayed_neutered),
                vaccination_status = COALESCE($13, vaccination_status),
                temperament = COALESCE($14, temperament),
                special_needs = COALESCE($15, special_needs),
                emergency_contact_name = COALESCE($16, emergency_contact_name),
                emergency_contact_phone = COALESCE($17, emergency_contact_phone),
                vet_name = COALESCE($18, vet_name),
                vet_phone = COALESCE($19, vet_phone),
                photo_url = COALESCE($20, photo_url),
                notes = COALESCE($21, notes),
                is_active = COALESCE($22, is_active),
                updated_at = NOW()
            WHERE id = $1 AND organization_id = $2 AND owner_id = $3
            RETURNING *
            "#,
        )
        .bind(pet_id)
        .bind(organization_id)
        .bind(owner_id)
        .bind(&input.name)
        .bind(&input.species)
        .bind(&input.breed)
        .bind(&input.date_of_birth)
        .bind(&input.weight_lbs)
        .bind(&input.gender)
        .bind(&input.color)
        .bind(&input.microchip_id)
        .bind(input.is_spayed_neutered)
        .bind(&input.vaccination_status)
        .bind(&input.temperament)
        .bind(&input.special_needs)
        .bind(&input.emergency_contact_name)
        .bind(&input.emergency_contact_phone)
        .bind(&input.vet_name)
        .bind(&input.vet_phone)
        .bind(&input.photo_url)
        .bind(&input.notes)
        .bind(input.is_active)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(pet)
    }

    /// Soft delete a pet (set is_active = false)
    pub async fn delete(
        pool: &PgPool,
        organization_id: OrganizationId,
        owner_id: UserId,
        pet_id: Uuid,
    ) -> Result<bool, AppError> {
        let result = sqlx::query(
            r#"
            UPDATE pets SET is_active = false, updated_at = NOW()
            WHERE id = $1 AND organization_id = $2 AND owner_id = $3
            "#,
        )
        .bind(pet_id)
        .bind(organization_id)
        .bind(owner_id)
        .execute(pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(result.rows_affected() > 0)
    }
}
