use std::{collections::BTreeMap, marker::PhantomData};

use actix_http::StatusCode;
use itertools::Itertools;
use requires::RequiredPermissionSet;
use serde_json::json;
use utoipa::{
    openapi::{
        example::{Example, ExampleBuilder},
        ContentBuilder,
        RefOr,
        ResponseBuilder,
        ResponsesBuilder,
    },
    ToSchema,
};

use crate::api::errors::{
    ErrorReason,
    ErrorReasonName,
    InvalidJsonBodyReason,
    ResponseWithErrorReason,
};

pub mod requires;



pub struct MissingAuthentication;

impl utoipa::IntoResponses for MissingAuthentication {
    fn responses() -> BTreeMap<String, utoipa::openapi::RefOr<utoipa::openapi::response::Response>> {
        // Constructs the first kind of possible response: a failure to authenticate at all.
        let missing_user_authentication_401_response = ResponseBuilder::new()
            .description(
                "Missing user authentication, provide an `Authorization: Bearer your-token-here` header."
            )
            .build();

        ResponsesBuilder::new()
            .response(
                StatusCode::UNAUTHORIZED.as_u16().to_string(),
                missing_user_authentication_401_response,
            )
            .build()
            .into()
    }
}



/// A `utoipa` endpoint response for when an endpoint requires authentication and some permission.
///
/// TODO needs a documentation update, a lot has changed
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
///
/// TODO to through these usages and add MissingAuthentication where auth is required
pub struct MissingPermissions<P, const N: usize>
where
    P: RequiredPermissionSet<N>,
{
    _marker: PhantomData<P>,
}

// TODO perhaps we should split this into requires-permission and requires-permissions?

impl<P, const N: usize> utoipa::IntoResponses for MissingPermissions<P, N>
where
    P: RequiredPermissionSet<N>,
{
    /// This will panic if [`ResponseWithErrorReason`] fails to serialize for
    /// a given [`ErrorReason::missing_permission`] (which has no reason to happen,
    /// at least given the current schema).
    fn responses() -> BTreeMap<String, RefOr<utoipa::openapi::response::Response>> {
        // Constructs responses that indicate missing permissions.
        let mut missing_user_permission_403_examples =
            Vec::<(String, RefOr<Example>)>::with_capacity(N);

        for required_permission in P::permissions() {
            let missing_permission_example_json_object =
                serde_json::to_value(ResponseWithErrorReason::new(
                    ErrorReason::missing_permission(required_permission),
                ))
                .expect("failed to serialize ResponseWithErrorReason for a missing permission");

            let missing_permission_example = ExampleBuilder::new()
                .value(Some(missing_permission_example_json_object))
                .build();

            missing_user_permission_403_examples.push((
                format!(
                    "Missing permission: `{}`",
                    required_permission.name()
                ),
                RefOr::T(missing_permission_example),
            ));
        }

        let missing_permission_response_description = if N > 1 {
            format!(
                "Missing one or more of the required permissions: {}.",
                P::permissions()
                    .into_iter()
                    .map(|permission| format!("`{}`", permission.name()))
                    .join(", ")
            )
        } else {
            format!(
                "Missing a required permission: {}.",
                P::permissions()
                    .into_iter()
                    .map(|permission| format!("`{}`", permission.name()))
                    .join(", ")
            )
        };

        let missing_user_permission_403_response = ResponseBuilder::new()
            .description(missing_permission_response_description)
            .content(
                mime::APPLICATION_JSON.to_string(),
                ContentBuilder::new()
                    .examples_from_iter(missing_user_permission_403_examples)
                    .schema(ResponseWithErrorReason::schema().1)
                    .build(),
            )
            .build();


        ResponsesBuilder::new()
            .response(
                StatusCode::FORBIDDEN.as_u16().to_string(),
                missing_user_permission_403_response,
            )
            .build()
            .into()
    }
}




/*
impl<P: RequiredPermissionSet> utoipa::IntoResponses for FailedAuthentication<P> {
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
} */


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
pub struct Unmodified;

