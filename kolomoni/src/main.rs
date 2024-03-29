use actix_web::error::JsonPayloadError;
use actix_web::{web, HttpServer};
use clap::Parser;
use kolomoni::connect_and_set_up_database;
use kolomoni_configuration::Configuration;
use miette::{Context, IntoDiagnostic, Result};
use tracing::info;

mod api;
mod authentication;
mod cli;
mod logging;
mod state;

#[cfg(feature = "with_test_facilities")]
mod testing;

use crate::api::api_router;
use crate::api::errors::APIError;
use crate::cli::CLIArgs;
use crate::logging::initialize_tracing;
use crate::state::ApplicationStateInner;



#[tokio::main]
async fn main() -> Result<()> {
    #[cfg(feature = "with_test_facilities")]
    {
        println!("-------------------------------------");
        println!("THIS IS AN INCREDIBLY IMPORTANT ERROR");
        println!("-------------------------------------");
        println!(
            "THIS BINARY HAS BEEN COMPILED WITH THE with_test_facilities FEATURE FLAG, \n\
            WHICH MEANS IT SHOULD ONLY BE USED FOR TESTING. IF YOU USE THIS IN PRODUCTION, \n\
            ANYONE CAN WIPE YOUR DATABASE REMOTELY. YOU HAVE BEEN WARNED"
        );
        println!("-------------------------------------");
        println!("THIS IS AN INCREDIBLY IMPORTANT ERROR");
        println!("-------------------------------------");
    }


    // Parse CLI arguments.
    let arguments = CLIArgs::parse();

    // Load configuration.
    let configuration = match arguments.configuration_file_path.as_ref() {
        Some(path) => {
            println!("Loading configuration: {}", path.display());
            Configuration::load_from_path(path)
        }
        None => {
            println!("Loading configuration at default path.");
            Configuration::load_from_default_path()
        }
    }
    .with_context(|| "Failed to load configuration file.")?;

    info!(
        file_path = configuration.file_path.to_string_lossy().as_ref(),
        "Configuration loaded."
    );


    let guard = initialize_tracing(
        configuration.logging.console_output_level_filter(),
        configuration.logging.log_file_output_level_filter(),
        &configuration.logging.log_file_output_directory,
        "kolomoni.log",
    )
    .wrap_err("Failed to initialize tracing.")?;


    // TODO Introduce request rate-limiting.

    let mut state_inner = ApplicationStateInner::new(configuration.clone()).await?;

    state_inner
        .search
        .engine
        .initialize_with_fresh_entries()
        .await?;


    let state = web::Data::new(state_inner);


    // Initialize and start the actix HTTP server.
    #[rustfmt::skip]
    #[allow(clippy::let_and_return)]
    let server = HttpServer::new(move || {
        let json_extractor_config = actix_web::web::JsonConfig::default()
            .error_handler(|payload_error, request| {
                match payload_error {
                    JsonPayloadError::ContentType  => {
                        APIError::client_error(
                            "non-JSON body. If your request body contains JSON, please signal that \
                            with the Content-Type: application/json header."
                        ).into()
                    },
                    JsonPayloadError::Serialize(error) => {
                        APIError::internal_reason(
                            format!("Failed to serialize to JSON: {:?}. Request: {:?}", error, request)
                        ).into()
                    },
                    JsonPayloadError::Deserialize(_) => {
                        APIError::client_error(
                            "invalid JSON body."
                        ).into()
                    },
                    JsonPayloadError::Overflow { .. } | JsonPayloadError::OverflowKnownLength { .. } => {
                        APIError::client_error(
                            "request body is too large."
                        ).into()
                    },
                    error => {
                        APIError::internal_reason(format!(
                            "Unrecognized error: {:?}",
                            error
                        )).into()
                    }
                }
            });

        // FIXME Modify permissive CORS to something more safe in production.
        let cors = actix_cors::Cors::permissive().expose_headers(vec![
            "Date",
            "Content-Type",
            "Last-Modified",
            "Content-Length",
        ]);

        let mut app = actix_web::App::new()
            .wrap(actix_web::middleware::Compress::default())
            .wrap(actix_web::middleware::NormalizePath::trim())
            .wrap(cors)
            .wrap(tracing_actix_web::TracingLogger::default())
            .app_data(json_extractor_config)
            .app_data(state.clone())
            .service(api_router());

        #[cfg(feature = "with_test_facilities")]
        {
            info!("Enabling testing endpoints.");

            app = app.service(testing::testing_router());
        }

        app
    })
        .bind((
            configuration.http.host.as_str(),
            configuration.http.port as u16,
        ))
        .into_diagnostic()
        .wrap_err("Failed to set up actix HTTP server.")?;


    #[cfg(feature = "with_test_facilities")]
    {
        // We use this line to check (in the logs) that the server
        // is alive and running. We use println instead of tracing
        // because it goes directly to stdout.

        // TODO Maybe a proper health check (e.g. a /ping) would be better?

        println!("Server initialized and running.");
    }

    info!(
        host = configuration.http.host.as_str(),
        port = configuration.http.port as u16,
        "HTTP server initialized and running."
    );

    // Run HTTP server until stopped.
    server
        .run()
        .await
        .into_diagnostic()
        .wrap_err("Errored while running actix HTTP server.")?;


    drop(guard);

    Ok(())
}
