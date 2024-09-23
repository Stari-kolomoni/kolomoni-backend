//! Macros to avoid repeating code (JSON response builders, authentication-related macros).

use actix_web::body::{BoxBody, MessageBody};
use actix_web::http::header::{self, HeaderValue, InvalidHeaderValue};
use actix_web::http::StatusCode;
use actix_web::{http, HttpResponse, ResponseError};
use chrono::{DateTime, Utc};
use serde::Serialize;
use thiserror::Error;

use super::errors::APIError;


/// Given a `last_modification_time`, this function tries to construct
/// a [`HeaderValue`] corresponding to the `Last-Modified` header name.
///
/// The reason this function exists is because the date and time format is a bit peculiar.
///
/// See [Last-Modified documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Last-Modified).
pub fn construct_last_modified_header_value(
    last_modification_time: &DateTime<Utc>,
) -> Result<HeaderValue, InvalidHeaderValue> {
    let date_time_formatter = last_modification_time.format("%a, %d %b %Y %H:%M:%S GMT");
    HeaderValue::from_str(date_time_formatter.to_string().as_str())
}


pub fn construct_not_modified_response(
    last_modified_at: &DateTime<Utc>,
) -> Result<HttpResponse, APIError> {
    let mut not_modified_response = HttpResponse::new(StatusCode::NOT_MODIFIED);

    not_modified_response.headers_mut().append(
        header::LAST_MODIFIED,
        construct_last_modified_header_value(last_modified_at).map_err(|_| {
            APIError::internal_error_with_reason("unable to construct Last-Modified header")
        })?,
    );

    Ok(not_modified_response)
}


#[derive(Debug, Error)]
pub enum KolomoniResponseBuilderJSONError {
    #[error("failed to encode value as JSON")]
    JsonError {
        #[from]
        #[source]
        error: serde_json::Error,
    },
}

#[derive(Debug, Error)]
pub enum KolomoniResponseBuilderLMAError {
    #[error("failed to encode Last-Modified-At header")]
    JsonError {
        #[from]
        #[source]
        error: InvalidHeaderValue,
    },
}


/// A builder struct for a HTTP response with a JSON body.
///
/// Most commonly obtained by implementing [`IntoKolomoniResponseBuilder`] on
/// a [`Serialize`]-implementing struct and calling
/// [`into_response_builder`][IntoKolomoniResponseBuilder::into_response_builder] on it.
/// **Use [`impl_json_response_builder`][crate::impl_json_response_builder]
/// instead of manually implementing this trait.**
///
/// See documentation of [`impl_json_response_builder`][crate::impl_json_response_builder] for more info.
pub struct KolomoniResponseBuilder {
    /// Status code to respond with.
    status_code: StatusCode,

    /// Serialized HTTP response body.
    body: String,

    /// Additional headers to append to the HTTP response.
    additional_headers: http::header::HeaderMap,
}

impl KolomoniResponseBuilder {
    /// Construct a new [`KolomoniResponseBuilder`] by providing
    /// a [`Serialize`]-implementing `value` (e.g. a struct).
    ///
    /// The value will be serialized as JSON and prepared to be included
    /// in the body of the HTTP response.
    pub fn new_json<S>(value: S) -> Result<Self, KolomoniResponseBuilderJSONError>
    where
        S: Serialize,
    {
        let body = serde_json::to_string(&value)?;

        let mut additional_headers = http::header::HeaderMap::with_capacity(1);
        additional_headers.append(
            http::header::CONTENT_TYPE,
            HeaderValue::from_static(mime::APPLICATION_JSON.as_ref()),
        );

        Ok(Self {
            status_code: StatusCode::OK,
            body,
            additional_headers,
        })
    }

    /// Set the response status code. If not called, this will default to `200 OK`.
    #[allow(dead_code)]
    pub fn status_code(mut self, status_code: StatusCode) -> Self {
        self.status_code = status_code;

        self
    }

    /// Set the `Last-Modified` HTTP response header to some date and time.
    /// This has no default --- the header will not be included in the response
    /// if this is not called.
    pub fn last_modified_at(
        mut self,
        last_modified_at: DateTime<Utc>,
    ) -> Result<Self, KolomoniResponseBuilderLMAError> {
        // See <https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Last-Modified#directives>
        self.additional_headers.append(
            http::header::LAST_MODIFIED,
            construct_last_modified_header_value(&last_modified_at)?,
        );

        Ok(self)
    }

