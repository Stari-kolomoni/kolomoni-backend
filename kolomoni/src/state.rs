//! Application-wide state (shared between endpoint functions).

use actix_web::web::Data;
use kolomoni_auth::JsonWebTokenManager;
use kolomoni_configuration::Configuration;
use kolomoni_database::mutation::ArgonHasher;
use kolomoni_search::KolomoniSearchEngine;
use miette::Result;
use sea_orm::DatabaseConnection;

use crate::connect_and_set_up_database;



/// Central application state.
///
/// Use [`ApplicationState`] instead as it already wraps this struct
/// in [`actix_web::web::Data`]!
///
/// If you need mutable state, opt for internal mutability as the struct
/// is internally essentially wrapped in an `Arc` by actix.
/// For more information about mutable state, see
/// <https://actix.rs/docs/application#shared-mutable-state>.
pub struct ApplicationStateInner {
    /// The configuration that this server was loaded with.
    pub configuration: Configuration,

    /// Password hasher helper struct.
    pub hasher: ArgonHasher,

    /// PostgreSQL database connection.
    pub database: DatabaseConnection,

    /// Authentication token manager (JSON Web Token).
    pub jwt_manager: JsonWebTokenManager,

    pub search: KolomoniSearchEngine,
}

impl ApplicationStateInner {
    pub async fn new(configuration: Configuration) -> Result<Self> {
        let hasher = ArgonHasher::new(&configuration)?;
        let database = connect_and_set_up_database(&configuration).await?;
        let jwt_manager = JsonWebTokenManager::new(&configuration.json_web_token.secret);
        let search = KolomoniSearchEngine::new(&configuration).await?;

        Ok(Self {
            configuration,
            hasher,
            database,
            jwt_manager,
            search,
        })
    }
}


/// Central application state, wrapped in an actix [`Data`] wrapper-
///
///
/// This enables usage in endpoint functions.///
/// See <https://actix.rs/docs/application#state> for more information.
///
/// # Examples
/// ```no_run
/// # use actix_web::{post, web};
/// # use kolomoni::api::errors::EndpointResult;
/// # use kolomoni::state::ApplicationState;
/// #[post("")]
/// pub async fn some_endpoint(
///     state: ApplicationState,
/// ) -> EndpointResult {
///     // state.database, state.configuration, ...
///     # todo!();
/// }
/// ```
pub type ApplicationState = Data<ApplicationStateInner>;
