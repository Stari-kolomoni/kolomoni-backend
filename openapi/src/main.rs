use std::env;
use std::env::VarError;
use std::net::Ipv4Addr;

use actix_web::{App, HttpServer};
use anyhow::{Context, Result};
use stari_kolomoni_backend::api::errors;
use stari_kolomoni_backend::api::v1::login;
use stari_kolomoni_backend::api::v1::ping;
use stari_kolomoni_backend::api::v1::users;
use tracing::info;
use tracing_actix_web::TracingLogger;
use tracing_subscriber::{EnvFilter, FmtSubscriber};
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::{Modify, OpenApi};
use utoipa_swagger_ui::SwaggerUi;


#[derive(OpenApi)]
#[openapi(
    paths(
        ping::ping,
        login::login,
        login::refresh_login,
        users::get_all_registered_users,
        users::register_user,
        users::get_current_user_info,
        users::get_current_user_permissions,
        users::update_current_user_display_name,
        users::get_specific_user_info,
        users::get_specific_user_permissions,
        users::update_specific_user_display_name,
        users::add_permissions_to_specific_user,
        users::remove_permissions_from_specific_user,
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
            url = "https://github.com/Stari-kolomoni/kolomoni-backend-rust/blob/master/LICENSE.md"
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
        let components = openapi.components.as_mut().unwrap(); // we can unwrap safely since there already is components registered.

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


#[actix_web::main]
async fn main() -> Result<()> {
    // Initialize logging and tracing.
    if let Err(error) = env::var("RUST_LOG") {
        if error == VarError::NotPresent {
            env::set_var("RUST_LOG", "INFO");
        }
    }

    let tracing_subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .finish();

    tracing::subscriber::set_global_default(tracing_subscriber)
        .with_context(|| "Failed to set up tracing formatter.")?;

    // Initialize compile-time generated OpenApi documentation.
    let open_api = APIDocumentation::openapi();

    // Start actix HTTP server to serve the documentation.
    let server = HttpServer::new(move || {
        App::new().wrap(TracingLogger::default()).service(
            SwaggerUi::new("/api-documentation/{_:.*}").url(
                "/api-documentation/openapi.json",
                open_api.clone(),
            ),
        )
    })
    .bind((Ipv4Addr::LOCALHOST, 8877))
    .with_context(|| "Failed to set up actix HTTP server.")?;

    info!("HTTP server initialized, running.");

    server
        .run()
        .await
        .with_context(|| "Errored while running HTTP server.")?;

    Ok(())
}
