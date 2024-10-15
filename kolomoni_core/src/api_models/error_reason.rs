use std::borrow::Cow;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    permissions::{Permission, PermissionSet},
    roles::Role,
};



/// An [`ErrorReason`]-related trait providing a quick static description for a given error reason.
pub trait ErrorReasonName {
    fn reason_description(&self) -> &'static str;
}


/// Pertains to all endpoints under:
/// - `/dictionary/english`, and
/// - `/dictionary/slovene`
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone, ToSchema)]
#[serde(tag = "word-error-type")]
#[non_exhaustive]
pub enum WordErrorReason {
    #[serde(rename = "word-with-given-lemma-already-exists")]
    WordWithGivenLemmaAlreadyExists,

    #[serde(rename = "word-not-found")]
    WordNotFound,

    #[serde(rename = "identical-word-meaning-already-exists")]
    IdenticalWordMeaningAlreadyExists,

    #[serde(rename = "word-meaning-not-found")]
    WordMeaningNotFound,
}

impl WordErrorReason {
    /// Encountered when:
    /// - an english word with a given lemma already exists,
    /// - a slovene word with a given lemma already exists.
    pub const fn word_with_given_lemma_already_exists() -> Self {
        Self::WordWithGivenLemmaAlreadyExists
    }

    /// Encountered when:
    /// - an english word cannot be found by lemma or ID,
    /// - a slovene word cannot be found by lemma or ID.
    pub const fn word_not_found() -> Self {
        Self::WordNotFound
    }

    // TODO
    #[allow(dead_code)]
    pub const fn identical_word_meaning_already_exists() -> Self {
        Self::IdenticalWordMeaningAlreadyExists
    }

    /// Encountered when:
    /// - an english word meaning cannot be found by ID,
    /// - a slovene word meaning cannot be found by ID.
    pub const fn word_meaning_not_found() -> Self {
        Self::WordMeaningNotFound
    }
}

impl ErrorReasonName for WordErrorReason {
    fn reason_description(&self) -> &'static str {
        match self {
            Self::WordWithGivenLemmaAlreadyExists => "word with given lemma already exists",
            Self::WordNotFound => "word not found",
            Self::IdenticalWordMeaningAlreadyExists => "identical word meaning already exists",
            Self::WordMeaningNotFound => "word meaning not found",
        }
    }
}




/// Pertains to all endpoints under `/dictionary/translation`
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone, ToSchema)]
#[serde(tag = "translation-error-type")]
#[non_exhaustive]
pub enum TranslationsErrorReason {
    #[serde(rename = "english-word-meaning-not-found")]
    EnglishWordMeaningNotFound,

    #[serde(rename = "slovene-word-meaning-not-found")]
    SloveneWordMeaningNotFound,

    #[serde(rename = "translation-relationship-not-found")]
    TranslationRelationshipNotFound,

    #[serde(rename = "translation-relationship-already-exists")]
    TranslationRelationshipAlreadyExists,
}

impl TranslationsErrorReason {
    pub const fn english_word_meaning_not_found() -> Self {
        Self::EnglishWordMeaningNotFound
    }

    pub const fn slovene_word_meaning_not_found() -> Self {
        Self::SloveneWordMeaningNotFound
    }

    pub const fn translation_relationship_not_found() -> Self {
        Self::TranslationRelationshipNotFound
    }

    pub const fn translation_relationship_already_exists() -> Self {
        Self::TranslationRelationshipAlreadyExists
    }
}

impl ErrorReasonName for TranslationsErrorReason {
    fn reason_description(&self) -> &'static str {
        match self {
            Self::EnglishWordMeaningNotFound => "english word meaning not found",
            Self::SloveneWordMeaningNotFound => "slovene word meaning not found",
            Self::TranslationRelationshipNotFound => "translation relationship not found",
            Self::TranslationRelationshipAlreadyExists => "translation relationship already exists",
        }
    }
}




/// Pertains to all endpoints under `/login`.
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone, ToSchema)]
#[serde(tag = "login-error-type")]
#[non_exhaustive]
pub enum LoginErrorReason {
    #[serde(rename = "invalid-login-credentials")]
    InvalidLoginCredentials,

