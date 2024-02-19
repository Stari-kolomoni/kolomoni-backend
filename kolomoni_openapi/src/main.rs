use std::net::Ipv4Addr;

use actix_web::{App, HttpServer};
use kolomoni::api::errors;
use kolomoni::api::v1::dictionary;
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
        /***
         * Annotated paths are relative to `kolomoni/src/api/v1`.
         */

        // ping.rs
        ping::ping,

        // login.rs
        login::login,
        login::refresh_login,

        // users/all.rs
        users::all::get_all_registered_users,

        // users/current.rs
        users::current::get_current_user_info,
        users::current::get_current_user_roles,
        users::current::get_current_user_effective_permissions,
        users::current::update_current_user_display_name,

        // users/registration.rs
        users::registration::register_user,

        // users/specific.rs
        users::specific::get_specific_user_info,
        users::specific::get_specific_user_roles,
        users::specific::get_specific_user_effective_permissions,
        users::specific::update_specific_user_display_name,
        users::specific::add_roles_to_specific_user,
        users::specific::remove_roles_from_specific_user,

        // dictionary/slovene_word.rs
        dictionary::slovene_word::get_all_slovene_words,
        dictionary::slovene_word::create_slovene_word,
        dictionary::slovene_word::get_specific_slovene_word,
        dictionary::slovene_word::update_specific_slovene_word,
        dictionary::slovene_word::delete_specific_slovene_word,

        // dictionary/english_word.rs
        dictionary::english_word::get_all_english_words,
        dictionary::english_word::create_english_word,
        dictionary::english_word::get_specific_english_word,
        dictionary::english_word::update_specific_english_word,
        dictionary::english_word::delete_specific_english_word,
    ),
    components(
        schemas(
            /***
             * Annotated paths are relative to `kolomoni/src/api/v1`.
             */

            // ping.rs
            ping::PingResponse,

            // login.rs
            login::UserLoginRequest,
            login::UserLoginResponse,
            login::UserLoginRefreshRequest,
            login::UserLoginRefreshResponse,

            // users.rs
            users::UserInformation,
            users::UserInfoResponse,
            users::UserDisplayNameChangeRequest,
            users::UserDisplayNameChangeResponse,
            users::UserRolesResponse,
            users::UserPermissionsResponse,

            // users/all.rs
            users::all::RegisteredUsersListResponse,

            // users/current.rs
            // (none)
            // users/registration.rs
            users::registration::UserRegistrationRequest,
            users::registration::UserRegistrationResponse,

            // users/specific.rs
            users::specific::UserRoleAddRequest,
            users::specific::UserRoleRemoveRequest,

            // ../errors.rs
            errors::ErrorReasonResponse,

            // dictionary/slovene_word.rs
            dictionary::slovene_word::SloveneWord,
            dictionary::slovene_word::SloveneWordsResponse,
            dictionary::slovene_word::SloveneWordCreationRequest,
            dictionary::slovene_word::SloveneWordCreationResponse,
            dictionary::slovene_word::SloveneWordInfoResponse,
            dictionary::slovene_word::SloveneWordUpdateRequest,

            // dictionary/english_word.rs
            dictionary::english_word::EnglishWord,
            dictionary::english_word::EnglishWordsResponse,
            dictionary::english_word::EnglishWordCreationRequest,
            dictionary::english_word::EnglishWordCreationResponse,
            dictionary::english_word::EnglishWordInfoResponse,
            dictionary::english_word::EnglishWordUpdateRequest,
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