    /// Build the [`HttpResponse`].
    pub fn build(self) -> HttpResponse<BoxBody> {
        self.into_response()
    }
}

impl ContextlessResponder for KolomoniResponseBuilder {
    type Body = BoxBody;

    fn into_response(self) -> HttpResponse<Self::Body> {
        let mut response =
            actix_web::HttpResponse::with_body(self.status_code, BoxBody::new(self.body));

        let response_headers = response.headers_mut();
        for (additional_header_name, additional_header_value) in self.additional_headers {
            if response_headers.contains_key(&additional_header_name) {
                response_headers.remove(&additional_header_name);
            }

            response_headers.append(additional_header_name, additional_header_value);
        }

        response
    }
}

/// A trait that allows a [`Serialize`]-implementing type
/// to be serialized as JSON body and obtain a [`KolomoniResponseBuilder`]
/// to further customize the response.
///
/// If you do not need further customization of the response and
/// are instead satisfied with a simple `200 OK` with a JSON body,
/// look at [`ContextlessResponder`] and the documentation provided in
/// [`impl_json_response_builder`][crate::impl_json_response_builder].
pub trait IntoKolomoniResponseBuilder: Serialize {
    fn into_response_builder(self) -> Result<KolomoniResponseBuilder, APIError>;
}



/// Simple responder trait (similar to [`actix_web::Responder`]).
///
/// The main difference is that our `into_response` method does not require
/// a reference to [`HttpRequest`][actix_web::HttpRequest],
/// i.e. the response must be built without a request when using this trait.
/// This can make the call signature more sensible in certain cases.
///
/// See documentation for [`impl_json_response_builder`][crate::impl_json_response_builder] for reasoning.
pub trait ContextlessResponder {
    type Body: MessageBody + 'static;

    /// Serializes `self` as JSON and return a `HTTP 200 OK` response
    /// with a JSON-encoded body.  
    fn into_response(self) -> HttpResponse<Self::Body>;
}


/// Generates a simple HTTP `200 OK` response. The body
/// will contain the `value` serialized as JSON.
///
/// If the value can not be serialized due to some error,
/// an HTTP `500 Internal Server Error` is generated.
pub fn generate_simple_http_ok_response<S>(value: S) -> HttpResponse<BoxBody>
where
    S: Serialize,
{
    match serde_json::to_vec(&value) {
        Ok(serialized_value) => {
            let mut response =
                HttpResponse::with_body(StatusCode::OK, BoxBody::new(serialized_value));

            let response_headers = response.headers_mut();
            response_headers.append(
                header::CONTENT_TYPE,
                HeaderValue::from_static(mime::APPLICATION_JSON.as_ref()),
            );

            response
        }
        Err(_) => APIError::internal_error_with_reason("Failed to serialize value to JSON.")
            .error_response(),
    }
}


#[macro_export]
macro_rules! obtain_database_connection {
    ($state:expr) => {
        $state.database.acquire().await?
    };
}



