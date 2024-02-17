//! Defines commonly used OpenAPI parameters and responses
//! to be used in conjunction with the [`utiopa::path`][utoipa::path] proc macro on actix handlers.

use std::{collections::BTreeMap, marker::PhantomData};

use utoipa::{
    openapi::{
        example::ExampleBuilder,
        ContentBuilder,
        Ref,
        RefOr,
        Response,
        ResponseBuilder,
        ResponsesBuilder,
    },
    ToSchema,
};

use super::errors::ErrorReasonResponse;


/// A "required permission" trait.
/// Relates to [`FailedAuthenticationResponses`].
pub trait RequiredPermission {
    fn name() -> &'static str;
}

/// Given a variant name for [`Permission`][kolomoni_auth::Permission], this macro will generate
/// an empty struct with the name `RequiredPermissionNameHere`.
///
/// For example, calling `generate_standalone_requirement_struct!(UserSelfRead)`
/// will result in the following code:
///
/// ```
/// # use kolomoni::api::openapi::RequiredPermission;
/// # use kolomoni_auth::Permission;
/// pub struct RequiresUserSelfRead;
/// impl RequiredPermission for RequiresUserSelfRead {
///     fn name() -> &'static str {
///         Permission::UserSelfRead.name()
///     }
/// }
/// ```
macro_rules! generate_standalone_requirement_struct {
    ($permission_variant:ident) => {
        ::paste::paste! {
            #[doc = concat!(
                "Corresponds to the [`Permission::",
                stringify!($permission_variant),
                "`][kolomoni_auth::Permission::",
                stringify!($permission_variant),
                "] permission.")
            ]
            #[doc =
                "Use in conjunction with [`FailedAuthenticationResponses`][crate::api::openapi::FailedAuthenticationResponses] \
                to indicate that the permission is required."
            ]
            #[doc = ""]
            #[doc =
                "See documentation on [`FailedAuthenticationResponses`][crate::api::openapi::FailedAuthenticationResponses] \
                for more information on usage."
            ]
            pub struct [< Requires $permission_variant >];
            impl RequiredPermission for [< Requires $permission_variant >] {
                fn name() -> &'static str {
                    kolomoni_auth::Permission::$permission_variant.name()
                }
            }
        }
    };
}

// The macro calls below generate empty structs for all available permissions,
// making them usable as a parameter for the [`FailedAuthenticationResponses`] generic.

generate_standalone_requirement_struct!(UserSelfRead);
generate_standalone_requirement_struct!(UserSelfWrite);
generate_standalone_requirement_struct!(UserAnyRead);
generate_standalone_requirement_struct!(UserAnyWrite);
generate_standalone_requirement_struct!(WordCreate);
generate_standalone_requirement_struct!(WordRead);
generate_standalone_requirement_struct!(WordUpdate);
generate_standalone_requirement_struct!(WordDelete);



