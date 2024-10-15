//! [`utoipa`] (OpenAPI) response annotations for Stari Kolomoni endpoints.
//!
//! # Usage
//! The types in this module are meant for different use-cases (described individually).
//! However, they they all implement [`utoipa::IntoResponses`] / [`utoipa::ToResponse`],
//! allowing us to insert them into our [`utoipa::path`] endpoint annotations.
//!
//! **It is fully up to your endpoint implementation to ensure
//! what you annotate it with actually happens. Adding an [`utoipa`] annotation from this module
//! only means
//! that it will append/modify the OpenAPI documentation.**
//!
//! <br>
//!
//! ## [`utoipa::IntoResponses`]-implementing types
//! Types that implement [`utoipa::IntoResponses`] can be used
//! inside the `responses` section (example based on the [`InternalServerError`] annotation):
//! ```no_run
//! use kolomoni::api::errors::EndpointResult;
//! use kolomoni::api::openapi::response::InternalServerError;
//!
//! #[utoipa::path(
//!     post,
//!     path = "/",
//!     responses(
//!         // This will generate the appropriate HTTP `500 Internal Server Error`
//!         // documentation on the endpoint's OpenAPI schema.
//!         InternalServerError
//!     )
//! )]
//! #[actix_web::post("/")]
//! pub async fn foo_bar() -> EndpointResult {
//!     todo!();
//! }
//! ```
//!
//!
//! ## [`utoipa::ToResponse`]-implementing types
//! Types that implement [`utoipa::ToResponse`] can be used
//! inside an individual response in the `responses` section (example based on the [`AsErrorReason`] annotation):
//! ```no_run
//! use kolomoni::declare_openapi_error_reason_response;
//! use kolomoni::api::errors::EndpointResult;
//! use kolomoni::api::errors::WordErrorReason;
//! use kolomoni::api::openapi::response::InternalServerError;
//!
//! declare_openapi_error_reason_response!(
//!     pub struct MyCustomErrorReason {
//!         description => "Custom error reason.",
//!         reason => WordErrorReason::word_not_found()
//!     }
//! );
//!
//! #[utoipa::path(
//!     post,
//!     path = "/",
//!     responses(
//!         // This will generate the appropriate response that includes a strongly-typed reason
//!         // on the endpoint's OpenAPI schema.
//!         (
//!             status = 404,
//!             response = inline(AsErrorReason<MyCustomErrorReason>)
//!         )
//!     )
//! )]
//! #[actix_web::post("/")]
//! pub async fn foo_bar() -> EndpointResult {
//!     todo!();
//! }
//! ```

use std::{collections::BTreeMap, marker::PhantomData};

use actix_http::StatusCode;
use itertools::Itertools;
use kolomoni_core::api_models::{
    ErrorReason,
    ErrorReasonName,
    InvalidJsonBodyReason,
    ResponseWithErrorReason,
};
use requires::RequiredPermissionSet;
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


pub mod requires;



/// Indicates that an endpoint requires authentication via the `Authorization` header.
///
/// Annotate an endpoint with this to document the appropriate `401 Unauthorized` HTTP error response
/// in cases when the `Authorization` header is not provided.
///
///
/// # Usage
/// This is an endpoint OpenAPI schema documentation type that implements [`utoipa::IntoResponses`].
/// See [module-level documentation] on how to apply this set of responses to an endpoint's OpenAPI documentation.
/// **As with all types in this module, it is fully up to your endpoint implementation to ensure
/// what you annotate it with actually happens. Adding this annotation only means
/// that it will append/modify the OpenAPI documentation.**
///
/// # Example
/// ```no_run
/// use kolomoni::api::openapi::response::MissingAuthentication;
/// use kolomoni::api::errors::EndpointResult;
/// use kolomoni::require_user_authentication;
/// use kolomoni::authentication::UserAuthenticationExtractor;
///
/// #[utoipa::path(
///     post,
///     path = "/submit",
///     responses(
///         MissingAuthentication
///     )
/// )]
/// #[actix_web::post("/submit")]
/// async fn submit(
///     authentication: UserAuthenticationExtractor,
/// ) -> EndpointResult {
///     // This macro will early-return a `401 Unauthorized` with
///     // `ErrorReason::missing_authentication()` if the caller did not
///     // provide an access token. For more information, see the macro's documentation.
///     let authenticated_user = require_user_authentication!(authentication);
///
///     // ...
///     # todo!();
/// }
/// ```
///
/// # Generated documentation
/// This type appends the following responses to the documentation:
/// - `401 Unauthorized` when:
///     - the API caller is not authenticated.
///
///
/// [module-level documentation]: self
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