    #[serde(rename = "expired-refresh-token")]
    ExpiredRefreshToken,

    /// Not in the sense that is has expired or that it is *not* a refresh token,
    /// but in the sense that the given JWT couldn't be parsed or decoded.
    #[serde(rename = "invalid-refresh-json-web-token")]
    InvalidRefreshJsonWebToken,

    /// Expected a refresh token, but got an access JWT instead.
    #[serde(rename = "not-a-refresh-token")]
    NotARefreshToken,
}

impl LoginErrorReason {
    pub const fn invalid_login_credentials() -> Self {
        Self::InvalidLoginCredentials
    }

    pub const fn expired_refresh_token() -> Self {
        Self::ExpiredRefreshToken
    }

    pub const fn invalid_refresh_json_web_token() -> Self {
        Self::InvalidRefreshJsonWebToken
    }

    pub const fn not_a_refresh_token() -> Self {
        Self::NotARefreshToken
    }
}

impl ErrorReasonName for LoginErrorReason {
    fn reason_description(&self) -> &'static str {
        match self {
            Self::InvalidLoginCredentials => "invalid login credentials",
            Self::ExpiredRefreshToken => "expired refresh token",
            Self::InvalidRefreshJsonWebToken => "invalid refresh JWT",
            Self::NotARefreshToken => "not a refresh token",
        }
    }
}




/// Pertains to all endpoints under `/users`.
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone, ToSchema)]
#[serde(tag = "users-error-type")]
#[non_exhaustive]
pub enum UsersErrorReason {
    /*
     * General user-related errors
     */
    #[serde(rename = "user-not-found")]
    UserNotFound,

    /*
     * Registration errors
     */
    #[serde(rename = "username-already-exists")]
    UsernameAlreadyExists,

    /*
     * Registration / user modification errors
     */
    #[serde(rename = "display-name-already-exists")]
    DisplayNameAlreadyExists,

    /*
     * User modification errors
     */
    #[serde(rename = "cannot-modify-your-own-account")]
    CannotModifyYourOwnAccount,

    #[serde(rename = "invalid-role-name")]
    InvalidRoleName { role_name: String },

    #[serde(rename = "unable-to-give-out-unowned-role")]
    UnableToGiveOutUnownedRole {
        #[schema(value_type = String)]
        role: Role,
    },

    #[serde(rename = "unable-to-take-away-unowned-role")]
    UnableToTakeAwayUnownedRole {
        #[schema(value_type = String)]
        role: Role,
    },
}

impl UsersErrorReason {
    pub const fn user_not_found() -> Self {
        Self::UserNotFound
    }

    pub const fn username_already_exists() -> Self {
        Self::UsernameAlreadyExists
    }

    pub const fn display_name_already_exists() -> Self {
        Self::DisplayNameAlreadyExists
    }

    pub const fn cannot_modify_your_own_account() -> Self {
        Self::CannotModifyYourOwnAccount
    }

    pub fn invalid_role_name(role_name: String) -> Self {
        // To avoid resending a huge chunk of data
        // if the "wrong" role name is something large.
        if role_name.len() > 120 {
            return Self::InvalidRoleName {
                role_name: "[redacted]".to_string(),
            };
        }

        Self::InvalidRoleName { role_name }
    }

    pub const fn unable_to_give_out_unowned_role(role: Role) -> Self {
        Self::UnableToGiveOutUnownedRole { role }
    }

    pub const fn unable_to_take_away_unowned_role(role: Role) -> Self {
        Self::UnableToTakeAwayUnownedRole { role }
    }
}

impl ErrorReasonName for UsersErrorReason {
    fn reason_description(&self) -> &'static str {
        match self {
            Self::UserNotFound => "user not found",
            Self::UsernameAlreadyExists => "username already exists",
            Self::DisplayNameAlreadyExists => "display name already exists",
            Self::CannotModifyYourOwnAccount => "cannot modify your own account",
            Self::InvalidRoleName { .. } => "invalid role name",
            Self::UnableToGiveOutUnownedRole { .. } => "unable to give out unowned role",
            Self::UnableToTakeAwayUnownedRole { .. } => "unable to take away unowned role",
        }
    }
}




