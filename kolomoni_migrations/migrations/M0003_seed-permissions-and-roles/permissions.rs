/// This is the list of available permissions as of this migration.
///
/// **IMPORTANT: When still in the unstable phase, this permission list (or ones on related migrations) 
/// should be kept in sync with `kolomoni_auth/src/permissions.rs`.**
///
/// We don't keep them in sync automatically because that would mean a migration could be
/// modified without touching its directory, which would be unexpected. 
/// 
/// We can modify this sanely if any only if we're still in the unstable prototyping phase. 
/// Otherwise, opt for a new migration that adds the new permissions.
#[derive(Clone, Copy, Debug)]
pub enum StandardPermission {
    UserSelfRead,
    UserSelfWrite,
    UserAnyRead,
    UserAnyWrite,
    WordCreate,
    WordRead,
    WordUpdate,
    WordDelete,
    // SuggestionCreate,
    // SuggestionDelete,
    TranslationCreate,
    TranslationDelete,
    CategoryCreate,
    CategoryUpdate,
    CategoryDelete,
}

impl StandardPermission {
    pub fn all_permissions() -> Vec<Self> {
        vec![
            Self::UserSelfRead,
            Self::UserSelfWrite,
            Self::UserAnyRead,
            Self::UserAnyWrite,
            Self::WordCreate,
            Self::WordRead,
            Self::WordUpdate,
            Self::WordDelete,
            // Self::SuggestionCreate,
            // Self::SuggestionDelete,
            Self::TranslationCreate,
            Self::TranslationDelete,
            Self::CategoryCreate,
            Self::CategoryUpdate,
            Self::CategoryDelete
        ]
    }

    pub fn internal_id(&self) -> i32 {
        match self {
            StandardPermission::UserSelfRead => 1,
            StandardPermission::UserSelfWrite => 2,
            StandardPermission::UserAnyRead => 3,
            StandardPermission::UserAnyWrite => 4,
            StandardPermission::WordCreate => 5,
            StandardPermission::WordRead => 6,
            StandardPermission::WordUpdate => 7,
            StandardPermission::WordDelete => 8,
            // StandardPermission::SuggestionCreate => 9,
            // StandardPermission::SuggestionDelete => 10,
            StandardPermission::TranslationCreate => 11,
            StandardPermission::TranslationDelete => 12,
            StandardPermission::CategoryCreate => 13,
            StandardPermission::CategoryUpdate => 14,
            StandardPermission::CategoryDelete => 15,
        }
    }

    pub fn external_key(&self) -> &'static str {
        match self {
            StandardPermission::UserSelfRead => "user.self:read",
            StandardPermission::UserSelfWrite => "user.self:write",
            StandardPermission::UserAnyRead => "user.any:read",
            StandardPermission::UserAnyWrite => "user.any:write",
            StandardPermission::WordCreate => "word:create",
            StandardPermission::WordRead => "word:read",
            StandardPermission::WordUpdate => "word:update",
            StandardPermission::WordDelete => "word:delete",
            // StandardPermission::SuggestionCreate => "word.suggestion:create",
            // StandardPermission::SuggestionDelete => "word.suggestion:delete",
            StandardPermission::TranslationCreate => "word.translation:create",
            StandardPermission::TranslationDelete => "word.translation:delete",
            StandardPermission::CategoryCreate => "category:create",
            StandardPermission::CategoryUpdate => "category:update",
            StandardPermission::CategoryDelete => "category:delete",
        }
    }

    #[rustfmt::skip]
    pub fn english_description(&self) -> &'static str {
        match self {
            StandardPermission::UserSelfRead =>
                "Allows users to log in and view their account information.",
            StandardPermission::UserSelfWrite =>
                "Allows users to update their account information.",
            StandardPermission::UserAnyRead =>
                "Allows users to view public account information of any other user.",
            StandardPermission::UserAnyWrite =>
                "Allows users to update account information of any other user.",
            StandardPermission::WordCreate =>
                "Allows users to create words in the dictionary.",
            StandardPermission::WordRead =>
                "Allows users to read words in the dictionary.",
            StandardPermission::WordUpdate =>
                "Allows users to update existing words in the dictionary (but not delete them).",
            StandardPermission::WordDelete =>
                "Allows users to delete words from the dictionary.",
            // StandardPermission::SuggestionCreate => 
            //     "Allows the user to create a translation suggestion.",
            // StandardPermission::SuggestionDelete => 
            //     "Allows the user to remove a translation suggestion.",
            StandardPermission::TranslationCreate =>
                "Allows users to translate a word.",
            StandardPermission::TranslationDelete => 
                "Allows users to remove a word translation.",
            StandardPermission::CategoryCreate => 
                "Allows users to create a word category.",
            StandardPermission::CategoryUpdate => 
                "Allows users to update an existing word category.",
            StandardPermission::CategoryDelete => 
                "Allows users to delete a word category.",
        }
    }

    pub fn slovene_description(&self) -> &'static str {
        match self {
            StandardPermission::UserSelfRead => "Omogoča prijavo in ogled podrobnosti uporabniškega računa.",
            StandardPermission::UserSelfWrite => "Omogoča spreminjanje podrobnosti uporabniškega računa.",
            StandardPermission::UserAnyRead => "Omogoča ogled javnih podatkov ostalih uporabniških računov.",
            StandardPermission::UserAnyWrite => "Omogoča spreminjanje podrobnosti ostalih uporabniških računov.",
            StandardPermission::WordCreate => "Omogoča ustvarjanje novih besed in njenih pomenov v slovarju.",
            StandardPermission::WordRead => "Omogoča branje obstoječih besed in povezanih pomenov v slovarju.",
            StandardPermission::WordUpdate => "Omogoča spreminjanje podrobnosti obstoječih besed in povezanih pomenov v slovarju.",
            StandardPermission::WordDelete => "Omogoča brisanje besed in besednih pomenov iz slovarja.",
            // StandardPermission::SuggestionCreate => todo!(),
            // StandardPermission::SuggestionDelete => todo!(),
            StandardPermission::TranslationCreate => "Omogoča ustvarjanje prevodov.",
            StandardPermission::TranslationDelete => "Omogoča brisanje prevodov.",
            StandardPermission::CategoryCreate => "Omogoča ustvarjanje besednih kategorij.",
            StandardPermission::CategoryUpdate => "Omogoča spreminjanje podrobnosti obstoječih besednih kategorij.",
            StandardPermission::CategoryDelete => "Omogoča brisanje obstoječih besedilnih kategorij."
        }
    }
}
