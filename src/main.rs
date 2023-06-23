// TODO Integrate Atlas for migrations
// TODO Integrate utoipa for OpenAPI documentation

use anyhow::{Context, Result};

mod api;
mod configuration;

use actix_web::{App, HttpServer};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use crate::api::ping::ping;
use crate::configuration::Config;

#[tokio::main]
async fn main() -> Result<()> {
    let tracing_subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish();

    tracing::subscriber::set_global_default(tracing_subscriber)
        .with_context(|| "Failed to set up tracing formatter.")?;

    let configuration =
        Config::load_from_default_path().with_context(|| "Failed to load configuration.")?;
    info!(
        file_path = configuration.file_path.to_string_lossy().as_ref(),
        "Configuration loaded."
    );

    #[rustfmt::skip]
    let server = HttpServer::new(
        || App::new()
            .service(api::api_router())
            .service(ping)
    )
        .bind((
            configuration.server.host.as_str(),
            configuration.server.port as u16,
        ))
        .with_context(|| "Failed to set up actix HTTP server.")?;
    info!(
        host = configuration.server.host.as_str(),
        port = configuration.server.port as u16,
        "HTTP server initialized, running."
    );

    server
        .run()
        .await
        .with_context(|| "Errored while running HTTP server.")?;

    Ok(())
}
