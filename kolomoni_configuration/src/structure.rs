use std::fs;
use std::path::{Path, PathBuf};

use miette::{Context, IntoDiagnostic, Result};
use serde::Deserialize;

mod base_paths;
mod database;
mod http;
mod json_web_token;
mod logging;
mod search;
mod secrets;

pub use base_paths::BasePathsConfiguration;
use base_paths::UnresolvedBasePathsConfiguration;
pub use database::DatabaseConfiguration;
use database::UnresolvedDatabaseConfiguration;
pub use http::HttpConfiguration;
use http::UnresolvedHttpConfiguration;
pub use json_web_token::JsonWebTokenConfiguration;
use json_web_token::UnresolvedJsonWebTokenConfiguration;
pub use logging::LoggingConfiguration;
use logging::UnresolvedLoggingConfiguration;
pub use search::SearchConfiguration;
use search::UnresolvedSearchConfiguration;
pub use secrets::SecretsConfiguration;
use secrets::UnresolvedSecretsConfiguration;

use crate::traits::{ResolvableConfiguration, ResolvableConfigurationWithContext};
use crate::utilities::get_default_configuration_file_path;

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
    /// Base paths
    pub base_paths: BasePathsConfiguration,

    /// This is the file path this `Config` instance was loaded from.
    pub file_path: PathBuf,

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


impl ResolvableConfigurationWithContext for UnresolvedConfiguration {
    type Resolved = Configuration;
    type Context = PathBuf;

    fn resolve(self, context: Self::Context) -> Result<Self::Resolved> {
        let base_paths = self
            .base_paths
            .resolve()
            .wrap_err("Failed to resolve base_paths table.")?;

        let logging = self
            .logging
            .resolve()
            .wrap_err("Failed to resolve logging table.")?;

        let http = self
            .http
            .resolve()
            .wrap_err("Failed to resolve http table.")?;

        let database = self
            .database
            .resolve()
            .wrap_err("Failed to resolve database table.")?;

        let secrets = self
            .secrets
            .resolve()
            .wrap_err("Failed to resolve secrets table.")?;

        let json_web_token = self
            .json_web_token
            .resolve()
            .wrap_err("Failed to resolve json_web_token table.")?;

        let search = self
            .search
            .resolve(base_paths.clone())
            .wrap_err("Failed to resolve search table.")?;


        Ok(Configuration {
            base_paths,
            file_path: context,
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
    pub fn load_from_path<S: AsRef<Path>>(configuration_file_path: S) -> Result<Self> {
        // Read the configuration file into memory.
        let configuration_string = fs::read_to_string(configuration_file_path.as_ref())
            .expect("Could not read configuration file!");


        // Parse the string into the `UnresolvedConfiguration` structure and then resolve it.
        let unresolved_configuration =
            toml::from_str::<UnresolvedConfiguration>(&configuration_string)
                .into_diagnostic()
                .wrap_err("Could not load configuration file!")?;


        let configuration_file_path = dunce::canonicalize(configuration_file_path)
            .into_diagnostic()
            .wrap_err("Could not canonicalize configuration file path!")?;

        let resolved_configuration = unresolved_configuration
            .resolve(configuration_file_path)
            .wrap_err("Failed to resolve configuration.")?;

        Ok(resolved_configuration)
    }

    /// Load the configuration from the default path (`./data/configuration.toml`).
    pub fn load_from_default_path() -> Result<Configuration> {
        Configuration::load_from_path(
            get_default_configuration_file_path()
                .wrap_err_with(|| "Could not load configuration file at default path.")?,
        )
    }
}
