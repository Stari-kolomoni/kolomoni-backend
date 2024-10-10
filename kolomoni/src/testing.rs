//! A test-only API. Included only when
//! the `with_test_facilities` feature flag is enabled.

use actix_web::{post, web, HttpResponse, Scope};
use kolomoni_auth::{Role, RoleSet, DEFAULT_USER_ROLE};
use kolomoni_configuration::{Configuration, ForMigrationAtApiRuntimeDatabaseConfiguration};
use kolomoni_core::ids::UserId;
use kolomoni_database::entities;
use kolomoni_migrations::{
    core::{
        errors::{MigrationApplyError, MigrationRollbackError, StatusError},
        identifier::MigrationIdentifier,
        migrations::MigrationsWithStatusOptions,
    },
    migrations,
};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgConnectOptions, Acquire, ConnectOptions, PgConnection};
use thiserror::Error;
use tracing::{info, warn};
use uuid::Uuid;

use crate::{
    api::errors::{EndpointError, EndpointResult},
    obtain_database_connection,
    state::ApplicationState,
};


#[derive(Debug, Error)]
pub enum RollbackAndReapplyError {
    #[error("failed to get migration status")]
    StatusError {
        #[from]
        #[source]
        error: StatusError,
    },

    #[error("migration {} does not have a rollback script", .migration)]
    MissingRollbackScript { migration: MigrationIdentifier },

    #[error("database error encountered")]
    DatabaseError {
        #[from]
        #[source]
        error: sqlx::Error,
    },

    #[error("failed to rollback migration {}", .migration)]
    MigrationRollbackError {
        #[source]
        error: MigrationRollbackError,

        migration: MigrationIdentifier,
    },

    #[error("failed to apply migration {}", .migration)]
    MigrationApplyError {
        #[source]
        error: MigrationApplyError,

        migration: MigrationIdentifier,
    },
}


fn construct_database_migrator_connection_options(
    database_configuration: &ForMigrationAtApiRuntimeDatabaseConfiguration,
) -> PgConnectOptions {
    let mut connection_options = PgConnectOptions::new_without_pgpass()
        .application_name(&format!(
            "stari-kolomoni-backend-test_v{}",
            env!("CARGO_PKG_VERSION")
        ))
        .statement_cache_capacity(
            database_configuration
                .statement_cache_capacity
                .unwrap_or(200),
        )
        .host(&database_configuration.host)
        .port(database_configuration.port)
        .username(&database_configuration.username)
        .database(&database_configuration.database_name);

    if let Some(password) = &database_configuration.password {
        connection_options = connection_options.password(password.as_str());
    }

    connection_options
}


async fn rollback_and_reapply_non_privileged_migrations(
    migrator_user_connection_options: &PgConnectOptions,
) -> Result<(), RollbackAndReapplyError> {
    warn!("Rolling back and reapplying all non-privileged migrations.");

    let migrator = migrations::manager();


    info!("Fetching migration status.");

    let all_migrations = migrator
        .migrations_with_status_with_fallback(
            Some(migrator_user_connection_options),
            MigrationsWithStatusOptions::strict(),
        )
        .await?;

    // Ignores leading privileged migrations.
    let migrations_to_rollback_and_reapply = Vec::from_iter(
        all_migrations
            .into_iter()
            .skip_while(|migration| migration.configuration().run_as_privileged_user),
    );

    // Ensures all filtered migration scripts actually have a rollback script.
    for migration in &migrations_to_rollback_and_reapply {
        if !migration.has_rollback_script() {
            return Err(RollbackAndReapplyError::MissingRollbackScript {
                migration: migration.identifier().to_owned(),
            });
        }
    }

    info!(
        "Will rollback and reapply {} migrations.",
        migrations_to_rollback_and_reapply.len()
    );

    info!("Connecting to database as migrator...");


    let mut database_connection = migrator_user_connection_options.connect().await?;

    for migration_to_roll_back in migrations_to_rollback_and_reapply.iter().rev() {
        info!(
            "Rolling back migration {}...",
            migration_to_roll_back.identifier()
        );

        migration_to_roll_back
            .execute_down(&mut database_connection)
            .await
            .map_err(
                |error| RollbackAndReapplyError::MigrationRollbackError {
                    error,
                    migration: migration_to_roll_back.identifier().to_owned(),
                },
            )?;
    }

    for migration_to_apply in &migrations_to_rollback_and_reapply {
        info!(
            "Applying migration {}...",
            migration_to_apply.identifier()
        );

        migration_to_apply
            .execute_up(&mut database_connection)
            .await
            .map_err(
                |error| RollbackAndReapplyError::MigrationApplyError {
                    error,
                    migration: migration_to_apply.identifier().to_owned(),
                },
            )?;
    }


    info!("All migrations rolled back and reapplied.");

    Ok(())
}

#[post("/full-reset")]
pub async fn reset_server(state: ApplicationState) -> EndpointResult {
    warn!("Resetting database.");


    let migrator_connection_options = construct_database_migrator_connection_options(
        &state.configuration().database.for_migration_at_api_runtime,
    );

    rollback_and_reapply_non_privileged_migrations(&migrator_connection_options)
        .await
        .map_err(|error| EndpointError::internal_error(error))?;

    Ok(HttpResponse::Ok().finish())
}


#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct GiveFullUserPermissionsRequest {
    pub user_id: Uuid,
}

#[post("/give-user-full-permissions")]
pub async fn give_full_permissions_to_user(
    state: ApplicationState,
    request_body: web::Json<GiveFullUserPermissionsRequest>,
) -> EndpointResult {
    let mut database_connection = state.acquire_database_connection().await?;
    let mut transaction = database_connection.begin().await?;

    let target_user_id = UserId::new(request_body.user_id);


    warn!(
        "Giving full permissions to user {}.",
        target_user_id
    );

    entities::UserRoleMutation::add_roles_to_user(
        &mut transaction,
        target_user_id,
        RoleSet::from_roles(&[Role::User, Role::Administrator]),
    )
    .await?;


    transaction.commit().await?;


    Ok(HttpResponse::Ok().finish())
}



#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct ResetUserRolesRequest {
    pub user_id: Uuid,
}


#[post("/reset-user-roles-to-normal")]
pub async fn reset_user_roles_to_starting_user_roles(
    state: ApplicationState,
    request_body: web::Json<ResetUserRolesRequest>,
) -> EndpointResult {
    let mut database_connection = state.acquire_database_connection().await?;
    let mut transaction = database_connection.begin().await?;

    let target_user_id = UserId::new(request_body.user_id);


    warn!(
        "Resetting user {} to starting roles.",
        target_user_id
    );

    let current_roles_of_user =
        entities::UserRoleQuery::roles_for_user(&mut transaction, target_user_id).await?;


    entities::UserRoleMutation::remove_roles_from_user(
        &mut transaction,
        target_user_id,
        current_roles_of_user,
    )
    .await?;

    entities::UserRoleMutation::add_roles_to_user(
        &mut transaction,
        target_user_id,
        RoleSet::from_roles(&[DEFAULT_USER_ROLE]),
    )
    .await?;


    Ok(HttpResponse::Ok().finish())
}




#[rustfmt::skip]
pub fn testing_router() -> Scope {
    web::scope("/testing")
        .service(reset_server)
        .service(give_full_permissions_to_user)
        .service(reset_user_roles_to_starting_user_roles)
}
