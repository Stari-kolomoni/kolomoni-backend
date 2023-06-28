use anyhow::Result;
use sea_orm::{ActiveValue, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter};

use crate::api::auth::UserPermission;
use crate::database::entities;

#[allow(dead_code)]
pub struct UserPermissionsMutation {}

impl UserPermissionsMutation {
    pub async fn add_permissions_to_user_by_user_id<C: ConnectionTrait>(
        database: &C,
        user_id: i32,
        permissions: Vec<UserPermission>,
    ) -> Result<()> {
        let many_added_permissions: Vec<entities::user_permissions::ActiveModel> = permissions
            .into_iter()
            .map(
                |permission| entities::user_permissions::ActiveModel {
                    user_id: ActiveValue::Set(user_id),
                    permission_id: ActiveValue::Set(permission.to_id()),
                },
            )
            .collect();

        entities::user_permissions::Entity::insert_many(many_added_permissions)
            .exec(database)
            .await?;

        Ok(())
    }

    pub async fn remove_permissions_from_user_by_user_id<C: ConnectionTrait>(
        database: &C,
        user_id: i32,
        permissions_to_remove: Vec<UserPermission>,
    ) -> Result<()> {
        let permission_ids: Vec<i32> = permissions_to_remove
            .into_iter()
            .map(|permission| permission.to_id())
            .collect();

        entities::user_permissions::Entity::delete_many()
            .filter(
                entities::user_permissions::Column::UserId
                    .eq(user_id)
                    .and(entities::user_permissions::Column::PermissionId.is_in(permission_ids)),
            )
            .exec(database)
            .await?;

        Ok(())
    }
}