/// A `utoipa` endpoint response for when an endpoint requires authentication and some permission.
///
/// Specifying [`FailedAuthenticationResponses`]`<`[`RequiresUserSelfRead`]`>` semantically means that:
/// - Your endpoint function requires the user to provide an authentication token in the request,
///   and that it will return a `401 Unauthorized` response if not.
/// - Your endpoint function requires the user to have the `user.self:read` permission,
///   and that it will return a `403 Forbidden` response if not.
///
/// **It is, however, up to your function to ensure this happens. Adding this annotation only means
/// that the above will appear in the OpenAPI documentation.**
///
///
/// <br>
///
/// # Example
/// ```no_run
/// use actix_web::get;
/// use kolomoni::api::openapi;
/// use kolomoni::api::errors::EndpointResult;
///
/// #[utoipa::path(
///     get,
///     path = "/hello-world",
///     responses(
///         openapi::FailedAuthenticationResponses<openapi::RequiresUserSelfRead>
///     )
/// )]
/// #[get("/hello-world")]
/// pub async fn some_endpoint_function() -> EndpointResult {
///     // This route requires the `user.self:read` permission
///     // (which means it also requires authentication in general)!
///
///     // ... and so on ...
///     # todo!();
/// }
/// ```
///
/// The above is basically equivalent to specifying the following manual responses:
///
/// ```no_run
/// # use actix_web::get;
/// # use kolomoni::api::openapi;
/// # use kolomoni::api::errors::{EndpointResult, ErrorReasonResponse};
/// #[utoipa::path(
///     get,
///     path = "/hello-world",
///     responses(
///         (
///             status = 401,
///             description = "Missing authentication. Include an `Authorization: Bearer <token>` \
///                            header with your request to access this endpoint."
///         ),
///         (
///             status = 403,
///             description = "Missing the `user.self:read` permission.",
///             content_type = "application/json",
///             body = ErrorReasonResponse,
///             example = json!({ "reason": "Missing permission: user.self:read." })
///         ),
///     )
/// )]
/// #[get("/hello-world")]
/// pub async fn some_endpoint_function() -> EndpointResult {
///     // This route requires the `user.self:read` permission
///     // (which means it also requires authentication in general)!
///
///     // ... and so on ...
///     # todo!();
/// }
/// ```
///
/// [FailedAuthenticationResponses]: crate::api::openapi::FailedAuthenticationResponses
/// [RequiresUserSelfRead]: crate::api::openapi::RequiresUserSelfRead
pub struct FailedAuthenticationResponses<P: RequiredPermission> {
    _marker: PhantomData<P>,
}

impl<P: RequiredPermission> utoipa::IntoResponses for FailedAuthenticationResponses<P> {
    fn responses() -> BTreeMap<String, RefOr<Response>> {
        let missing_user_auth_response = ResponseBuilder::new()
            .description(
                "Missing user authentication, provide an `Authorization: Bearer your_token_here` header."
            )
            .build();

        let missing_user_permission_decription = format!("Missing the `{}` permission.", P::name());

        let mut missing_user_permission_example = serde_json::Map::with_capacity(1);
        missing_user_permission_example.insert(
            "reason".to_string(),
            serde_json::Value::String(format!("Missing permission: {}.", P::name())),
        );

        let missing_user_permission_response = ResponseBuilder::new()
            .description(missing_user_permission_decription)
            .content(
                mime::APPLICATION_JSON.to_string(),
                ContentBuilder::new()
                    .examples_from_iter(vec![(
                        "Missing permissions.",
                        ExampleBuilder::new()
                            .value(Some(serde_json::Value::Object(
                                missing_user_permission_example,
                            )))
                            .build(),
                    )])
                    .schema(RefOr::Ref(Ref::from_schema_name(
                        ErrorReasonResponse::schema().0,
                    )))
                    .build(),
            )
            .build();

        ResponsesBuilder::new()
            .response("401", missing_user_auth_response)
            .response("403", missing_user_permission_response)
            .build()
            .into()
    }
}