impl utoipa::IntoResponses for Unmodified {
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
///
/// TODO remove when usages are migrated to JsonBodyErrors
#[deprecated = "use JsonBodyErrors instead"]
pub struct MissingOrInvalidJsonRequestBody;


impl utoipa::IntoResponses for MissingOrInvalidJsonRequestBody {
    fn responses() -> BTreeMap<String, utoipa::openapi::RefOr<utoipa::openapi::response::Response>> {
        // TODO update schemas here, use ResponseWithErrorReason!
        let bad_request_response = ResponseBuilder::new()
            .description(
                "Bad request due to an invalid JSON request body. Possible reasons:\n\
                - `Content-Type` header is not specified or is not `application/json`.\n\
                - Incorrect structure of the JSON body or invalid JSON in general.\n\
                - Request body is too large (unlikely)."
            )
            .content(
                mime::APPLICATION_JSON.to_string(),
                ContentBuilder::new()
                    .examples_from_iter(vec![
                        (
                            "`Content-Type` header is missing or is not `application/json`.",
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
                    .schema(ResponseWithErrorReason::schema().1)
                    .build(),
            ).build();

        ResponsesBuilder::new()
            .response("400", bad_request_response)
            .build()
            .into()
    }
}



pub struct RequiredJsonBodyErrors;

impl utoipa::IntoResponses for RequiredJsonBodyErrors {
    #[allow(clippy::vec_init_then_push)]
    fn responses() -> BTreeMap<String, RefOr<utoipa::openapi::response::Response>> {
        let mut bad_request_response_examples = Vec::with_capacity(4);

        bad_request_response_examples.push((
            "Missing JSON body (either no body or `Content-Type` header is incorrect)",
            ExampleBuilder::new()
                .description(
                    "The JSON body or the associated `Content-Type: application/json` header is missing. \
                    Both are required for the server to acknowledge and parse the body as JSON."
                )
                .value(
                    Some(
                        serde_json::to_value(ResponseWithErrorReason::new(ErrorReason::missing_json_body()))
                            .expect("failed to serialize missing JSON body error response")
                    )
                )
                .build()
        ));

        bad_request_response_examples.push((
            "Invalid JSON body (invalid JSON syntax)",
            ExampleBuilder::new()
                .description(
                    "The provided JSON body is either not valid JSON at all, or \
                    there was an IO/EOF error while reading the contents.",
                )
                .value(Some(
                    serde_json::to_value(ResponseWithErrorReason::new(
                        ErrorReason::invalid_json_body(InvalidJsonBodyReason::NotJson),
                    ))
                    .expect("failed to serialize invalid JSON body error response"),
                ))
                .build(),
        ));

        bad_request_response_examples.push((
            "Invalid JSON schema (valid JSON, but invalid data)",
            ExampleBuilder::new()
                .description(
                    "The provided JSON body is valid JSON, but its data doesn't match \
                    the expected schema for the given endpoint.",
                )
                .value(Some(
                    serde_json::to_value(ResponseWithErrorReason::new(
                        ErrorReason::invalid_json_body(InvalidJsonBodyReason::InvalidData),
                    ))
                    .expect("failed to serialize invalid JSON data error response"),
                ))
                .build(),
        ));


        bad_request_response_examples.push((
            "Body is too large",
            ExampleBuilder::new()
                .description("The provided request body is too large.")
                .value(Some(
                    serde_json::to_value(ResponseWithErrorReason::new(
                        ErrorReason::invalid_json_body(InvalidJsonBodyReason::TooLarge),
                    ))
                    .expect("failed to serialize too large body error response"),
                ))
                .build(),
        ));



        let bad_request_response = ResponseBuilder::new()
            .description(
                "Invalid (or missing) JSON body. An expected JSON body can be invalid \
                 due to either the JSON syntax itself not being valid, or because the \
                 data itself (the schema) is invalid. Additionally, the server will refuse \
                 to process JSON payloads that exceed the configured maximum size (though \
                 this should be exceedingly rare).",
            )
            .content(
                mime::APPLICATION_JSON.to_string(),
                ContentBuilder::new()
                    .examples_from_iter(bad_request_response_examples)
                    .schema(ResponseWithErrorReason::schema().1)
                    .build(),
            )
            .build();


        ResponsesBuilder::new()
            .response(
                StatusCode::BAD_REQUEST.as_str(),
                bad_request_response,
            )
            .build()
            .into()
    }
}




pub struct OptionalJsonBodyErrors;

impl utoipa::IntoResponses for OptionalJsonBodyErrors {
    #[allow(clippy::vec_init_then_push)]
    fn responses() -> BTreeMap<String, RefOr<utoipa::openapi::response::Response>> {
        let mut bad_request_response_examples = Vec::with_capacity(3);

        bad_request_response_examples.push((
            "Invalid JSON body (invalid JSON syntax)",
            ExampleBuilder::new()
                .description(
                    "The provided JSON body is either not valid JSON at all, or \
                    there was an IO/EOF error while reading the contents.",
                )
                .value(Some(
                    serde_json::to_value(ResponseWithErrorReason::new(
                        ErrorReason::invalid_json_body(InvalidJsonBodyReason::NotJson),
                    ))
                    .expect("failed to serialize invalid JSON body error response"),
                ))
                .build(),
        ));

        bad_request_response_examples.push((
            "Invalid JSON schema (valid JSON, but invalid data)",
            ExampleBuilder::new()
                .description(
                    "The provided JSON body is valid JSON, but its data doesn't match \
                    the expected schema for the given endpoint.",
                )
                .value(Some(
                    serde_json::to_value(ResponseWithErrorReason::new(
                        ErrorReason::invalid_json_body(InvalidJsonBodyReason::InvalidData),
                    ))
                    .expect("failed to serialize invalid JSON data error response"),
                ))
                .build(),
        ));


        bad_request_response_examples.push((
            "Body is too large",
            ExampleBuilder::new()
                .description("The provided request body is too large.")
                .value(Some(
                    serde_json::to_value(ResponseWithErrorReason::new(
                        ErrorReason::invalid_json_body(InvalidJsonBodyReason::TooLarge),
                    ))
                    .expect("failed to serialize too large body error response"),
                ))
                .build(),
        ));



        let bad_request_response = ResponseBuilder::new()
            .description(
                "Invalid JSON body. A JSON body can be invalid \
                 due to either the JSON syntax itself not being valid, or because the \
                 data itself (the schema) is invalid. Additionally, the server will refuse \
                 to process JSON payloads that exceed the configured maximum size (though \
                 this should be exceedingly rare).",
            )
            .content(
                mime::APPLICATION_JSON.to_string(),
                ContentBuilder::new()
                    .examples_from_iter(bad_request_response_examples)
                    .schema(ResponseWithErrorReason::schema().1)
                    .build(),
            )
            .build();


        ResponsesBuilder::new()
            .response(
                StatusCode::BAD_REQUEST.as_str(),
                bad_request_response,
            )
            .build()
            .into()
    }
}




pub struct UuidUrlParameterError;

impl utoipa::IntoResponses for UuidUrlParameterError {
    fn responses() -> BTreeMap<String, utoipa::openapi::RefOr<utoipa::openapi::response::Response>> {
        let invalid_uuid_400_response = ResponseBuilder::new()
            .description(
                "One of the expected URL parameters was an UUID (string), but it was in an invalid format."
            )
            .content(
                mime::APPLICATION_JSON.to_string(),
                ContentBuilder::new()
                    .examples_from_iter([(
                        "Invalid UUID",
                        ExampleBuilder::new()
                            .description("The provided value is not a valid UUID.")
                            .value(Some(
                                serde_json::to_value(
                                    ResponseWithErrorReason::new(
                                        ErrorReason::invalid_uuid_format()
                                    )
                                ).expect("failed to serialize invalid UUID error response")
                            ))
                            .build()
                    )])
                    .schema(ResponseWithErrorReason::schema().1)
                    .build()
            )
            .build();


        ResponsesBuilder::new()
            .response(
                StatusCode::BAD_REQUEST.as_str(),
                invalid_uuid_400_response,
            )
            .build()
            .into()
    }
}




pub trait ErrorReasonNewtype {
    fn description() -> &'static str;
    fn stateless_error_reason() -> ErrorReason;
}


#[macro_export]
macro_rules! declare_openapi_error_reason_response {
    (
        $struct_visibility:vis struct $struct_name:ident {
            description => $description:expr,
            reason => $error_reason:expr
        }
    ) => {
        $struct_visibility struct $struct_name;

        impl $crate::api::openapi::response::ErrorReasonNewtype for $struct_name {
            fn description() -> &'static str {
                $description
            }

            fn stateless_error_reason() -> $crate::api::errors::ErrorReason {
                $error_reason.into()
            }
        }
    };
}


pub struct AsErrorReason<R>
where
    R: ErrorReasonNewtype,
{
    _marker: PhantomData<R>,
}

impl<'a, R> utoipa::ToResponse<'a> for AsErrorReason<R>
where
    R: ErrorReasonNewtype,
{
    fn response() -> (
        &'a str,
        RefOr<utoipa::openapi::response::Response>,
    ) {
        let response = ResponseBuilder::new()
            .description(R::description())
            .content(
                mime::APPLICATION_JSON.to_string(),
                ContentBuilder::new()
                    .examples_from_iter([(
                        format!(
                            "Reason: {}",
                            R::stateless_error_reason().reason_name()
                        ),
                        ExampleBuilder::new()
                            .value(Some(
                                serde_json::to_value(ResponseWithErrorReason::new(
                                    R::stateless_error_reason(),
                                ))
                                .expect("failed to serialize AsErrorReason example"),
                            ))
                            .build(),
                    )])
                    .schema(ResponseWithErrorReason::schema().1)
                    .build(),
            )
            .build();

        ("Reason", RefOr::T(response))
    }
}
