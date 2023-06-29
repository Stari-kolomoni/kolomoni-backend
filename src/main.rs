use std::env;
use std::env::VarError;

use actix_cors::Cors;
use anyhow::{Context, Result};

mod api;
mod cli;
mod configuration;
mod database;
pub mod jwt;
pub mod state;

use actix_web::{middleware, web, App, HttpServer};
use clap::Parser;
use migration::{Migrator, MigratorTrait};
use sea_orm::{Database, DatabaseConnection};
use tracing::info;
use tracing_actix_web::TracingLogger;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

use crate::cli::CLIArgs;
use crate::configuration::Config;
use crate::database::mutation::ArgonHasher;
use crate::jwt::JsonWebTokenManager;
use crate::state::AppState;

pub async fn connect_and_set_up_database(config: &Config) -> Result<DatabaseConnection> {
    let database = Database::connect(format!(
        "postgres://{}:{}@{}:{}/{}",
        config.database.username,
        config.database.password,
        config.database.host,
        config.database.port,
        config.database.database_name,
    ))
    .await
    .with_context(|| "Could not initialize connection to PostgreSQL database.")?;

    info!("Database connection established.");

    Migrator::up(&database, None)
        .await
        .with_context(|| "Could not apply database migration.")?;

    info!("Migrations applied.");

    Ok(database)
}

#[tokio::main]
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

    // Parse CLI arguments.
    let arguments = CLIArgs::parse();

    // Load configuration.
    let configuration = match arguments.configuration_file_path.as_ref() {
        Some(path) => Config::load_from_path(path),
        None => Config::load_from_default_path(),
    }
    .with_context(|| "Failed to load configuration file.")?;

    info!(
        file_path = configuration.file_path.to_string_lossy().as_ref(),
        "Configuration loaded."
    );

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
            // Maximum configured JSON payload size is 1 MB.
            let json_extractor_config = web::JsonConfig::default().limit(1048576);

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
        .with_context(|| "Failed to set up actix HTTP server.")?;

    info!(
        host = configuration.http.host.as_str(),
        port = configuration.http.port as u16,
        "HTTP server initialized and running."
    );

    server
        .run()
        .await
        .with_context(|| "Errored while running HTTP server.")?;

    Ok(())
}
