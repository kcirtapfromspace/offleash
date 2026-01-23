use axum::{
    extract::{Path, State},
    Json,
};
use chrono::NaiveDate;
use db::{models::CreatePet, models::UpdatePet, PetRepository};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use shared::AppError;
use uuid::Uuid;

use crate::{
    auth::{AuthUser, TenantContext},
    error::{ApiError, ApiResult},
    state::AppState,
};

#[derive(Debug, Serialize)]
pub struct PetResponse {
    pub id: String,
    pub name: String,
    pub species: String,
    pub breed: Option<String>,
    pub date_of_birth: Option<String>,
    pub age_years: Option<i32>,
    pub weight_lbs: Option<f64>,
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
    pub created_at: String,
}

impl From<db::models::Pet> for PetResponse {
    fn from(pet: db::models::Pet) -> Self {
        Self {
            id: pet.id.to_string(),
            name: pet.name,
            species: pet.species,
            breed: pet.breed,
            date_of_birth: pet.date_of_birth.map(|d| d.to_string()),
            age_years: pet.age_years(),
            weight_lbs: pet.weight_lbs.map(|w| w.to_string().parse().unwrap_or(0.0)),
            gender: pet.gender,
            color: pet.color,
            microchip_id: pet.microchip_id,
            is_spayed_neutered: pet.is_spayed_neutered,
            vaccination_status: pet.vaccination_status,
            temperament: pet.temperament,
            special_needs: pet.special_needs,
            emergency_contact_name: pet.emergency_contact_name,
            emergency_contact_phone: pet.emergency_contact_phone,
            vet_name: pet.vet_name,
            vet_phone: pet.vet_phone,
            photo_url: pet.photo_url,
            notes: pet.notes,
            created_at: pet.created_at.to_rfc3339(),
        }
    }
}

/// List all pets for the authenticated user
pub async fn list_pets(
    State(_state): State<AppState>,
    auth_user: AuthUser,
    tenant: TenantContext,
) -> ApiResult<Json<Vec<PetResponse>>> {
    let pets = PetRepository::list_for_owner(&tenant.pool, tenant.org_id, auth_user.user_id).await?;

    let response: Vec<PetResponse> = pets.into_iter().map(PetResponse::from).collect();

    Ok(Json(response))
}

/// Get a specific pet by ID
pub async fn get_pet(
    State(_state): State<AppState>,
    auth_user: AuthUser,
    tenant: TenantContext,
    Path(id): Path<String>,
) -> ApiResult<Json<PetResponse>> {
    let pet_id: Uuid = id
        .parse()
        .map_err(|_| ApiError::from(AppError::Validation("Invalid pet ID".to_string())))?;

    let pet = PetRepository::get_by_id(&tenant.pool, tenant.org_id, auth_user.user_id, pet_id)
        .await?
        .ok_or_else(|| ApiError::from(AppError::NotFound("Pet not found".to_string())))?;

    Ok(Json(PetResponse::from(pet)))
}

#[derive(Debug, Deserialize)]
pub struct CreatePetRequest {
    pub name: String,
    pub species: Option<String>,
    pub breed: Option<String>,
    pub date_of_birth: Option<String>,
    pub weight_lbs: Option<f64>,
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

/// Create a new pet
pub async fn create_pet(
    State(_state): State<AppState>,
    auth_user: AuthUser,
    tenant: TenantContext,
    Json(req): Json<CreatePetRequest>,
) -> ApiResult<Json<PetResponse>> {
    if req.name.trim().is_empty() {
        return Err(ApiError::from(AppError::Validation(
            "Pet name is required".to_string(),
        )));
    }

    let date_of_birth = if let Some(dob_str) = &req.date_of_birth {
        Some(
            NaiveDate::parse_from_str(dob_str, "%Y-%m-%d")
                .map_err(|_| ApiError::from(AppError::Validation("Invalid date format. Use YYYY-MM-DD".to_string())))?,
        )
    } else {
        None
    };

    let weight_lbs = req.weight_lbs.map(|w| Decimal::try_from(w).unwrap_or_default());

    let input = CreatePet {
        name: req.name.trim().to_string(),
        species: req.species,
        breed: req.breed,
        date_of_birth,
        weight_lbs,
        gender: req.gender,
        color: req.color,
        microchip_id: req.microchip_id,
        is_spayed_neutered: req.is_spayed_neutered,
        vaccination_status: req.vaccination_status,
        temperament: req.temperament,
        special_needs: req.special_needs,
        emergency_contact_name: req.emergency_contact_name,
        emergency_contact_phone: req.emergency_contact_phone,
        vet_name: req.vet_name,
        vet_phone: req.vet_phone,
        photo_url: req.photo_url,
        notes: req.notes,
    };

    let pet = PetRepository::create(&tenant.pool, tenant.org_id, auth_user.user_id, input).await?;

    Ok(Json(PetResponse::from(pet)))
}

#[derive(Debug, Deserialize)]
pub struct UpdatePetRequest {
    pub name: Option<String>,
    pub species: Option<String>,
    pub breed: Option<String>,
    pub date_of_birth: Option<String>,
    pub weight_lbs: Option<f64>,
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

/// Update a pet
pub async fn update_pet(
    State(_state): State<AppState>,
    auth_user: AuthUser,
    tenant: TenantContext,
    Path(id): Path<String>,
    Json(req): Json<UpdatePetRequest>,
) -> ApiResult<Json<PetResponse>> {
    let pet_id: Uuid = id
        .parse()
        .map_err(|_| ApiError::from(AppError::Validation("Invalid pet ID".to_string())))?;

    let date_of_birth = if let Some(dob_str) = &req.date_of_birth {
        Some(
            NaiveDate::parse_from_str(dob_str, "%Y-%m-%d")
                .map_err(|_| ApiError::from(AppError::Validation("Invalid date format. Use YYYY-MM-DD".to_string())))?,
        )
    } else {
        None
    };

    let weight_lbs = req.weight_lbs.map(|w| Decimal::try_from(w).unwrap_or_default());

    let input = UpdatePet {
        name: req.name.map(|n| n.trim().to_string()),
        species: req.species,
        breed: req.breed,
        date_of_birth,
        weight_lbs,
        gender: req.gender,
        color: req.color,
        microchip_id: req.microchip_id,
        is_spayed_neutered: req.is_spayed_neutered,
        vaccination_status: req.vaccination_status,
        temperament: req.temperament,
        special_needs: req.special_needs,
        emergency_contact_name: req.emergency_contact_name,
        emergency_contact_phone: req.emergency_contact_phone,
        vet_name: req.vet_name,
        vet_phone: req.vet_phone,
        photo_url: req.photo_url,
        notes: req.notes,
        is_active: None,
    };

    let pet = PetRepository::update(&tenant.pool, tenant.org_id, auth_user.user_id, pet_id, input)
        .await?
        .ok_or_else(|| ApiError::from(AppError::NotFound("Pet not found".to_string())))?;

    Ok(Json(PetResponse::from(pet)))
}

/// Delete a pet
pub async fn delete_pet(
    State(_state): State<AppState>,
    auth_user: AuthUser,
    tenant: TenantContext,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let pet_id: Uuid = id
        .parse()
        .map_err(|_| ApiError::from(AppError::Validation("Invalid pet ID".to_string())))?;

    let deleted =
        PetRepository::delete(&tenant.pool, tenant.org_id, auth_user.user_id, pet_id).await?;

    if !deleted {
        return Err(ApiError::from(AppError::NotFound("Pet not found".to_string())));
    }

    Ok(Json(serde_json::json!({ "success": true })))
}
