use std::collections::HashSet;

use kolomoni_core::ids::UserId;
use kolomoni_core::roles::{Role, RoleSet};
use sqlx::PgConnection;

use crate::{deserialize_json_from_value, QueryError, QueryResult};

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
        struct PreUpdateRoleIdsRaw {
            pre_update_roles_array: serde_json::Value,
        }

        let role_ids_nested = roles_to_add
            .clone()
            .into_roles()
            .into_iter()
            .map(|role| role.id())
            .collect::<Vec<_>>();

        let user_ids_nested =
            std::iter::repeat_n(user_id.into_uuid(), role_ids_nested.len()).collect::<Vec<_>>();

        // This will add the given roles to the user, then return
        // the user's role list **pre-update** (seemingly due to a limitation in PostgreSQL).
        // We'll then manually add our roles to the returned role set,
        // obtaining the current true role set for the user.
        let pre_update_user_role_set = sqlx::query_as!(
            PreUpdateRoleIdsRaw,
            "INSERT INTO kolomoni.user_role (user_id, role_id) \
                SELECT * FROM UNNEST($1::uuid[], $2::integer[]) \
                ON CONFLICT (role_id, user_id) \
                DO UPDATE SET \
                    user_id = excluded.user_id, \
                    role_id = excluded.role_id \
                RETURNING
                    (SELECT DISTINCT coalesce(jsonb_agg_strict(role_id), '[]'::jsonb)
                    FROM kolomoni.user_role
                    WHERE user_id = $3) as \"pre_update_roles_array!\"",
            user_ids_nested.as_slice(),
            role_ids_nested.as_slice(),
            user_id.into_uuid()
        )
        .fetch_one(database_connection)
        .await?;

        let pre_update_role_ids = deserialize_json_from_value!(
            pre_update_user_role_set.pre_update_roles_array => Vec<i32>
        )
        .map_err(|error_message| {
            QueryError::model_error(format!(
                "failed to deserialize the returned pre-update set of role IDs: {}",
                error_message
            ))
        })?;

        if pre_update_role_ids.is_empty() && roles_to_add.is_empty() {
            return Ok(RoleSet::new_empty());
        }

        let mut final_role_hash_set = HashSet::with_capacity(pre_update_role_ids.len());
        for raw_role_id in pre_update_role_ids {
            let Some(role) = Role::from_id(raw_role_id) else {
                return Err(QueryError::ModelError {
                    reason: format!("unexpected internal role ID: {}", raw_role_id).into(),
                });
            };

            final_role_hash_set.insert(role);
        }

        final_role_hash_set.extend(roles_to_add.into_roles());

        Ok(RoleSet::from_role_set(final_role_hash_set))
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

        Ok(RoleSet::from_role_set(role_hash_set))
    }
}