/// A macro that, given some struct type, implements the following two traits on it:
/// - [`ContextlessResponder`], allowing you to make a struct instance and call
///   [`into_response`][ContextlessResponder::into_response] on it, turning it into a
///   [`HttpResponse`] that just returns a `200 OK` with the struct serialized as JSON.
/// - [`IntoKolomoniResponseBuilder`], which is similar to [`ContextlessResponder`], but allows for more advanced operations
///   compared to above. As a user of a struct with this impl you need to call
///   [`.into_response_builder()`][IntoKolomoniResponseBuilder::into_response_builder] to get the builder.
///   You may then set the last modification header and other attributes.
///   Finally, to turn the builder into a response, call [`into_response`][ContextlessResponder::into_response].
///
/// # Example
///
/// ## Simple `200 OK` response with JSON-encoded body
/// ```
/// use actix_web::get;
/// use serde::Serialize;
/// use kolomoni::impl_json_response_builder;
/// use kolomoni::api::errors::EndpointResult;
/// use kolomoni::api::macros::ContextlessResponder;
///
/// #[derive(Serialize)]
/// struct SomeResponse {
///     value: i32,
/// }
///
/// impl_json_response_builder!(SomeResponse);
///
///
/// #[get("/some/path")]
/// async fn example_handler() -> EndpointResult {
///     // ...
///     
///     Ok(SomeResponse { value: 42 }.into_response())
///     //                           ^^^^^^^^^^^^^^^^
///     // By calling the implementor macro we gained the ability to call
///     // the `into_response` method, allowing us to ergonomically build
///     // an HTTP response with a JSON-encoded body.
/// }
/// ```
///
/// ## Advanced response with custom status code and other attributes
/// ```
/// use actix_web::get;
/// use actix_web::http::StatusCode;
/// use serde::Serialize;
/// use chrono::Utc;
/// use kolomoni::impl_json_response_builder;
/// use kolomoni::api::errors::EndpointResult;
/// use kolomoni::api::macros::{
///     IntoKolomoniResponseBuilder,
///     ContextlessResponder
/// };
///
/// #[derive(Serialize)]
/// struct SomeResponse {
///     value: i32,
/// }
///
/// impl_json_response_builder!(SomeResponse);
///
///
/// #[get("/some/path")]
/// async fn example_handler() -> EndpointResult {
///     // ...
///     
///     Ok(
///         SomeResponse { value: 42 }
///             .into_response_builder()?
///     //       ^^^^^^^^^^^^^^^^^^^^^
///     // By calling the implementor macro we gained the ability to call
///     // the `into_response_builder` method, allowing us to ergonomically build
///     // an HTTP response with a JSON-encoded body.
///             .status_code(StatusCode::IM_A_TEAPOT)
///             .last_modified_at(Utc::now())?
///     //       ^^^^^^^^^^^^^^^^
///     // We can use the methods on `KolomoniResponseBuilder` to specify
///     // advanced parameters, such as a different status code or a `Last-Modified` header.
///             .into_response()
///     //       ^^^^^^^^^^^^^
///     // Finally, call `into_response` to build a HTTP response you can return from the function.
///     )
/// }
/// ```
#[macro_export]
macro_rules! impl_json_response_builder {
    ($struct:ty) => {
        impl $crate::api::macros::ContextlessResponder for $struct {
            type Body = actix_web::body::BoxBody;

            fn into_response(self) -> actix_web::HttpResponse<Self::Body> {
                $crate::api::macros::generate_simple_http_ok_response(self)
            }
        }

        impl $crate::api::macros::IntoKolomoniResponseBuilder for $struct {
            fn into_response_builder(
                self,
            ) -> Result<$crate::api::macros::KolomoniResponseBuilder, $crate::api::errors::APIError>
            {
                Ok($crate::api::macros::KolomoniResponseBuilder::new_json(self)?)
            }
        }
    };
}



/// A macro for generating a [`HttpResponse`]
/// with a given status code and a JSON body containing the `reason` field
/// that describes the issue.
///
/// The first argument must be the [`StatusCode`]
/// to use in the response.
///
/// The second argument must be the value of the `reason` field to include.
/// The provided expression does not need to be a `String`; it must, however, implement `Into<String>`.
///
/// ## Example
/// ```
/// use actix_web::post;
/// use actix_web::http::StatusCode;
/// use kolomoni::api::macros::ContextlessResponder;
/// use kolomoni::api::errors::EndpointResult;
/// use kolomoni::error_response_with_reason;
///
/// #[post("/here")]
/// async fn here_endpoint() -> EndpointResult {
///     // ...
///     # let some_condition = true;
///
///     if some_condition {
///         return Ok(error_response_with_reason!(
///             StatusCode::CONFLICT,
///             "Here is a reason."
///         ));
///     }
///     
///     // ...
///     # todo!();
/// }
/// ```
#[macro_export]
macro_rules! json_error_response_with_reason {
    ($status_code:expr, $reason:expr) => {
        actix_web::HttpResponseBuilder::new($status_code)
            .json($crate::api::errors::ErrorReasonResponse::custom_reason($reason))
    };
}



