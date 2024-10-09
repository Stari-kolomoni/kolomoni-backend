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
use kolomoni_auth::{JWTCreationError, Permission, PermissionSet, Role};
use kolomoni_database::entities::UserQueryError;
use kolomoni_database::QueryError;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::error;
use utoipa::ToSchema;

use super::macros::construct_last_modified_header_value;
use crate::authentication::AuthenticatedUserError;

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
    pub const fn word_with_given_lemma_already_exists() -> Self {
        Self::WordWithGivenLemmaAlreadyExists
    }

    pub const fn word_not_found() -> Self {
        Self::WordNotFound
    }

    pub const fn identical_word_meaning_already_exists() -> Self {
        Self::IdenticalWordMeaningAlreadyExists
    }

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
    /*
     * General
     */
    #[serde(rename = "missing-authentication")]
    MissingAuthentication,

    #[serde(rename = "missing-permissions")]
    MissingPermissions { permissions: Vec<Permission> },

    /// Request is missing a JSON body.
    #[serde(rename = "missing-json-body")]
    MissingJsonBody,

    #[serde(rename = "invalid-json-body")]
    InvalidJsonBody {
        #[schema(value_type = String)]
        reason: InvalidJsonBodyReason,
    },

    #[serde(rename = "invalid-uuid-format")]
    InvalidUuidFormat,

    /*
     * Category-related
     */
    #[serde(rename = "category")]
    Category(CategoryErrorReason),

    /*
     * `/login` endpoint-related
     */
    #[serde(rename = "login")]
    Login(LoginErrorReason),

    /*
     * `/users` endpoint-related
     */
    #[serde(rename = "users")]
    Users(UsersErrorReason),

    /*
     * `/dictionary/translation`-endpoint related
     */
    #[serde(rename = "translations")]
    Translations(TranslationsErrorReason),

    /// Pertains to all endpoints under:
    /// - `/dictionary/english`, and
    /// - `/dictionary/slovene`
    #[serde(rename = "word")]
    Word(WordErrorReason),

    /*
     * Other
     */
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
    pub fn missing_permissions(permission_set: &PermissionSet) -> Self {
        Self::MissingPermissions {
            permissions: permission_set.set().iter().copied().collect(),
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



/// **Do not use directly in endpoint code.**
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




#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvalidJsonBodyReason {
    /// Signals an IO / syntax / EOF error while parsing.
    #[serde(rename = "not-json")]
    NotJson,

    #[serde(rename = "invalid-data")]
    InvalidData,

    #[serde(rename = "too-large")]
    TooLarge,
}



/// General-purpose Stari Kolomoni API error type.
///
/// TODO needs documentation rework
///
/// Use this type alongside an [`EndpointResult`] return type in your actix endpoint handlers
/// to allow you to easily
/// [`?`](https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html#a-shortcut-for-propagating-errors-the--operator)-return
/// errors and automatically convert them into HTTP 4xx and 5xx errors!
/// For more details on how this works, consult the
/// [Actix documentation on errors](https://actix.rs/docs/errors) and the
/// `impl `[`ResponseError`]` for `[`APIError`] block.
///
/// <br>
///
/// # Usage examples
///
/// ## 1.1 Internal errors
/// If the function you're calling returns a [`miette::Result`], you can simply
/// map it to an [`APIError::InternalError`] and use `?` to return early if an error occurred.
/// If it returns a std-compatible [`Error`][std::error::Error], you must first call
/// [`error.into_diagnostic()`][miette::IntoDiagnostic::into_diagnostic].
///
/// ```
/// # use miette::miette;
/// # use serde::Serialize;
/// # use actix_web::get;
/// # use kolomoni::impl_json_response_builder;
/// # use kolomoni::api::errors::APIError;
/// # use kolomoni::api::macros::ContextlessResponder;
/// # use kolomoni::api::errors::EndpointResult;
/// #[derive(Serialize)]
/// struct RandomValueResponse {
///     value: i32,
/// }
///
/// impl_json_response_builder!(RandomValueResponse);
///
///
/// #[get("/some/path")]
/// async fn example_internal_error() -> EndpointResult {
///     let some_value: i32 = Result::Err(miette!("This is some error."))
///         .map_err(APIError::InternalError)?;
///     
///     println!("{}", some_value);
///     Ok(RandomValueResponse { value: some_value }.into_response())
/// }
/// ```
///
/// > Note that in this case `.map_err(APIError::InternalError)` is the same as
/// > the longer version: `.map_err(|error| APIError::InternalError(error))`.
///
/// As mentioned, returning an [`APIError`] from a [`EndpointResult`]-returning actix handler
/// will mean actix will automatically generate a relevant 4xx/5xx error, including any additional info,
/// as configured.
///
/// ---
///
/// Similarly, you can map a [`sea_orm::error::DbErr`] to [`APIError::InternalDatabaseError`].
/// If you are working with some other type of `Result`, you can do something like this instead
/// to produce an `500 Internal Server Error` on `Err`:
///
/// ```
/// # use miette::Error;
/// # use kolomoni::api::errors::{APIError, EndpointResult};
/// async fn example_string_internal_error() -> EndpointResult {
///     let some_value: i32 = Result::<i32, Error>::Ok(42)
///         .map_err(|_| APIError::internal_reason("Failed this operation!"))?;
///
///     # todo!();
/// }
/// ```
///
///
/// ## 1.2 Other errors (not found, missing permissions, etc.)
/// Just like [`APIError::internal_reason`], which returns a constructed [`APIError::InternalReason`]
/// with your message, there are other helper methods, such as:
/// - [`APIError::not_found`],
/// - [`APIError::not_found_with_reason`],
/// - [`APIError::missing_permission`],
/// - [`APIError::missing_specific_permission`], and
/// - [`APIError::missing_specific_permissions`].
///
///
/// <br>
///
/// # Full authentication example
/// When the user is not authenticated at all, you can use the
/// [`APIError::NotAuthenticated`] error variant.
///
/// What follows is a full authentication and permission example, requiring the user to
/// be authenticated and have the `user.self:read` permission.
///
/// ```
/// use actix_web::post;
/// use kolomoni::{require_permission, require_authentication};
/// use kolomoni::authentication::UserAuthenticationExtractor;
/// use kolomoni::state::ApplicationState;
/// use kolomoni_auth::Permission;
/// use kolomoni::api::errors::{APIError, EndpointResult};
///
/// #[post("/some/path")]
/// async fn example_auth(
///     state: ApplicationState,
///     authentication: UserAuthenticationExtractor,
/// ) -> EndpointResult {
///     let authenticated_user = require_authentication!(authentication);
///     require_permission!(state, authenticated_user, Permission::UserSelfRead);
///
///     // ... the rest of the function ...
///     # todo!();
/// }
/// ```
#[derive(Debug, Error)]
pub enum EndpointError {
    /*
     * Client errors.
     *
     * Reasons are exposed as a HTTP status code + optionally a JSON body.
     */
    /*
    /// User is not authenticated
    /// (missing `Authorization: Bearer <token>` HTTP header).
    NotAuthenticated,

    /// User does not have the required permission (or permissions).
    /// If `Some`, this specifies the missing permission (or permissions).
    NotEnoughPermissions {
        missing_permission: Option<Vec<Permission>>,
    },
    /// Resource could not be found. If `Some`, this describes the reason for a 404.
    NotFound {
        reason: Option<ErrorResponseWithReason>,
    },

    /// Bad client request with a reason; will produce a `400 Bad Request`.
    /// The `reason` will also be sent along in the response.
    OtherClientError { reason: Cow<'static, str> },
    */
    /* DEPRECATED
    /// This "error variant" exists on this type only for purposes of control flow,
    /// so that e.g. [`parse_uuid`] can propagate its UUID parsing error upwards,
    /// including the ability to respond with an [`ErrorReason`] body (or some other response).
    ///
    /// **This is not intended to be used directly by endpoints.**
    ///
    ///
    /// [`parse_uuid`]: crate::api::v1::dictionary::parse_uuid
    NoErrorButRespondWith { response: EndpointResponseBuilder }, */

    /*
     * Client errors.
     *
     * Specific status codes and included JSON bodies are specific to each error.
     * Avoid using this as much as possible, and use the
     *
     * ```rust,no_run
     * # fn hello_world() -> EndpointResult {
     * // ...
     * return EndpointResponseBuilder::ok()
     *      .with_error_response(todo!())
     *      .build;
     * # }
     * ```
     *
     * pattern instead.
     */
    /// The endpoint expected a JSON body, but there was either:
    /// - no JSON body sent with the request,
    /// - or there was an incorrect `Content-Type` header (expected: `application/json`).
    MissingJsonBody,

    /// Invalid JSON body, either due to a deserialization error,
    /// or because the body is too large.
    InvalidJsonBody {
        reason: InvalidJsonBodyReason,
    },

    InvalidUuidFormat {
        #[source]
        error: uuid::Error,
    },

    /*
     * Server errors.
     *
     * Reasons are not shown externally.
     */
    /// Internal error with a string reason.
    /// Triggers a `500 Internal Server Error` (**reason doesn't leak through the API**).
    InternalErrorWithReason {
        reason: Cow<'static, str>,
    },

    /// Internal error, constructed from a boxed [`Error`].
    /// Triggers a `500 Internal Server Error` (**error doesn't leak through the API**).
    InternalGenericError {
        #[from]
        #[source]
        error: Box<dyn std::error::Error>,
    },

    /// Internal error, constructed from a [`sqlx::Error`].
    /// Triggers a `500 Internal Server Error` (*doesn't leak the error through the API*).
    InternalDatabaseError {
        #[from]
        #[source]
        error: sqlx::Error,
    },

    InvalidDatabaseState {
        problem: Cow<'static, str>,
    },
}

impl EndpointError {
    /*
    /// Initialize a new not found API error without a specific reason.
    #[inline]
    pub const fn not_found() -> Self {
        Self::NotFound { reason: None }
    }

    /// Initialize a new not found API error with a specific reason.
    #[allow(dead_code)]
    #[inline]
    pub fn not_found_with_reason<M: Into<String>>(reason: M) -> Self {
        Self::NotFound {
            reason: Some(ErrorResponseWithReason::custom_reason(reason)),
        }
    }

    /// Initialize a new API error, clarifying that the user is missing
    /// a permission (or multiple permissions), but without clarification as to which those are.
    #[allow(dead_code)]
    #[inline]
    pub const fn missing_permission() -> Self {
        Self::NotEnoughPermissions {
            missing_permission: None,
        }
    }

    pub fn client_error<S>(reason: S) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        Self::OtherClientError {
            reason: reason.into(),
        }
    } */

    /*
    /// Initialize a new API error, clarifying that the user is missing
    /// some permission.
    #[inline]
    pub fn missing_specific_permission(permission: Permission) -> Self {
        Self::NotEnoughPermissions {
            missing_permission: Some(vec![permission]),
        }
    }

    /// Initialize a new API error, clarifying that the user is missing
    /// some set of permissions.
    #[inline]
    #[allow(unused)]
    pub const fn missing_specific_permissions(permissions: Vec<Permission>) -> Self {
        Self::NotEnoughPermissions {
            missing_permission: Some(permissions),
        }
    } */

    pub const fn missing_json_body() -> Self {
        Self::MissingJsonBody
    }

    pub const fn invalid_json_body(reason: InvalidJsonBodyReason) -> Self {
        Self::InvalidJsonBody { reason }
    }

    #[allow(unused)]
    pub fn internal_error<E>(error: E) -> Self
    where
        E: std::error::Error + 'static,
    {
        Self::InternalGenericError {
            error: Box::new(error),
        }
    }

    #[allow(unused)]
    pub fn internal_database_error(error: sqlx::Error) -> Self {
        Self::InternalDatabaseError { error }
    }

    /// Initialize a new internal API error using an internal reason string.
    /// When constructing an HTTP response using this error variant, the **reason
    /// is not leaked through the API.**
    #[inline]
    pub fn internal_error_with_reason<S>(reason: S) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        Self::InternalErrorWithReason {
            reason: reason.into(),
        }
    }

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




#[derive(Debug, Error)]
pub enum EndpointResponseBuilderError {
    #[error("failed to serialize value as JSON")]
    JsonSerializationError {
        #[from]
        #[source]
        error: serde_json::Error,
    },
}



pub struct EndpointResponseBuilder {
    status_code: StatusCode,

    body: Option<Result<Vec<u8>, serde_json::Error>>,

    additional_headers: Vec<(HeaderName, HeaderValue)>,
}

impl EndpointResponseBuilder {
    pub fn new(status_code: StatusCode) -> Self {
        Self {
            status_code,
            body: None,
            additional_headers: Vec::with_capacity(1),
        }
    }

    #[inline]
    pub fn ok() -> Self {
        Self::new(StatusCode::OK)
    }

    #[inline]
    pub fn bad_request() -> Self {
        Self::new(StatusCode::BAD_REQUEST)
    }

    #[inline]
    pub fn forbidden() -> Self {
        Self::new(StatusCode::FORBIDDEN)
    }

    #[inline]
    pub fn conflict() -> Self {
        Self::new(StatusCode::CONFLICT)
    }

    #[inline]
    pub fn not_found() -> Self {
        Self::new(StatusCode::NOT_FOUND)
    }

    #[inline]
    pub fn not_modified() -> Self {
        Self::new(StatusCode::NOT_MODIFIED)
    }

    #[inline]
    pub fn internal_server_error() -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR)
    }

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

    pub fn with_error_reason<R>(self, reason: R) -> Self
    where
        R: Into<ErrorReason>,
    {
        self.with_json_body(ResponseWithErrorReason::new(reason.into()))
    }

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




/// Short for [`Result`]`<`[`HttpResponse`]`, `[`APIError`]`>`, intended to be used in most
/// places in handlers of the Stari Kolomoni API.
///
/// The generic parameter (`Body`) specifies which body type is used inside [`HttpResponse`]
/// and defaults to [`BoxBody`], which is what
/// [`KolomoniResponseBuilder`][super::macros::KolomoniResponseBuilder]
/// uses and will likely be the most common body type.
///
/// See documentation for [`APIError`] for more info.
pub type EndpointResult<Body = BoxBody> = Result<HttpResponse<Body>, EndpointError>;
