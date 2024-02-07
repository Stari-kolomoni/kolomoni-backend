use kolomoni_auth::Permission;
use miette::{Context, IntoDiagnostic, Result};
use sea_orm::{ActiveValue, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter};

use crate::entities;

/// Mutations for the [`crate::entities::user_permission::Entity`] entity.
#[allow(dead_code)]
pub struct UserPermissionMutation {}

impl UserPermissionMutation {
    /// Add a set of permissions to a user's permission list. The user is looked up by their ID.
    pub async fn add_permissions_to_user_by_user_id<C: ConnectionTrait>(
        database: &C,
        user_id: i32,
        permissions: Vec<Permission>,
    ) -> Result<()> {
        let many_added_permissions: Vec<entities::user_permission::ActiveModel> = permissions
            .into_iter()
            .map(
                |permission| entities::user_permission::ActiveModel {
                    user_id: ActiveValue::Set(user_id),
                    permission_id: ActiveValue::Set(permission.id()),
                },
            )
            .collect();

        entities::user_permission::Entity::insert_many(many_added_permissions)
            .exec(database)
            .await
            .into_diagnostic()
            .wrap_err("Failed to add user permissions.")?;

        Ok(())
    }

    /// Remove a set of permissions to a user's permission list. The user is looked up by their ID.
    pub async fn remove_permissions_from_user_by_user_id<C: ConnectionTrait>(
        database: &C,
        user_id: i32,
        permissions_to_remove: Vec<Permission>,
    ) -> Result<()> {
        let permission_ids: Vec<i32> = permissions_to_remove
            .into_iter()
            .map(|permission| permission.id())
            .collect();

        entities::user_permission::Entity::delete_many()
            .filter(
                entities::user_permission::Column::UserId
                    .eq(user_id)
                    .and(entities::user_permission::Column::PermissionId.is_in(permission_ids)),
            )
            .exec(database)
            .await
            .into_diagnostic()
            .wrap_err("Failed to delete user permissions.")?;

        Ok(())
    }
}
