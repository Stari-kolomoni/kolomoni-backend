use std::collections::HashSet;

use kolomoni_core::ids::UserId;
use kolomoni_core::roles::{Role, RoleSet};
use sqlx::PgConnection;

use crate::{QueryError, QueryResult};

pub struct UserRoleMutation;

impl UserRoleMutation {
    /// Gives the specified user a set of roles. If the user
    /// already had one or more of the specified roles, nothing bad happens
    /// (those are ignored).
    ///
    /// Returns a full updated set of roles the user has.
    pub async fn add_roles_to_user(
        database_connection: &mut PgConnection,
        user_id: UserId,
        roles_to_add: RoleSet,
    ) -> QueryResult<RoleSet> {
        struct SelectedRoleId {
            role_id: i32,
        }

        let role_ids_nested = roles_to_add
            .into_roles()
            .into_iter()
            .map(|role| role.id())
            .collect::<Vec<_>>();

        let user_ids_nested = std::iter::repeat(user_id.into_uuid())
            .take(role_ids_nested.len())
            .collect::<Vec<_>>();

        let updated_full_user_role_set = sqlx::query_as!(
            SelectedRoleId,
            "INSERT INTO kolomoni.user_role (user_id, role_id) \
                SELECT * FROM UNNEST($1::uuid[], $2::integer[]) \
                ON CONFLICT DO NOTHING \
                RETURNING \
                    (SELECT DISTINCT role_id \
                    FROM kolomoni.user_role \
                    WHERE user_id = $3) as \"role_id!\"",
            user_ids_nested.as_slice(),
            role_ids_nested.as_slice(),
            user_id.into_uuid()
        )
        .fetch_all(database_connection)
        .await?;

        if updated_full_user_role_set.is_empty() {
            return Ok(RoleSet::new_empty());
        }


        let mut role_hash_set = HashSet::with_capacity(updated_full_user_role_set.len());
        for raw_role in updated_full_user_role_set {
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

    /// Removes a set of roles from the specified user.
    /// If the user did not have any of the specified roles,
    /// nothing bad happens (non-matches are ignored).
    ///
    /// Returns a full updated set of roles the user has.
    pub async fn remove_roles_from_user(
        database_connection: &mut PgConnection,
        user_id: UserId,
        roles_to_remove: RoleSet,
    ) -> QueryResult<RoleSet> {
        struct SelectedRoleId {
            role_id: i32,
        }

        let role_ids_nested = roles_to_remove
            .into_roles()
            .into_iter()
            .map(|role| role.id())
            .collect::<Vec<_>>();


        let updated_full_user_role_set = sqlx::query_as!(
            SelectedRoleId,
            "DELETE FROM kolomoni.user_role \
                WHERE user_id = $1 AND role_id = ANY($2::integer[]) \
                RETURNING \
                    (SELECT DISTINCT role_id \
                    FROM kolomoni.user_role \
                    WHERE user_id = $1) as \"role_id!\"",
            user_id.into_uuid(),
            role_ids_nested.as_slice()
        )
        .fetch_all(database_connection)
        .await?;

        if updated_full_user_role_set.is_empty() {
            return Ok(RoleSet::new_empty());
        }


        let mut role_hash_set = HashSet::with_capacity(updated_full_user_role_set.len());
        for raw_role in updated_full_user_role_set {
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
}
