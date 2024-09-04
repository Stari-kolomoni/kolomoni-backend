use kolomoni_migrations_core::{context::MigrationContext, errors::MigrationApplyError};

use super::{permissions::StandardPermission, roles::StandardRole};



#[kolomoni_migrations_macros::up]
pub async fn up(mut context: MigrationContext<'_>) -> Result<(), MigrationApplyError> {
    let database_connection = context.database_connection();


    for permission in StandardPermission::all_permissions() {
        sqlx::query(
            "INSERT INTO kolomoni.permission (id, key, description_en, description_sl) \
            VALUES ($1, $2, $3, $4)",
        )
        .bind(permission.internal_id())
        .bind(permission.external_key())
        .bind(permission.english_description())
        .bind(permission.slovene_description())
        .execute(&mut *database_connection)
        .await
        .map_err(|error| MigrationApplyError::FailedToExecuteQuery { error })?;
    }

    for role in StandardRole::all_roles() {
        sqlx::query(
            "INSERT INTO kolomoni.role (id, key, description_en, description_sl) \
            VALUES ($1, $2, $3, $4)",
        )
        .bind(role.internal_id())
        .bind(role.external_key())
        .bind(role.english_description())
        .bind(role.slovene_description())
        .execute(&mut *database_connection)
        .await
        .map_err(|error| MigrationApplyError::FailedToExecuteQuery { error })?;

        for assigned_permission in role.permission_list() {
            sqlx::query(
                "INSERT INTO kolomoni.role_permission (role_id, permission_id) \
                VALUES ($1, $2)",
            )
            .bind(role.internal_id())
            .bind(assigned_permission.internal_id())
            .execute(&mut *database_connection)
            .await
            .map_err(|error| MigrationApplyError::FailedToExecuteQuery { error })?;
        }
    }


    Ok(())
}
