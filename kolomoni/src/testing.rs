//! A test-only API. Compiled into the binary and enabled only when
//! the `with_test_facilities` feature flag is enabled.

use actix_web::{post, web, HttpResponse, Scope};
use kolomoni_auth::{Role, DEFAULT_USER_ROLE};
use kolomoni_database::{mutation, query};
use kolomoni_migrations::Migrator;
use miette::{Context, IntoDiagnostic, Result};
use sea_orm::DatabaseConnection;
use sea_orm_migration::MigratorTrait;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::{
    api::errors::{APIError, EndpointResult},
    state::ApplicationState,
};

pub async fn drop_database_and_reapply_migrations(
    database_connection: &DatabaseConnection,
) -> Result<()> {
    warn!("Dropping the entire database and reapplying all migrations.");

    Migrator::fresh(database_connection)
        .await
        .into_diagnostic()
        .wrap_err("Failed to drop database and reapply migrations.")?;

    info!("Database reset.");

    Ok(())
}

#[post("/full-reset")]
pub async fn reset_server(state: ApplicationState) -> EndpointResult {
    warn!("Resetting database.");

    drop_database_and_reapply_migrations(&state.database)
        .await
        .map_err(APIError::InternalError)?;

    Ok(HttpResponse::Ok().finish())
}


#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct GiveFullUserPermissionsRequest {
    pub user_id: i32,
}

#[post("/give-user-full-permissions")]
pub async fn give_full_permissions_to_user(
    state: ApplicationState,
    request_body: web::Json<GiveFullUserPermissionsRequest>,
) -> EndpointResult {
    let target_user_id = request_body.user_id;

    warn!(
        "Giving full permissions to user {}.",
        target_user_id
    );

    mutation::UserRoleMutation::add_roles_to_user(
        &state.database,
        target_user_id,
        &[Role::User, Role::Administrator],
    )
    .await
    .map_err(APIError::InternalError)?;

    Ok(HttpResponse::Ok().finish())
}


#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct ResetUserRolesRequest {
    pub user_id: i32,
}

#[post("/reset-user-roles-to-normal")]
pub async fn reset_user_roles_to_starting_user_roles(
    state: ApplicationState,
    request_body: web::Json<ResetUserRolesRequest>,
) -> EndpointResult {
    let target_user_id = request_body.user_id;

    warn!(
        "Resetting user {} to starting roles.",
        target_user_id
    );

    let previous_role_set = query::UserRoleQuery::user_roles(&state.database, target_user_id)
        .await
        .map_err(APIError::InternalError)?;

    mutation::UserRoleMutation::remove_roles_from_user(
        &state.database,
        target_user_id,
        &previous_role_set
            .into_roles()
            .into_iter()
            .collect::<Vec<_>>(),
    )
    .await
    .map_err(APIError::InternalError)?;

    mutation::UserRoleMutation::add_roles_to_user(
        &state.database,
        target_user_id,
        &[DEFAULT_USER_ROLE],
    )
    .await
    .map_err(APIError::InternalError)?;


    Ok(HttpResponse::Ok().finish())
}


#[rustfmt::skip]
pub fn testing_router() -> Scope {
    web::scope("/testing")
        .service(reset_server)
        .service(give_full_permissions_to_user)
        .service(reset_user_roles_to_starting_user_roles)
}
