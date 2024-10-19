use std::{borrow::Cow, collections::HashSet};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use utoipa::ToSchema;

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
#[derive(
    Serialize,
    Deserialize,
    Eq,
    PartialEq,
    Hash,
    Copy,
    Clone,
    Debug,
    ToSchema
)]
#[repr(u16)]
pub enum Permission {
    /// Allows the user to log in and view their account information.
    #[serde(rename = "user.self:read")]
    UserSelfRead = 1,

    /// Allows the user to update their account information.
    #[serde(rename = "user.self:write")]
    UserSelfWrite = 2,

    /// Allows the user to view public account information of any other user.
    #[serde(rename = "user.any:read")]
    UserAnyRead = 3,

    /// Allows the user to update public account information of any other user and
    /// give or remove their permissions (but only if they have them themselves).
    #[serde(rename = "user.any:write")]
    UserAnyWrite = 4,

    /// Allows the user to create a new word.
    #[serde(rename = "word:create")]
    WordCreate = 5,

    /// Allows the user to see information about individual words
    /// and  obtain list of all of them.
    #[serde(rename = "word:read")]
    WordRead = 6,

    /// Allows the user to update an existing word's information.
    ///
    /// This includes creating, updating, and deleting meanings for an existing word).
    ///
    /// This also includes linking and unlinking categores from a word meaning.
    #[serde(rename = "word:update")]
    WordUpdate = 7,

    /// Allows the user to delete a word from the dictionary.
    #[serde(rename = "word:delete")]
    WordDelete = 8,

    /// Allows the user to create a translation relationship between
    /// an english and slovene word.
    #[serde(rename = "word.translation:create")]
    TranslationCreate = 11,

    /// Allows the user to remove a translation relationship between
    /// an english and slovene word.
    #[serde(rename = "word.translation:delete")]
    TranslationDelete = 12,

    /// Allows the user to create a category.
    #[serde(rename = "category:create")]
    CategoryCreate = 13,

    /// Allows the user to see information about individual categories
    /// and obtain a list of all of them.
    #[serde(rename = "category:read")]
    CategoryRead = 14,

    /// Allows the user to update an existing category's information.
    #[serde(rename = "category:update")]
    CategoryUpdate = 15,

    /// Allows the user to delete a category from the dictionary.
    #[serde(rename = "category:delete")]
    CategoryDelete = 16,
}


impl Permission {
    pub fn from_id(internal_permission_id: u16) -> Option<Self> {
        match internal_permission_id {
            id if id == (Self::UserSelfRead as u16) => Some(Self::UserSelfRead),
            id if id == (Self::UserSelfWrite as u16) => Some(Self::UserSelfWrite),
            id if id == (Self::UserAnyRead as u16) => Some(Self::UserAnyRead),
            id if id == (Self::UserAnyWrite as u16) => Some(Self::UserAnyWrite),
            id if id == (Self::WordCreate as u16) => Some(Self::WordCreate),
            id if id == (Self::WordRead as u16) => Some(Self::WordRead),
            id if id == (Self::WordUpdate as u16) => Some(Self::WordUpdate),
            id if id == (Self::WordDelete as u16) => Some(Self::WordDelete),
            id if id == (Self::TranslationCreate as u16) => Some(Self::TranslationCreate),
            id if id == (Self::TranslationDelete as u16) => Some(Self::TranslationDelete),
            id if id == (Self::CategoryCreate as u16) => Some(Self::CategoryCreate),
            id if id == (Self::CategoryRead as u16) => Some(Self::CategoryRead),
            id if id == (Self::CategoryUpdate as u16) => Some(Self::CategoryUpdate),
            id if id == (Self::CategoryDelete as u16) => Some(Self::CategoryDelete),
            _ => None,
        }
    }

