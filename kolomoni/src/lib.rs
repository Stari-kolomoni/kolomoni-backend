use std::time::Duration;

use kolomoni_configuration::ForApiDatabaseConfiguration;
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    PgPool,
};

pub mod api;
pub(crate) mod authentication;
pub(crate) mod cli;
pub mod logging;
pub(crate) mod state;

pub async fn establish_database_connection_pool(
    database_configuration: &ForApiDatabaseConfiguration,
) -> Result<PgPool, sqlx::Error> {
    let mut connection_options = PgConnectOptions::new_without_pgpass()
        .application_name(&format!(
            "stari-kolomoni-backend-api_v{}",
            env!("CARGO_PKG_VERSION")
        ))
        .statement_cache_capacity(
            database_configuration
                .statement_cache_capacity
                .unwrap_or(200),
        )
        .host(&database_configuration.host)
        .port(database_configuration.port)
        .username(&database_configuration.username)
        .database(&database_configuration.database_name);

    if let Some(password) = &database_configuration.password {
        connection_options = connection_options.password(password.as_str());
    }


    PgPoolOptions::new()
        .idle_timeout(Some(Duration::from_secs(60 * 20)))
        .max_lifetime(Some(Duration::from_secs(60 * 60)))
        .min_connections(1)
        .max_connections(10)
        .test_before_acquire(true)
        .connect_with(connection_options)
        .await
}
