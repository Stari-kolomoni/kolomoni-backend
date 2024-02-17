use actix_web::{web, HttpServer};
use clap::Parser;
use kolomoni::connect_and_set_up_database;
use kolomoni_auth::JsonWebTokenManager;
use kolomoni_configuration::Configuration;
use kolomoni_database::mutation::ArgonHasher;
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
            println!(
                "Loading configuration: {}",
                arguments.configuration_file_path
            );
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

    // Initialize database connection and other static structs.
    let database = connect_and_set_up_database(&configuration).await?;
    let hasher = ArgonHasher::new(&configuration)?;
    let json_web_token_manager = JsonWebTokenManager::new(&configuration.json_web_token.secret);

    let state = web::Data::new(ApplicationStateInner {
        configuration: configuration.clone(),
        hasher,
        database,
        jwt_manager: json_web_token_manager,
    });


    // Initialize and start the actix HTTP server.
    #[rustfmt::skip]
    #[allow(clippy::let_and_return)]
    let server = HttpServer::new(move || {
        let json_extractor_config = actix_web::web::JsonConfig::default();

        // FIXME Modify permissive CORS to something more safe in production.
        let cors = actix_cors::Cors::permissive().expose_headers(vec![
            "Date",
            "Content-Type",
            "Last-Modified",
            "Content-Length",
        ]);

        let mut app = actix_web::App::new()
            .wrap(actix_web::middleware::NormalizePath::trim())
            .wrap(cors)
            .wrap(tracing_actix_web::TracingLogger::default())
            .app_data(json_extractor_config)
            .app_data(state.clone())
            .service(api_router());

        #[cfg(feature = "with_test_facilities")]
        {
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
