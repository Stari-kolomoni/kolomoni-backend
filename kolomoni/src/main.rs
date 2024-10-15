//! Stari Kolomoni backend API project.
//!
//! TODO This needs an update.
//!
//! # Workspace structure
//! - [`kolomoni`][crate] *(this crate)* --- provides the entire API surface,
//!   with [`actix_web`] as the server software.
//! - [`kolomoni_auth`] --- contains authentication, role,
//!   and JSON Web Token-related code.
//! - [`kolomoni_configuration`] --- contains the entire configuration schema,
//!   including code to load it and validate it.
//! - [`kolomoni_database`] --- handles the entire PostgreSQL
//!   database interaction (with [SeaORM][sea_orm] as an ORM layer).
//! - [`kolomoni_migrations`] --- contains database migrations from which the
//!   entire schema is autogenerated in [`kolomoni_database`].
//! - [`kolomoni_openapi`](../kolomoni_openapi/index.html ) --- generates an OpenAPI schema for the entire API surface.
//!   Most annotations from which this is generated are present near each endpoint function in
//!   [`kolomoni::api::v1`][crate::api::v1], but the finishing touches are done in this crate. This crate also has a binary
//!   that serves the API schema interactively through a [RapiDoc](https://rapidocweb.com/) frontend.
//! - [`kolomoni_test`](../kolomoni_test/index.html) --- contains end-to-end tests for the backend.
//! - [`kolomoni_search`](../kolomoni_search/index.html) --- contains search engine logic.
//! - [`kolomoni_test_util`](../kolomoni_test_util/index.html) --- contains shared code for the end-to-end tests in the
//!   [`kolomoni_test`](../kolomoni_test/index.html) crate.
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
//! |   > Authentication-related code, namely an Actix extractor that
//! |   > allows us to ergonomically check for roles and permissions.
//! |
//! |-> cli.rs
//! |   > Definition of the command-line interface.
//! |
//! |-> logging.rs
//! |   > Sets up logging via the `tracing` crate.
//! |
//! |-> state.rs
//! |   > Houses the entire application state that is shared between workers.
//! |   > It contains things like the current configuration and database connection.
//! ```
//!

use actix_web::error::JsonPayloadError;
use actix_web::{web, HttpServer};
use clap::Parser;
use kolomoni_configuration::Configuration;
use kolomoni_core::api_models::InvalidJsonBodyReason;
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
use crate::api::errors::EndpointError;
use crate::cli::CLIArgs;
use crate::logging::initialize_tracing;
use crate::state::ApplicationStateInner;


/*
#[derive(Debug, Error)]
pub enum PendingMigrationApplyError {
    #[error("failed to retrieve database migration status")]
    StatusError(
        #[from]
        #[source]
        StatusError,
    ),

    #[error("failed to apply migration")]
    MigrationApplyError(
        #[from]
        #[source]
        MigrationApplyError,
    ),
}

// TODO needs logging
pub async fn apply_pending_migrations(
    database_connection: &mut PgConnection,
) -> Result<(), PendingMigrationApplyError> {
    let manager = kolomoni_migrations::migrations::manager();

    let migrations = manager
        .migrations_with_status(
            database_connection,
            MigrationsWithStatusOptions::strict(),
        )
        .await?;

    let pending_migrations = migrations
        .into_iter()
        .filter(|migration| migration.status() == &MigrationStatus::Pending)
        .collect::<Vec<_>>();


    if pending_migrations.is_empty() {
        return Ok(());
    }


    for pending_migration in pending_migrations {
        pending_migration.execute_up(database_connection).await?;
    }

    Ok(())
} */


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

    let state_inner = ApplicationStateInner::new(configuration)
        .await
        .into_diagnostic()?;

    /* TODO pending rewrite
    state_inner
        .search
        .engine
        .initialize_with_fresh_entries()
        .await?; */


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

        #[cfg(feature = "with_test_facilities")]
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


    #[cfg(feature = "with_test_facilities")]
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