/// A macro that takes a [`UserAuthenticationExtractor`][`crate::authentication::UserAuthenticationExtractor`]
/// and verifies that the user did authenticate on the request.
///
/// The resulting expression, which you can, for example, assign to a variable,
/// is of type [`AuthenticatedUser`][crate::authentication::AuthenticatedUser].
///
/// Note that calling this will not perform any database lookups yet.
///
///
/// # Early-return values
/// If there is no authentication provided in the request, this macro early-returns a
/// `Err(`[`APIError::NotAuthenticated`]`)`
/// from the caller function. This results in a `401 Unauthorized` HTTP response,
/// indicating to the API caller that authentication is required on the endpoint.
///
///
/// # Example
/// ```
/// use actix_web::get;
/// use kolomoni::state::ApplicationState;
/// use kolomoni::authentication::{
///     AuthenticatedUser,
///     UserAuthenticationExtractor
/// };
/// use kolomoni::api::errors::{EndpointResult, APIError};
/// use kolomoni::{require_permission, require_authentication};
/// use kolomoni_auth::Permission;
///
/// #[get("")]
/// async fn get_all_registered_users(
///     state: ApplicationState,
///     authentication: UserAuthenticationExtractor,
/// ) -> EndpointResult {
///     // This will ensure the user provided a valid authentication token
///     // with the request.
///     let authenticated_user: AuthenticatedUser = require_authentication!(authentication);
///
///     // The following will ensure the user has the `user.any:read` permission by early-returning
///     // an `APIError::NotEnoughPermissions` if the user does not have it.
///     // This call performs a database lookup (*the previous one did not*).
///     require_permission!(state, authenticated_user, Permission::UserAnyRead);
///     
///     // ...
///     # todo!();
/// }
/// ```
#[macro_export]
macro_rules! require_user_authentication {
    ($user_auth_extractor:expr) => {
        $user_auth_extractor
            .authenticated_user()
            .ok_or($crate::api::errors::APIError::NotAuthenticated)?
    };
}


#[macro_export]
macro_rules! require_permission_with_optional_authentication {
    ($database_connection:expr, $user_auth_extractor:expr, $permission:expr) => {
        match $user_auth_extractor.authenticated_user() {
            Some(_authenticated_user) => {
                if !_authenticated_user
                    .transitively_has_permission($database_connection, $permission)
                    .await?
                {
                    return Err(
                        $crate::api::errors::APIError::missing_specific_permission($permission),
                    );
                }

                Some(_authenticated_user)
            }
            None => {
                if !$user_auth_extractor.is_permission_granted_to_all($permission) {
                    return Err(
                        $crate::api::errors::APIError::missing_specific_permission($permission),
                    );
                }

                None
            }
        }
    };
}



