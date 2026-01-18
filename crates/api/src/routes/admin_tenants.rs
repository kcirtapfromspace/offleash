use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use axum::{extract::State, Json};
use db::models::{CreateOrganization, CreateTenantDatabase, CreateUser, UserRole};
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
            slug: req.slug,
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
