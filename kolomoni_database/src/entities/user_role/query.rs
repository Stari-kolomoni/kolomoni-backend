use std::collections::HashSet;

use kolomoni_auth::{Permission, PermissionSet, Role, RoleSet};
use kolomoni_core::id::UserId;
use sqlx::PgConnection;

use crate::{QueryError, QueryResult};



pub struct UserRoleQuery;

impl UserRoleQuery {
    pub async fn roles_for_user(
        connection: &mut PgConnection,
        user_id: UserId,
    ) -> QueryResult<RoleSet> {
        struct SelectedRoleId {
            role_id: i32,
        }

        let raw_roles = sqlx::query_as!(
            SelectedRoleId,
            "SELECT DISTINCT role_id \
                FROM kolomoni.user_role \
                WHERE user_id = $1",
            user_id.into_uuid(),
        )
        .fetch_all(connection)
        .await?;


        if raw_roles.is_empty() {
            return Ok(RoleSet::new_empty());
        }


        let mut role_hash_set = HashSet::with_capacity(raw_roles.len());
        for raw_role in raw_roles {
            let Some(role) = Role::from_id(raw_role.role_id) else {
                return Err(QueryError::ModelError {
                    reason: format!(
                        "unexpected internal role ID: {}",
                        raw_role.role_id
                    )
                    .into(),
                });
            };

            role_hash_set.insert(role);
        }

        Ok(RoleSet::from_role_hash_set(role_hash_set))
    }

    pub async fn transitive_permissions_for_user(
        connection: &mut PgConnection,
        user_id: UserId,
    ) -> QueryResult<PermissionSet> {
        struct SelectedPermissionId {
            permission_id: i32,
        }

        let raw_permissions = sqlx::query_as!(
            SelectedPermissionId,
            "SELECT DISTINCT role_permission.permission_id as \"permission_id\" \
                FROM kolomoni.role_permission \
                INNER JOIN kolomoni.user_role \
                    ON role_permission.role_id = user_role.role_id \
                WHERE user_role.user_id = $1",
            user_id.into_uuid()
        )
        .fetch_all(connection)
        .await?;


        if raw_permissions.is_empty() {
            return Ok(PermissionSet::new_empty());
        }


        let mut permission_hash_set = HashSet::with_capacity(raw_permissions.len());
        for raw_permission in raw_permissions {
            let permission_id_u16 = u16::try_from(raw_permission.permission_id).map_err(|_| {
                QueryError::model_error("Invalid permission ID: outside of u16 range.")
            })?;

            let Some(permission) = Permission::from_id(permission_id_u16) else {
                return Err(QueryError::model_error(format!(
                    "unrecognized internal permission ID: {}",
                    raw_permission.permission_id
                )));
            };

            permission_hash_set.insert(permission);
        }


        Ok(PermissionSet::from_permission_hash_set(
            permission_hash_set,
        ))
    }

    /// # Performance
    /// This is slightly faster than [`Self::transitive_permissions_for_user`].
    /// However, if you need to query for more than one permission, consider calling
    /// [`Self::transitive_permissions_for_user`] once and checking the resulting permission set.
    pub async fn user_has_permission_transitively(
        connection: &mut PgConnection,
        user_id: UserId,
        permission: Permission,
    ) -> QueryResult<bool> {
        let query_result = sqlx::query_scalar!(
            "SELECT EXISTS( \
                SELECT 1 \
                FROM kolomoni.role_permission \
                INNER JOIN kolomoni.user_role \
                ON role_permission.role_id = user_role.role_id \
                WHERE \
                    user_role.user_id = $1 AND role_permission.permission_id = $2 \
            )",
            user_id.into_uuid(),
            permission.id() as i32
        )
        .fetch_one(connection)
        .await?;

        Ok(query_result.unwrap_or(false))
    }
}
