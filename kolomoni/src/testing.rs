//! A test-only API. Included only when
//! the `e2e-testing` feature flag is enabled.

use actix_web::{get, post, web, HttpResponse, Scope};
use kolomoni_configuration::ForMigrationAtApiRuntimeDatabaseConfiguration;
use kolomoni_core::{
    api_models::{GiveAdministratorRoleRequest, ResetUserRolesRequest},
    ids::UserId,
    roles::{Role, RoleSet, DEFAULT_USER_ROLE_SET},
};
use kolomoni_database::{
    entities::user_role::{UserRoleMutation, UserRoleQuery},
    DatabaseConnectionOptions,
};
use kolomoni_migrations::{
    core::{
        errors::{MigrationApplyError, MigrationRollbackError, StatusError},
        identifier::MigrationIdentifier,
    },
    migrations,
};
use sqlx::ConnectOptions;
use thiserror::Error;
use tracing::{info, warn};

use crate::{
    api::errors::{EndpointError, EndpointResponseBuilder, EndpointResult},
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

    #[error("unable to rollback through a non-leading privileged migration: {}", .migration)]
    UnableToRollbackPrivilegedMigration { migration: MigrationIdentifier },

    #[error("database error encountered")]
    DatabaseError {
        #[from]
        #[source]
        error: sqlx::Error,
    },

    #[error("migration integrity error: {}", .reason)]
    MigrationIntegrityError { reason: String },

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


#[inline]
fn construct_database_migrator_connection_options(
    migration_connection_configuration: &ForMigrationAtApiRuntimeDatabaseConfiguration,
) -> DatabaseConnectionOptions {
    DatabaseConnectionOptions::from_parameters(
        &migration_connection_configuration.host,
        migration_connection_configuration.port,
        &migration_connection_configuration.username,
        migration_connection_configuration.password.as_deref(),
        &migration_connection_configuration.database_name,
        migration_connection_configuration.statement_cache_capacity,
    )
}


async fn rollback_and_reapply_non_privileged_migrations(
    migrator_user_connection_options: &DatabaseConnectionOptions,
) -> Result<(), RollbackAndReapplyError> {
    warn!("Rolling back and reapplying all non-privileged migrations.");

    let migrator = migrations::manager();


    info!("Fetching migration status.");

    let all_migrations = migrator
        .migrations_with_status_with_fallback(
            migrator_user_connection_options.as_pg_connect_options(),
        )
        .await?;


    if !all_migrations.integrity().has_down_integrity() {
        return Err(RollbackAndReapplyError::MigrationIntegrityError {
            reason: format!("{:?}", all_migrations.integrity()),
        });
    }


    // Ignores leading privileged migrations.
    let migrations_to_rollback_and_reapply = Vec::from_iter(
        all_migrations
            .migrations()
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

    // Ensures no non-leading privileged migrations are in the execution path.
    for migration in &migrations_to_rollback_and_reapply {
        if migration.configuration().run_as_privileged_user {
            return Err(
                RollbackAndReapplyError::UnableToRollbackPrivilegedMigration {
                    migration: migration.identifier().to_owned(),
                },
            );
        }
    }


    info!(
        "Will rollback and reapply {} migrations.",
        migrations_to_rollback_and_reapply.len()
    );

    info!("Connecting to database as migrator...");


    let mut database_connection = migrator_user_connection_options
        .as_pg_connect_options()
        .connect()
        .await?;

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


#[get("/enabled")]
pub async fn is_testing_enabled() -> EndpointResult {
    EndpointResponseBuilder::ok().build()
}


#[post("/state/reset")]
pub async fn reset_server(state: ApplicationState) -> EndpointResult {
    warn!("Resetting database.");


    let migrator_connection_options = construct_database_migrator_connection_options(
        &state.configuration().database.for_migration_at_api_runtime,
    );

    rollback_and_reapply_non_privileged_migrations(&migrator_connection_options)
        .await
        .inspect_err(|error| {
            tracing::error!(
                "errored while rolling back and reapplying privileged migrations: {:?}",
                error
            );
        })
        .map_err(|error| EndpointError::internal_error(error))?;

    Ok(HttpResponse::Ok().finish())
}



#[post("/user/give-administrator-role")]
pub async fn give_administrator_role_to_user(
    state: ApplicationState,
    request_body: web::Json<GiveAdministratorRoleRequest>,
) -> EndpointResult {
    let mut database_connection = state.acquire_database_connection().await?;
    let mut transaction = database_connection.transaction().begin().await?;

    let target_user_id = UserId::new(request_body.user_id);


    warn!(
        "Giving administrator permissions to user {}.",
        target_user_id
    );

    UserRoleMutation::add_roles_to_user(
        &mut transaction,
        target_user_id,
        RoleSet::from_roles(&[Role::Administrator]),
    )
    .await?;


    transaction.commit().await?;


    Ok(HttpResponse::Ok().finish())
}



#[post("/user/reset-roles-to-default")]
pub async fn reset_user_roles_to_starting_user_roles(
    state: ApplicationState,
    request_body: web::Json<ResetUserRolesRequest>,
) -> EndpointResult {
    let mut database_connection = state.acquire_database_connection().await?;
    let mut transaction = database_connection.transaction().begin().await?;

    let target_user_id = UserId::new(request_body.user_id);


    warn!(
        "Resetting user {} to starting roles.",
        target_user_id
    );

    let current_roles_of_user =
        UserRoleQuery::roles_for_user(&mut transaction, target_user_id).await?;


    let remaining_roles = UserRoleMutation::remove_roles_from_user(
        &mut transaction,
        target_user_id,
        current_roles_of_user,
    )
    .await?;

    assert!(remaining_roles.is_empty());

    UserRoleMutation::add_roles_to_user(
        &mut transaction,
        target_user_id,
        (*DEFAULT_USER_ROLE_SET).clone(),
    )
    .await?;


    Ok(HttpResponse::Ok().finish())
}




#[rustfmt::skip]
pub fn testing_router() -> Scope {
    web::scope("/testing")
        .service(is_testing_enabled)
        .service(reset_server)
        .service(give_administrator_role_to_user)
        .service(reset_user_roles_to_starting_user_roles)
}
