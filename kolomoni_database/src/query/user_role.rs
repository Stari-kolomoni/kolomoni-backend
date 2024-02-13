use std::collections::HashSet;

use kolomoni_auth::{Permission, PermissionSet, Role, RoleSet};
use miette::{miette, Result};
use miette::{Context, IntoDiagnostic};
use sea_orm::sea_query::Expr;
use sea_orm::{
    ColumnTrait,
    ConnectionTrait,
    EntityTrait,
    FromQueryResult,
    JoinType,
    QueryFilter,
    QuerySelect,
};

use crate::entities;


pub struct UserRoleQuery;

impl UserRoleQuery {
    pub async fn effective_user_permissions_from_user_id<C: ConnectionTrait>(
        database: &C,
        user_id: i32,
    ) -> Result<PermissionSet> {
        #[derive(Debug, FromQueryResult, PartialEq, Eq, Hash)]
        struct PermissionIdSelect {
            permission_id: i32,
        }

        // Functionally equivalent to the following SQL query:
        //      SELECT DISTINCT role_permission.permission_id FROM role_permission
        //        INNER JOIN user_role ON user_role.role_id = role_permission.role_id
        //        WHERE user_role.user_id = <some user id>;

        let distinct_permission_ids = entities::role_permission::Entity::find()
            .select_only()
            .column(entities::role_permission::Column::PermissionId)
            .distinct()
            .join(
                JoinType::InnerJoin,
                entities::role_permission::Entity::belongs_to(entities::user_role::Entity)
                    .from(entities::role_permission::Column::RoleId)
                    .to(entities::user_role::Column::RoleId)
                    .into(),
            )
            .filter(entities::user_role::Column::UserId.eq(user_id))
            .into_model::<PermissionIdSelect>()
            .all(database)
            .await
            .into_diagnostic()
            .wrap_err("Failed while looking up aggregated permission list.")?;



        if distinct_permission_ids.is_empty() {
            return Ok(PermissionSet::new_empty());
        }

        let permission_id_set = distinct_permission_ids
            .into_iter()
            .map(|permission_struct| permission_struct.permission_id)
            .collect::<HashSet<_>>();

        let parsed_permission_set = permission_id_set
            .into_iter()
            .map(|permission_id| {
                Permission::from_id(permission_id)
                    .ok_or_else(|| miette!("Failed to deserialize permission from database: unrecognized permission id {permission_id}!"))
            })
            .collect::<Result<HashSet<_>, _>>()?;

        Ok(PermissionSet::from_permission_set(
            parsed_permission_set,
        ))
    }

    pub async fn user_roles<C: ConnectionTrait>(database: &C, user_id: i32) -> Result<RoleSet> {
        #[derive(Debug, FromQueryResult, PartialEq, Eq, Hash)]
        struct RoleIdSelect {
            role_id: i32,
        }

        let user_roles = entities::user_role::Entity::find()
            .select_only()
            .column(entities::user_role::Column::RoleId)
            .filter(entities::user_role::Column::UserId.eq(user_id))
            .into_model::<RoleIdSelect>()
            .all(database)
            .await
            .into_diagnostic()
            .wrap_err("Failed to retrieve list of user roles.")?;


        let role_set = user_roles
            .into_iter()
            .map(|select_result| select_result.role_id)
            .map(|role_id| {
                Role::from_id(role_id).ok_or_else(|| {
                    miette!(
                        "Failed to deserialize database response: unrecognized role ID {role_id}!"
                    )
                })
            })
            .collect::<Result<HashSet<_>>>()?;

        Ok(RoleSet::from_role_set(role_set))
    }

    pub async fn user_has_permission<C: ConnectionTrait>(
        database: &C,
        user_id: i32,
        permission: Permission,
    ) -> Result<bool> {
        #[derive(Debug, FromQueryResult, PartialEq, Eq, Hash)]
        struct PermissionCheckCountResult {
            count: i64,
        }

        // Functionally equivalent to the following SQL query:
        //      SELECT COUNT(1) AS "count" FROM "role_permission"
        //        INNER JOIN "user_role" ON "role_permission"."role_id" = "user_role"."role_id"
        //        WHERE "user_role"."user_id" = <user id> AND "role_permission"."permission_id" = <permission id>;

        let mut count_query = entities::role_permission::Entity::find().select_only();

        // TODO Revert back to normal chaining when SeaORM updates this method to take `self` instead of `&mut self`
        //      (is marked as FIXME in their source).
        count_query.expr_as(Expr::val(1).count(), "count");

        let count_result = count_query
            .join(
                JoinType::InnerJoin,
                entities::role_permission::Entity::belongs_to(entities::user_role::Entity)
                    .from(entities::role_permission::Column::RoleId)
                    .to(entities::user_role::Column::RoleId)
                    .into(),
            )
            .filter(entities::user_role::Column::UserId.eq(user_id))
            .filter(entities::role_permission::Column::PermissionId.eq(permission.id()))
            .into_model::<PermissionCheckCountResult>()
            .one(database)
            .await
            .into_diagnostic()
            .wrap_err("Failed while looking up whether the user has a permission.")?;


        match count_result {
            Some(check) => Ok(check.count == 1),
            None => Ok(false),
        }
    }
}


#[allow(async_fn_in_trait)]
pub trait UserPermissionsExt {
    async fn from_database_by_user_id<C: ConnectionTrait>(
        database: &C,
        user_id: i32,
    ) -> Result<Self>
    where
        Self: Sized;
}

impl UserPermissionsExt for PermissionSet {
    async fn from_database_by_user_id<C: ConnectionTrait>(database: &C, user_id: i32) -> Result<Self>
    where
        Self: Sized,
    {
        let permission_set =
            UserRoleQuery::effective_user_permissions_from_user_id(database, user_id).await?;

        Ok(permission_set)
    }
}
