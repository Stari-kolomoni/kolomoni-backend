use std::{borrow::Cow, collections::HashSet};

use serde::{Deserialize, Serialize};

use crate::permissions::{Permission, PermissionSet};



/// User roles that we have.
///
/// Roles can be assigned to users, granting them
/// all permissions associated with the role.
///
/// # Maintenance
/// **The defined roles must match with the `*_seed_roles.rs` file
/// in `kolomoni_migrations`!**
#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum Role {
    /// A normal Kolomoni user. Grants access to their own account
    /// and most read permissions.
    #[serde(rename = "user")]
    User,

    #[serde(rename = "administrator")]
    Administrator,
}

impl Role {
    /// Attempts to deserialize a [`Role`] from its internal database ID
    /// (e.g. 1).
    pub fn from_id(role_id: i32) -> Option<Self> {
        match role_id {
            1 => Some(Role::User),
            2 => Some(Role::Administrator),
            _ => None,
        }
    }

    /// Returns an internal database ID associated with the role.
    pub fn id(&self) -> i32 {
        match self {
            Role::User => 1,
            Role::Administrator => 2,
        }
    }

    /// Attempt to deserialize a [`Role`] from its lower-case name
    /// (e.g. "user").
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "user" => Some(Self::User),
            "administrator" => Some(Self::Administrator),
            _ => None,
        }
    }

    /// Returns the lower-case name associated with the role.
    pub fn name(&self) -> &'static str {
        match self {
            Role::User => "user",
            Role::Administrator => "administrator",
        }
    }

    /// Returns a description of the role.
    #[rustfmt::skip]
    pub fn description(&self) -> &'static str {
        match self {
            Role::User =>
                "Normal user with most read permissions.",
            Role::Administrator =>
                "Administrator with almost all permission, including deletions.",
        }
    }

    /// Returns a list of permissions that the role grants.
    pub fn permissions_granted(&self) -> Vec<Permission> {
        match self {
            Role::User => vec![
                Permission::UserSelfRead,
                Permission::UserSelfWrite,
                Permission::UserAnyRead,
                Permission::WordRead,
                Permission::SuggestionCreate,
            ],
            Role::Administrator => vec![
                Permission::UserAnyWrite,
                Permission::WordCreate,
                Permission::WordUpdate,
                Permission::WordDelete,
                Permission::SuggestionDelete,
                Permission::TranslationCreate,
                Permission::TranslationDelete,
                Permission::CategoryCreate,
                Permission::CategoryUpdate,
                Permission::CategoryDelete,
            ],
        }
    }
}

/// The default role given to newly-registered users.
pub const DEFAULT_USER_ROLE: Role = Role::User;


/// Set of roles, usually associated with some user.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RoleSet {
    /// Set of roles.
    roles: HashSet<Role>,
}

impl RoleSet {
    #[inline]
    pub fn new_empty() -> Self {
        Self {
            roles: HashSet::with_capacity(0),
        }
    }

    pub fn from_roles(roles: &[Role]) -> Self {
        let roles = roles.iter().copied().collect();

        Self::from_role_hash_set(roles)
    }

    /// Initialize a role set from a [`HashSet`] of [`Role`]s.
    #[inline]
    pub fn from_role_hash_set(role_set: HashSet<Role>) -> Self {
        Self { roles: role_set }
    }

    /// Checks whether the role set contains a specific role.
    pub fn has_role(&self, role: &Role) -> bool {
        self.roles.contains(role)
    }

    pub fn granted_permission_set(&self) -> PermissionSet {
        let mut permission_hash_set = HashSet::new();

        for role in self.roles.iter() {
            permission_hash_set.extend(role.permissions_granted());
        }

        PermissionSet::from_permission_hash_set(permission_hash_set)
    }

    /// Consumes the [`RoleSet`] and returns a raw [`HashSet`] of [`Role`]s.
    pub fn into_roles(self) -> HashSet<Role> {
        self.roles
    }

    /// Returns a reference to the set of roles.
    pub fn roles(&self) -> &HashSet<Role> {
        &self.roles
    }

    /// Returns a `Vec` of role names.
    pub fn role_names(&self) -> Vec<String> {
        self.roles
            .iter()
            .map(|role| role.name().to_string())
            .collect()
    }

    pub fn role_names_cow(&self) -> Vec<Cow<'static, str>> {
        self.roles
            .iter()
            .map(|role| Cow::Borrowed(role.name()))
            .collect()
    }
}
