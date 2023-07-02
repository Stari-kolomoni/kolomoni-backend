use sea_orm::DatabaseConnection;

use crate::auth::token::JsonWebTokenManager;
use crate::configuration::Config;
use crate::database::mutation::ArgonHasher;

/// Central application state.
/// See https://actix.rs/docs/application#state for more information.
///
/// *Careful with any kind of shared mutable state, read this first,
/// as it won't work by default (may need an `Arc`, depending on use-case):*
/// https://actix.rs/docs/application#shared-mutable-state
///
/// ## Examples
/// ```
/// #[post("")]
/// pub async fn some_endpoint(
///     state: web::Data<AppState>,
/// ) -> EndpointResult {
///     // state.database, state.configuration, ...
/// }
/// ```
pub struct AppState {
    pub configuration: Config,

    pub hasher: ArgonHasher,

    pub database: DatabaseConnection,

    pub jwt_manager: JsonWebTokenManager,
}
