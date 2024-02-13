use std::net::Ipv4Addr;

use actix_web::{App, HttpServer};
use kolomoni::api::errors;
use kolomoni::api::v1::login;
use kolomoni::api::v1::ping;
use kolomoni::api::v1::users;
use kolomoni::logging::initialize_tracing;
use miette::Context;
use miette::IntoDiagnostic;
use miette::Result;
use tracing::info;
use tracing_actix_web::TracingLogger;
use tracing_subscriber::EnvFilter;
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::{openapi::OpenApi, Modify, OpenApi as OpenApiDerivable};
use utoipa_rapidoc::RapiDoc;


#[derive(OpenApiDerivable)]
#[openapi(
    paths(
        ping::ping,
        login::login,
        login::refresh_login,
        users::all_users::get_all_registered_users,
        users::register::register_user,
        users::current_user::get_current_user_info,
        users::current_user::get_current_user_effective_permissions,
        users::current_user::update_current_user_display_name,
        users::specific_user::get_specific_user_info,
        users::specific_user::get_specific_user_effective_permissions,
        users::specific_user::get_specific_user_roles,
        users::specific_user::add_roles_to_specific_user,
        users::specific_user::remove_roles_from_specific_user,
        users::specific_user::update_specific_user_display_name,
    ),
    components(
        schemas(
            users::UserInformation,
            users::UserInfoResponse,
            users::UserPermissionsResponse,
            users::UserDisplayNameChangeRequest,
            users::UserDisplayNameChangeResponse,
            errors::ErrorReasonResponse
        ),
    ),
    info(
        title = "Stari Kolomoni API",
        description = "",
        contact(
            name = "Stari Kolomoni Team",
            email = "stari.kolomoni@gmail.com"
        ),
        license(
            name = "GPL-3.0-only",
            url = "https://github.com/Stari-kolomoni/kolomoni-backend/blob/master/LICENSE.md"
        )
    ),
    servers(
        (
            url = "http://127.0.0.1:8866/api/v1/",
            description = "Local development server"
        )
    ),
    modifiers(
        &JWTBearerTokenModifier
    )
)]
struct APIDocumentation;


struct JWTBearerTokenModifier;

impl Modify for JWTBearerTokenModifier {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        // We can unwrap safely since there already are components registered.
        let components = openapi.components.as_mut().unwrap();

        components.add_security_scheme(
            "access_token",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .description(Some(
                        "Provide a `Bearer` JSON Web Token to authenticate as a user.",
                    ))
                    .build(),
            ),
        );
    }
}

const PRIVATE_COMMENTS_SEPARATOR: &str = "--- EXCLUDE FROM PUBLIC API DOCUMENTATION ---";

fn remove_private_comments(string: &mut String) {
    if let Some(starting_index) = string.find(PRIVATE_COMMENTS_SEPARATOR) {
        *string = string[..starting_index].to_string();
    }
}

fn remove_duplicated_summary_from_description(summary: &str, paragraph: &mut String) {
    if paragraph.starts_with(summary) {
        *paragraph = paragraph[summary.len()..].trim_start().to_string();
    }
}

/// Filters out private comments from the API documentation
/// (marked by `--- EXCLUDE FROM PUBLIC API DOCUMENTATION ---`) and
/// removes duplicated summary paragraphs from description fields.
fn clean_up_documentation(documentation: &mut OpenApi) {
    if let Some(info_description) = documentation.info.description.as_mut() {
        remove_private_comments(info_description);
    }

    for path in documentation.paths.paths.values_mut() {
        if let Some(path_description) = path.description.as_mut() {
            if let Some(path_summary) = path.summary.as_ref() {
                remove_duplicated_summary_from_description(path_summary, path_description);
            }

            remove_private_comments(path_description);
        }

        for operation in path.operations.values_mut() {
            if let Some(operation_description) = operation.description.as_mut() {
                if let Some(operation_summary) = operation.summary.as_ref() {
                    remove_duplicated_summary_from_description(
                        operation_summary,
                        operation_description,
                    );
                }

                remove_private_comments(operation_description);
            }
        }
    }
}


#[actix_web::main]
async fn main() -> Result<()> {
    // Initialize logging and tracing.
    let guard = initialize_tracing(
        EnvFilter::builder().from_env_lossy(),
        EnvFilter::builder().from_env_lossy(),
        "./logs",
        "kolomoni-openapi.log",
    );

    // Initialize compile-time generated OpenApi documentation.
    let mut open_api: OpenApi = APIDocumentation::openapi();
    clean_up_documentation(&mut open_api);

    // Start actix HTTP server to serve the documentation.
    // The interactive documentation page will be served at `/api-documentation`,
    // and the OpenAPI JSON file at `/api-documentation/openapi.json`.

    let server = HttpServer::new(move || {
        App::new().wrap(TracingLogger::default()).service(
            RapiDoc::with_openapi(
                "/api-documentation/openapi.json",
                open_api.clone(),
            )
            .path("/api-documentation"),
        )
    })
    .bind((Ipv4Addr::LOCALHOST, 8877))
    .into_diagnostic()
    .wrap_err("Failed to set up actix HTTP server.")?;

    info!("HTTP server initialized, running.");

    server
        .run()
        .await
        .into_diagnostic()
        .wrap_err("Errored while running actix HTTP server.")?;

    drop(guard);
    Ok(())
}
