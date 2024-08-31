use kolomoni_migrations_core::errors::MigrationRollbackError;
use sqlx::PgConnection;

use super::{permissions::StandardPermission, roles::StandardRole};



#[kolomoni_migrations_macros::down]
pub async fn down(database_connection: &mut PgConnection) -> Result<(), MigrationRollbackError> {
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
