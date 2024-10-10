//! Provides ways of handling errors in API endpoint functions
//! and ways to have those errors automatically turned into correct
//! HTTP error responses when returned as `Err(error)` from those functions.

use std::borrow::{Borrow, Cow};
use std::fmt::{Display, Formatter};

use actix_http::header::{HeaderName, HeaderValue};
use actix_web::body::{BoxBody, MessageBody};
use actix_web::http::{header, StatusCode};
use actix_web::{HttpResponse, ResponseError};
use chrono::{DateTime, Utc};
use kolomoni_core::permissions::{Permission, PermissionSet};
use kolomoni_core::roles::Role;
use kolomoni_core::token::JWTCreationError;
use kolomoni_database::entities::UserQueryError;
use kolomoni_database::QueryError;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::error;
use utoipa::ToSchema;

use super::macros::construct_last_modified_header_value;
use crate::authentication::AuthenticatedUserError;



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




#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone, ToSchema)]
#[serde(tag = "type", content = "data")]
#[non_exhaustive]
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
#[cfg_attr(feature = "with_test_facilities", derive(serde::Deserialize))]
pub struct ResponseWithErrorReason {
    reason: ErrorReason,
}

impl ResponseWithErrorReason {
    #[inline]
    pub fn new(reason: ErrorReason) -> Self {
        Self { reason }
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



/// The most general endpoint handler error type.
///
/// Almost all of the endpoint handler functions tend (or should)
/// return a `Result` with the [`EndpointError`] error type.
/// The reason for this is that throughout the codebase, we've integrated
/// various functions to return either this error type, or an error type that
/// can be losslessly converted into it.
///
/// For example, [`QueryError`]s, which come from
/// the `kolomoni_database` crate, can be `?`-propagated, because we implement
/// `From<QueryError> for EndpointError`, allowing the actual database calls
/// that happen in endpoints to not worry about error conversion.
///
/// See also: [`EndpointResult`].
#[derive(Debug, Error)]
pub enum EndpointError {
    /*
     * Client errors.
     *
     * Reasons are exposed as a HTTP status code + optionally a JSON body.
     */
    /// The endpoint expected a JSON body, but there was either:
    /// - no JSON body sent with the request, or
    /// - an incorrect `Content-Type` header (expected `application/json`).
    MissingJsonBody,

    /// Invalid JSON body, either due to invalid JSON syntax,
    /// a deserialization error, or because the body is too large.
    InvalidJsonBody { reason: InvalidJsonBodyReason },

    /// Invalid UUID format encountered when trying to convert from a string.
    InvalidUuidFormat {
        #[source]
        error: uuid::Error,
    },

    /*
     * Server errors.
     *
     * Strings and boxed errors are not exposed externally.
     */
    /// Internal error with a reason string.
    ///
    /// Triggers a `500 Internal Server Error`. **The reason doesn't leak through the API**.
    InternalErrorWithReason { reason: Cow<'static, str> },

    /// Internal error, constructed from a boxed [`Error`].
    ///
    /// Triggers a `500 Internal Server Error`. **The reason doesn't leak through the API**.
    InternalGenericError {
        #[from]
        #[source]
        error: Box<dyn std::error::Error>,
    },

    /// Internal error, constructed from a database error ([`sqlx::Error`]).
    ///
    /// Triggers a `500 Internal Server Error`. **The reason doesn't leak through the API**.
    InternalDatabaseError {
        #[from]
        #[source]
        error: sqlx::Error,
    },

    /// Internal database inconsistency, with more details provided by the `problem` field.
    ///
    /// Triggers a `500 Internal Server Error`. **The reason doesn't leak through the API**.
    InvalidDatabaseState { problem: Cow<'static, str> },
}

impl EndpointError {
    /// Creates a client error specifying that a JSON body was expected (but not found).
    ///
    /// This can also happen when the `Content-Type` header isn't set properly.
    pub const fn missing_json_body() -> Self {
        Self::MissingJsonBody
    }

    /// Creates a client error specifying that a JSON body was found, but was invalid.
    ///
    /// Reasons include: invalid JSON syntax, data (schema) error when deserializing,
    /// or body size.
    pub const fn invalid_json_body(reason: InvalidJsonBodyReason) -> Self {
        Self::InvalidJsonBody { reason }
    }

    /// Creates an internal server error based on any [`Error`]-implementing type.
    ///
    /// The error will be boxed internally, but its details will not be exposed
    /// in the API response.
    #[allow(unused)]
    pub fn internal_error<E>(error: E) -> Self
    where
        E: std::error::Error + 'static,
    {
        Self::InternalGenericError {
            error: Box::new(error),
        }
    }

    /// Creates an internal server error based on a [`sqlx::Error`].
    ///
    /// The details of the error will not be exposed in the API response.
    #[allow(unused)]
    pub fn internal_database_error(error: sqlx::Error) -> Self {
        Self::InternalDatabaseError { error }
    }

    /// Creates an internal server error based on a string describing the cause.
    ///
    /// The reason is not exposed in the API response.
    #[inline]
    pub fn internal_error_with_reason<S>(reason: S) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        Self::InternalErrorWithReason {
            reason: reason.into(),
        }
    }

    /// Creates an internal server error based on a string describing
    /// how a database state is invalid or inconsistent.
    ///
    /// We tend to use this as sanity checks when performing a sequence
    /// of operation in e.g. a transaction: we fetch some data, verify that it matches,
    /// then e.g. delete the row. If we see that the deletion failed, we would consider
    /// that an invalid database state, because we were in a transaction and the row
    /// previously existed.
    ///
    /// The details of this problem will not be exposed in the API response.
    #[inline]
    pub fn invalid_database_state<S>(problem: S) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        Self::InvalidDatabaseState {
            problem: problem.into(),
        }
    }
}

impl Display for EndpointError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingJsonBody => {
                write!(f, "Expected a JSON body.")
            }
            Self::InvalidJsonBody { reason } => match reason {
                InvalidJsonBodyReason::NotJson => {
                    write!(f, "Invalid JSON body: not JSON.")
                }
                InvalidJsonBodyReason::InvalidData => {
                    write!(f, "Invalid JSON body: invalid data.")
                }
                InvalidJsonBodyReason::TooLarge => {
                    write!(f, "Invalid JSON body: too large.")
                }
            },
            Self::InvalidUuidFormat { error } => {
                write!(f, "Invalid UUID format: {}.", error)
            }
            Self::InternalErrorWithReason { reason } => write!(
                f,
                "Internal server error (with reason): {reason}."
            ),
            Self::InternalGenericError { error } => {
                write!(f, "Internal server error (generic): {error:?}")
            }
            Self::InternalDatabaseError { error } => {
                write!(
                    f,
                    "Internal server error (database error): {error}."
                )
            }
            Self::InvalidDatabaseState { problem: reason } => {
                write!(
                    f,
                    "Inconsistent internal database state: {}",
                    reason
                )
            }
        }
    }
}

