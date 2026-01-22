use shared::types::{OrganizationId, UserId};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{
    CreateWalkerProfile, CreateWalkerSpecialization, UpdateWalkerProfile, WalkerProfile,
    WalkerSpecialization, WalkerSpecializationType,
};

pub struct WalkerProfileRepository;

impl WalkerProfileRepository {
    // MARK: - Profile CRUD

    pub async fn find_by_user(
        pool: &PgPool,
        org_id: OrganizationId,
        user_id: UserId,
    ) -> Result<Option<WalkerProfile>, sqlx::Error> {
        sqlx::query_as::<_, WalkerProfile>(
            r#"
            SELECT id, user_id, organization_id, bio, profile_photo_url,
                   emergency_contact_name, emergency_contact_phone, emergency_contact_relationship,
                   years_experience, created_at, updated_at
            FROM walker_profiles
            WHERE user_id = $1 AND organization_id = $2
            "#,
        )
        .bind(user_id.as_uuid())
        .bind(org_id.as_uuid())
        .fetch_optional(pool)
        .await
    }

    pub async fn find_by_id(
        pool: &PgPool,
        org_id: OrganizationId,
        id: Uuid,
    ) -> Result<Option<WalkerProfile>, sqlx::Error> {
        sqlx::query_as::<_, WalkerProfile>(
            r#"
            SELECT id, user_id, organization_id, bio, profile_photo_url,
                   emergency_contact_name, emergency_contact_phone, emergency_contact_relationship,
                   years_experience, created_at, updated_at
            FROM walker_profiles
            WHERE id = $1 AND organization_id = $2
            "#,
        )
        .bind(id)
        .bind(org_id.as_uuid())
        .fetch_optional(pool)
        .await
    }

    pub async fn create(
        pool: &PgPool,
        profile: CreateWalkerProfile,
    ) -> Result<WalkerProfile, sqlx::Error> {
        sqlx::query_as::<_, WalkerProfile>(
            r#"
            INSERT INTO walker_profiles (user_id, organization_id, bio, profile_photo_url,
                emergency_contact_name, emergency_contact_phone, emergency_contact_relationship,
                years_experience)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, user_id, organization_id, bio, profile_photo_url,
                emergency_contact_name, emergency_contact_phone, emergency_contact_relationship,
                years_experience, created_at, updated_at
            "#,
        )
        .bind(profile.user_id.as_uuid())
        .bind(profile.organization_id.as_uuid())
        .bind(&profile.bio)
        .bind(&profile.profile_photo_url)
        .bind(&profile.emergency_contact_name)
        .bind(&profile.emergency_contact_phone)
        .bind(&profile.emergency_contact_relationship)
        .bind(profile.years_experience)
        .fetch_one(pool)
        .await
    }

    pub async fn update(
        pool: &PgPool,
        org_id: OrganizationId,
        user_id: UserId,
        update: UpdateWalkerProfile,
    ) -> Result<Option<WalkerProfile>, sqlx::Error> {
        sqlx::query_as::<_, WalkerProfile>(
            r#"
            UPDATE walker_profiles
            SET bio = COALESCE($3, bio),
                profile_photo_url = COALESCE($4, profile_photo_url),
                emergency_contact_name = COALESCE($5, emergency_contact_name),
                emergency_contact_phone = COALESCE($6, emergency_contact_phone),
                emergency_contact_relationship = COALESCE($7, emergency_contact_relationship),
                years_experience = COALESCE($8, years_experience),
                updated_at = NOW()
            WHERE user_id = $1 AND organization_id = $2
            RETURNING id, user_id, organization_id, bio, profile_photo_url,
                emergency_contact_name, emergency_contact_phone, emergency_contact_relationship,
                years_experience, created_at, updated_at
            "#,
        )
        .bind(user_id.as_uuid())
        .bind(org_id.as_uuid())
        .bind(&update.bio)
        .bind(&update.profile_photo_url)
        .bind(&update.emergency_contact_name)
        .bind(&update.emergency_contact_phone)
        .bind(&update.emergency_contact_relationship)
        .bind(update.years_experience)
        .fetch_optional(pool)
        .await
    }

