use sea_orm::DatabaseConnection;

use crate::configuration::Config;
use crate::database::mutation::users::ArgonHasher;
use crate::jwt::JsonWebTokenManager;

pub struct AppState {
    pub configuration: Config,
    pub hasher: ArgonHasher,
    pub database: DatabaseConnection,
    pub jwt_manager: JsonWebTokenManager,
}
