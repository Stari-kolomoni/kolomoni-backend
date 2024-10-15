use std::net::Ipv4Addr;

use actix_web::{App, HttpServer};
use kolomoni::api::v1::dictionary;
use kolomoni::api::v1::health;
use kolomoni::api::v1::login;
use kolomoni::api::v1::users;
use kolomoni::logging::initialize_tracing;
use kolomoni_core::api_models;
use miette::Context;
use miette::IntoDiagnostic;
use miette::Result;
use tracing::info;
use tracing_actix_web::TracingLogger;
use tracing_subscriber::EnvFilter;
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::{openapi::OpenApi, Modify, OpenApi as OpenApiDerivable};
use utoipa_rapidoc::RapiDoc;

// TODO update to include all the current endpoints

#[derive(OpenApiDerivable)]
#[openapi(
    paths(
        /***
         * Annotated paths are relative to `kolomoni/src/api/v1`.
         */

        // kolomoni::api::v1::health
        health::ping,


        // kolomoni::api::v1::login
        login::login,
        login::refresh_login,


        // kolomoni::api::v1::users::all
        users::all::get_all_registered_users,

        // kolomoni::api::v1::users::current
        users::current::get_current_user_info,
        users::current::get_current_user_roles,
        users::current::get_current_user_effective_permissions,
        users::current::update_current_user_display_name,

        // kolomoni::api::v1::users::registration
        users::registration::register_user,

        // kolomoni::api::v1::users::specific
        users::specific::get_specific_user_info,
        users::specific::get_specific_user_roles,
        users::specific::get_specific_user_effective_permissions,
        users::specific::update_specific_user_display_name,
        users::specific::add_roles_to_specific_user,
        users::specific::remove_roles_from_specific_user,


        // kolomoni::api::v1::dictionary::slovene::endpoints::word
        dictionary::slovene::get_all_slovene_words,
        dictionary::slovene::create_slovene_word,
        dictionary::slovene::get_slovene_word_by_id,
        dictionary::slovene::get_slovene_word_by_lemma,
        dictionary::slovene::update_slovene_word,
        dictionary::slovene::delete_slovene_word,

        // kolomoni::api::v1::dictionary::slovene::endpoints::meaning
        // TODO
        // dictionary::slovene::get_all_slovene_word_meanings,
        // dictionary::slovene::create_slovene_word_meaning,
        // dictionary::slovene::update_slovene_word_meaning,
        // dictionary::slovene::delete_slovene_word_meaning,


        // kolomoni::api::v1::dictionary::english::endpoints::word
        dictionary::english::get_all_english_words,
        dictionary::english::create_english_word,
        dictionary::english::get_english_word_by_id,
        dictionary::english::get_english_word_by_lemma,
        dictionary::english::update_english_word,
        dictionary::english::delete_english_word,

        // kolomoni::api::v1::dictionary::english::endpoints::meaning
        // TODO
        // dictionary::english::get_all_english_word_meanings,
        // dictionary::english::create_english_word_meaning,
        // dictionary::english::update_english_word_meaning,
        // dictionary::english::delete_english_word_meaning,


        // kolomoni::api::v1::dictionary::translations
        dictionary::translations::create_translation,
        dictionary::translations::delete_translation,


        // dictionary/search.rs
        // TODO
        // dictionary::search::perform_search,
    ),
    components(
        schemas(
            // kolomoni_core::api_models::error_reason
            api_models::ErrorReason,
            api_models::CategoryErrorReason,
            api_models::LoginErrorReason,
            api_models::UsersErrorReason,
            api_models::TranslationsErrorReason,
            api_models::WordErrorReason,
            api_models::ResponseWithErrorReason,

            // kolomoni_auth
            kolomoni_core::permissions::Permission,

            // kolomoni_core::id
            kolomoni_core::ids::CategoryId,
            kolomoni_core::ids::EditId,
            kolomoni_core::ids::UserId,
            kolomoni_core::ids::WordId,
            kolomoni_core::ids::WordMeaningId,
            kolomoni_core::ids::EnglishWordId,
            kolomoni_core::ids::EnglishWordMeaningId,
            kolomoni_core::ids::SloveneWordId,
            kolomoni_core::ids::SloveneWordMeaningId,
            kolomoni_core::ids::PermissionId,
            kolomoni_core::ids::RoleId,

            // kolomoni_core::api_models::health
            api_models::PingResponse,


            // kolomoni_core::api_models::users
            api_models::UserLoginRequest,
            api_models::UserLoginRefreshRequest,
            api_models::UserLoginRefreshResponse,
            api_models::UserLoginResponse,
            api_models::UserInfo,
            api_models::UserInfoResponse,
            api_models::UserDisplayNameChangeRequest,
            api_models::UserDisplayNameChangeResponse,
            api_models::UserRolesResponse,
            api_models::UserPermissionsResponse,
            api_models::RegisteredUsersListResponse,
            api_models::UserRegistrationRequest,
            api_models::UserRegistrationResponse,
            api_models::UserRoleAddRequest,
            api_models::UserRoleRemoveRequest,


            // kolomoni_core::dictionary::categories
            api_models::Category,
            api_models::CategoryCreationRequest,
            api_models::CategoryCreationResponse,
            api_models::CategoriesResponse,
            api_models::CategoryResponse,
            api_models::CategoryUpdateRequest,


            // kolomoni_core::dictionary::translations
            api_models::TranslationCreationRequest,
            api_models::TranslationDeletionRequest,


            // kolomoni_core::dictionary::slovene::word
            api_models::SloveneWordWithMeanings,
            api_models::SloveneWordsResponse,
            api_models::SloveneWordsListRequest,
            api_models::SloveneWordCreationRequest,
            api_models::SloveneWordCreationResponse,
            api_models::SloveneWordInfoResponse,
            api_models::SloveneWordUpdateRequest,

            // kolomoni_core::dictionary::slovene::meaning
            api_models::ShallowSloveneWordMeaning,
            api_models::SloveneWordMeaning,
            api_models::SloveneWordMeaningWithCategoriesAndTranslations,
            api_models::SloveneWordMeaningsResponse,
            api_models::NewSloveneWordMeaningRequest,
            api_models::NewSloveneWordMeaningCreatedResponse,
            api_models::SloveneWordMeaningUpdateRequest,
            api_models::SloveneWordMeaningUpdatedResponse,


            // kolomoni_core::dictionary::english::word
            api_models::EnglishWordWithMeanings,
            api_models::EnglishWordsResponse,
            api_models::EnglishWordsListRequest,
            api_models::EnglishWordCreationRequest,
            api_models::EnglishWordCreationResponse,
            api_models::EnglishWordInfoResponse,
            api_models::EnglishWordUpdateRequest,

            // kolomoni_core::dictionary::english::meaning
            api_models::ShallowEnglishWordMeaning,
            api_models::EnglishWordMeaning,
            api_models::EnglishWordMeaningWithCategoriesAndTranslations,
            api_models::EnglishWordMeaningsResponse,
            api_models::NewEnglishWordMeaningRequest,
            api_models::NewEnglishWordMeaningCreatedResponse,
            api_models::EnglishWordMeaningUpdateRequest,
            api_models::EnglishWordMeaningUpdatedResponse,
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

fn process_with_expanded_markdown_support(string: &mut String) {
    // Replace triple dash with em dash.
    *string = string.replace("---", "—");

    // Replace double dash with en dash.
    *string = string.replace("--", "–");
}

/// Filters out private comments from the API documentation
/// (marked by `--- EXCLUDE FROM PUBLIC API DOCUMENTATION ---`) and
/// removes duplicated summary paragraphs from description fields.
fn clean_up_documentation(documentation: &mut OpenApi) {
    if let Some(info_description) = documentation.info.description.as_mut() {
        remove_private_comments(info_description);
        process_with_expanded_markdown_support(info_description);
    }

    for path in documentation.paths.paths.values_mut() {
        if let Some(path_description) = path.description.as_mut() {
            if let Some(path_summary) = path.summary.as_mut() {
                remove_duplicated_summary_from_description(path_summary, path_description);
                process_with_expanded_markdown_support(path_summary);
            }

            remove_private_comments(path_description);
            process_with_expanded_markdown_support(path_description);
        }

        for operation in path.operations.values_mut() {
            if let Some(operation_description) = operation.description.as_mut() {
                if let Some(operation_summary) = operation.summary.as_mut() {
                    remove_duplicated_summary_from_description(
                        operation_summary,
                        operation_description,
                    );
                    process_with_expanded_markdown_support(operation_summary);
                }

                remove_private_comments(operation_description);
                process_with_expanded_markdown_support(operation_description);
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