    /// Get the internal ID of the given [`Permission`].
    /// This ID is used primarily in the database and should not be visible externally.
    pub fn id(&self) -> u16 {
        *self as u16
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
            Self::TranslationCreate => "Allows the user to translate a word.",
            Self::TranslationDelete => "Allows the user to remove a word translation.",
            Self::CategoryCreate => "Allows the user to create a word category.",
            Self::CategoryRead => "Allows the user to read categories.",
            Self::CategoryUpdate => "Allows the user to update an existing word category.",
            Self::CategoryDelete => "Allows the user to delete a word category.",
        }
    }
}

impl AsRef<Permission> for Permission {
    fn as_ref(&self) -> &Permission {
        self
    }
}


/// List of permissions that are given to **ANY API CALLER**,
/// authenticated or not.
pub const BLANKET_PERMISSION_GRANT: [Permission; 3] = [
    Permission::WordRead,
    Permission::UserAnyRead,
    Permission::CategoryRead,
];


#[derive(Debug, Error)]
pub enum FromPermissionNamesError {
    #[error("no such permission (by name): {}", .name)]
    NoSuchPermissionByName { name: String },
}


/// Set of permissions, usually associated with some user.
#[derive(Debug)]
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

    #[inline]
    pub fn from_permissions(permissions: &[Permission]) -> Self {
        Self {
            permissions: HashSet::from_iter(permissions.iter().copied()),
        }
    }

    /// Initialize a permission set from a [`HashSet`] of [`Permission`]s.
    #[inline]
    pub const fn from_permission_hash_set(permission_set: HashSet<Permission>) -> Self {
        Self {
            permissions: permission_set,
        }
    }

    /// Initialize [`PermissionSet`] given a `Vec` of permission names.
    /// Returns `Err` if a permission name doesn't resolve to a [`Permission`].
    pub fn from_permission_names<P>(
        permission_names: Vec<P>,
    ) -> Result<Self, FromPermissionNamesError>
    where
        P: AsRef<str>,
    {
        let permissions = permission_names
            .into_iter()
            .map(|permission_name| {
                Permission::from_name(permission_name.as_ref()).ok_or_else(|| {
                    FromPermissionNamesError::NoSuchPermissionByName {
                        name: permission_name.as_ref().to_string(),
                    }
                })
            })
            .collect::<Result<HashSet<_>, _>>()?;

        Ok(Self { permissions })
    }

    pub fn is_subset_of(&self, other: &Self) -> bool {
        self.set().is_subset(other.set())
    }

    /// Returns `true` if the user has the specified permission, `false` otherwise.
    ///
    /// This will also check the blanket permission grant (see `BLANKET_ANY_USER_PERMISSION_GRANT`)
    /// and return `true` regardless of the user's effective permissions (if the required permission
    /// has a blanket grant).
    pub fn has_permission_or_is_blanket_granted<P>(&self, permission: P) -> bool
    where
        P: AsRef<Permission>,
    {
        if BLANKET_PERMISSION_GRANT.contains(permission.as_ref()) {
            return true;
        }

        if self.permissions.contains(permission.as_ref()) {
            return true;
        }

        false
    }

    /// Consumes the [`PermissionSet`] and returns a raw [`HashSet`] of [`Permission`]s.
    pub fn into_permissions(self) -> HashSet<Permission> {
        self.permissions
    }

    /// Returns a reference to the set of permissions.
    pub fn set(&self) -> &HashSet<Permission> {
        &self.permissions
    }

    /// Returns a `Vec` of permission names.
    pub fn permission_names(&self) -> Vec<&'static str> {
        self.permissions
            .iter()
            .map(|permission| permission.name())
            .collect()
    }

    pub fn permission_names_cow(&self) -> Vec<Cow<'static, str>> {
        self.permissions
            .iter()
            .map(|permission| Cow::Borrowed(permission.name()))
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

        assert!(permissions.has_permission_or_is_blanket_granted(Permission::UserSelfRead));
        assert!(permissions.has_permission_or_is_blanket_granted(Permission::UserSelfWrite));
        assert!(permissions.has_permission_or_is_blanket_granted(Permission::UserAnyRead));
        assert!(permissions.has_permission_or_is_blanket_granted(Permission::UserAnyWrite));
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