// TODO
/// Pertains to all endpoints under `/dictionary/category`.
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone, ToSchema)]
#[serde(tag = "category-error-type")]
#[non_exhaustive]
pub enum CategoryErrorReason {
    #[serde(rename = "category-not-found")]
    CategoryNotFound,

    /*
     * Category creation/update errors
     */
    /// This error is returned when:
    /// - attempting to create a category where the provided
    ///   slovene category name is already present on another category,
    /// - attempting to set an existing category's slovene name to
    ///   one that is already present on another category.
    #[serde(rename = "slovene-name-already-exists")]
    SloveneNameAlreadyExists,

    /// This error is returned when:
    /// - attempting to create a category where the provided
    ///   english category name is already present on another category,
    /// - attempting to set an existing category's english name to
    ///   one that is already present on another category.
    #[serde(rename = "english-name-already-exists")]
    EnglishNameAlreadyExists,

    /*
     * Category update errors
     */
    /// This error is returned when:
    /// - calling the category update endpoint with the request
    ///   body not indicating any fields to update (no fields present).
    #[serde(rename = "no-fields-to-update")]
    NoFieldsToUpdate,
}

impl CategoryErrorReason {
    pub const fn category_not_found() -> Self {
        Self::CategoryNotFound
    }

    pub const fn slovene_name_already_exists() -> Self {
        Self::SloveneNameAlreadyExists
    }

    pub const fn english_name_already_exists() -> Self {
        Self::EnglishNameAlreadyExists
    }

    pub const fn no_fields_to_update() -> Self {
        Self::NoFieldsToUpdate
    }
}

impl ErrorReasonName for CategoryErrorReason {
    fn reason_description(&self) -> &'static str {
        match self {
            Self::CategoryNotFound => "category not found",
            Self::SloveneNameAlreadyExists => "slovene name already exists",
            Self::EnglishNameAlreadyExists => "english name already exists",
            Self::NoFieldsToUpdate => "no fields to update",
        }
    }
}



/// Reasons for a JSON body to not be accepted by the server.
///
/// See also: [`EndpointError::invalid_json_body`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvalidJsonBodyReason {
    /// Indicates that the provided JSON data was invalid,
    /// possibly due to an IO / syntax / EOF error while parsing.
    #[serde(rename = "not-json")]
    NotJson,

    /// Indicates that the provided JSON data was valid,
    /// but its data did not match the expected scheme / format
    /// (deserialization error).
    #[serde(rename = "invalid-data")]
    InvalidData,

    /// Indicates that the provided JSON data was too large.
    #[serde(rename = "too-large")]
    TooLarge,
}



#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone, ToSchema)]
#[serde(tag = "type", content = "data")]
pub enum ErrorReason {
    /// Indicates that authentication is required on the endpoint,
    /// but the caller did not provide an access token.
    #[serde(rename = "missing-authentication")]
    MissingAuthentication,

    /// Indicates that a permission is required to access an endpoint,
    /// which was either not blanket granted or not one of the user's permissions.
    #[serde(rename = "missing-permissions")]
    MissingPermissions { permissions: Vec<Permission> },

    /// Indicates that the request is missing a JSON body.
    #[serde(rename = "missing-json-body")]
    MissingJsonBody,

    /// Indicates that the request has an invalid JSON body (see [`InvalidJsonBodyReason`]).
    #[serde(rename = "invalid-json-body")]
    InvalidJsonBody {
        /// Describes why the JSON body is invalid.
        #[schema(value_type = String)]
        reason: InvalidJsonBodyReason,
    },

    /// Indicates that some provided UUID parameter (in string format)
    /// was not a valid UUID.
    #[serde(rename = "invalid-uuid-format")]
    InvalidUuidFormat,

    /// Pertains to all category-related endpoints.
    #[serde(rename = "category")]
    Category(CategoryErrorReason),

    /// Pertains to all endpoints under:
    /// - `/login`
    #[serde(rename = "login")]
    Login(LoginErrorReason),

