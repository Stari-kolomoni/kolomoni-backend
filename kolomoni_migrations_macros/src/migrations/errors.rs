use std::{borrow::Cow, path::PathBuf};

use kolomoni_migrations_core::{
    configuration::MigrationConfigurationError,
    identifier::{InvalidMigrationIdentifierError, MigrationIdentifier},
};
use thiserror::Error;



#[derive(Debug, Error)]
pub(crate) enum MigrationScanError {
    #[error(
        "invalid structure for migration entry at \"{}\": {}",
        .migration_directory_path.display(),
        .reason
    )]
    InvalidMigrationStructure {
        migration_directory_path: PathBuf,

        reason: Cow<'static, str>,
    },

    #[error(
        "invalid local migration identifier \"{}\"",
        .identifier,
    )]
    InvalidMigrationIdentifier {
        identifier: String,

        #[source]
        error: InvalidMigrationIdentifierError,
    },

    #[error(
        "migration version {} is not unique",
        .version
    )]
    MigrationVersionIsNotUnique { version: i64 },

    #[error(
        "failed to read migrations directory at \"{}\"",
        .directory_path.display()
    )]
    UnableToScanMigrationsDirectory {
        directory_path: PathBuf,

        #[source]
        error: fs_more::error::DirectoryScanError,
    },

    #[error(
        "failed to parse migration configuration for {}",
        .identifier
    )]
    ConfigurationError {
        identifier: MigrationIdentifier,

        #[source]
        error: MigrationConfigurationError,
    },

    #[error(
        "failed to load script at {}",
        .path.display()
    )]
    ScriptError {
        path: PathBuf,

        #[source]
        error: MigrationScriptError,
    },
}



#[derive(Debug, Error)]
pub(crate) enum MigrationScriptError {
    #[error(
        "unable to read file: {}",
        .file_path.display()
    )]
    UnableToReadFile {
        file_path: PathBuf,

        #[source]
        error: std::io::Error,
    },

    #[error(
        "file has unrecognized extension (expected rs or sql): {}",
        .file_path.display()
    )]
    UnrecognizedFileExtension { file_path: PathBuf },

    #[error(
        "file path is not UTF-8: {}",
        .file_path.display()
    )]
    FilePathNotUtf8 { file_path: PathBuf },
}
