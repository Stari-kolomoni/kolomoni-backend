//! Stari Kolomoni backend API (and API client) project.
//!
//! # Workspace structure
//! - [`kolomoni`][crate] *(this crate)* provides the entire V1 API (using [`actix-web`]).
//! - [`kolomoni_core`] contains reusable functionality (API models and other reusable types,
//!   JWT token code, permission and role systems, etc.).
//! - [`kolomoni_configuration`] defines the entire configuration schema and the code to load it from disk and validate it.
//! - [`kolomoni_cache`] implements a full type-safe memory cache for database entities (this is used by our search engine
//!   for very fast search results).
//! - [`kolomoni_database`] handles PostgreSQL database querying and modification (using [`sqlx`]).
//! - [`kolomoni_search`] implements our custom search engine (based on [`tantivy`]).
//! - [`kolomoni_search_core`] contains shared search engine models and code for reuse.
//! - [`kolomoni_migrations`] implements a custom database migration system (see `migrations` subfolder).
//! - [`kolomoni_migrations_core`] defines reusable migration models and functionality.
//! - [`kolomoni_migrations_macros`] defines handy proc macros for embedding migrations into the binary at build time
//!   and for being able to elegantly write migrations in pure Rust instead of SQL (if more complex migration logic is required).
//! - [`kolomoni_seeder`] implementst a system for seeding our database with existing words, meanings, and translations
//!   from our previous translation system (based on spreadsheets and CSV).
//! - [`kolomoni_api_client`] implements a full-featured API client (with proper error handling for each endpoint) for our Stari Kolomoni API.
//!   This is particularly useful for our end-to-end tests.
//! - [`kolomoni_openapi`] assembles an OpenAPI schema for the entire API surface (it can emit it as a JSON file or serve it through the [RapiDoc](https://rapidocweb.com/) frontend).
//!   The final OpenAPI schema is assembled from individual annotations that are present near each endpoint function in
//!   [`kolomoni::api::v1`][crate::api::v1].
//! - [`kolomoni_test`] contains end-to-end tests for our entire backend.
//! - [`kolomoni_test_core`] contains shared code for the end-to-end tests, wrapping the [`kolomoni_api_client`] crate to provide a nice test interface
//!   as well as providing some sample data to test on.
//! - [`kolomoni_test_macros`] defines a simple custom `#[test]` macro for setting up asynchronous tests and other features.
//!
//!
//! # Structure of this crate
//! ```markdown
//! kolomoni/src
//! |
//! |-| api/
//! | |
//! | |-| v1/
//! | |   > Contains the entire API surface.
//! | |
//! | |-> errors.rs
//! | |   > Ways of handling errors, namely the `APIError` struct, which allows
//! | |   > you to simply return an `Err(APIError)` and have it automatically
//! | |   > return a relevant HTTP error response. Also important: `EndpointResult`.
//! | |
//! | |-> macros.rs
//! | |   > Macros to avoid repeating code, such as `impl_json_response_builder`,
//! | |   > which enables structs to automatically convert to 200 OK JSON via `into_response`.
//! | |   > Also has macros to handle authentication and require permissions.
//! | |
//! | |-> openapi.rs
//! | |   > Defines commonly-used OpenAPI / `utoipa` parameters and responses,
//! | |   > which you can then use when documenting endpoint functions with
//! | |   > the `utoipa::path` macro.
//! |
//! |-> authentication.rs
//! |   > Authentication-related code, namely an actix_web extractor that
//! |   > allows us to ergonomically check for roles and permissions.
//! |
//! |-> cli.rs
//! |   > Definition of the command-line interface.
//! |
//! |-> logging.rs
//! |   > Sets up console and file logging via the `tracing` crate.
//! |
//! |-> state.rs
//! |   > Houses the entire application state that is shared between workers
//! |   > (things like the current configuration, entity cache, and a database connection).
//! |
//! |-> testing.rs
//! |   > Defines additional test-build-only API endpoints that allow us to
//! |   > access restricted parts of our system. This module is only compiled
//! |   > into the binary if the `e2e-testing` feature flag is set.
//! ```
//!

use std::time::Duration;

use actix_web::error::JsonPayloadError;
use actix_web::{web, HttpServer};
use clap::Parser;
use kolomoni_configuration::Configuration;
use kolomoni_core::api_models::InvalidJsonBodyReason;
use kolomoni_core::cancellation::CancellationToken;
use miette::{Context, IntoDiagnostic, Result};
use tokio::runtime;
use tracing::info;

pub mod api;
pub mod authentication;
pub mod cli;
pub mod logging;
pub mod state;

#[cfg(feature = "e2e-testing")]
mod testing;

use crate::api::api_router;
use crate::api::errors::EndpointError;
use crate::cli::CLIArgs;
use crate::logging::initialize_tracing;
use crate::state::ApplicationStateInner;