/// Indicates that an endpoint requires a set of permissions.
///
/// Note: since we have a few permissions that are currently granted to all users
/// as well as unauthenticated API calls (e.g. [`Permission::WordRead`]),
/// the responses documented by this type do not include errors
/// about missing authentication — if you wish to document that an endpoint requires
/// authentication *and* some permissions, use both this
/// and the [`MissingAuthentication`] response types.
///
///
/// See also: [`requires`].
///
/// <br>
///
/// # Usage
/// This is an endpoint OpenAPI schema documentation type that implements [`utoipa::IntoResponses`].
/// See [module-level documentation] on how to apply this set of responses to an endpoint's OpenAPI documentation.
///
/// **As with all types in this module, it is fully up to your endpoint implementation to ensure
/// what you annotate it with actually happens. Adding this annotation only means
/// that it will append/modify the OpenAPI documentation.**
///
/// Additionally, this type has a generic named `P` and a const `usize` generic named `N`.
/// The first generic is used to provide one or more of the required permissions on the type-level.
///
/// The second generic simply counts how many permissions are required (how many permissions are combined in `P`).
/// Sadly, it is not possible to elide this at the moment due to Rust limitations.
///
/// # Examples
/// The following is an example requiring one permission (and not necessarily authentication):
/// ```no_run
/// use kolomoni::api::openapi::response::{
///     MissingPermissions,
///     requires
/// };
/// use kolomoni::api::errors::EndpointResult;
/// use kolomoni::require_user_authentication;
/// use kolomoni::authentication::UserAuthenticationExtractor;
///
/// #[utoipa::path(
///     get,
///     path = "/fetch",
///     responses(
///         // We specified the WordRead permission, and also had to specify 1
///         // on the second generic (it represents the number of required permissions,
///         // which we sadly can't elide yet due to language limits).
///         MissingPermissions<requires::WordRead, 1>
///     )
/// )]
/// #[actix_web::get("/fetch")]
/// async fn fetch_something(
///     authentication: UserAuthenticationExtractor,
/// ) -> EndpointResult {
///     // This macro will early-return a `401 Unauthorized` with
///     // `ErrorReason::missing_authentication()` if the caller did not
///     // provide an access token. For more information, see the macro's documentation.
///     let authenticated_user = require_user_authentication!(authentication);
///
///     // ...
///     # todo!();
/// }
/// ```
///
/// <br>
///
/// The following is an example requiring authntication *and* two permissions:
/// ```no_run
/// use kolomoni::state::ApplicationState;
/// use kolomoni::api::openapi::response::{
///     MissingAuthentication,
///     MissingPermissions
/// };
/// use kolomoni::api::openapi::response::requires;
/// use kolomoni::api::openapi::response::requires::And;
/// use kolomoni::api::errors::EndpointResult;
/// use kolomoni::require_user_authentication_and_permissions;
/// use kolomoni::authentication::UserAuthenticationExtractor;
///
/// #[utoipa::path(
///     post,
///     path = "/update",
///     responses(
///         MissingAuthentication,
///         // Notice how we used the `And` operator to combine two permission requirements.
///         // Additionally, we had to set MissingPermissions' second generic to 2, since
///         // we require two permissions, not just one.
///         MissingPermissions<And<requires::WordRead, requires::WordUpdate>, 2>
///     )
/// )]
/// #[actix_web::post("/update")]
/// async fn update_something(
///     state: ApplicationState,
///     authentication: UserAuthenticationExtractor,
/// ) -> EndpointResult {
///     let mut database_connection = state.acquire_database_connection().await?;
///
///     // This macro will early-return a `401 Unauthorized` with
///     // `ErrorReason::missing_authentication()` if the caller did not
///     // provide an access token. Additionally, it will early-return a
///     // `403 Forbidden` with `ErrorReason::missing_permissions(...)`
///     // if the user is missing any of the specified permissions.
///     // For more information, see the macro's documentation.
///     require_user_authentication_and_permissions!(
///         &mut database_connection,
///         authentication,
///         [Permission::WordRead, Permission::WordUpdate]
///     );
///
///     // ...
///     # todo!();
/// }
/// ```
///
/// # Generated documentation
/// This type appends the following responses to the documentation:
/// - `403 Forbidden` when:
///     - the API caller is missing one or more of the required permissions.
///
///
/// [module-level documentation]: self
/// [`Permission::WordRead`]: kolomoni_core::permissions::Permission::WordRead
#[allow(private_bounds)]
pub struct MissingPermissions<P, const N: usize>
where
    P: RequiredPermissionSet<N>,
{
    _marker: PhantomData<P>,
}

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


