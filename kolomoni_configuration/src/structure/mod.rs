use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

mod base_paths;
mod database;
mod http;
mod json_web_token;
mod logging;
mod search;
mod secrets;

pub use base_paths::*;
pub use database::*;
pub use http::*;
pub use json_web_token::*;
pub use logging::*;
pub use search::*;
pub use secrets::*;

use crate::traits::{Resolve, ResolveWithContext, TryResolve, TryResolveWithContext};
use crate::utilities::get_default_configuration_file_path;
use crate::{ConfigurationLoadingError, ConfigurationResolutionError};



#[derive(Deserialize, Debug)]
pub(crate) struct UnresolvedConfiguration {
    /// Base paths.
    base_paths: UnresolvedBasePathsConfiguration,

    /// Logging-related configuration.
    logging: UnresolvedLoggingConfiguration,

    /// Configuration related to the HTTP server.
    http: UnresolvedHttpConfiguration,

    /// Configuration related to the database.
    database: UnresolvedDatabaseConfiguration,

    /// Password-related configuration.
    secrets: UnresolvedSecretsConfiguration,

    /// Json Web Token-related configuration.
    json_web_token: UnresolvedJsonWebTokenConfiguration,

    /// Search-related configuration.
    search: UnresolvedSearchConfiguration,
}


/// The entire Stari Kolomoni backend configuration.
#[derive(Debug, Clone)]
pub struct Configuration {
    /// This is the file path this [`Configuration`] instance was loaded from.
    pub configuration_file_path: PathBuf,

    /// Base paths
    pub base_paths: BasePathsConfiguration,

    /// Logging-related configuration.
    pub logging: LoggingConfiguration,

    /// Configuration related to the HTTP server.
    pub http: HttpConfiguration,

    /// Configuration related to the database.
    pub database: DatabaseConfiguration,

    /// Password-related configuration.
    pub secrets: SecretsConfiguration,

    /// Json Web Token-related configuration.
    pub json_web_token: JsonWebTokenConfiguration,

    /// Search-related configuration.
    pub search: SearchConfiguration,
}



pub(crate) struct ConfigurationResolutionContext {
    configuration_file_path: PathBuf,
}


impl TryResolveWithContext for UnresolvedConfiguration {
    type Resolved = Configuration;
    type Context = ConfigurationResolutionContext;
    type Error = ConfigurationResolutionError;

    fn try_resolve_with_context(
        self,
        context: Self::Context,
    ) -> Result<Self::Resolved, Self::Error> {
        let base_paths = self.base_paths.resolve();
        let logging = self.logging.try_resolve()?;
        let http = self.http.resolve();
        let database = self.database.resolve();
        let secrets = self.secrets.resolve();
        let json_web_token = self.json_web_token.resolve();
        let search = self.search.resolve_with_context(&base_paths);

        Ok(Configuration {
            base_paths,
            configuration_file_path: context.configuration_file_path,
            logging,
            http,
            database,
            secrets,
            json_web_token,
            search,
        })
    }
}


impl Configuration {
    /// Load the configuration from a specific file path.
    pub fn load_from_path<S: AsRef<Path>>(
        configuration_file_path: S,
    ) -> Result<Self, ConfigurationLoadingError> {
        // Read the configuration file into memory as a string.
        let configuration_string =
            fs::read_to_string(configuration_file_path.as_ref()).map_err(|error| {
                ConfigurationLoadingError::UnableToReadConfigurationFile {
                    path: configuration_file_path.as_ref().to_path_buf(),
                    error,
                }
            })?;

        // Parse the string into the [`UnresolvedConfiguration`] structure and then resolve it.
        let unresolved_configuration =
            toml::from_str::<UnresolvedConfiguration>(&configuration_string)
                .map_err(|error| ConfigurationLoadingError::ParsingError { error })?;

        let resolved_configuration =
            unresolved_configuration.try_resolve_with_context(ConfigurationResolutionContext {
                configuration_file_path: configuration_file_path.as_ref().to_path_buf(),
            })?;

        Ok(resolved_configuration)
    }

    /// Load the configuration from the default path (`./data/configuration.toml`).
    pub fn load_from_default_path() -> Result<Self, ConfigurationLoadingError> {
        Configuration::load_from_path(get_default_configuration_file_path())
    }
}
