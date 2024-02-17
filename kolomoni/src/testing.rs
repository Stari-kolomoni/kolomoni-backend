//! A test-only API. Compiled into the binary and enabled only when
//! the `with_test_facilities` feature flag is enabled.

use actix_web::{post, web, HttpResponse, Scope};
use kolomoni_migrations::Migrator;
use miette::{Context, IntoDiagnostic, Result};
use sea_orm::DatabaseConnection;
use sea_orm_migration::MigratorTrait;
use tracing::info;

use crate::{
    api::errors::{APIError, EndpointResult},
    state::ApplicationState,
};

pub async fn drop_database_and_reapply_migrations(
    database_connection: &DatabaseConnection,
) -> Result<()> {
    use tracing::warn;

    warn!("Dropping the entire database and reapplying all migrations.");

    Migrator::fresh(database_connection)
        .await
        .into_diagnostic()
        .wrap_err("Failed to drop database and reapply migrations.")?;

    info!("Database reset.");

    Ok(())
}

#[post("/full-reset")]
pub async fn reset_server(state: ApplicationState) -> EndpointResult {
    drop_database_and_reapply_migrations(&state.database)
        .await
        .map_err(APIError::InternalError)?;

    Ok(HttpResponse::Ok().finish())
}


#[rustfmt::skip]
pub fn testing_router() -> Scope {
    web::scope("/testing")
        .service(reset_server)
}
