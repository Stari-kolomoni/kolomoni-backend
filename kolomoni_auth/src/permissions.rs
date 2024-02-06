use std::collections::HashSet;

use miette::{miette, Result};
use serde::{Deserialize, Serialize};


/// User permissions that we have (inspired by the scope system in OAuth).
///
/// **The defined permissions must match with the `*_seed_permissions.rs` file in `migrations`!**
#[derive(Serialize, Deserialize, Eq, PartialEq, Hash, Copy, Clone, Debug)]
#[allow(clippy::enum_variant_names)]
pub enum Permission {
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

impl Permission {
    /// Attempt to parse a [`Permission`] from its name.
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "user.self:read" => Some(Self::UserSelfRead),
            "user.self:write" => Some(Self::UserSelfWrite),
            "user.any:read" => Some(Self::UserAnyRead),
            "user.any:write" => Some(Self::UserAnyWrite),
            _ => None,
        }
    }

    /// Get the internal ID of the given [`Permission`].
    /// This ID is used primarily in the database and should not be visible externally.
    pub fn id(&self) -> i32 {
        match self {
            Permission::UserSelfRead => 1,
            Permission::UserSelfWrite => 2,
            Permission::UserAnyRead => 3,
            Permission::UserAnyWrite => 4,
        }
    }

    /// Get the name of the given [`Permission`].
    pub fn name(&self) -> &'static str {
        match self {
            Permission::UserSelfRead => "user.self:read",
            Permission::UserSelfWrite => "user.self:write",
            Permission::UserAnyRead => "user.any:read",
            Permission::UserAnyWrite => "user.any:write",
        }
    }

    /// Get the description of the given [`Permission`].
    pub fn description(&self) -> &'static str {
        match self {
            Permission::UserSelfRead => {
                "Allows the user to log in and view their account information."
            }
            Permission::UserSelfWrite => "Allows the user to update their account information.",
            Permission::UserAnyRead => {
                "Allows the user to view public account information of any other user."
            }
            Permission::UserAnyWrite => {
                "Allows the user to update account information of any other user."
            }
        }
    }
}

/// List of user permissions given to newly-registered users.
pub const DEFAULT_USER_PERMISSIONS: [Permission; 3] = [
    Permission::UserSelfRead,
    Permission::UserSelfWrite,
    Permission::UserAnyRead,
];



/// Set of permissions for a specific user.
///
/// Not to be confused with `kolomoni_database::entities::UserPermission`,
/// which is a raw database entity.
pub struct UserPermissionSet {
    /// Permission set.
    permissions: HashSet<Permission>,
}

impl UserPermissionSet {
    /// Initialize [`UserPermissionSet`] given a `Vec` of permission names.
    /// Returns `Err` if a permission name doesn't resolve to a [`Permission`].
    pub fn from_permission_names<P>(permission_names: Vec<P>) -> Result<Self>
    where
        P: AsRef<str>,
    {
        let permissions = permission_names
            .into_iter()
            .map(|permission_name| {
                Permission::from_name(permission_name.as_ref()).ok_or_else(|| {
                    miette!(
                        "BUG: No such permission: {}!",
                        permission_name.as_ref()
                    )
                })
            })
            .collect::<Result<HashSet<Permission>>>()?;

        Ok(Self { permissions })
    }

    /// Returns `true` if the user has the specified permission, `false` otherwise.
    pub fn has_permission(&self, permission: Permission) -> bool {
        self.permissions.contains(&permission)
    }

    /// Returns a `Vec` of permission names (inverse of `from_permission_names`).
    pub fn to_permission_names(&self) -> Vec<String> {
        self.permissions
            .iter()
            .map(|permission| permission.name().to_string())
            .collect()
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parses_from_name() {
        let permissions = UserPermissionSet::from_permission_names(vec![
            "user.self:read",
            "user.self:write",
            "user.any:read",
            "user.ayn:write",
        ])
        .unwrap();

        assert!(permissions.has_permission(Permission::UserSelfRead));
        assert!(permissions.has_permission(Permission::UserSelfWrite));
        assert!(permissions.has_permission(Permission::UserAnyRead));
        assert!(permissions.has_permission(Permission::UserAnyWrite));
    }
}
