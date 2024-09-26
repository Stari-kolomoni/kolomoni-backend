use serde::Deserialize;

use crate::traits::Resolve;


pub(crate) type UnresolvedForApiDatabaseConfiguration = ForApiDatabaseConfiguration;

/// PostgreSQL-related configuration.
#[derive(Deserialize, Debug, Clone)]
pub struct ForApiDatabaseConfiguration {
    /// Host where the database resides, or a Unix socket on which the database socket is available.
    pub host: String,

    /// Database port.
    pub port: u16,

    /// User to login as.
    pub username: String,

    /// Login password.
    pub password: Option<String>,

    /// Database to connect to.
    pub database_name: String,

    /// The maximum number of prepared statements to store in the cache.
    pub statement_cache_capacity: Option<usize>,
}

impl Resolve for UnresolvedForApiDatabaseConfiguration {
    type Resolved = ForApiDatabaseConfiguration;

    fn resolve(self) -> Self::Resolved {
        self
    }
}



pub(crate) type UnresolvedForMigrationAtApiRuntimeDatabaseConfiguration =
    ForMigrationAtApiRuntimeDatabaseConfiguration;

/// PostgreSQL-related configuration.
#[derive(Deserialize, Debug, Clone)]
pub struct ForMigrationAtApiRuntimeDatabaseConfiguration {
    /// Host where the database resides, or a Unix socket on which the database socket is available.
    pub host: String,

    /// Database port.
    pub port: u16,

    /// User to login as.
    pub username: String,

    /// Login password.
    pub password: Option<String>,

    /// Database to connect to.
    pub database_name: String,

    /// The maximum number of prepared statements to store in the cache.
    pub statement_cache_capacity: Option<usize>,
}

impl Resolve for UnresolvedForMigrationAtApiRuntimeDatabaseConfiguration {
    type Resolved = ForMigrationAtApiRuntimeDatabaseConfiguration;

    fn resolve(self) -> Self::Resolved {
        self
    }
}




#[derive(Deserialize, Debug, Clone)]
pub(crate) struct UnresolvedDatabaseConfiguration {
    pub(crate) for_api: UnresolvedForApiDatabaseConfiguration,

    pub(crate) for_migration_at_api_runtime: UnresolvedForMigrationAtApiRuntimeDatabaseConfiguration,
}

#[derive(Debug, Clone)]
pub struct DatabaseConfiguration {
    pub for_api: ForApiDatabaseConfiguration,

    pub for_migration_at_api_runtime: ForMigrationAtApiRuntimeDatabaseConfiguration,
}

impl Resolve for UnresolvedDatabaseConfiguration {
    type Resolved = DatabaseConfiguration;

    fn resolve(self) -> Self::Resolved {
        Self::Resolved {
            for_api: self.for_api.resolve(),
            for_migration_at_api_runtime: self.for_migration_at_api_runtime.resolve(),
        }
    }
}