/// A `utoipa` endpoint response for when an endpoint may return
/// a `304 Not Modified` HTTP response indicating that the resource did not change.
///
/// **However: as with all other structures in this module it is fully up to
/// your function to ensure this can happen. Adding this annotation only means
/// that the above will appear in the OpenAPI documentation.**
///
/// # Example
/// ```
/// use actix_web::{get, http::{header, StatusCode}};
/// use actix_web::HttpResponse;
/// use chrono::Utc;
/// use miette::IntoDiagnostic;
/// use kolomoni::api::openapi;
/// use kolomoni::api::errors::{
///     APIError,
///     EndpointResult,
///     ErrorReasonResponse
/// };
/// use kolomoni::api::macros::construct_last_modified_header_value;
///
/// #[utoipa::path(
///     get,
///     path = "/hello-world",
///     responses(
///         openapi::UnmodifiedConditionalResponse,
///     )
/// )]
/// #[get("/hello-world")]
/// pub async fn some_endpoint_function() -> EndpointResult {
///     # let unmodified = false;
///     # let some_modification_time = Utc::now();
///     if unmodified {
///         let mut response = HttpResponse::new(StatusCode::NOT_MODIFIED);
///         
///         response.headers_mut().append(
///             header::LAST_MODIFIED,
///             construct_last_modified_header_value(&some_modification_time)
///                 .into_diagnostic()
///                 .map_err(APIError::InternalError)?
///         );
///
///         return Ok(response);
///     }
///
///     // ... and so on ...
///     # todo!();
/// }
/// ```
///
/// The above is basically equivalent to specifying the following manual responses:
///
/// ```
/// # use actix_web::{get, http::{header, StatusCode}};
/// # use actix_web::HttpResponse;
/// # use chrono::Utc;
/// # use miette::IntoDiagnostic;
/// # use kolomoni::api::openapi;
/// # use kolomoni::api::errors::{
/// #     APIError,
/// #     EndpointResult,
/// #     ErrorReasonResponse
/// # };
/// # use kolomoni::api::macros::construct_last_modified_header_value;
///
/// #[utoipa::path(
///     get,
///     path = "/hello-world",
///     responses(
///         (
///             status = 304,
///             description =
///                 "Resource hasn't been modified since the timestamp specified \
///                 in the `If-Modified-Since` header. As such, this status code \
///                 can only be returned if that header is provided in the request."
///         ),
///     )
/// )]
/// #[get("/hello-world")]
/// pub async fn some_endpoint_function() -> EndpointResult {
///     # let unmodified = false;
///     # let some_modification_time = Utc::now();
///     if unmodified {
///         // ...
///         # todo!();
///     }
///
///     // ... and so on ...
///     # todo!();
/// }
/// ```
///
pub struct UnmodifiedConditionalResponse;

impl utoipa::IntoResponses for UnmodifiedConditionalResponse {
    fn responses() -> BTreeMap<String, utoipa::openapi::RefOr<utoipa::openapi::response::Response>> {
        let unmodified_data_response = ResponseBuilder::new()
            .description(
                "Resource hasn't been modified since the timestamp specified in the `If-Modified-Since` header. \
                As such, this status code can only be returned if that header is provided in the request."
            )
            .build();

        ResponsesBuilder::new()
            .response("304", unmodified_data_response)
            .build()
            .into()
    }
}



/// A `utoipa` endpoint response for when and endpoint may return a `500 Internal Server Error` HTTP response
/// indicating that something went wrong internally.
///
/// This should be present on basically all routes, as even most extractors
/// can cause this to happen.
///
/// # Example
/// ```
/// use actix_web::get;
/// use actix_web::HttpResponse;
/// use kolomoni::state::ApplicationState;
/// use kolomoni::api::openapi;
/// use kolomoni::api::errors::{APIError, EndpointResult};
/// use kolomoni_database::query::UserQuery;
///
/// #[utoipa::path(
///     get,
///     path = "/hello-world",
///     responses(
///         (
///             status = 404,
///             description = "The user could not be found."
///         ),
///         openapi::InternalServerErrorResponse,
///     )
/// )]
/// #[get("/hello-world")]
/// pub async fn some_endpoint_function(
///     state: ApplicationState,
/// ) -> EndpointResult {
///     # let user_id: i32 = 1;
///     let user_data = UserQuery::get_user_by_id(
///         &state.database,
///         user_id
///     )
///         .await
///         .map_err(APIError::InternalError)?
///     //           ^^^^^^^^^^^^^^^^^^^^^^^
///     // The query above can cause a database error, which we map
///     // into an `APIError::InternalError`. When this error is
///     // returned, the error is automatically converted into
///     // a `500 Internal Server Error` response.
///     //
///     // As such, we can annotate our endpoint with `InternalServerErrorResponse`
///     // like this is done above to make the OpenAPI schema correctly list
///     // it as a possible response.
///         .ok_or_else(APIError::not_found)?;
///
///     // ... and so on
///     # todo!();
/// }
/// ```
pub struct InternalServerErrorResponse;

