use axum::{
    extract::{Path, State},
    Json,
};
use db::models::{CreateWalkerProfile, WalkerSpecializationType};
use db::{UserRepository, WalkerProfileRepository};
use serde::{Deserialize, Serialize};
use shared::AppError;

use crate::{
    auth::{AuthUser, TenantContext},
    error::{ApiError, ApiResult},
    state::AppState,
};

// MARK: - Request/Response Types

#[derive(Debug, Serialize)]
pub struct WalkerProfileResponse {
    pub id: String,
    pub user_id: String,
    pub bio: Option<String>,
    pub profile_photo_url: Option<String>,
    pub emergency_contact_name: Option<String>,
    pub emergency_contact_phone: Option<String>,
    pub emergency_contact_relationship: Option<String>,
    pub years_experience: i32,
    pub specializations: Vec<SpecializationResponse>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct SpecializationResponse {
    pub specialization: String,
    pub display_name: String,
    pub certified: bool,
    pub certification_date: Option<String>,
    pub certification_expiry: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateWalkerProfileRequest {
    pub bio: Option<String>,
    pub profile_photo_url: Option<String>,
    pub emergency_contact_name: Option<String>,
    pub emergency_contact_phone: Option<String>,
    pub emergency_contact_relationship: Option<String>,
    pub years_experience: Option<i32>,
    pub specializations: Option<Vec<String>>,
}

// MARK: - Endpoints

/// Get current walker's profile
pub async fn get_my_profile(
    State(_state): State<AppState>,
    tenant: TenantContext,
    auth: AuthUser,
) -> ApiResult<Json<WalkerProfileResponse>> {
    // Verify user is a walker
    let user = UserRepository::find_by_id(&tenant.pool, tenant.org_id, auth.user_id)
        .await?
        .ok_or_else(|| ApiError::from(AppError::Forbidden))?;

    if !user.is_walker() {
        return Err(ApiError::from(AppError::Forbidden));
    }

    // Get or create profile
    let profile =
        match WalkerProfileRepository::find_by_user(&tenant.pool, tenant.org_id, auth.user_id)
            .await?
        {
            Some(p) => p,
            None => {
                // Create empty profile
                WalkerProfileRepository::create(
                    &tenant.pool,
                    CreateWalkerProfile {
                        user_id: auth.user_id,
                        organization_id: tenant.org_id,
                        bio: None,
                        profile_photo_url: None,
                        emergency_contact_name: None,
                        emergency_contact_phone: None,
                        emergency_contact_relationship: None,
                        years_experience: None,
                    },
                )
                .await?
            }
        };

    // Get specializations
    let specs = WalkerProfileRepository::get_specializations(&tenant.pool, profile.id).await?;

    Ok(Json(WalkerProfileResponse {
        id: profile.id.to_string(),
        user_id: profile.user_id.to_string(),
        bio: profile.bio,
        profile_photo_url: profile.profile_photo_url,
        emergency_contact_name: profile.emergency_contact_name,
        emergency_contact_phone: profile.emergency_contact_phone,
        emergency_contact_relationship: profile.emergency_contact_relationship,
        years_experience: profile.years_experience.unwrap_or(0),
        specializations: specs
            .into_iter()
            .map(|s| SpecializationResponse {
                specialization: format!("{:?}", s.specialization).to_lowercase(),
                display_name: s.specialization.display_name().to_string(),
                certified: s.certified,
                certification_date: s.certification_date.map(|d| d.to_string()),
                certification_expiry: s.certification_expiry.map(|d| d.to_string()),
            })
            .collect(),
        created_at: profile.created_at.to_rfc3339(),
        updated_at: profile.updated_at.to_rfc3339(),
    }))
}

/// Update current walker's profile
pub async fn update_my_profile(
    State(_state): State<AppState>,
    tenant: TenantContext,
    auth: AuthUser,
    Json(req): Json<UpdateWalkerProfileRequest>,
) -> ApiResult<Json<WalkerProfileResponse>> {
    // Verify user is a walker
    let user = UserRepository::find_by_id(&tenant.pool, tenant.org_id, auth.user_id)
        .await?
        .ok_or_else(|| ApiError::from(AppError::Forbidden))?;

    if !user.is_walker() {
        return Err(ApiError::from(AppError::Forbidden));
    }

    // Upsert profile
    let profile = WalkerProfileRepository::upsert(
        &tenant.pool,
        CreateWalkerProfile {
            user_id: auth.user_id,
            organization_id: tenant.org_id,
            bio: req.bio,
            profile_photo_url: req.profile_photo_url,
            emergency_contact_name: req.emergency_contact_name,
            emergency_contact_phone: req.emergency_contact_phone,
            emergency_contact_relationship: req.emergency_contact_relationship,
            years_experience: req.years_experience,
        },
    )
    .await?;

    // Update specializations if provided
    if let Some(spec_names) = req.specializations {
        let spec_types: Vec<WalkerSpecializationType> = spec_names
            .iter()
            .filter_map(|name| parse_specialization(name))
            .collect();

        WalkerProfileRepository::set_specializations(&tenant.pool, profile.id, spec_types).await?;
    }

    // Get updated specializations
    let specs = WalkerProfileRepository::get_specializations(&tenant.pool, profile.id).await?;

    Ok(Json(WalkerProfileResponse {
        id: profile.id.to_string(),
        user_id: profile.user_id.to_string(),
        bio: profile.bio,
        profile_photo_url: profile.profile_photo_url,
        emergency_contact_name: profile.emergency_contact_name,
        emergency_contact_phone: profile.emergency_contact_phone,
        emergency_contact_relationship: profile.emergency_contact_relationship,
        years_experience: profile.years_experience.unwrap_or(0),
        specializations: specs
            .into_iter()
            .map(|s| SpecializationResponse {
                specialization: format!("{:?}", s.specialization).to_lowercase(),
                display_name: s.specialization.display_name().to_string(),
                certified: s.certified,
                certification_date: s.certification_date.map(|d| d.to_string()),
                certification_expiry: s.certification_expiry.map(|d| d.to_string()),
            })
            .collect(),
        created_at: profile.created_at.to_rfc3339(),
        updated_at: profile.updated_at.to_rfc3339(),
    }))
}

/// Get a walker's profile (admin endpoint)
pub async fn get_walker_profile(
    State(_state): State<AppState>,
    tenant: TenantContext,
    auth: AuthUser,
    Path(walker_id): Path<String>,
) -> ApiResult<Json<WalkerProfileResponse>> {
    // Verify requester is admin
    let user = UserRepository::find_by_id(&tenant.pool, tenant.org_id, auth.user_id)
        .await?
        .ok_or_else(|| ApiError::from(AppError::Forbidden))?;

    if !user.is_admin() {
        return Err(ApiError::from(AppError::Forbidden));
    }

    let walker_uuid = walker_id
        .parse()
        .map_err(|_| ApiError::from(AppError::Validation("Invalid walker ID".to_string())))?;

    let walker_user_id = shared::types::UserId::from(walker_uuid);

    // Get or create profile
    let profile =
        match WalkerProfileRepository::find_by_user(&tenant.pool, tenant.org_id, walker_user_id)
            .await?
        {
            Some(p) => p,
            None => {
                WalkerProfileRepository::create(
                    &tenant.pool,
                    CreateWalkerProfile {
                        user_id: walker_user_id,
                        organization_id: tenant.org_id,
                        bio: None,
                        profile_photo_url: None,
                        emergency_contact_name: None,
                        emergency_contact_phone: None,
                        emergency_contact_relationship: None,
                        years_experience: None,
                    },
                )
                .await?
            }
        };

    let specs = WalkerProfileRepository::get_specializations(&tenant.pool, profile.id).await?;

    Ok(Json(WalkerProfileResponse {
        id: profile.id.to_string(),
        user_id: profile.user_id.to_string(),
        bio: profile.bio,
        profile_photo_url: profile.profile_photo_url,
        emergency_contact_name: profile.emergency_contact_name,
        emergency_contact_phone: profile.emergency_contact_phone,
        emergency_contact_relationship: profile.emergency_contact_relationship,
        years_experience: profile.years_experience.unwrap_or(0),
        specializations: specs
            .into_iter()
            .map(|s| SpecializationResponse {
                specialization: format!("{:?}", s.specialization).to_lowercase(),
                display_name: s.specialization.display_name().to_string(),
                certified: s.certified,
                certification_date: s.certification_date.map(|d| d.to_string()),
                certification_expiry: s.certification_expiry.map(|d| d.to_string()),
            })
            .collect(),
        created_at: profile.created_at.to_rfc3339(),
        updated_at: profile.updated_at.to_rfc3339(),
    }))
}

/// Update a walker's profile (admin endpoint)
pub async fn update_walker_profile(
    State(_state): State<AppState>,
    tenant: TenantContext,
    auth: AuthUser,
    Path(walker_id): Path<String>,
    Json(req): Json<UpdateWalkerProfileRequest>,
) -> ApiResult<Json<WalkerProfileResponse>> {
    // Verify requester is admin
    let user = UserRepository::find_by_id(&tenant.pool, tenant.org_id, auth.user_id)
        .await?
        .ok_or_else(|| ApiError::from(AppError::Forbidden))?;

    if !user.is_admin() {
        return Err(ApiError::from(AppError::Forbidden));
    }

    let walker_uuid = walker_id
        .parse()
        .map_err(|_| ApiError::from(AppError::Validation("Invalid walker ID".to_string())))?;

    let walker_user_id = shared::types::UserId::from(walker_uuid);

    // Upsert profile
    let profile = WalkerProfileRepository::upsert(
        &tenant.pool,
        CreateWalkerProfile {
            user_id: walker_user_id,
            organization_id: tenant.org_id,
            bio: req.bio,
            profile_photo_url: req.profile_photo_url,
            emergency_contact_name: req.emergency_contact_name,
            emergency_contact_phone: req.emergency_contact_phone,
            emergency_contact_relationship: req.emergency_contact_relationship,
            years_experience: req.years_experience,
        },
    )
    .await?;

    // Update specializations if provided
    if let Some(spec_names) = req.specializations {
        let spec_types: Vec<WalkerSpecializationType> = spec_names
            .iter()
            .filter_map(|name| parse_specialization(name))
            .collect();

        WalkerProfileRepository::set_specializations(&tenant.pool, profile.id, spec_types).await?;
    }

    let specs = WalkerProfileRepository::get_specializations(&tenant.pool, profile.id).await?;

    Ok(Json(WalkerProfileResponse {
        id: profile.id.to_string(),
        user_id: profile.user_id.to_string(),
        bio: profile.bio,
        profile_photo_url: profile.profile_photo_url,
        emergency_contact_name: profile.emergency_contact_name,
        emergency_contact_phone: profile.emergency_contact_phone,
        emergency_contact_relationship: profile.emergency_contact_relationship,
        years_experience: profile.years_experience.unwrap_or(0),
        specializations: specs
            .into_iter()
            .map(|s| SpecializationResponse {
                specialization: format!("{:?}", s.specialization).to_lowercase(),
                display_name: s.specialization.display_name().to_string(),
                certified: s.certified,
                certification_date: s.certification_date.map(|d| d.to_string()),
                certification_expiry: s.certification_expiry.map(|d| d.to_string()),
            })
            .collect(),
        created_at: profile.created_at.to_rfc3339(),
        updated_at: profile.updated_at.to_rfc3339(),
    }))
}

/// Get list of all available specializations
pub async fn list_specializations() -> ApiResult<Json<Vec<SpecializationOption>>> {
    let specs: Vec<SpecializationOption> = WalkerSpecializationType::all()
        .into_iter()
        .map(|s| SpecializationOption {
            value: format!("{:?}", s).to_lowercase(),
            display_name: s.display_name().to_string(),
        })
        .collect();

    Ok(Json(specs))
}

#[derive(Debug, Serialize)]
pub struct SpecializationOption {
    pub value: String,
    pub display_name: String,
}

// MARK: - Helpers

fn parse_specialization(name: &str) -> Option<WalkerSpecializationType> {
    match name.to_lowercase().as_str() {
        "puppies" => Some(WalkerSpecializationType::Puppies),
        "seniordogs" | "senior_dogs" => Some(WalkerSpecializationType::SeniorDogs),
        "largebreeds" | "large_breeds" => Some(WalkerSpecializationType::LargeBreeds),
        "smallbreeds" | "small_breeds" => Some(WalkerSpecializationType::SmallBreeds),
        "anxiousreactive" | "anxious_reactive" => Some(WalkerSpecializationType::AnxiousReactive),
        "multipledogs" | "multiple_dogs" => Some(WalkerSpecializationType::MultipleDogs),
        "petfirstaid" | "pet_first_aid" => Some(WalkerSpecializationType::PetFirstAid),
        "dogtraining" | "dog_training" => Some(WalkerSpecializationType::DogTraining),
        "catcare" | "cat_care" => Some(WalkerSpecializationType::CatCare),
        "medicationadministration" | "medication_administration" => {
            Some(WalkerSpecializationType::MedicationAdministration)
        }
        _ => None,
    }
}
