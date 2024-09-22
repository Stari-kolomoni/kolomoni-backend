use serde::Deserialize;

use crate::traits::ResolvableConfiguration;


pub(crate) type UnresolvedDatabaseConfiguration = DatabaseConfiguration;

/// PostgreSQL-related configuration.
#[derive(Deserialize, Debug, Clone)]
pub struct DatabaseConfiguration {
    /// Host of the database.
    pub host: String,

    /// Port the database is listening at.
    pub port: u16,

    /// Login username.
    pub username: String,

    /// Login password.
    pub password: Option<String>,

    /// Database name.
    pub database_name: String,
}

impl ResolvableConfiguration for UnresolvedDatabaseConfiguration {
    type Resolved = DatabaseConfiguration;

    fn resolve(self) -> miette::Result<Self::Resolved> {
        Ok(self)
    }
}
