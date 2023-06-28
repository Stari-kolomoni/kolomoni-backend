use std::net::Ipv4Addr;

use actix_web::middleware::Logger;
use actix_web::{App, HttpServer};
use anyhow::{Context, Result};
use stari_kolomoni_backend::api::errors;
use stari_kolomoni_backend::api::v1::login;
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::{Modify, OpenApi};
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    paths(login::login),
    components(
        schemas(
            login::UserLoginRequest,
            login::UserLoginResponse,
            errors::ErrorReasonResponse
        ),
    ),
    info(
        title = "Stari Kolomoni API",
        description = "",
        contact(
            name = "Stari Kolomoni Team",
            email = "<stari.kolomoni@gmail.com>"
        ),
        license(
            name = "GPL-3.0-only",
            url = "https://github.com/Stari-kolomoni/kolomoni-backend-rust/blob/master/LICENSE.md"
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
            "bearer_token",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .build(),
            ),
        );
    }
}

#[actix_web::main]
async fn main() -> Result<()> {
    // TODO
    let open_api = APIDocumentation::openapi();

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .service(SwaggerUi::new("/api-documentation/{_:.*}").url(
                "/api-documentation/openapi.json",
                open_api.clone(),
            ))
    })
    .bind((Ipv4Addr::LOCALHOST, 8877))
    .with_context(|| "Failed to set up actix HTTP server.")?
    .run()
    .await
    .with_context(|| "Errored while running HTTP server.")?;

    Ok(())
}
