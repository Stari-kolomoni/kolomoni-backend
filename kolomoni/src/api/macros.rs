use actix_web::body::{BoxBody, MessageBody};
use actix_web::http::header::{self, HeaderValue, InvalidHeaderValue};
use actix_web::http::StatusCode;
use actix_web::{http, HttpResponse, ResponseError};
use chrono::{DateTime, Utc};
use miette::{Context, IntoDiagnostic, Result};
use serde::Serialize;

use super::errors::APIError;


pub fn construct_last_modified_header_value(
    last_modification_time: &DateTime<Utc>,
) -> Result<HeaderValue, InvalidHeaderValue> {
    let date_time_formatter = last_modification_time.format("%a, %d %b %Y %H:%M:%S GMT");
    HeaderValue::from_str(date_time_formatter.to_string().as_str())
}

pub struct KolomoniResponseBuilder {
    status_code: StatusCode,
    body: String,
    additional_headers: http::header::HeaderMap,
}

impl KolomoniResponseBuilder {
    pub fn new_json<S>(value: S) -> Result<Self>
    where
        S: Serialize,
    {
        let body = serde_json::to_string(&value)
            .into_diagnostic()
            .wrap_err("Failed to serialize JSON body.")?;

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

    #[allow(dead_code)]
    pub fn status_code(mut self, status_code: StatusCode) -> Self {
        self.status_code = status_code;

        self
    }

    pub fn last_modified_at(mut self, last_modified_at: DateTime<Utc>) -> Result<Self, APIError> {
        // See <https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Last-Modified#directives>
        self.additional_headers.append(
            http::header::LAST_MODIFIED,
            construct_last_modified_header_value(&last_modified_at)
                .into_diagnostic()
                .map_err(APIError::InternalError)?,
        );

        Ok(self)
    }

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
/// See documentation for [`impl_json_responder`][crate::impl_json_responder] for reasoning.
pub trait ContextlessResponder {
    type Body: MessageBody + 'static;

    /// Serializes `self` as JSON and return a `HTTP 200 OK` response
    /// with a JSON-encoded body.  
    fn into_response(self) -> HttpResponse<Self::Body>;
}


pub(crate) fn generate_simple_http_ok_response<S>(value: S) -> HttpResponse<BoxBody>
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
        Err(_) => APIError::InternalReason("Failed to serialize value to JSON.".to_string())
            .error_response(),
    }
}


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
                $crate::api::macros::KolomoniResponseBuilder::new_json(self)
                    .map_err($crate::api::errors::APIError::InternalError)
            }
        }
    };
}

/// Macro that implements two traits for the given struct:
/// - [`actix_web::Responder`], allowing you to return this struct in an actix endpoint handler, and
/// - [`ContextlessResponder`], which is a simpler internal trait that has the `into_response` method that
///   does basically the same as [`actix_web::Responder::respond_to`], but without having to provide
///   a reference to [`HttpRequest`][actix_web::HttpRequest], making code cleaner.
///
/// The provided struct must already implement [`Serialize`][serde::Serialize].
///
///
/// # Example
/// ```
/// use actix_web::get;
/// use serde::Serialize;
/// use kolomoni::impl_json_responder;
/// use kolomoni::api::errors::EndpointResult;
/// use kolomoni::api::macros::ContextlessResponder;
///
/// #[derive(Serialize)]
/// struct SomeResponse {
///     value: i32,
/// }
///
/// impl_json_responder!(SomeResponse);
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
#[macro_export]
#[deprecated]
macro_rules! impl_json_responder {
    ($struct:ty) => {
        impl actix_web::Responder for $struct {
            type Body = actix_web::body::BoxBody;

            fn respond_to(
                self,
                _req: &actix_web::HttpRequest,
            ) -> actix_web::HttpResponse<Self::Body> {
                actix_web::HttpResponse::Ok().json(&self)
            }
        }

        impl $crate::api::macros::ContextlessResponder for $struct {
            type Body = actix_web::body::BoxBody;

            fn into_response(self) -> actix_web::HttpResponse<Self::Body> {
                actix_web::HttpResponse::Ok().json(&self)
            }
        }
    };
}