async fn async_main(cancellation_token: CancellationToken) -> Result<()> {
    #[cfg(feature = "e2e-testing")]
    {
        println!("-------------------------------------");
        println!("THIS IS AN INCREDIBLY IMPORTANT ERROR");
        println!("-------------------------------------");
        println!(
            "THIS BINARY HAS BEEN COMPILED WITH THE e2e-testing FEATURE FLAG, \n\
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
    .into_diagnostic()
    .wrap_err("Failed to load configuration file.")?;

    info!(
        file_path = configuration
            .configuration_file_path
            .to_string_lossy()
            .as_ref(),
        "Configuration loaded."
    );


    configuration
        .base_paths
        .create_base_data_directory_if_missing()
        .into_diagnostic()?;

    configuration
        .search
        .create_search_index_directory_if_missing()
        .into_diagnostic()?;



    let logging_guard = initialize_tracing(
        configuration.logging.console_output_level_filter(),
        configuration.logging.log_file_output_level_filter(),
        &configuration.logging.log_file_output_directory,
        "kolomoni.log",
    )
    .wrap_err("Failed to initialize tracing.")?;


    // TODO Introduce request rate-limiting.


    let http_server_configuration = configuration.http.clone();


    // TODO add ctrlc handler to set the background task cancellation task?
    //      then test if the program stops correctly
    let state_inner = ApplicationStateInner::new(configuration, cancellation_token)
        .await
        .into_diagnostic()
        .wrap_err("failed to initialize application state")?;

    let state = web::Data::new(state_inner);


    // Initialize and start the actix HTTP server.
    #[rustfmt::skip]
    #[allow(clippy::let_and_return)]
    let server = HttpServer::new(move || {
        let json_extractor_config = actix_web::web::JsonConfig::default()
            .error_handler(|payload_error, request| {
                match payload_error {
                    JsonPayloadError::ContentType  => {
                        EndpointError::missing_json_body().into()
                    },
                    JsonPayloadError::Serialize(error) => {
                        EndpointError::internal_error_with_reason(
                            format!("Failed to serialize to JSON: {:?}. Request: {:?}", error, request)
                        ).into()
                    },
                    JsonPayloadError::Deserialize(error) => {
                        match error.classify() {
                            serde_json::error::Category::Io | serde_json::error::Category::Syntax | serde_json::error::Category::Eof => {
                                EndpointError::invalid_json_body(InvalidJsonBodyReason::NotJson).into()
                            }
                            serde_json::error::Category::Data => {
                                EndpointError::invalid_json_body(InvalidJsonBodyReason::InvalidData).into()
                            }
                        }
                    },
                    JsonPayloadError::Overflow { .. } | JsonPayloadError::OverflowKnownLength { .. } => {
                        EndpointError::invalid_json_body(InvalidJsonBodyReason::TooLarge).into()
                    },
                    error => {
                        EndpointError::internal_error_with_reason(format!(
                            "Unhandled JSON error: {:?}",
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

        #[allow(unused_mut)]
        let mut app = actix_web::App::new()
            .wrap(actix_web::middleware::Compress::default())
            .wrap(actix_web::middleware::NormalizePath::trim())
            .wrap(cors)
            .wrap(tracing_actix_web::TracingLogger::default())
            .app_data(json_extractor_config)
            .app_data(state.clone())
            .service(api_router());

        #[cfg(feature = "e2e-testing")]
        {
            info!("Enabling testing endpoints.");

            app = app.service(testing::testing_router());
        }

        app
    })
        .bind((
            http_server_configuration.host.as_str(),
            http_server_configuration.port as u16,
        ))
        .into_diagnostic()
        .wrap_err("Failed to set up actix HTTP server.")?;


    #[cfg(feature = "e2e-testing")]
    {
        // We use this line to check (in the logs) that the server
        // is alive and running. We use println instead of tracing
        // because it goes directly to stdout.

        // TODO Maybe a proper health check (e.g. a /ping) would be better?

        println!("Server initialized and running.");
    }

    info!(
        host = http_server_configuration.host.as_str(),
        port = http_server_configuration.port as u16,
        "HTTP server initialized and running."
    );

    // Run HTTP server until stopped.
    server
        .run()
        .await
        .into_diagnostic()
        .wrap_err("Errored while running actix HTTP server.")?;


    drop(logging_guard);

    Ok(())
}



fn main() -> Result<()> {
    println!("Initializing tokio async runtime.");

    let cancellation_token = CancellationToken::new();

    let async_runtime = runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .into_diagnostic()
        .wrap_err("failed to initialize async runtime")?;

    println!("Executing async entry point.");

    let entry_point_result = async_runtime.block_on(async_main(cancellation_token.clone()));

    println!(
        "Async entry point has returned (is_err={}); setting cancellation token and waiting 500 ms.",
        entry_point_result.is_err()
    );

    cancellation_token.cancel();

    std::thread::sleep(Duration::from_millis(500));

    entry_point_result
}
