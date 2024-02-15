use kolomoni_auth::Role;
use miette::{Context, IntoDiagnostic, Result};
use sea_orm::{
    sea_query::OnConflict,
    ActiveValue,
    ColumnTrait,
    ConnectionTrait,
    EntityTrait,
    QueryFilter,
};

use crate::entities;

pub struct UserRoleMutation;

impl UserRoleMutation {
    pub async fn add_roles_to_user<C: ConnectionTrait>(
        database: &C,
        user_id: i32,
        roles: &[Role],
    ) -> Result<()> {
        if roles.is_empty() {
            return Ok(());
        }

        let role_models = roles
            .iter()
            .map(|role| entities::user_role::ActiveModel {
                user_id: ActiveValue::Set(user_id),
                role_id: ActiveValue::Set(role.id()),
            })
            .collect::<Vec<_>>();

        entities::user_role::Entity::insert_many(role_models)
            .on_conflict(OnConflict::new().do_nothing().to_owned())
            .exec_without_returning(database)
            .await
            .into_diagnostic()
            .wrap_err("Failed while adding roles to user.")?;

        Ok(())
    }

    pub async fn remove_roles_from_user<C: ConnectionTrait>(
        database: &C,
        user_id: i32,
        roles: &[Role],
    ) -> Result<()> {
        if roles.is_empty() {
            return Ok(());
        }

        // The following code generates a set of AND and OR expressions
        // to the following effect:
        //     (user_id matches) AND ((first role_id matches) OR (second role_id matches) OR ...)
        // This allows us to remove all the specified roles with one database interaction.

        let base_removal_condition = entities::user_role::Column::UserId.eq(user_id);

        let role_id_removal_conditions = roles
            .iter()
            .map(|role| entities::user_role::Column::RoleId.eq(role.id()))
            .collect::<Vec<_>>();
        let merged_role_id_conditions = {
            let mut condition_iterator = role_id_removal_conditions.into_iter();

            // PANIC SAFETY: We checked that `roles` wasn't empty.
            let mut current_condition = condition_iterator.next().unwrap();

            for next_condition in condition_iterator {
                current_condition = current_condition.or(next_condition);
            }

            current_condition
        };


        let final_master_condition = base_removal_condition.and(merged_role_id_conditions);

        entities::user_role::Entity::delete_many()
            .filter(final_master_condition)
            .exec(database)
            .await
            .into_diagnostic()
            .wrap_err("Failed while removing roles from user.")?;

        Ok(())
    }
}
