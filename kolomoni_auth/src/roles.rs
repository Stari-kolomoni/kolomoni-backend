use std::collections::HashSet;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum Role {
    #[serde(rename = "user")]
    User,

    #[serde(rename = "administrator")]
    Administrator,
}

impl Role {
    pub fn from_id(role_id: i32) -> Option<Self> {
        match role_id {
            1 => Some(Role::User),
            2 => Some(Role::Administrator),
            _ => None
        }
    }

    pub fn id(&self) -> i32 {
        match self {
            Role::User => 1,
            Role::Administrator => 2,
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "user" => Some(Self::User),
            "administrator" => Some(Self::Administrator),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Role::User => "user",
            Role::Administrator => "administrator",
        }
    }

    #[rustfmt::skip]
    pub fn description(&self) -> &'static str {
        match self {
            Role::User => 
                "Normal user with most read permissions.",
            Role::Administrator => 
                "Administrator with almost all permission, including deletions.",
        }
    }
}

/// Default role given to newly-registered users.
pub const DEFAULT_USER_ROLE: Role = Role::User;


pub struct RoleSet {
    roles: HashSet<Role>
}

impl RoleSet {
    pub fn from_role_set(role_set: HashSet<Role>) -> Self {
        Self {
            roles: role_set
        }
    }

    pub fn has_role(&self, role: &Role) -> bool {
        self.roles.contains(role)
    }

    pub fn into_roles(self) -> HashSet<Role> {
        self.roles
    }

    /// Returns a set of roles the associated user effectively has.
    pub fn roles(&self) -> &HashSet<Role> {
        &self.roles
    }

    /// Returns a set of role names the associated user effectively has.
    pub fn role_names(&self) -> Vec<String> {
        self.roles.iter()
            .map(|role| role.name().to_string())
            .collect()
    }
}