/// Indicates that an endpoint may return a `304 Not Modified` HTTP response
/// if the underlying resource did not change.
///
///
/// # Usage
/// This is an endpoint OpenAPI schema documentation type that implements [`utoipa::IntoResponses`].
/// See [module-level documentation] on how to apply this set of responses to an endpoint's OpenAPI documentation.
///
/// **As with all types in this module, it is fully up to your endpoint implementation to ensure
/// what you annotate it with actually happens. Adding this annotation only means
/// that it will append/modify the OpenAPI documentation.**
///
/// # Examples
/// ```no_run
/// use kolomoni::api::OptionalIfModifiedSince;
/// use kolomoni::api::openapi::response::Unmodified;
/// use kolomoni::api::errors::EndpointResult;
///
/// #[utoipa::path(
///     get,
///     path = "/fetch",
///     responses(
///         Unmodified
///     )
/// )]
/// #[actix_web::get("/fetch")]
/// async fn fetch_something(
///     if_modified_since_header: OptionalIfModifiedSince,
/// ) -> EndpointResult {
///     // ...
///     # let some_time = chrono::Utc::now();
///
///     if if_modified_since_header.enabled_and_has_not_changed_since(&some_time) {
///         return EndpointResponseBuilder::not_modified()
///             .with_last_modified_at(&some_time)
///             .build();
///     }
///
///     // ...
///     # todo!();
/// }
/// ```
///
/// # Generated documentation
/// This type appends the following responses to the documentation:
/// - `304 Not Modified` with an empty body; implementation details are
///   up to the endpoint on which this is defined.
///
///
/// [module-level documentation]: self
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


/// Indicates that an endpoint may return a `500 Internal Server Error` HTTP response
/// indicating that something went wrong internally (e.g. database connection issues,
/// JSON serialization error, ...).
///
/// This should be present on basically all routes,
/// as even most extractors can cause this to happen.
///
/// # Usage
/// This is an endpoint OpenAPI schema documentation type that implements [`utoipa::IntoResponses`].
/// See [module-level documentation] on how to apply this set of responses to an endpoint's OpenAPI documentation.
///
/// **As with all types in this module, it is fully up to your endpoint implementation to ensure
/// what you annotate it with actually happens. Adding this annotation only means
/// that it will append/modify the OpenAPI documentation.**
///
///
/// # Examples
/// ```no_run
/// use kolomoni::api::OptionalIfModifiedSince;
/// use kolomoni::api::openapi::response::InternalServerError;
/// use kolomoni::api::errors::EndpointResult;
///
/// #[utoipa::path(
///     get,
///     path = "/fetch",
///     responses(
///         InternalServerError
///     )
/// )]
/// #[actix_web::get("/fetch")]
/// async fn fetch_something(
///     state: ApplicationState,
/// ) -> EndpointResult {
///     // For example, this can cause an internal error if the database connection fails.
///     let mut database_connection = state.acquire_database_connection().await?;
///
///     // ...
///     # todo!();
/// }
/// ```
///
/// # Generated documentation
/// This type appends the following responses to the documentation:
/// - `500 Internal Server Error` without any further details.
///
///
/// [module-level documentation]: self
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



/// Indicates that an endpoint may return a `400 Bad Request` response for a variety
/// of reasons related to the required JSON body provided by the caller.
///
///
/// # Usage
/// This is an endpoint OpenAPI schema documentation type that implements [`utoipa::IntoResponses`].
/// See [module-level documentation] on how to apply this set of responses to an endpoint's OpenAPI documentation.
///
/// **As with all types in this module, it is fully up to your endpoint implementation to ensure
/// what you annotate it with actually happens. Adding this annotation only means
/// that it will append/modify the OpenAPI documentation.**
///
///
/// # Examples
/// ```no_run
/// use actix_web::web;
/// use kolomoni::api::openapi::response::RequiredJsonBodyErrors;
/// use kolomoni::api::errors::EndpointResult;
///
/// #[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
/// struct SomeBodyData {
///     pub some_field: String,
/// }
///
/// #[utoipa::path(
///     post,
///     path = "/change",
///     responses(
///         RequiredJsonBodyErrors
///     )
/// )]
/// #[actix_web::get("/change")]
/// async fn change_something(
///     request_body: web::Json<SomeBodyData>,
/// ) -> EndpointResult {
///     println!("{:?}", request_body);    
///
///     // ...
///     # todo!();
/// }
/// ```
///
///
/// # Generated documentation
/// This type appends the following responses to the documentation:
/// - `400 Bad Request`, when:
///     - the body is missing (or the `Content-Type` header isn't set to `application/json`),
///     - the body is not valid JSON,
///     - the JSON data is not in a valid schema (e.g. a missing `some_field` above), or
///     - the body is too large.
///
///
/// [module-level documentation]: self
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