    /// Pertains to all endpoints under:
    /// - `/users`
    #[serde(rename = "users")]
    Users(UsersErrorReason),

    /// Pertains to all endpoints under:
    /// - `/dictionary/translation`
    #[serde(rename = "translations")]
    Translations(TranslationsErrorReason),

    /// Pertains to all endpoints under:
    /// - `/dictionary/english`, and
    /// - `/dictionary/slovene`
    #[serde(rename = "word")]
    Word(WordErrorReason),

    #[serde(rename = "other")]
    Other { reason: Cow<'static, str> },
}

impl ErrorReason {
    pub const fn missing_authentication() -> Self {
        Self::MissingAuthentication
    }

    pub fn missing_permission(permission: Permission) -> Self {
        Self::MissingPermissions {
            permissions: vec![permission],
        }
    }

    #[allow(dead_code)]
    pub fn missing_permissions_from_set(permission_set: &PermissionSet) -> Self {
        Self::MissingPermissions {
            permissions: permission_set.set().iter().copied().collect(),
        }
    }

    #[allow(dead_code)]
    pub fn missing_permissions_from_slice(permissions: &[Permission]) -> Self {
        Self::MissingPermissions {
            permissions: permissions.to_vec(),
        }
    }

    pub const fn missing_json_body() -> Self {
        Self::MissingJsonBody
    }

    pub const fn invalid_json_body(reason: InvalidJsonBodyReason) -> Self {
        Self::InvalidJsonBody { reason }
    }

    pub const fn invalid_uuid_format() -> Self {
        Self::InvalidUuidFormat
    }
}

impl ErrorReasonName for ErrorReason {
    fn reason_description(&self) -> &'static str {
        match self {
            Self::MissingAuthentication => "missing authentication",
            Self::MissingPermissions { .. } => "missing permissions",
            Self::MissingJsonBody => "missing JSON body",
            Self::InvalidJsonBody { .. } => "invalid JSON body",
            Self::InvalidUuidFormat => "invalid UUID format",
            Self::Category(category_error_reason) => category_error_reason.reason_description(),
            Self::Login(login_error_reason) => login_error_reason.reason_description(),
            Self::Users(users_error_reason) => users_error_reason.reason_description(),
            Self::Translations(translations_error_reason) => {
                translations_error_reason.reason_description()
            }
            Self::Word(word_error_reason) => word_error_reason.reason_description(),
            Self::Other { .. } => "other reason",
        }
    }
}

impl From<CategoryErrorReason> for ErrorReason {
    fn from(value: CategoryErrorReason) -> Self {
        Self::Category(value)
    }
}

impl From<LoginErrorReason> for ErrorReason {
    fn from(value: LoginErrorReason) -> Self {
        Self::Login(value)
    }
}

impl From<UsersErrorReason> for ErrorReason {
    fn from(value: UsersErrorReason) -> Self {
        Self::Users(value)
    }
}

impl From<TranslationsErrorReason> for ErrorReason {
    fn from(value: TranslationsErrorReason) -> Self {
        Self::Translations(value)
    }
}

impl From<WordErrorReason> for ErrorReason {
    fn from(value: WordErrorReason) -> Self {
        Self::Word(value)
    }
}




/// A JSON-serializable model containing a single field named `reason` ([`ErrorReason`]).
///
/// This type is used when responding with strongly-typed error reasons,
/// **do not use directly in endpoint code**, use e.g. [`EndpointResponseBuilder`] with
/// its [`with_error_reason`] builder method instead.
///
///
/// [`with_error_reason`]: EndpointResponseBuilder::with_error_reason
#[derive(Serialize, PartialEq, Eq, Clone, Debug, ToSchema)]
#[cfg_attr(
    feature = "serde_impls_for_client_on_models",
    derive(serde::Deserialize)
)]
pub struct ResponseWithErrorReason {
    pub reason: ErrorReason,
}

impl ResponseWithErrorReason {
    #[inline]
    pub fn new(reason: ErrorReason) -> Self {
        Self { reason }
    }
}
