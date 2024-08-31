use kolomoni_migrations_core::errors::MigrationApplyError;
use shared::StandardPermission;
use sqlx::PgConnection;

mod shared;

#[kolomoni_migrations_macros::up]
pub(super) async fn up(database_connection: &mut PgConnection) -> Result<(), MigrationApplyError> {
    for permission in StandardPermission::all_permissions() {
        sqlx::query(
            "INSERT INTO kolomoni.permission (id, name, description) \
            VALUES ($1, $2, $3)",
        )
        .bind(permission.id())
        .bind(permission.name())
        .bind(permission.description())
        .execute(&mut *database_connection)
        .await
        .map_err(|error| MigrationApplyError::FailedToExecuteQuery { error })?;
    }

    Ok(())
}
