use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use axum::{
    extract::{Path, Query, State},
    Json,
};
use db::models::{CreateOrganization, CreateTenantDatabase, CreateUser, TenantDbStatus, UserRole};
use db::{OrganizationRepository, TenantDatabaseRepository, UserRepository};
use serde::{Deserialize, Serialize};
use shared::DomainError;

use crate::{
    auth::PlatformAdminAuth,
    error::{ApiError, ApiResult},
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct CreateTenantRequest {
    pub name: String,
    pub slug: String,
    pub admin_email: String,
    pub admin_password: String,
}

#[derive(Debug, Serialize)]
pub struct CreateTenantResponse {
    pub organization: OrganizationResponse,
    pub tenant_database: TenantDatabaseResponse,
    pub admin_user: AdminUserResponse,
}

#[derive(Debug, Serialize)]
pub struct OrganizationResponse {
    pub id: String,
    pub name: String,
    pub slug: String,
}

#[derive(Debug, Serialize)]
pub struct TenantDatabaseResponse {
    pub id: String,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct AdminUserResponse {
    pub id: String,
    pub email: String,
}

pub async fn create_tenant(
    _auth: PlatformAdminAuth,
    State(state): State<AppState>,
    Json(req): Json<CreateTenantRequest>,
) -> ApiResult<Json<CreateTenantResponse>> {
    // Check if slug already exists
    if OrganizationRepository::find_by_slug(&state.pool, &req.slug)
        .await?
        .is_some()
    {
        return Err(ApiError::from(DomainError::SlugAlreadyExists(
            req.slug.clone(),
        )));
    }

    // Create organization record with default branding settings
    let organization = OrganizationRepository::create(
        &state.pool,
        CreateOrganization {
            name: req.name,
            slug: req.slug.clone(),
            subdomain: Some(req.slug),
            custom_domain: None,
            settings: None, // Use default settings
        },
    )
    .await?;

    // Create tenant_database record with status: provisioning
    // Connection string will be set later during actual provisioning
    let tenant_database = TenantDatabaseRepository::create(
        &state.pool,
        CreateTenantDatabase {
            organization_id: organization.id,
            connection_string: String::new(), // Placeholder until actual provisioning
            status: None,                     // Defaults to Provisioning
        },
    )
    .await?;

    // Hash admin password
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(req.admin_password.as_bytes(), &salt)
        .map_err(|_| {
            ApiError::from(shared::AppError::Internal(
                "Password hashing failed".to_string(),
            ))
        })?
        .to_string();

    // Create admin user for the new organization
    let admin_user = UserRepository::create(
        &state.pool,
        CreateUser {
            organization_id: organization.id,
            email: req.admin_email,
            password_hash,
            role: UserRole::Admin,
            first_name: "Admin".to_string(),
            last_name: "User".to_string(),
            phone: None,
            timezone: None,
        },
    )
    .await?;

    Ok(Json(CreateTenantResponse {
        organization: OrganizationResponse {
            id: organization.id.to_string(),
            name: organization.name,
            slug: organization.slug,
        },
        tenant_database: TenantDatabaseResponse {
            id: tenant_database.id.to_string(),
            status: format!("{:?}", tenant_database.status).to_lowercase(),
        },
        admin_user: AdminUserResponse {
            id: admin_user.id.to_string(),
            email: admin_user.email,
        },
    }))
}

// ============================================================================
// List Tenants Endpoint
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct ListTenantsQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct ListTenantsResponse {
    pub tenants: Vec<TenantInfo>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Serialize)]
pub struct TenantInfo {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub custom_domain: Option<String>,
    pub settings: TenantSettingsResponse,
    pub status: String,
    pub subscription_tier: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct TenantSettingsResponse {
    pub primary_color: Option<String>,
    pub secondary_color: Option<String>,
    pub logo_url: Option<String>,
    pub favicon_url: Option<String>,
    pub font_family: Option<String>,
}

/// Row returned from the tenant list query with joined data
#[derive(Debug, sqlx::FromRow)]
struct TenantRow {
    id: uuid::Uuid,
    name: String,
    slug: String,
    custom_domain: Option<String>,
    settings: sqlx::types::Json<db::models::OrganizationSettings>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
    status: Option<String>,
    subscription_tier: Option<String>,
}

pub async fn list_tenants(
    _auth: PlatformAdminAuth,
    State(state): State<AppState>,
    Query(query): Query<ListTenantsQuery>,
) -> ApiResult<Json<ListTenantsResponse>> {
    let limit = query.limit.unwrap_or(20).min(100);
    let offset = query.offset.unwrap_or(0);

    // Get total count
    let total = OrganizationRepository::count(&state.pool).await?;

    // Query organizations with joined tenant_database status and organization_settings
    let rows = sqlx::query_as::<_, TenantRow>(
        r#"
        SELECT
            o.id,
            o.name,
            o.slug,
            o.custom_domain,
            o.settings,
            o.created_at,
            o.updated_at,
            td.status::text as status,
            os.subscription_tier
        FROM organizations o
        LEFT JOIN tenant_databases td ON td.organization_id = o.id
        LEFT JOIN organization_settings os ON os.organization_id = o.id
        ORDER BY o.created_at DESC
        LIMIT $1 OFFSET $2
        "#,
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.pool)
    .await?;

    let tenants = rows
        .into_iter()
        .map(|row| TenantInfo {
            id: row.id.to_string(),
            name: row.name,
            slug: row.slug,
            custom_domain: row.custom_domain,
            settings: TenantSettingsResponse {
                primary_color: row.settings.primary_color.clone(),
                secondary_color: row.settings.secondary_color.clone(),
                logo_url: row.settings.logo_url.clone(),
                favicon_url: row.settings.favicon_url.clone(),
                font_family: row.settings.font_family.clone(),
            },
            status: row.status.unwrap_or_else(|| "unknown".to_string()),
            subscription_tier: row.subscription_tier.unwrap_or_else(|| "free".to_string()),
            created_at: row.created_at.to_rfc3339(),
            updated_at: row.updated_at.to_rfc3339(),
        })
        .collect();

    Ok(Json(ListTenantsResponse {
        tenants,
        total,
        limit,
        offset,
    }))
}

// ============================================================================
// Get Tenant by ID Endpoint
// ============================================================================

pub async fn get_tenant(
    _auth: PlatformAdminAuth,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<Json<TenantInfo>> {
    let tenant_id: uuid::Uuid = id.parse().map_err(|_| {
        ApiError::from(shared::AppError::Validation(
            "Invalid tenant ID".to_string(),
        ))
    })?;

    // Query organization with joined tenant_database status and organization_settings
    let row = sqlx::query_as::<_, TenantRow>(
        r#"
        SELECT
            o.id,
            o.name,
            o.slug,
            o.custom_domain,
            o.settings,
            o.created_at,
            o.updated_at,
            td.status::text as status,
            os.subscription_tier
        FROM organizations o
        LEFT JOIN tenant_databases td ON td.organization_id = o.id
        LEFT JOIN organization_settings os ON os.organization_id = o.id
        WHERE o.id = $1
        "#,
    )
    .bind(tenant_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| ApiError::from(DomainError::TenantNotFound(id)))?;

    Ok(Json(TenantInfo {
        id: row.id.to_string(),
        name: row.name,
        slug: row.slug,
        custom_domain: row.custom_domain,
        settings: TenantSettingsResponse {
            primary_color: row.settings.primary_color.clone(),
            secondary_color: row.settings.secondary_color.clone(),
            logo_url: row.settings.logo_url.clone(),
            favicon_url: row.settings.favicon_url.clone(),
            font_family: row.settings.font_family.clone(),
        },
        status: row.status.unwrap_or_else(|| "unknown".to_string()),
        subscription_tier: row.subscription_tier.unwrap_or_else(|| "free".to_string()),
        created_at: row.created_at.to_rfc3339(),
        updated_at: row.updated_at.to_rfc3339(),
    }))
}

// ============================================================================
// Update Tenant Endpoint
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct UpdateTenantRequest {
    pub name: Option<String>,
    pub subscription_tier: Option<String>,
    pub custom_domain: Option<String>,
}

pub async fn update_tenant(
    _auth: PlatformAdminAuth,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateTenantRequest>,
) -> ApiResult<Json<TenantInfo>> {
    let tenant_id: uuid::Uuid = id.parse().map_err(|_| {
        ApiError::from(shared::AppError::Validation(
            "Invalid tenant ID".to_string(),
        ))
    })?;

    let org_id = shared::types::OrganizationId::from_uuid(tenant_id);

    // Verify tenant exists
    OrganizationRepository::find_by_id(&state.pool, org_id)
        .await?
        .ok_or_else(|| ApiError::from(DomainError::TenantNotFound(id.clone())))?;

    // Update organization if name or custom_domain provided
    if req.name.is_some() || req.custom_domain.is_some() {
        OrganizationRepository::update(
            &state.pool,
            org_id,
            db::models::UpdateOrganization {
                name: req.name.clone(),
                slug: None,
                custom_domain: req.custom_domain.clone(),
                settings: None,
            },
        )
        .await?;
    }

    // Update subscription_tier in organization_settings if provided
    if let Some(ref tier) = req.subscription_tier {
        sqlx::query(
            r#"
            INSERT INTO organization_settings (organization_id, subscription_tier)
            VALUES ($1, $2)
            ON CONFLICT (organization_id)
            DO UPDATE SET subscription_tier = $2, updated_at = NOW()
            "#,
        )
        .bind(tenant_id)
        .bind(tier)
        .execute(&state.pool)
        .await?;
    }

    // Fetch and return updated tenant info
    let row = sqlx::query_as::<_, TenantRow>(
        r#"
        SELECT
            o.id,
            o.name,
            o.slug,
            o.custom_domain,
            o.settings,
            o.created_at,
            o.updated_at,
            td.status::text as status,
            os.subscription_tier
        FROM organizations o
        LEFT JOIN tenant_databases td ON td.organization_id = o.id
        LEFT JOIN organization_settings os ON os.organization_id = o.id
        WHERE o.id = $1
        "#,
    )
    .bind(tenant_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| ApiError::from(DomainError::TenantNotFound(id)))?;

    Ok(Json(TenantInfo {
        id: row.id.to_string(),
        name: row.name,
        slug: row.slug,
        custom_domain: row.custom_domain,
        settings: TenantSettingsResponse {
            primary_color: row.settings.primary_color.clone(),
            secondary_color: row.settings.secondary_color.clone(),
            logo_url: row.settings.logo_url.clone(),
            favicon_url: row.settings.favicon_url.clone(),
            font_family: row.settings.font_family.clone(),
        },
        status: row.status.unwrap_or_else(|| "unknown".to_string()),
        subscription_tier: row.subscription_tier.unwrap_or_else(|| "free".to_string()),
        created_at: row.created_at.to_rfc3339(),
        updated_at: row.updated_at.to_rfc3339(),
    }))
}

// ============================================================================
// Delete Tenant Endpoint (Soft Delete)
// ============================================================================

#[derive(Debug, Serialize)]
pub struct DeleteTenantResponse {
    pub message: String,
    pub id: String,
    pub status: String,
}

pub async fn delete_tenant(
    _auth: PlatformAdminAuth,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<Json<DeleteTenantResponse>> {
    let tenant_id: uuid::Uuid = id.parse().map_err(|_| {
        ApiError::from(shared::AppError::Validation(
            "Invalid tenant ID".to_string(),
        ))
    })?;

    let org_id = shared::types::OrganizationId::from_uuid(tenant_id);

    // Verify tenant exists
    OrganizationRepository::find_by_id(&state.pool, org_id)
        .await?
        .ok_or_else(|| ApiError::from(DomainError::TenantNotFound(id.clone())))?;

    // Soft-delete: Update tenant_database status to 'inactive'
    TenantDatabaseRepository::update_status_by_org_id(
        &state.pool,
        org_id,
        TenantDbStatus::Inactive,
    )
    .await?;

    Ok(Json(DeleteTenantResponse {
        message: "Tenant deactivated successfully".to_string(),
        id,
        status: "inactive".to_string(),
    }))
}
