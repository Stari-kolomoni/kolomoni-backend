use kolomoni_migrations_core::errors::MigrationRollbackError;
use shared::StandardPermission;
use sqlx::PgConnection;

mod shared;

#[kolomoni_migrations_macros::down]
pub(super) async fn down(
    database_connection: &mut PgConnection,
) -> Result<(), MigrationRollbackError> {
    for permission in StandardPermission::all_permissions() {
        sqlx::query("DELETE FROM kolomoni.permission WHERE id = $1")
            .bind(permission.id())
            .execute(&mut *database_connection)
            .await
            .map_err(|error| MigrationRollbackError::FailedToExecuteQuery { error })?;
    }

    Ok(())
}
