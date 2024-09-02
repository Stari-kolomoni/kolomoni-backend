use kolomoni_migrations_core::{context::MigrationContext, errors::MigrationRollbackError};

use super::{permissions::StandardPermission, roles::StandardRole};



#[kolomoni_migrations_macros::down]
pub async fn down(mut context: MigrationContext<'_>) -> Result<(), MigrationRollbackError> {
    let database_connection = context.database_connection();


    for permission in StandardPermission::all_permissions() {
        sqlx::query("DELETE FROM kolomoni.permission WHERE id = $1")
            .bind(permission.id())
            .execute(&mut *database_connection)
            .await
            .map_err(|error| MigrationRollbackError::FailedToExecuteQuery { error })?;
    }

    for role in StandardRole::all_roles() {
        sqlx::query("DELETE FROM kolomoni.role WHERE id = $1")
            .bind(role.id())
            .execute(&mut *database_connection)
            .await
            .map_err(|error| MigrationRollbackError::FailedToExecuteQuery { error })?;
    }

    // Note: role-permission relations are removed automatically, since we have ON DELETE CASCADE set up.

    Ok(())
}
