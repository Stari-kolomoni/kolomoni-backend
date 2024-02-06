use std::fs;
use std::path::{Path, PathBuf};

use miette::{Context, IntoDiagnostic, Result};
use serde::Deserialize;

mod database;
mod http;
mod json_web_token;
mod logging;
mod secrets;

pub use self::database::DatabaseConfiguration;
use self::database::UnresolvedDatabaseConfiguration;
pub use self::http::HttpConfiguration;
use self::http::UnresolvedHttpConfiguration;
pub use self::json_web_token::JsonWebTokenConfiguration;
use self::json_web_token::UnresolvedJsonWebTokenConfiguration;
use self::logging::{LoggingConfiguration, UnresolvedLoggingConfiguration};
pub use self::secrets::SecretsConfiguration;
use self::secrets::UnresolvedSecretsConfiguration;
use crate::traits::{ResolvableConfiguration, ResolvableConfigurationWithContext};
use crate::utilities::get_default_configuration_file_path;

#[derive(Deserialize, Debug)]
pub(crate) struct UnresolvedConfiguration {
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
}


/// The entire Stari Kolomoni backend configuration.
#[derive(Debug, Clone)]
pub struct Configuration {
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
}


impl ResolvableConfigurationWithContext for UnresolvedConfiguration {
    type Resolved = Configuration;
    type Context = PathBuf;

    fn resolve(self, context: Self::Context) -> Result<Self::Resolved> {
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


        Ok(Configuration {
            file_path: context,
            logging,
            http,
            database,
            secrets,
            json_web_token,
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
