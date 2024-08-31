use std::{borrow::Cow, path::Path};

use kolomoni_migrations_core::{
    configuration::MigrationConfiguration,
    identifier::MigrationIdentifier,
};

use super::{errors::MigrationScanError, ScannedMigrationScript};




/// A local, compile-time scanned migration, i.e. without the database state.
#[derive(Clone, Debug)]
pub struct ScannedMigration {
    pub(crate) identifier: MigrationIdentifier,

    pub(crate) configuration: MigrationConfiguration,

    pub(crate) up: ScannedMigrationScript,

    pub(crate) down: Option<ScannedMigrationScript>,
}

impl ScannedMigration {
    /// Given a directory path leading to a specific migration,
    /// this function scans the directory and parses out information about the migration
    /// (and potentially its rollback script as well).
    pub(crate) fn load_from_directory<P>(
        single_migration_directory: P,
    ) -> Result<Self, MigrationScanError>
    where
        P: AsRef<Path>,
    {
        let migration_identifier = {
            let directory_name = single_migration_directory
                .as_ref()
                .file_name()
                .ok_or_else(|| MigrationScanError::InvalidMigrationStructure {
                    migration_directory_path: single_migration_directory.as_ref().to_path_buf(),
                    reason: "migration entry directory has no name".into(),
                })?
                .to_str()
                .ok_or_else(|| MigrationScanError::InvalidMigrationStructure {
                    migration_directory_path: single_migration_directory.as_ref().to_path_buf(),
                    reason: "migration entry directory has non-UTF-8 name".into(),
                })?;

            MigrationIdentifier::parse_from_str(directory_name).map_err(|error| {
                MigrationScanError::InvalidMigrationIdentifier {
                    identifier: directory_name.to_string(),
                    error,
                }
            })?
        };


        let migration_configuration =
            MigrationConfiguration::load_from_directory(single_migration_directory.as_ref())
                .map_err(|error| MigrationScanError::ConfigurationError {
                    identifier: migration_identifier.clone(),
                    error,
                })?
                .unwrap_or_default();


        let up_migration_script = {
            let up_sql_file_path = single_migration_directory.as_ref().join("up.sql");
            if up_sql_file_path.exists() {
                ScannedMigrationScript::load_from_path(&up_sql_file_path).map_err(|error| {
                    MigrationScanError::ScriptError {
                        path: up_sql_file_path.clone(),
                        error,
                    }
                })?
            } else {
                let up_rs_file_path = single_migration_directory.as_ref().join("up.rs");

                if !up_rs_file_path.exists() {
                    return Err(MigrationScanError::InvalidMigrationStructure {
                        migration_directory_path: single_migration_directory.as_ref().to_path_buf(),
                        reason: Cow::Borrowed("no up.sql file"),
                    });
                }

                ScannedMigrationScript::load_from_path(&up_rs_file_path).map_err(|error| {
                    MigrationScanError::ScriptError {
                        path: up_rs_file_path.clone(),
                        error,
                    }
                })?
            }
        };


        let down_migration_script = {
            let down_sql_file_path = single_migration_directory.as_ref().join("down.sql");

            if down_sql_file_path.exists() {
                Some(
                    ScannedMigrationScript::load_from_path(&down_sql_file_path).map_err(
                        |error| MigrationScanError::ScriptError {
                            path: down_sql_file_path.clone(),
                            error,
                        },
                    )?,
                )
            } else {
                let down_rs_file_path = single_migration_directory.as_ref().join("down.rs");

                if down_rs_file_path.exists() {
                    Some(
                        ScannedMigrationScript::load_from_path(&down_rs_file_path).map_err(
                            |error| MigrationScanError::ScriptError {
                                path: down_sql_file_path.clone(),
                                error,
                            },
                        )?,
                    )
                } else {
                    None
                }
            }
        };


        Ok(Self {
            identifier: migration_identifier,
            configuration: migration_configuration,
            up: up_migration_script,
            down: down_migration_script,
        })
    }

    pub fn identifier(&self) -> &MigrationIdentifier {
        &self.identifier
    }

    pub fn configuration(&self) -> &MigrationConfiguration {
        &self.configuration
    }

    pub fn up(&self) -> &ScannedMigrationScript {
        &self.up
    }

    pub fn down(&self) -> Option<&ScannedMigrationScript> {
        self.down.as_ref()
    }
}
