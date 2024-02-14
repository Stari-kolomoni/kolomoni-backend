use kolomoni_auth::JsonWebTokenManager;
use kolomoni_configuration::Configuration;
use kolomoni_database::mutation::ArgonHasher;
use sea_orm::DatabaseConnection;



/// Central application state.
///
/// Use [`ApplicationState`] instead as it already wraps this struct
/// in [`actix_web::web::Data`]!
///
/// See <https://actix.rs/docs/application#state> for more information.
///
/// If you need mutable state, opt for internal mutability as the struct
/// is internally essentially wrapped in an `Arc` by actix.
/// For more information about mutable state, see
/// <https://actix.rs/docs/application#shared-mutable-state>.
///
/// ## Examples
/// ```
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
pub struct ApplicationStateInner {
    /// The configuration that this server was loaded with.
    pub configuration: Configuration,

    /// Password hasher helper struct.
    pub hasher: ArgonHasher,

    /// PostgreSQL database connection.
    pub database: DatabaseConnection,

    /// Authentication token manager (JSON Web Token).
    pub jwt_manager: JsonWebTokenManager,
}


/// An [`actix_web::web::Data`] wrapper for [`ApplicationStateInner`].
///
/// See [`ApplicationStateInner`] and <https://actix.rs/docs/application#state>
/// for more information.
pub type ApplicationState = actix_web::web::Data<ApplicationStateInner>;
