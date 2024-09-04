use super::permissions::StandardPermission;


/// This is the list of available roles.
///
/// **IMPORTANT: When still in the unstable phase, this role list (or ones on related migrations)
/// should be kept in sync with `kolomoni_auth/src/roles.rs`.**
///
/// We don't keep them in sync automatically because that would mean a migration could be
/// modified without touching its directory, which would be unexpected.
///
/// We can modify this sanely if any only if we're still in the unstable prototyping phase.
/// Otherwise, opt for a new migration that adds the new roles.
#[derive(Clone, Copy, Debug)]
pub enum StandardRole {
    User,
    Administrator,
}

impl StandardRole {
    pub fn all_roles() -> Vec<Self> {
        vec![Self::User, Self::Administrator]
    }

    pub fn internal_id(&self) -> i32 {
        match self {
            StandardRole::User => 1,
            StandardRole::Administrator => 2,
        }
    }

    pub fn external_key(&self) -> &'static str {
        match self {
            StandardRole::User => "user",
            StandardRole::Administrator => "administrator",
        }
    }

    pub fn english_description(&self) -> &'static str {
        match self {
            StandardRole::User => "Normal user with read permissions.",
            StandardRole::Administrator => {
                "Powerful user with almost all permissions, including deletions."
            }
        }
    }

    pub fn slovene_description(&self) -> &'static str {
        match self {
            StandardRole::User => "Navaden_a uporabnica_k, ki si lahko ogleduje vsebino.",
            StandardRole::Administrator => {
                "Uporabnica_k s skoraj vsemi dovoljenji, vključno z možnostjo brisanja."
            }
        }
    }

    pub fn permission_list(&self) -> Vec<StandardPermission> {
        match self {
            StandardRole::User => vec![
                StandardPermission::UserSelfRead,
                StandardPermission::UserSelfWrite,
                StandardPermission::UserAnyRead,
                StandardPermission::WordRead,
                // StandardPermission::SuggestionCreate,
            ],
            StandardRole::Administrator => vec![
                StandardPermission::UserAnyWrite,
                StandardPermission::WordCreate,
                StandardPermission::WordUpdate,
                StandardPermission::WordDelete,
                // StandardPermission::SuggestionDelete,
                StandardPermission::TranslationCreate,
                StandardPermission::TranslationDelete,
                StandardPermission::CategoryCreate,
                StandardPermission::CategoryUpdate,
                StandardPermission::CategoryDelete,
            ],
        }
    }
}
