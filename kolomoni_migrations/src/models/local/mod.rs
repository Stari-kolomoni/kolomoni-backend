use std::{borrow::Cow, fs, path::Path};

use configuration::MigrationConfiguration;

use super::{MigrationIdentifier, MigrationScript};
use crate::{errors::LocalMigrationError, sha256::Sha256Hash};

pub mod configuration;




/// A partial, local-only [`RemoteMigration`], i.e. without the database state.
#[derive(Clone, Debug)]
pub struct LocalMigration {
    pub(crate) identifier: MigrationIdentifier,

    pub(crate) configuration: MigrationConfiguration,

    pub(crate) up: MigrationScript,

    pub(crate) down: Option<MigrationScript>,
}

impl LocalMigration {
    pub(crate) fn load_from_directory<P>(
        single_migration_directory: P,
    ) -> Result<Self, LocalMigrationError>
    where
        P: AsRef<Path>,
    {
        let migration_identifier = {
            let directory_name = single_migration_directory
                .as_ref()
                .file_name()
                .ok_or_else(
                    || LocalMigrationError::InvalidMigrationStructure {
                        migration_directory_path: single_migration_directory.as_ref().to_path_buf(),
                        reason: "migration entry directory has no name".into(),
                    },
                )?
                .to_str()
                .ok_or_else(
                    || LocalMigrationError::InvalidMigrationStructure {
                        migration_directory_path: single_migration_directory.as_ref().to_path_buf(),
                        reason: "migration entry directory has non-UTF-8 name".into(),
                    },
                )?;

            MigrationIdentifier::parse_from_str(directory_name).map_err(|error| {
                LocalMigrationError::InvalidMigrationIdentifier {
                    identifier: directory_name.to_string(),
                    error,
                }
            })?
        };


        let migration_configuration =
            MigrationConfiguration::load_from_directory(single_migration_directory.as_ref())
                .map_err(|error| LocalMigrationError::ConfigurationError {
                    identifier: migration_identifier.clone(),
                    error,
                })?
                .unwrap_or_default();



        let up_migration_script = {
            let up_sql_file_path = single_migration_directory.as_ref().join("up.sql");
            if !up_sql_file_path.exists() {
                return Err(LocalMigrationError::InvalidMigrationStructure {
                    migration_directory_path: single_migration_directory.as_ref().to_path_buf(),
                    reason: Cow::Borrowed("no up.sql file"),
                });
            }

            let up_sql = fs::read_to_string(&up_sql_file_path).map_err(|error| {
                LocalMigrationError::UnableToReadMigration {
                    path: up_sql_file_path,
                    error,
                }
            })?;

            let up_sql_sha256_hash = Sha256Hash::calculate(up_sql.as_bytes());


            MigrationScript {
                sql: up_sql,
                sha256_hash: up_sql_sha256_hash,
            }
        };


        let down_migration_script = {
            let down_sql_file_path = single_migration_directory.as_ref().join("down.sql");

            if down_sql_file_path.exists() {
                let down_sql = fs::read_to_string(&down_sql_file_path).map_err(|error| {
                    LocalMigrationError::UnableToReadMigration {
                        path: down_sql_file_path,
                        error,
                    }
                })?;

                let down_sql_sha256_hash = Sha256Hash::calculate(down_sql.as_bytes());

                Some(MigrationScript {
                    sql: down_sql,
                    sha256_hash: down_sql_sha256_hash,
                })
            } else {
                None
            }
        };


        Ok(Self {
            identifier: migration_identifier,
            configuration: migration_configuration,
            up: up_migration_script,
            down: down_migration_script,
        })
    }
}