/// Indicates that an endpoint does [`String`] to [`Uuid`] parsing and, as such,
/// can fail to parse a string as an UUID, returning `400 Bad Request` with details
/// about the reason.
///
///
/// # Usage
/// This is an endpoint OpenAPI schema documentation type that implements [`utoipa::IntoResponses`].
/// See [module-level documentation] on how to apply this set of responses to an endpoint's OpenAPI documentation.
///
/// **As with all types in this module, it is fully up to your endpoint implementation to ensure
/// what you annotate it with actually happens. Adding this annotation only means
/// that it will append/modify the OpenAPI documentation.**
///
///
/// # Example
/// ```no_run
/// use actix_web::web;
/// use kolomoni_core::id::EnglishWordId;
/// use kolomoni::api::openapi::response::UuidUrlParameterError;
/// use kolomoni::api::errors::EndpointResult;
///
/// #[utoipa::path(
///     post,
///     path = "/change/{english_word_uuid}",
///     responses(
///         UuidUrlParameterError
///     )
/// )]
/// #[actix_web::get("/change/{english_word_uuid}")]
/// async fn change_something(
///     parameters: web::Path<(String,)>,
/// ) -> EndpointResult {
///     // This function can fail to parse the string as a valid UUID,
///     // and will return [`EndpointError::InvalidUuidFormat`] when that happens.
///     let change_target_uuid = parse_uuid::<EnglishWordId>(parameters.into_inner().0)?;
///
///     println!("{:?}", change_target_uuid);    
///
///     // ...
///     # todo!();
/// }
/// ```
///
///
/// [`Uuid`]: uuid::Uuid
/// [module-level documentation]: self
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



/// A hidden trait related to the [`declare_openapi_error_reason_response!`] macro.
/// Avoid implementing directly.
///
/// [`declare_openapi_error_reason_response!`]: crate::declare_openapi_error_reason_response
pub trait ErrorReasonNewtype {
    /// A concrete description of the reason for this error to occur.
    fn description() -> &'static str;

    /// Returns a correct variant (but otherwise a mock) [`ErrorReason`].
    /// E.g. you may return [`ErrorReason::MissingPermissions`],
    /// but note that the inner state of that variant is ignored when generating
    /// generic reason descriptions (see [`ErrorReasonName`]).
    ///
    /// [`ErrorReasonName`]: crate::api::errors::ErrorReasonName
    fn stateless_error_reason() -> ErrorReason;
}


/// A macro for declaring rich custom endpoint responses that include
/// a JSON-serialized [`ResponseWithErrorReason`] in their body, describing
/// the precise reason for the error.
///
/// For more details, see [`AsErrorReason`].
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

            fn stateless_error_reason() -> kolomoni_core::api_models::ErrorReason {
                $error_reason.into()
            }
        }
    };
}


/// Indicates that an endpoint returns a JSON-serialized error reason.
///
/// Alongside this response type, users will need to declare an error reason
/// newtype using [`declare_openapi_error_reason_response!`].
///
/// # Usage
/// This is an endpoint OpenAPI schema documentation type that implements [`utoipa::ToResponse`].
/// See [module-level documentation] on how to apply this response to an endpoint's OpenAPI documentation.
///
/// **As with all types in this module, it is fully up to your endpoint implementation to ensure
/// what you annotate it with actually happens. Adding this annotation only means
/// that it will append/modify the OpenAPI documentation.**
///
///
/// # Example
/// ```no_run
/// use kolomoni::declare_openapi_error_reason_response;
/// use kolomoni::api::error::WordErrorReason;
///
/// declare_openapi_error_reason_response!(
///     pub struct EnglishWordNotFound {
///         description => "The requested english word does not exist.",
///         reason => WordErrorReason::word_not_found()
///     }
/// );
///
/// #[utoipa::path(
///     get,
///     path = "/dictionary/english/{word_lemma}",
///     tag = "dictionary:english",
///     params(
///         (
///             "word_lemma" = String,
///             Path,
///             description = "Exact lemma of the english word."
///         )
///     ),
///     responses(
///         // ...
///         (
///             status = 404,
///             // This is where the magic happens. We specify an inlined
///             // `EnglishWordNotFound` type that we just declared, wrapped
///             // inside an `AsErrorReason`, which gives it its correct schema and description.
///             response = inline(AsErrorReason<EnglishWordNotFound>)
///         ),
///         // ...
///     )
/// )]
/// #[get("/{word_lemma}")]
/// async fn get_english_word_by_lemma(
///     // ...
///     parameters: web::Path<(String,)>,
/// ) -> EndpointResult {
///     // ...
///     # let word_exists = false;
///
///     if !word_exists {
///         return EndpointResponseBuilder::not_found()
///             .with_error_reason(WordErrorReason::word_not_found())
///             .build;
///     }
///
///     // ...
///     # todo!();
/// }
/// ```
///
/// [`declare_openapi_error_reason_response!`]: crate::declare_openapi_error_reason_response
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
                            R::stateless_error_reason().reason_description()
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
