use actix_cors::Cors;
use kolomoni_auth::token::JsonWebTokenManager;
use kolomoni_configuration::Configuration;
use kolomoni_database::mutation::ArgonHasher;
use miette::{Context, IntoDiagnostic, Result};

mod api;
mod authentication;
mod cli;
mod logging;
mod state;

use actix_web::{middleware, web, App, HttpServer};
use clap::Parser;
use kolomoni_migrations::{Migrator, MigratorTrait};
use sea_orm::{Database, DatabaseConnection};
use tracing::info;
use tracing_actix_web::TracingLogger;

use crate::cli::CLIArgs;
use crate::logging::initialize_tracing;
use crate::state::AppState;


/// Connect to PostgreSQL database as specified in the configuration file
/// and apply any pending migrations.
pub async fn connect_and_set_up_database(config: &Configuration) -> Result<DatabaseConnection> {
    let database = Database::connect(format!(
        "postgres://{}:{}@{}:{}/{}",
        config.database.username,
        config.database.password,
        config.database.host,
        config.database.port,
        config.database.database_name,
    ))
    .await
    .into_diagnostic()
    .wrap_err("Could not initialize connection to PostgreSQL database.")?;

    info!("Database connection established.");

    Migrator::up(&database, None)
        .await
        .into_diagnostic()
        .wrap_err("Could not apply database migration.")?;

    info!("Migrations applied.");

    Ok(database)
}


#[tokio::main]
async fn main() -> Result<()> {
    // Parse CLI arguments.
    let arguments = CLIArgs::parse();

    // Load configuration.
    let configuration = match arguments.configuration_file_path.as_ref() {
        Some(path) => Configuration::load_from_path(path),
        None => Configuration::load_from_default_path(),
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
    )
    .wrap_err("Failed to initialize tracing.")?;


    // TODO Introduce request rate-limiting.

    // Initialize database connection and other static structs.
    let database = connect_and_set_up_database(&configuration).await?;
    let hasher = ArgonHasher::new(&configuration)?;
    let json_web_token_manager = JsonWebTokenManager::new(&configuration);

    let state = web::Data::new(AppState {
        configuration: configuration.clone(),
        hasher,
        database,
        jwt_manager: json_web_token_manager,
    });

    // Initialize and start the actix HTTP server.
    #[rustfmt::skip]
    let server = HttpServer::new(
        move || {
            let json_extractor_config = web::JsonConfig::default();

            // FIXME Modify permissive CORS to something more safe in production.
            let cors = Cors::permissive();
            
            App::new()
                .wrap(middleware::NormalizePath::trim())
                .wrap(cors)
                .wrap(TracingLogger::default())
                .service(api::api_router())
                .app_data(json_extractor_config)
                .app_data(state.clone())
        }
    )
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