/// A macro for generating a [`HttpResponse`]
/// with a given status code and a JSON body containing the `reason` field
/// that describes the issue.
///
/// The first argument must be the [`StatusCode`][actix_web::http::StatusCode]
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
/// use kolomoni::response_with_reason;
///
/// #[post("/here")]
/// async fn here_endpoint() -> EndpointResult {
///     // ...
///     # let some_condition = true;
///
///     if some_condition {
///         return Ok(response_with_reason!(
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
macro_rules! error_response_with_reason {
    ($status_code:expr, $reason:expr) => {
        actix_web::HttpResponseBuilder::new($status_code)
            .json($crate::api::errors::ErrorReasonResponse::custom_reason($reason))
    };
}


/// A macro that takes an [`ApplicationState`][crate::state::ApplicationState]
/// and a [`UserAuth`][`crate::authentication::UserAuth`] and attempts to look up the
/// authenticated user's permissions.
///
/// The resulting expression is a tuple
/// `(`[`&JWTClaims`][kolomoni_auth::JWTClaims]`, `[`UserPermissionSet`][kolomoni_auth::UserPermissionSet]`)`.
///
///
/// # Early-return values
/// If there is no authentication, the macro early-returns a
/// `Err(`[`APIError::NotAuthenticated`][crate::api::errors::APIError::NotAuthenticated]`)`,
/// which results in a `401 Unauthorized` HTTP status code, indicating to the API caller
/// that authentication is required.
///
/// If the macro fails to look up the authenticated user's permissions, it early-returns a
/// `Err(`[`APIError::InternalError`][crate::api::errors::APIError::InternalError]`)`,
/// which results in a `500 Internal Server Error` HTTP status code, indicating that something
/// went wrong on our side, not the caller's.
///
///
/// # Examples
/// ```
/// use actix_web::get;
/// use kolomoni::state::ApplicationState;
/// use kolomoni::authentication::UserAuth;
/// use kolomoni::api::errors::{EndpointResult, APIError};
/// use kolomoni::{require_permission, require_authentication};
/// use kolomoni_auth::{UserPermissionSet, Permission, JWTClaims};
///
/// #[get("")]
/// async fn get_all_registered_users(
///     state: ApplicationState,
///     user_auth: UserAuth,
/// ) -> EndpointResult {
///     // This will ensure the user is authenticated.
///     // `permissions` will contain the user's permissions
///     // (this will perform a database lookup).
///     let (token, permissions): (JWTClaims, UserPermissionSet)
///         = require_authentication!(state, user_auth);
///
///     // This will ensure the user has the `user.any:read` permission by early-returning
///     // an `APIError::NotEnoughPermissions` if the user does not have it.
///     require_permission!(permissions, Permission::UserAnyRead);
///     
///     // ...
///     # todo!();
/// }
/// ```
#[macro_export]
macro_rules! require_authentication {
    ($state:expr, $user_auth:expr) => {
        $user_auth
            .token_and_permissions_if_authenticated(&$state.database)
            .await
            .map_err($crate::api::errors::APIError::InternalError)?
            .ok_or_else(|| $crate::api::errors::APIError::NotAuthenticated)?
    };
}


/// A macro that early-returns an
/// `Err(`[`APIError::NotEnoughPermissions`][crate::api::errors::APIError::NotEnoughPermissions]`)`
/// if the given permissions struct doesn't have the required permission.
/// This essentially generates a `403 Forbidden` with JSON-encoded reasons
/// in the body of the response (see [`APIError`][crate::api::errors::APIError] for more information).
///
/// The first argument must be the
/// [`UserPermissionSet`][kolomoni_auth::UserPermissionSet] struct.
///
/// The second argument must be the permission you require
/// (a [`Permission`][kolomoni_auth::Permission]).
#[macro_export]
macro_rules! require_permission {
    ($user_permissions:expr, $required_permission:expr) => {
        if !$user_permissions.has_permission($required_permission) {
            return Err(
                $crate::api::errors::APIError::missing_specific_permission($required_permission),
            );
        }
    };
}
