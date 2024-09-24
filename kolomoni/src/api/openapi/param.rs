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
/// ```no_run
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
pub struct IfModifiedSince;

impl utoipa::IntoParams for IfModifiedSince {
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
