use kolomoni_auth::permissions::UserPermissions;
use miette::{miette, Context, IntoDiagnostic, Result};
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter};

use crate::entities::{permission, user};


/// Raw permissions query implementation.
/// For something more high-level, see [`UserPermissions`],
/// especially the methods in its [`UserPermissionExt`] impl.
#[allow(dead_code)]
pub struct UserPermissionsQuery {}

impl UserPermissionsQuery {
    pub async fn get_user_permission_names_by_username<C: ConnectionTrait>(
        database: &C,
        username: &str,
    ) -> Result<Option<Vec<String>>> {
        let results = user::Entity::find()
            .filter(user::Column::Username.eq(username))
            .find_with_related(permission::Entity)
            .all(database)
            .await
            .into_diagnostic()
            .wrap_err(
                "Failed while looking up permissions for user in the database (by username).",
            )?;

        if results.is_empty() {
            return Ok(None);
        } else if results.len() != 1 {
            return Err(miette!(
                "BUG: more than one result for user by username!",
            ));
        }

        let (_, permissions) = &results[0];

        Ok(Some(
            permissions
                .iter()
                .map(|permission| permission.name.clone())
                .collect(),
        ))
    }

    pub async fn get_user_permission_names_by_user_id<C: ConnectionTrait>(
        database: &C,
        user_id: i32,
    ) -> Result<Option<Vec<String>>> {
        let results = user::Entity::find_by_id(user_id)
            .find_with_related(permission::Entity)
            .all(database)
            .await
            .into_diagnostic()
            .wrap_err("Failed while looking up permissions for user in the database (by ID).")?;

        if results.is_empty() {
            return Ok(None);
        } else if results.len() != 1 {
            return Err(miette!(
                "BUG: more than one result for user by ID!"
            ));
        }

        let (_, permissions) = &results[0];

        Ok(Some(
            permissions
                .iter()
                .map(|permission| permission.name.clone())
                .collect(),
        ))
    }
}


#[allow(async_fn_in_trait)]
pub trait UserPermissionsExt {
    async fn get_from_database_by_username<C: ConnectionTrait>(
        database: &C,
        username: &str,
    ) -> Result<Option<Self>>
    where
        Self: Sized;

    async fn get_from_database_by_user_id<C: ConnectionTrait>(
        database: &C,
        user_id: i32,
    ) -> Result<Option<Self>>
    where
        Self: Sized;
}


impl UserPermissionsExt for UserPermissions {
    /// Initialize `UserPermissions` by loading permissions from
    /// the database.
    async fn get_from_database_by_username<C: ConnectionTrait>(
        database: &C,
        username: &str,
    ) -> Result<Option<Self>> {
        let permission_names =
            UserPermissionsQuery::get_user_permission_names_by_username(database, username)
                .await
                .with_context(|| "Failed to get user permissions from database.")?;

        let Some(names) = permission_names else {
            return Ok(None);
        };

        Ok(Some(Self::from_permission_names(names)?))
    }

    /// Initialize `UserPermissions` by loading permissions from
    /// the database.
    async fn get_from_database_by_user_id<C: ConnectionTrait>(
        database: &C,
        user_id: i32,
    ) -> Result<Option<Self>> {
        let permission_names =
            UserPermissionsQuery::get_user_permission_names_by_user_id(database, user_id)
                .await
                .with_context(|| "Failed to get user permissions from database.")?;

        let Some(names) = permission_names else {
            return Ok(None);
        };

        Ok(Some(Self::from_permission_names(names)?))
    }
}