    pub async fn upsert(
        pool: &PgPool,
        profile: CreateWalkerProfile,
    ) -> Result<WalkerProfile, sqlx::Error> {
        sqlx::query_as::<_, WalkerProfile>(
            r#"
            INSERT INTO walker_profiles (user_id, organization_id, bio, profile_photo_url,
                emergency_contact_name, emergency_contact_phone, emergency_contact_relationship,
                years_experience)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (user_id, organization_id)
            DO UPDATE SET
                bio = COALESCE(EXCLUDED.bio, walker_profiles.bio),
                profile_photo_url = COALESCE(EXCLUDED.profile_photo_url, walker_profiles.profile_photo_url),
                emergency_contact_name = COALESCE(EXCLUDED.emergency_contact_name, walker_profiles.emergency_contact_name),
                emergency_contact_phone = COALESCE(EXCLUDED.emergency_contact_phone, walker_profiles.emergency_contact_phone),
                emergency_contact_relationship = COALESCE(EXCLUDED.emergency_contact_relationship, walker_profiles.emergency_contact_relationship),
                years_experience = COALESCE(EXCLUDED.years_experience, walker_profiles.years_experience),
                updated_at = NOW()
            RETURNING id, user_id, organization_id, bio, profile_photo_url,
                emergency_contact_name, emergency_contact_phone, emergency_contact_relationship,
                years_experience, created_at, updated_at
            "#,
        )
        .bind(profile.user_id.as_uuid())
        .bind(profile.organization_id.as_uuid())
        .bind(&profile.bio)
        .bind(&profile.profile_photo_url)
        .bind(&profile.emergency_contact_name)
        .bind(&profile.emergency_contact_phone)
        .bind(&profile.emergency_contact_relationship)
        .bind(profile.years_experience)
        .fetch_one(pool)
        .await
    }

    // MARK: - Specializations

    pub async fn get_specializations(
        pool: &PgPool,
        profile_id: Uuid,
    ) -> Result<Vec<WalkerSpecialization>, sqlx::Error> {
        sqlx::query_as::<_, WalkerSpecialization>(
            r#"
            SELECT id, walker_profile_id, specialization, certified,
                   certification_date, certification_expiry, notes, created_at
            FROM walker_specializations
            WHERE walker_profile_id = $1
            ORDER BY specialization
            "#,
        )
        .bind(profile_id)
        .fetch_all(pool)
        .await
    }

    pub async fn add_specialization(
        pool: &PgPool,
        spec: CreateWalkerSpecialization,
    ) -> Result<WalkerSpecialization, sqlx::Error> {
        sqlx::query_as::<_, WalkerSpecialization>(
            r#"
            INSERT INTO walker_specializations (walker_profile_id, specialization, certified,
                certification_date, certification_expiry, notes)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (walker_profile_id, specialization)
            DO UPDATE SET
                certified = EXCLUDED.certified,
                certification_date = EXCLUDED.certification_date,
                certification_expiry = EXCLUDED.certification_expiry,
                notes = EXCLUDED.notes
            RETURNING id, walker_profile_id, specialization, certified,
                certification_date, certification_expiry, notes, created_at
            "#,
        )
        .bind(spec.walker_profile_id)
        .bind(spec.specialization)
        .bind(spec.certified)
        .bind(spec.certification_date)
        .bind(spec.certification_expiry)
        .bind(&spec.notes)
        .fetch_one(pool)
        .await
    }

    pub async fn remove_specialization(
        pool: &PgPool,
        profile_id: Uuid,
        specialization: WalkerSpecializationType,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"
            DELETE FROM walker_specializations
            WHERE walker_profile_id = $1 AND specialization = $2
            "#,
        )
        .bind(profile_id)
        .bind(specialization)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn set_specializations(
        pool: &PgPool,
        profile_id: Uuid,
        specializations: Vec<WalkerSpecializationType>,
    ) -> Result<Vec<WalkerSpecialization>, sqlx::Error> {
        // Delete existing
        sqlx::query("DELETE FROM walker_specializations WHERE walker_profile_id = $1")
            .bind(profile_id)
            .execute(pool)
            .await?;

        // Insert new ones
        let mut results = Vec::new();
        for spec_type in specializations {
            let spec = Self::add_specialization(
                pool,
                CreateWalkerSpecialization {
                    walker_profile_id: profile_id,
                    specialization: spec_type,
                    certified: false,
                    certification_date: None,
                    certification_expiry: None,
                    notes: None,
                },
            )
            .await?;
            results.push(spec);
        }

        Ok(results)
    }
}