/// A macro that early-returns an
/// `Err(`[`APIError::NotEnoughPermissions`]`)`
/// if the user doesn't have the required permission.
///
/// The early return essentially generates a `403 Forbidden` with a JSON-encoded reason
/// in the body of the response (see [`APIError`] for more information).
///
/// # Arguments and examples
/// ## Variant 1 (three arguments, most common)
/// - The first argument must be somethings that derefs to [`PgConnection`][sqlx::PgConnection]
///   (e.g. [`PoolConnection`][sqlx::PoolConnection]).
/// - The second argument must be an
///   [`AuthenticatedUser`][crate::authentication::AuthenticatedUser] instance, which is usually obtained
///   by calling the [`require_authentication!`][crate::api::macros::require_authentication] macro.
/// - The third argument must be the [`Permission`][kolomoni_auth::Permission] you want to ensure the user has.
///
/// ```
/// use actix_web::get;
/// use kolomoni::state::ApplicationState;
/// use kolomoni::authentication::{
///     AuthenticatedUser,
///     UserAuthenticationExtractor
/// };
/// use kolomoni::api::errors::{EndpointResult, APIError};
/// use kolomoni::{require_permission, require_authentication};
/// use kolomoni_auth::Permission;
///
/// #[get("")]
/// async fn get_all_registered_users(
///     state: ApplicationState,
///     authentication: UserAuthenticationExtractor,
/// ) -> EndpointResult {
///     // This will ensure the user provided a valid authentication token
///     // with the request.
///     let authenticated_user: AuthenticatedUser = require_authentication!(authentication);
///
///     // The following will ensure the user has the `user.any:read` permission by early-returning
///     // an `APIError::NotEnoughPermissions` if the user does not have it.
///     // This call performs a database lookup (*the previous one did not*).
///     require_permission!(state, authenticated_user, Permission::UserAnyRead);
///     
///     // ...
///     # todo!();
/// }
/// ```
///
/// ## Variant 2 (two arguments)
/// This variant comes in handy when you already have a set of user permissions
/// ([`PermissionSet`][kolomoni_auth::PermissionSet]) and wish to assert that
/// the user has a certain permission by looking in that set instead of
/// going to the database again.
///
/// - The first argument must be the [`PermissionSet`][kolomoni_auth::PermissionSet].
/// - THe second argument must be the [`Permission`][kolomoni_auth::Permission]
///   you wish to check for.
///
/// ```
/// use actix_web::get;
/// use kolomoni::state::ApplicationState;
/// use kolomoni::authentication::{
///     AuthenticatedUser,
///     UserAuthenticationExtractor
/// };
/// use kolomoni::api::errors::{EndpointResult, APIError};
/// use kolomoni::{require_permission, require_authentication};
/// use kolomoni_auth::{Permission, PermissionSet};
///
/// #[get("")]
/// async fn get_all_registered_users(
///     state: ApplicationState,
///     authentication: UserAuthenticationExtractor,
/// ) -> EndpointResult {
///     // This will ensure the user provided a valid authentication token
///     // with the request.
///     let authenticated_user: AuthenticatedUser
///         = require_authentication!(authentication);
///     let authenticated_user_permissions: PermissionSet
///         = authenticated_user
///             .permissions(&state.database)
///             .await
///             .map_err(APIError::InternalError)?;
///
///     // The following will ensure the user has the `user.any:read` permission by early-returning
///     // an `APIError::NotEnoughPermissions` if the user does not have it.
///     // Unlike the three-argument variant, this call does not perform a database lookup.
///     require_permission!(authenticated_user_permissions, Permission::UserAnyRead);
///     
///     // ...
///     # todo!();
/// }
/// ```
///
#[macro_export]
#[deprecated]
macro_rules! require_permission_OLD {
    ($permission_set:expr, $required_permission:expr) => {
        if !$permission_set.has_permission($required_permission) {
            return Err(
                $crate::api::errors::APIError::missing_specific_permission($required_permission),
            );
        }
    };

    ($database_connection:expr, $authenticated_user:expr, $required_permission:expr) => {
        if !$authenticated_user
            .transitively_has_permission($database_connection, $required_permission)
            .await?
        {
            return Err(
                $crate::api::errors::APIError::missing_specific_permission($required_permission),
            );
        }
    };
}


#[macro_export]
macro_rules! require_permission_in_set {
    ($permission_set:expr, $required_permission:expr) => {
        if !$permission_set.has_permission($required_permission) {
            return Err(
                $crate::api::errors::APIError::missing_specific_permission($required_permission),
            );
        }
    };
}


#[macro_export]
macro_rules! require_permission_on_user {
    ($database_connection:expr, $authenticated_user:expr, $required_permission:expr) => {{
        if !$authenticated_user
            .transitively_has_permission($database_connection, $required_permission)
            .await?
        {
            return Err(
                $crate::api::errors::APIError::missing_specific_permission($required_permission),
            );
        }

        $authenticated_user
    }};
}

#[macro_export]
macro_rules! require_user_authentication_and_permission {
    ($database_connection:expr, $authentication_extractor:expr, $required_permission:expr) => {{
        let __authenticated_user = $crate::require_user_authentication!($authentication_extractor);

        $crate::require_permission_on_user!(
            $database_connection,
            __authenticated_user,
            $required_permission
        )
    }};
}

#[deprecated]
#[macro_export]
macro_rules! require_permissionOLD2 {
    ($database_connection:expr, on authentication extractor $authentication_extractor:expr, $required_permission:expr) => {
        if let Some(authenticated_user) = $authentication_extractor.authenticated_user() {
            require_permission!(
                $database_connection,
                authenticated_user,
                $required_permission
            )
        } else {
            if !$authentication_extractor.is_permission_granted_to_all($required_permission) {
                return Err(
                    $crate::api::errors::APIError::missing_specific_permission($required_permission),
                );
            }
        }

        if !$authentication_extractor
            .transitively_has_permission($database_connection, $required_permission)
            .await?
        {
            return Err(
                $crate::api::errors::APIError::missing_specific_permission($required_permission),
            );
        }
    };

    ($database_connection:expr, on user $authenticated_user:expr, $required_permission:expr) => {
        if !$authenticated_user
            .transitively_has_permission($database_connection, $required_permission)
            .await?
        {
            return Err(
                $crate::api::errors::APIError::missing_specific_permission($required_permission),
            );
        }
    };
}
