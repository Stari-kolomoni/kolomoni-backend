use kolomoni_migrations_core::errors::MigrationApplyError;
use sqlx::PgConnection;

use super::{permissions::StandardPermission, roles::StandardRole};



#[kolomoni_migrations_macros::up]
pub async fn up(database_connection: &mut PgConnection) -> Result<(), MigrationApplyError> {
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

    for role in StandardRole::all_roles() {
        sqlx::query(
            "INSERT INTO kolomoni.role (id, name, description) \
            VALUES ($1, $2, $3)",
        )
        .bind(role.id())
        .bind(role.name())
        .bind(role.description())
        .execute(&mut *database_connection)
        .await
        .map_err(|error| MigrationApplyError::FailedToExecuteQuery { error })?;

        for assigned_permission in role.permission_list() {
            sqlx::query(
                "INSERT INTO kolomoni.role_permission (role_id, permission_id) \
                VALUES ($1, $2)",
            )
            .bind(role.id())
            .bind(assigned_permission.id())
            .execute(&mut *database_connection)
            .await
            .map_err(|error| MigrationApplyError::FailedToExecuteQuery { error })?;
        }
    }


    Ok(())
}
