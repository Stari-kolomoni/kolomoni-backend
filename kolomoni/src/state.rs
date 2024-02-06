use kolomoni_auth::token::JsonWebTokenManager;
use kolomoni_configuration::Configuration;
use kolomoni_database::mutation::ArgonHasher;
use sea_orm::DatabaseConnection;


/// Central application state.
///
/// See <https://actix.rs/docs/application#state> for more information.
///
/// *Careful with any kind of shared mutable state; read this first,
/// as it won't work by default (may need an `Arc`, depending on use-case):*
/// <https://actix.rs/docs/application#shared-mutable-state>
///
/// ## Examples
/// ```
/// # use actix_web::{post, web};
/// # use kolomoni::api::errors::EndpointResult;
/// # use kolomoni::state::AppState;
/// #[post("")]
/// pub async fn some_endpoint(
///     state: web::Data<AppState>,
/// ) -> EndpointResult {
///     // state.database, state.configuration, ...
/// # todo!();
/// }
/// ```
pub struct AppState {
    pub configuration: Configuration,

    pub hasher: ArgonHasher,

    pub database: DatabaseConnection,

    pub jwt_manager: JsonWebTokenManager,
}
