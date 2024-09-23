use std::collections::HashSet;

use miette::{miette, Result};
use serde::{Deserialize, Serialize};

// TODO Make sure this and roles are synced with the database migrations.

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

    #[serde(rename = "word.suggestion:create")]
    SuggestionCreate,

    #[serde(rename = "word.suggestion:delete")]
    SuggestionDelete,

    #[serde(rename = "word.translation:create")]
    TranslationCreate,

    #[serde(rename = "word.translation:delete")]
    TranslationDelete,

    #[serde(rename = "category:create")]
    CategoryCreate,

    #[serde(rename = "category:read")]
    CategoryRead,

    #[serde(rename = "category:update")]
    CategoryUpdate,

    #[serde(rename = "category:delete")]
    CategoryDelete,
}


impl Permission {
    pub fn from_id(internal_permission_id: i32) -> Option<Self> {
        match internal_permission_id {
            1 => Some(Self::UserSelfRead),
            2 => Some(Self::UserSelfWrite),
            3 => Some(Self::UserAnyRead),
            4 => Some(Self::UserAnyWrite),
            5 => Some(Self::WordCreate),
            6 => Some(Self::WordRead),
            7 => Some(Self::WordUpdate),
            8 => Some(Self::WordDelete),
            9 => Some(Self::SuggestionCreate),
            10 => Some(Self::SuggestionDelete),
            11 => Some(Self::TranslationCreate),
            12 => Some(Self::TranslationDelete),
            13 => Some(Self::CategoryCreate),
            14 => Some(Self::CategoryRead),
            15 => Some(Self::CategoryUpdate),
            16 => Some(Self::CategoryDelete),
            _ => None,
        }
    }

    /// Get the internal ID of the given [`Permission`].
    /// This ID is used primarily in the database and should not be visible externally.
    pub fn id(&self) -> i32 {
        match self {
            Self::UserSelfRead => 1,
            Self::UserSelfWrite => 2,
            Self::UserAnyRead => 3,
            Self::UserAnyWrite => 4,
            Self::WordCreate => 5,
            Self::WordRead => 6,
            Self::WordUpdate => 7,
            Self::WordDelete => 8,
            Self::SuggestionCreate => 9,
            Self::SuggestionDelete => 10,
            Self::TranslationCreate => 11,
            Self::TranslationDelete => 12,
            Self::CategoryCreate => 13,
            Self::CategoryRead => 14,
            Self::CategoryUpdate => 15,
            Self::CategoryDelete => 16,
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
            "word.suggestion:create" => Some(Self::SuggestionCreate),
            "word.suggestion:delete" => Some(Self::SuggestionDelete),
            "word.translation:create" => Some(Self::TranslationCreate),
            "word.translation:delete" => Some(Self::TranslationDelete),
            "category:create" => Some(Self::CategoryCreate),
            "category:read" => Some(Self::CategoryRead),
            "category:update" => Some(Self::CategoryUpdate),
            "category:delete" => Some(Self::CategoryDelete),
            _ => None,
        }
    }

    /// Get the name of the given [`Permission`].
    pub fn name(&self) -> &'static str {
        match self {
            Self::UserSelfRead => "user.self:read",
            Self::UserSelfWrite => "user.self:write",
            Self::UserAnyRead => "user.any:read",
            Self::UserAnyWrite => "user.any:write",
            Self::WordCreate => "word:create",
            Self::WordRead => "word:read",
            Self::WordUpdate => "word:update",
            Self::WordDelete => "word:delete",
            Self::SuggestionCreate => "word.suggestion:create",
            Self::SuggestionDelete => "word.suggestion:delete",
            Self::TranslationCreate => "word.translation:create",
            Self::TranslationDelete => "word.translation:delete",
            Self::CategoryCreate => "category:create",
            Self::CategoryRead => "category:read",
            Self::CategoryUpdate => "category:update",
            Self::CategoryDelete => "category:delete",
        }
    }

    /// Get the description of the given [`Permission`].
    pub fn description(&self) -> &'static str {
        match self {
            Self::UserSelfRead => "Allows the user to log in and view their account information.",
            Self::UserSelfWrite => "Allows the user to update their account information.",
            Self::UserAnyRead => {
                "Allows the user to view public account information of any other user."
            }
            Self::UserAnyWrite => "Allows the user to update account information of any other user.",
            Self::WordCreate => "Allows the user to create words in the dictionary.",
            Self::WordRead => "Allows the user to read words in the dictionary.",
            Self::WordUpdate => {
                "Allows the user to update existing words in the dictionary (but not delete them)."
            }
            Self::WordDelete => "Allows the user to delete words from the dictionary.",
            Self::SuggestionCreate => "Allows the user to create a translation suggestion.",
            Self::SuggestionDelete => "Allows the user to remove a translation suggestion.",
            Self::TranslationCreate => "Allows the user to translate a word.",
            Self::TranslationDelete => "Allows the user to remove a word translation.",
            Self::CategoryCreate => "Allows the user to create a word category.",
            Self::CategoryRead => "Allows the user to read categories.",
            Self::CategoryUpdate => "Allows the user to update an existing word category.",
            Self::CategoryDelete => "Allows the user to delete a word category.",
        }
    }
}

/// List of permissions that are given to **ANY API CALLER**,
/// authenticated or not.
pub const BLANKET_PERMISSION_GRANT: [Permission; 3] = [
    Permission::WordRead,
    Permission::UserAnyRead,
    Permission::CategoryRead,
];



/// Set of permissions, usually associated with some user.
pub struct PermissionSet {
    /// Set of permissions.
    permissions: HashSet<Permission>,
}

impl PermissionSet {
    /// Initialize an empty permission set.
    #[inline]
    pub fn new_empty() -> Self {
        Self {
            permissions: HashSet::with_capacity(0),
        }
    }

    /// Initialize a permission set from a [`HashSet`] of [`Permission`]s.
    #[inline]
    pub fn from_permission_hash_set(permission_set: HashSet<Permission>) -> Self {
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
        if BLANKET_PERMISSION_GRANT.contains(&permission) {
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

    pub fn permission_names_owned(&self) -> Vec<String> {
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
