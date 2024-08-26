use std::{fmt::Display, hash::Hash};

use thiserror::Error;


/// Failed to parse migration identifier from e.g. `str`.
#[derive(Error, Debug)]
#[error("failed to parse migration identifier from string")]
pub struct InvalidMigrationIdentifierError;


/// Represents a version and name pair of a migration.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct MigrationIdentifier {
    pub version: i64,
    pub name: String,
}

impl MigrationIdentifier {
    pub(crate) fn new(version: i64, name: String) -> Self {
        Self { version, name }
    }

    pub fn parse_from_str(identifier: &str) -> Result<Self, InvalidMigrationIdentifierError> {
        let (m_prefixed_migration_version, migration_name) = identifier
            .split_once('_')
            .ok_or_else(|| InvalidMigrationIdentifierError)?;


        let migration_version = m_prefixed_migration_version
            .to_ascii_lowercase()
            .strip_prefix('m')
            .unwrap_or(m_prefixed_migration_version)
            .to_string();


        let migration_version =
            str::parse::<i64>(&migration_version).map_err(|_| InvalidMigrationIdentifierError)?;


        Ok(Self {
            name: migration_name.to_string(),
            version: migration_version,
        })
    }

    pub fn to_directory_name(&self) -> String {
        format!("M{:04}_{}", self.version, self.name)
    }
}

impl Hash for MigrationIdentifier {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.version.hash(state)
    }
}

impl Display for MigrationIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_directory_name())
    }
}
