// TODO Integrate Atlas for migrations (?)

// TODO Integrate utoipa for OpenAPI documentation

// TODO Integrate SeaORM for Postgres database (or maybe raw sqlx with SeaQuery??)

use std::env;
use std::env::VarError;

use anyhow::{Context, Result};

mod api;
mod configuration;
mod database;
pub mod state;

use actix_web::{web, App, HttpServer};
use migration::{Migrator, MigratorTrait};
use sea_orm::{Database, DatabaseConnection};
use tracing::info;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

use crate::configuration::Config;
use crate::database::mutation::users::ArgonHasher;
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

    // Load configuration
    let configuration =
        Config::load_from_default_path().with_context(|| "Failed to load configuration.")?;
    info!(
        file_path = configuration.file_path.to_string_lossy().as_ref(),
        "Configuration loaded."
    );

    let database = connect_and_set_up_database(&configuration).await?;
    let hasher = ArgonHasher::new(&configuration)?;

    let state = web::Data::new(AppState {
        configuration: configuration.clone(),
        hasher,
        database,
    });

    #[rustfmt::skip]
    let server = HttpServer::new(
        move || {
            // Maximum configured JSON payload size is 1 MB.
            let json_extractor_config = web::JsonConfig::default().limit(1048576);
            
            App::new()
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
        "HTTP server initialized, running."
    );

    server
        .run()
        .await
        .with_context(|| "Errored while running HTTP server.")?;

    Ok(())
}