impl utoipa::IntoResponses for InternalServerErrorResponse {
    fn responses() -> BTreeMap<String, utoipa::openapi::RefOr<utoipa::openapi::response::Response>> {
        let internal_error_response = ResponseBuilder::new()
            .description("Internal server error.")
            .build();

        ResponsesBuilder::new()
            .response("500", internal_error_response)
            .build()
            .into()
    }
}



/// A `utoipa` endpoint parameter for when an endpoint supports specifying
/// the [`If-Modified-Since` header](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/If-Modified-Since).
///
/// For a real-life example, see the [`get_current_user_info`][crate::api::v1::users::current::get_current_user_info]
/// endpoint function.
///
/// # Example
/// This example uses the `If-Modified-Since` extractor, see
/// [`OptionalIfModifiedSince`][crate::api::OptionalIfModifiedSince]
/// for more info.
///
/// ```
/// use miette::IntoDiagnostic;
/// use actix_web::{get, http::{StatusCode, header}};
/// use actix_web::HttpResponse;
/// use kolomoni::state::ApplicationState;
/// use kolomoni::api::OptionalIfModifiedSince;
/// use kolomoni::api::openapi;
/// use kolomoni::api::errors::{APIError, EndpointResult};
/// use kolomoni::api::macros::construct_last_modified_header_value;
///
/// #[utoipa::path(
///     get,
///     path = "/hello-world",
///     params(
///         openapi::IfModifiedSinceParameter,
///     ),
///     responses(
///         openapi::InternalServerErrorResponse,
///     )
/// )]
/// #[get("/hello-world")]
/// pub async fn some_endpoint_function(
///     state: ApplicationState,
///     if_modified_since: OptionalIfModifiedSince,
/// ) -> EndpointResult {
///     # let last_modification_time = chrono::Utc::now();
///     // ...
///
///     if if_modified_since.has_not_changed_since(&last_modification_time) {
///         let mut unchanged_response = HttpResponse::new(StatusCode::NOT_MODIFIED);
///
///         unchanged_response
///             .headers_mut()
///             .append(
///                 header::LAST_MODIFIED,
///                 construct_last_modified_header_value(&last_modification_time)
///                     .into_diagnostic()
///                     .map_err(APIError::InternalError)?,
///             );
///         
///         return Ok(unchanged_response);
///     }
///
///     // ... and so on
///     # todo!();
/// }
/// ```
pub struct IfModifiedSinceParameter;

impl utoipa::IntoParams for IfModifiedSinceParameter {
    fn into_params(
        _parameter_in_provider: impl Fn() -> Option<utoipa::openapi::path::ParameterIn>,
    ) -> Vec<utoipa::openapi::path::Parameter> {
        let description
            = "If specified, this header makes the server return `304 Not Modified` without \
              content (instead of `200 OK` with the usual response) if the requested data \
              hasn't changed since the specified timestamp.\n\n See \
              [this article on MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/If-Modified-Since) \
              for more information about this conditional header.";

        let example = "Wed, 21 Oct 2015 07:28:00 GMT";

        vec![utoipa::openapi::path::ParameterBuilder::new()
            .name("If-Modified-Since")
            .parameter_in(utoipa::openapi::path::ParameterIn::Header)
            .description(Some(description))
            .required(utoipa::openapi::Required::True)
            .example(Some(serde_json::Value::String(
                example.to_string(),
            )))
            .schema(Some(
                utoipa::openapi::ObjectBuilder::new()
                    .schema_type(utoipa::openapi::SchemaType::String)
                    .read_only(Some(true)),
            ))
            .build()]
    }
}
