use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::configuration::utilities::get_default_configuration_file_path;


/// The entire Stari Kolomoni backend configuration.
#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    /// Configuration related to the HTTP server.
    pub http: ConfigHTTP,

    /// Configuration related to the database.
    pub database: ConfigDatabase,

    /// Password-related configuration.
    pub password: ConfigPasswords,

    /// Json Web Token-related configuration.
    pub jsonwebtoken: ConfigJsonWebToken,

    /// This is the file path this `Config` instance was loaded from.
    #[serde(skip)]
    pub file_path: PathBuf,
}

#[allow(dead_code)]
impl Config {
    /// Load the configuration from a specific file path.
    pub fn load_from_path<S: AsRef<Path>>(configuration_file_path: S) -> Result<Self> {
        // Read the configuration file into memory.
        let configuration_string = fs::read_to_string(configuration_file_path.as_ref())
            .expect("Could not read configuration file!");

        // Parse the string into the `Config` structure.
        let mut config = toml::from_str::<Config>(&configuration_string)
            .expect("Could not load configuration file!");

        config.file_path = dunce::canonicalize(configuration_file_path.as_ref())
            .expect("Could not canonicalize configuration file path even though it has loaded!");

        Ok(config)
    }

    /// Load the configuration from the default path (`./data/configuration.toml`).
    pub fn load_from_default_path() -> Result<Config> {
        Config::load_from_path(
            get_default_configuration_file_path()
                .with_context(|| "Could not load configuration file at default path.")?,
        )
    }
}

/// Actix HTTP server-related configuration.
#[derive(Deserialize, Debug, Clone)]
pub struct ConfigHTTP {
    /// Host to bind the HTTP server to.
    pub host: String,

    /// Port to bind the HTTP server to.
    pub port: usize,
}

/// PostgreSQL-related configuration.
#[derive(Deserialize, Debug, Clone)]
pub struct ConfigDatabase {
    /// Host of the database.
    pub host: String,

    /// Port the database is listening at.
    pub port: usize,

    /// Login username.
    pub username: String,

    /// Login password.
    pub password: String,

    /// Database name.
    pub database_name: String,
}

/// Password hashing-related configuration.
#[derive(Deserialize, Debug, Clone)]
pub struct ConfigPasswords {
    pub hash_salt: String,
}

/// JSON Web Token-related configuration.
#[derive(Deserialize, Debug, Clone)]
pub struct ConfigJsonWebToken {
    pub secret: String,
}
