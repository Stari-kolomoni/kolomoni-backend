use std::collections::HashSet;

use miette::{miette, Result};
use serde::{Deserialize, Serialize};


/// Permissions that we have (inspired by the scope system in OAuth).
///
/// **Note that permissions can be assigned to roles, not users.**
/// If you wish to assign certain permissions to a user, assign them
/// a matching role instead.
///
/// See also [`Role`][super::roles::Role].
///
/// # Maintenance
/// **The defined permissions must match with the `*_seed_permissions.rs` file
/// in `kolomoni_migrations`!**
#[derive(Serialize, Deserialize, Eq, PartialEq, Hash, Copy, Clone, Debug)]
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

    #[serde(rename = "word:create")]
    WordCreate,

    #[serde(rename = "word:read")]
    WordRead,

    #[serde(rename = "word:update")]
    WordUpdate,

    #[serde(rename = "word:delete")]
    WordDelete,
}


impl Permission {
    pub fn from_id(internal_permission_id: i32) -> Option<Self> {
        match internal_permission_id {
            1 => Some(Permission::UserSelfRead),
            2 => Some(Permission::UserSelfWrite),
            3 => Some(Permission::UserAnyRead),
            4 => Some(Permission::UserAnyWrite),
            5 => Some(Permission::WordCreate),
            6 => Some(Permission::WordRead),
            7 => Some(Permission::WordUpdate),
            8 => Some(Permission::WordDelete),
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
            Permission::WordCreate => 5,
            Permission::WordRead => 6,
            Permission::WordUpdate => 7,
            Permission::WordDelete => 8,
        }
    }

    /// Attempt to parse a [`Permission`] from its name.
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "user.self:read" => Some(Self::UserSelfRead),
            "user.self:write" => Some(Self::UserSelfWrite),
            "user.any:read" => Some(Self::UserAnyRead),
            "user.any:write" => Some(Self::UserAnyWrite),
            "word:create" => Some(Self::WordCreate),
            "word:read" => Some(Self::WordRead),
            "word:update" => Some(Self::WordUpdate),
            "word:delete" => Some(Self::WordDelete),
            _ => None,
        }
    }

    /// Get the name of the given [`Permission`].
    pub fn name(&self) -> &'static str {
        match self {
            Permission::UserSelfRead => "user.self:read",
            Permission::UserSelfWrite => "user.self:write",
            Permission::UserAnyRead => "user.any:read",
            Permission::UserAnyWrite => "user.any:write",
            Permission::WordCreate => "word:create",
            Permission::WordRead => "word:read",
            Permission::WordUpdate => "word:update",
            Permission::WordDelete => "word:delete",
        }
    }

    /// Get the description of the given [`Permission`].
    #[rustfmt::skip]
    pub fn description(&self) -> &'static str {
        match self {
            Permission::UserSelfRead =>
                "Allows the user to log in and view their account information.",
            Permission::UserSelfWrite =>
                "Allows the user to update their account information.",
            Permission::UserAnyRead =>
                "Allows the user to view public account information of any other user.",
            Permission::UserAnyWrite =>
                "Allows the user to update account information of any other user.",
            Permission::WordCreate =>
                "Allows the user to create words in the dictionary.",
            Permission::WordRead =>
                "Allows the user to read words in the dictionary.",
            Permission::WordUpdate =>
                "Allows the user to update existing words in the dictionary (but not delete them).",
            Permission::WordDelete =>
                "Allows the user to delete words from the dictionary.",
        }
    }
}

/// List of permissions that are given to **ANY API CALLER**,
/// authenticated or not.
pub const BLANKET_ANY_USER_PERMISSION_GRANT: [Permission; 2] =
    [Permission::WordRead, Permission::UserAnyRead];



/// Set of permissions, usually associated with some user.
pub struct PermissionSet {
    /// Set of permissions.
    permissions: HashSet<Permission>,
}

impl PermissionSet {
    /// Initialize an empty permission set.
    pub fn new_empty() -> Self {
        Self {
            permissions: HashSet::with_capacity(0),
        }
    }

    /// Initialize a permission set from a [`HashSet`] of [`Permission`]s.
    pub fn from_permission_set(permission_set: HashSet<Permission>) -> Self {
        Self {
            permissions: permission_set,
        }
    }

    /// Initialize [`PermissionSet`] given a `Vec` of permission names.
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
    ///
    /// This will also check the blanket permission grant (see `BLANKET_ANY_USER_PERMISSION_GRANT`)
    /// and return `true` regardless of the user's effective permissions (if the required permission
    /// has a blanket grant).
    pub fn has_permission(&self, permission: Permission) -> bool {
        if BLANKET_ANY_USER_PERMISSION_GRANT.contains(&permission) {
            return true;
        }

        if self.permissions.contains(&permission) {
            return true;
        }

        false
    }

    /// Consumes the [`PermissionSet`] and returns a raw [`HashSet`] of [`Permission`]s.
    pub fn into_permissions(self) -> HashSet<Permission> {
        self.permissions
    }

    /// Returns a reference to the set of permissions.
    pub fn permissions(&self) -> &HashSet<Permission> {
        &self.permissions
    }

    /// Returns a `Vec` of permission names.
    pub fn permission_names(&self) -> Vec<&'static str> {
        self.permissions
            .iter()
            .map(|permission| permission.name())
            .collect()
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parses_from_name() {
        let permissions = PermissionSet::from_permission_names(vec![
            "user.self:read",
            "user.self:write",
            "user.any:read",
            "user.any:write",
        ])
        .unwrap();

        assert!(permissions.has_permission(Permission::UserSelfRead));
        assert!(permissions.has_permission(Permission::UserSelfWrite));
        assert!(permissions.has_permission(Permission::UserAnyRead));
        assert!(permissions.has_permission(Permission::UserAnyWrite));
    }

    #[test]
    fn converts_to_name() {
        assert_eq!(Permission::UserSelfRead.name(), "user.self:read");

        assert_eq!(
            Permission::UserSelfWrite.name(),
            "user.self:write",
        );

        assert_eq!(Permission::UserAnyRead.name(), "user.any:read");

        assert_eq!(Permission::UserAnyWrite.name(), "user.any:write");
    }
}
