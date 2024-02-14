//! This module defines commonly used OpenAPI parameters and responses
//! to be used in conjunction with the [`utiopa::path`] proc macro on actix handlers.

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


pub trait RequiredPermission {
    fn name() -> &'static str;
}

macro_rules! generate_standalone_requirement_struct {
    ($permission_variant:ident) => {
        ::paste::paste! {
            pub struct [< Requires $permission_variant >];
            impl RequiredPermission for [< Requires $permission_variant >] {
                fn name() -> &'static str {
                    kolomoni_auth::Permission::$permission_variant.name()
                }
            }
        }
    };
}

// The generated structs and implementations look like the following:
//
// pub struct RequiresUserSelfRead;
// impl RequiredPermission for RequiresUserSelfRead {
//     fn name() -> &'static str {
//         Permission::UserSelfRead.name()
//     }
// }
//

generate_standalone_requirement_struct!(UserSelfRead);
generate_standalone_requirement_struct!(UserSelfWrite);
generate_standalone_requirement_struct!(UserAnyRead);
generate_standalone_requirement_struct!(UserAnyWrite);
generate_standalone_requirement_struct!(WordCreate);
generate_standalone_requirement_struct!(WordRead);
generate_standalone_requirement_struct!(WordUpdate);
generate_standalone_requirement_struct!(WordDelete);



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



pub struct UnmodifiedConditionalResponse;

impl utoipa::IntoResponses for UnmodifiedConditionalResponse {
    fn responses() -> BTreeMap<String, utoipa::openapi::RefOr<utoipa::openapi::response::Response>> {
        let unmodified_data_response = ResponseBuilder::new()
            .description(
                "User hasn't been modified since the timestamp specified in the `If-Modified-Since` header. \
                As such, this status code can only be returned if that header is provided in the request."
            )
            .build();

        ResponsesBuilder::new()
            .response("304", unmodified_data_response)
            .build()
            .into()
    }
}



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
