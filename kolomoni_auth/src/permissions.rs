use std::collections::HashSet;

use miette::{miette, Result};
use serde::{Deserialize, Serialize};


/// User permissions that we have (inspired by the scope system in OAuth).
///
/// **The defined permissions must match with the `*_seed_permissions.rs` file in `migrations`!**
#[derive(Serialize, Deserialize, Eq, PartialEq, Hash, Copy, Clone, Debug)]
#[allow(clippy::enum_variant_names)]
pub enum UserPermission {
    /// Allows the user to log in and view their account information.
    #[serde(rename = "user.self:read")]
    UserSelfRead,

    /// Allows the user to update their account information.
    #[serde(rename = "user.self:write")]
    UserSelfWrite,

    /// Allows the user to view public account information of any other user.
    #[serde(rename = "user.any:read")]
    UserAnyRead,

    /// Allows the user to update public account information of any other user and
    /// give or remove their permissions.
    #[serde(rename = "user.any:write")]
    UserAnyWrite,
}

impl UserPermission {
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "user.self:read" => Some(Self::UserSelfRead),
            "user.self:write" => Some(Self::UserSelfWrite),
            "user.any:read" => Some(Self::UserAnyRead),
            "user.any:write" => Some(Self::UserAnyWrite),
            _ => None,
        }
    }

    pub fn to_id(self) -> i32 {
        match self {
            UserPermission::UserSelfRead => 1,
            UserPermission::UserSelfWrite => 2,
            UserPermission::UserAnyRead => 3,
            UserPermission::UserAnyWrite => 4,
        }
    }

    pub fn to_name(self) -> &'static str {
        match self {
            UserPermission::UserSelfRead => "user.self:read",
            UserPermission::UserSelfWrite => "user.self:write",
            UserPermission::UserAnyRead => "user.any:read",
            UserPermission::UserAnyWrite => "user.any:write",
        }
    }

    pub fn to_description(self) -> &'static str {
        match self {
            UserPermission::UserSelfRead => {
                "Allows the user to log in and view their account information."
            }
            UserPermission::UserSelfWrite => "Allows the user to update their account information.",
            UserPermission::UserAnyRead => {
                "Allows the user to view public account information of any other user."
            }
            UserPermission::UserAnyWrite => {
                "Allows the user to update account information of any other user."
            }
        }
    }
}

// List of user permissions given to newly-registered users.
pub const DEFAULT_USER_PERMISSIONS: [UserPermission; 3] = [
    UserPermission::UserSelfRead,
    UserPermission::UserSelfWrite,
    UserPermission::UserAnyRead,
];



/// Set of permissions for a specific user.
pub struct UserPermissions {
    permissions: HashSet<UserPermission>,
}

impl UserPermissions {
    /// Initialize `UserPermissions` given a `Vec` of permission names.
    /// Returns `Err` if a permission name doesn't resolve to a `UserPermission`.
    pub fn from_permission_names(permission_names: Vec<String>) -> Result<Self> {
        let permissions = permission_names
            .into_iter()
            .map(|permission_name| {
                UserPermission::from_name(&permission_name)
                    .ok_or_else(|| miette!("BUG: No such permission: {permission_name}!"))
            })
            .collect::<Result<HashSet<UserPermission>>>()?;

        Ok(Self { permissions })
    }

    /* /// Initialize `UserPermissions` by loading permissions from
       /// the database.
       pub async fn get_from_database_by_username<C: ConnectionTrait>(
           database: &C,
           username: &str,
       ) -> Result<Option<Self>> {
           let permission_names =
               query::UserPermissionsQuery::get_user_permission_names_by_username(database, username)
                   .await
                   .with_context(|| "Failed to get user permissions from database.")?;

           let Some(names) = permission_names else {
               return Ok(None);
           };

           Ok(Some(Self::from_permission_names(names)?))
       }

       /// Initialize `UserPermissions` by loading permissions from
       /// the database.
       pub async fn get_from_database_by_user_id<C: ConnectionTrait>(
           database: &C,
           user_id: i32,
       ) -> Result<Option<Self>> {
           let permission_names =
               query::UserPermissionsQuery::get_user_permission_names_by_user_id(database, user_id)
                   .await
                   .with_context(|| "Failed to get user permissions from database.")?;

           let Some(names) = permission_names else {
               return Ok(None);
           };

           Ok(Some(Self::from_permission_names(names)?))
       }
    */
    /// Returns `true` if the user has the specified permission.
    pub fn has_permission(&self, permission: UserPermission) -> bool {
        self.permissions.contains(&permission)
    }

    /// Returns a `Vec` of permission names (inverse of `from_permission_names`).
    pub fn to_permission_names(&self) -> Vec<String> {
        self.permissions
            .iter()
            .map(|permission| permission.to_name().to_string())
            .collect()
    }
}