impl ResponseError for EndpointError {
    /// In reality, because we implemented error_response below,
    /// this function will never be called (status codes from error_response will be used).
    /// (see [`ResponseError::status_code`]).
    fn status_code(&self) -> StatusCode {
        match self {
            Self::MissingJsonBody => StatusCode::BAD_REQUEST,
            Self::InvalidJsonBody { .. } => StatusCode::BAD_GATEWAY,
            Self::InvalidUuidFormat { .. } => StatusCode::BAD_REQUEST,
            Self::InternalErrorWithReason { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            Self::InternalGenericError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            Self::InternalDatabaseError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            Self::InvalidDatabaseState { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        // TODO Find a way to log certain types of these errors (maybe via tracing?)

        let fallibly_built_response = match self {
            Self::MissingJsonBody => EndpointResponseBuilder::bad_request()
                .with_error_reason(ErrorReason::missing_json_body())
                .build(),
            Self::InvalidJsonBody { reason } => EndpointResponseBuilder::bad_request()
                .with_error_reason(ErrorReason::invalid_json_body(*reason))
                .build(),
            Self::InvalidUuidFormat { .. } => EndpointResponseBuilder::bad_request()
                .with_error_reason(ErrorReason::invalid_uuid_format())
                .build(),
            Self::InternalErrorWithReason { .. } => {
                EndpointResponseBuilder::internal_server_error().build()
            }
            Self::InternalGenericError { .. } => {
                EndpointResponseBuilder::internal_server_error().build()
            }
            Self::InternalDatabaseError { .. } => {
                EndpointResponseBuilder::internal_server_error().build()
            }
            Self::InvalidDatabaseState { .. } => {
                EndpointResponseBuilder::internal_server_error().build()
            }
        };


        fallibly_built_response.unwrap_or_else(|_| HttpResponse::InternalServerError().finish())
    }
}


impl From<QueryError> for EndpointError {
    fn from(value: QueryError) -> Self {
        match value {
            QueryError::SqlxError { error } => Self::InternalDatabaseError { error },
            QueryError::ModelError { reason } => Self::InternalErrorWithReason { reason },
            QueryError::DatabaseInconsistencyError { problem: reason } => {
                Self::InternalErrorWithReason { reason }
            }
        }
    }
}

impl From<UserQueryError> for EndpointError {
    fn from(value: UserQueryError) -> Self {
        match value {
            UserQueryError::SqlxError { error } => Self::InternalDatabaseError { error },
            UserQueryError::ModelError { reason } => Self::InternalErrorWithReason { reason },
            UserQueryError::HasherError { error } => Self::InternalGenericError {
                error: Box::new(error),
            },
            UserQueryError::DatabaseConsistencyError { reason } => {
                Self::InternalErrorWithReason { reason }
            }
        }
    }
}

impl From<AuthenticatedUserError> for EndpointError {
    fn from(value: AuthenticatedUserError) -> Self {
        match value {
            AuthenticatedUserError::QueryError { error } => Self::from(error),
        }
    }
}

impl From<JWTCreationError> for EndpointError {
    fn from(value: JWTCreationError) -> Self {
        match value {
            JWTCreationError::JWTError { error } => Self::InternalGenericError {
                error: Box::new(error),
            },
        }
    }
}




/// An endpoint response building error
/// (returned from the [`EndpointResponseBuilder::build`] method).
#[derive(Debug, Error)]
pub enum EndpointResponseBuilderError {
    #[error("failed to serialize value as JSON")]
    JsonSerializationError {
        #[from]
        #[source]
        error: serde_json::Error,
    },
}



/// A builder for HTTP responses returned from endpoint handlers.
pub struct EndpointResponseBuilder {
    status_code: StatusCode,

    body: Option<Result<Vec<u8>, serde_json::Error>>,

    additional_headers: Vec<(HeaderName, HeaderValue)>,
}

/// Endpoint response builder initialization methods, named after
/// the status codes they initialize with.
impl EndpointResponseBuilder {
    fn new(status_code: StatusCode) -> Self {
        Self {
            status_code,
            body: None,
            additional_headers: Vec::with_capacity(1),
        }
    }

    /// Initializes a response builder with [`StatusCode::OK`].
    #[inline]
    pub fn ok() -> Self {
        Self::new(StatusCode::OK)
    }

    /// Initializes a response builder with [`StatusCode::BAD_REQUEST`].
    #[inline]
    pub fn bad_request() -> Self {
        Self::new(StatusCode::BAD_REQUEST)
    }

    /// Initializes a response builder with [`StatusCode::FORBIDDEN`].
    #[inline]
    pub fn forbidden() -> Self {
        Self::new(StatusCode::FORBIDDEN)
    }

    /// Initializes a response builder with [`StatusCode::UNAUTHORIZED`].
    #[inline]
    pub fn unauthorized() -> Self {
        Self::new(StatusCode::UNAUTHORIZED)
    }

    /// Initializes a response builder with [`StatusCode::CONFLICT`].
    #[inline]
    pub fn conflict() -> Self {
        Self::new(StatusCode::CONFLICT)
    }

    /// Initializes a response builder with [`StatusCode::NOT_FOUND`].
    #[inline]
    pub fn not_found() -> Self {
        Self::new(StatusCode::NOT_FOUND)
    }

    /// Initializes a response builder with [`StatusCode::NOT_MODIFIED`].
    #[inline]
    pub fn not_modified() -> Self {
        Self::new(StatusCode::NOT_MODIFIED)
    }

    /// Initializes a response builder with [`StatusCode::INTERNAL_SERVER_ERROR`].
    #[inline]
    pub fn internal_server_error() -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

/// Endpoint response builder response customization methods.
impl EndpointResponseBuilder {
    /// Sets the JSON body to be included in the response body.
    ///
    /// Potential serialization errors are propagated to the [`build`] method.
    ///
    ///
    /// [`build`]: Self::build
    pub fn with_json_body<D, S>(mut self, data: D) -> Self
    where
        S: Serialize,
        D: Borrow<S>,
    {
        let body = serde_json::to_vec(data.borrow());

        self.additional_headers.push((
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::APPLICATION_JSON.as_ref()),
        ));

        Self {
            status_code: self.status_code,
            body: Some(body),
            additional_headers: self.additional_headers,
        }
    }

    /// Sets the JSON body to be included in the response body
    /// to [`ResponseWithErrorReason`] with the given error reason
    /// (e.g. [`ErrorReason`], [`CategoryErrorReason`], ...).
    ///
    /// Potential serialization errors are propagated to the [`build`] method.
    ///
    ///
    /// [`build`]: Self::build
    pub fn with_error_reason<R>(self, reason: R) -> Self
    where
        R: Into<ErrorReason>,
    {
        self.with_json_body(ResponseWithErrorReason::new(reason.into()))
    }

    /// Sets the `Last-Modified` header to the specified date and time.
    pub fn with_last_modified_at(mut self, last_modified_at: &DateTime<Utc>) -> Self {
        self.additional_headers.push((
            header::LAST_MODIFIED,
            construct_last_modified_header_value(last_modified_at),
        ));

        Self {
            status_code: self.status_code,
            body: self.body,
            additional_headers: self.additional_headers,
        }
    }

    /// Finalizes the builder into a [`HttpResponse`].
    pub fn build(self) -> Result<HttpResponse<BoxBody>, EndpointError> {
        let optional_body = match self.body {
            Some(body_or_error) => match body_or_error {
                Ok(body) => Some(body),
                Err(serialization_error) => {
                    return Err(EndpointError::internal_error(serialization_error))
                }
            },
            None => None,
        };


        let mut response_builder = HttpResponse::build(self.status_code);

        for (header_name, header_value) in self.additional_headers {
            response_builder.insert_header((header_name, header_value));
        }


        match optional_body {
            Some(body) => response_builder
                .message_body(body.boxed())
                // This will, however, never produce an error (`type Error = Infallible`),
                // see <https://docs.rs/actix-web/4.9.0/actix_web/body/trait.MessageBody.html#impl-MessageBody-for-Vec%3Cu8%3E>.
                .map_err(EndpointError::internal_error),
            None => response_builder
                .message_body(().boxed())
                // This will, however, never produce an error (`type Error = Infallible`),
                // see <https://docs.rs/actix-web/4.9.0/actix_web/body/trait.MessageBody.html#impl-MessageBody-for-()>.
                .map_err(EndpointError::internal_error),
        }
    }
}




/// Short for [`Result`]`<`[`HttpResponse`]`, `[`EndpointError`]`>`, intended to be used in most
/// places in handlers of the Stari Kolomoni API.
///
/// The generic parameter (`Body`) specifies which body type is used inside [`HttpResponse`].
/// It defaults to [`BoxBody`], which is what [`EndpointResponseBuilder`] uses
/// and will likely be the most common body type.
///
/// See documentation for [`EndpointError`] for more info.
pub type EndpointResult<Body = BoxBody> = Result<HttpResponse<Body>, EndpointError>;
