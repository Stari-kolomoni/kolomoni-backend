use std::{collections::BTreeMap, marker::PhantomData};

use serde_json::json;
use utoipa::{
    openapi::{
        example::ExampleBuilder,
        ContentBuilder,
        RefOr,
        Response,
        ResponseBuilder,
        ResponsesBuilder,
    },
    ToSchema,
};

use crate::api::errors::ErrorResponseWithReason;


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
/// ```no_run
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
            // FIXME this first doc isn't emitted for some reason? not a problem, but it's a bit annoying
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
            pub struct $permission_variant;
            impl RequiredPermission for $permission_variant {
                fn name() -> &'static str {
                    kolomoni_auth::Permission::$permission_variant.name()
                }
            }
        }
    };
}

// The macro calls below generate empty structs for all available permissions,
// making them usable as a parameter for the [`FailedAuthenticationResponses`] generic.


pub mod requires {
    use super::RequiredPermission;

    generate_standalone_requirement_struct!(UserSelfRead);
    generate_standalone_requirement_struct!(UserSelfWrite);
    generate_standalone_requirement_struct!(UserAnyRead);
    generate_standalone_requirement_struct!(UserAnyWrite);
    generate_standalone_requirement_struct!(WordCreate);
    generate_standalone_requirement_struct!(WordRead);
    generate_standalone_requirement_struct!(WordUpdate);
    generate_standalone_requirement_struct!(WordDelete);
    generate_standalone_requirement_struct!(SuggestionCreate);
    generate_standalone_requirement_struct!(SuggestionDelete);
    generate_standalone_requirement_struct!(TranslationCreate);
    generate_standalone_requirement_struct!(TranslationDelete);
    generate_standalone_requirement_struct!(CategoryCreate);
    generate_standalone_requirement_struct!(CategoryRead);
    generate_standalone_requirement_struct!(CategoryUpdate);
    generate_standalone_requirement_struct!(CategoryDelete);
}



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
pub struct FailedAuthentication<P: RequiredPermission> {
    _marker: PhantomData<P>,
}

impl<P: RequiredPermission> utoipa::IntoResponses for FailedAuthentication<P> {
    fn responses() -> BTreeMap<String, RefOr<Response>> {
        let missing_user_auth_response = ResponseBuilder::new()
            .description(
                "Missing user authentication, provide an `Authorization: Bearer your_token_here` header."
            )
            .build();

        let missing_user_permission_description = format!("Missing the `{}` permission.", P::name());

        let mut missing_user_permission_example = serde_json::Map::with_capacity(1);
        missing_user_permission_example.insert(
            "reason".to_string(),
            serde_json::Value::String(format!("Missing permission: {}.", P::name())),
        );

        let missing_user_permission_response = ResponseBuilder::new()
            .description(missing_user_permission_description)
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
                    .schema(ErrorResponseWithReason::schema().1)
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
/// ```no_run
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
/// ```no_run
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
pub struct UnmodifiedConditional;

impl utoipa::IntoResponses for UnmodifiedConditional {
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
/// ```no_run
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
pub struct InternalServerError;

impl utoipa::IntoResponses for InternalServerError {
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


/// A `utoipa` endpoint response for when and endpoint may return a `400 Bad Request` HTTP response
/// indicating that a JSON body included in the request is not valid.
///
/// This should be present on routes that have a [`web::Json<...>`][actix_web::web::Json]
/// parameter. For more information on how JSON extractor errors are handled,
/// see the [`JsonConfig`][actix_web::web::JsonConfig] that is instantiated in the server
/// initialization closure in `main.rs`.
///
/// # Example
/// ```no_run
/// use serde::Deserialize;
/// use utoipa::ToSchema;
/// use actix_web::{get, web, http, HttpResponse};
/// use kolomoni::state::ApplicationState;
/// use kolomoni::api::openapi;
/// use kolomoni::api::errors::{APIError, EndpointResult};
/// use kolomoni_database::query::UserQuery;
///
/// #[derive(Deserialize, PartialEq, Eq, Debug, ToSchema)]
/// struct HelloWorldRequest {
///    text: String,
/// }
///
/// #[utoipa::path(
///     get,
///     path = "/hello-world",
///     responses(
///         (
///             status = 200,
///             description = "Hello world to you too!"
///         ),
///         openapi::MissingOrInvalidJsonRequestBodyResponse,
///         openapi::InternalServerErrorResponse,
///     )
/// )]
/// #[get("/hello-world")]
/// pub async fn some_endpoint_function(
///     state: ApplicationState,
///     json_body: web::Json<HelloWorldRequest>,
/// ) -> EndpointResult {
///     println!("{}", json_body.text);
///     
///     // ... and so on
///     
///     Ok(HttpResponse::Ok().finish())
/// }
/// ```
pub struct MissingOrInvalidJsonRequestBody;

impl utoipa::IntoResponses for MissingOrInvalidJsonRequestBody {
    fn responses() -> BTreeMap<String, utoipa::openapi::RefOr<utoipa::openapi::response::Response>> {
        let bad_request_response = ResponseBuilder::new()
            .description(
                "Bad request due to an invalid JSON request body. Possible reasons:\n\
                - `Content-Type` header is not specified or does not equal `application/json`.\n\
                - Incorrect structure of the JSON body or invalid JSON in general.\n\
                - Request body is too large (highly unlikely)."
            )
            .content(
                mime::APPLICATION_JSON.to_string(),
                ContentBuilder::new()
                    .examples_from_iter(vec![
                        (
                            "`Content-Type` header is missing or does not equal `application/json`.",
                            ExampleBuilder::new()
                                .value(Some(json!({
                                    "reason": "Client error: non-JSON body. If your request body contains JSON, \
                                              please signal that with the `Content-Type: application/json` header."
                                })))
                                .build(),
                        ),
                        (
                            "Provided request body does not contain valid JSON data.",
                            ExampleBuilder::new()
                                .value(Some(json!({
                                    "reason": "Client error: invalid JSON body."
                                })))
                                .build(),
                        ),
                        (
                            "Provided request body does not matches the expected JSON structure \
                            (e.g. it's missing some fields or has a field of the incorrect type).",
                            ExampleBuilder::new()
                                .value(Some(json!({
                                    "reason": "Client error: invalid JSON body."
                                })))
                                .build(),
                        ),
                        (
                            "Provided request body is too large. Unlikely, but possible.",
                            ExampleBuilder::new()
                                .value(Some(json!({
                                    "reason": "Client error: request body is too large."
                                })))
                                .build(),
                        )
                    ])
                    .schema(ErrorResponseWithReason::schema().1)
                    .build(),
            ).build();

        ResponsesBuilder::new()
            .response("400", bad_request_response)
            .build()
            .into()
    }
}
