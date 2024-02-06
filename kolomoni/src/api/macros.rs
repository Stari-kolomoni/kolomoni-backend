use actix_web::body::MessageBody;
use actix_web::HttpResponse;

/// Simple responder trait similar to `Responder` from `actix_web`.
/// The main difference is that the `into_response` method does not require
/// a reference to `HttpRequest` (i.e. the response must be built without a request).
///
/// See documentation for `impl_json_responder` for reasoning.
pub trait DumbResponder {
    type Body: MessageBody + 'static;

    /// Serialize self as JSON and return a `HTTP 200 OK` response with JSON-encoded body.  
    fn into_response(self) -> HttpResponse<Self::Body>;
}

/// Macro that implements two traits:
/// - [`actix_web::Responder`], allowing you to return this struct in an endpoint handler, and
/// - [`DumbResponder`], which is a simpler internal trait that has the `into_response` method that
///   does basically the same as [`actix_web::Responder::respond_to`], but without having to provide
///   a reference to [`HttpRequest`][actix_web::HttpRequest], making code cleaner.
///
/// The provided struct must already implement [`Serialize`][serde::Serialize].
///
///
/// # Example
/// ```
/// # use actix_web::get;
/// # use serde::Serialize;
/// # use kolomoni::impl_json_responder;
/// # use kolomoni::api::errors::EndpointResult;
/// # use kolomoni::api::macros::DumbResponder;
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
///     // What we gain is essentially this `.into_response()` method
///     // that builds the `HttpResponse` with the JSON-encoded body.
///     Ok(SomeResponse { value: 42 }.into_response())
/// }
/// ```
#[macro_export]
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

        impl $crate::api::macros::DumbResponder for $struct {
            type Body = actix_web::body::BoxBody;

            fn into_response(self) -> actix_web::HttpResponse<Self::Body> {
                actix_web::HttpResponse::Ok().json(&self)
            }
        }
    };
}

/// A macro for generating a `HttpResponse` with a given status code and
/// a JSON body containing the `reason` field.
///
/// First argument is the `actix_web::StatusCode` status code and
/// the second argument is the reason to respond with (must implement `Into<String>`).
///
/// ## Example
/// ```
/// # use actix_web::post;
/// # use actix_web::http::StatusCode;
/// # use kolomoni::api::macros::DumbResponder;
/// # use kolomoni::api::errors::EndpointResult;
/// # use kolomoni::response_with_reason;
/// #[post("/here")]
/// async fn here_endpoint() -> EndpointResult {
///     # let some_condition = true;
///     if some_condition {
///         return Ok(response_with_reason!(
///             StatusCode::CONFLICT,
///             "Here is a reason."
///         ));
///     }
///
///     # todo!();
/// }
/// ```
#[macro_export]
macro_rules! response_with_reason {
    ($status_code:expr, $reason:expr) => {
        actix_web::HttpResponseBuilder::new($status_code)
            .json($crate::api::errors::ErrorReasonResponse::custom_reason($reason))
    };
}

/// A macro that early-returns an `Err(APIError::missing_specific_permission)` if the given permissions
/// struct doesn't have the required permission. This essentially generates a `403 Forbidden`
/// with JSON-encoded reasons in the body of the response (see `APIError` for more information).
///
/// The first argument is the `UserPermissions` struct.
/// The second argument is the permission you require (`UserPermission` variant).
///
/// See documentation for `APIError` for more information.
#[macro_export]
macro_rules! require_permission {
    ($user_permissions:expr, $required_permission:expr) => {
        if !$user_permissions.has_permission($required_permission) {
            return Err(APIError::missing_specific_permission(
                $required_permission,
            ));
        }
    };
}
