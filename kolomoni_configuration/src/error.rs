use std::{borrow::Cow, io, path::PathBuf};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum MissingBasePathCreationError {
    #[error("{} exists, but is not a directory", .path.display())]
    NotADirectory { path: PathBuf },

    #[error("unable to create directory {} due to IO error", .directory_path.display())]
    UnableToCreateDirectory {
        directory_path: PathBuf,

        #[source]
        error: io::Error,
    },
}

#[derive(Debug, Error)]
pub enum MissingLoggingDirectoryCreationError {
    #[error("{} exists, but is not a directory", .path.display())]
    NotADirectory { path: PathBuf },

    #[error("unable to create directory {} due to IO error", .directory_path.display())]
    UnableToCreateDirectory {
        directory_path: PathBuf,

        #[source]
        error: io::Error,
    },
}

#[derive(Debug, Error)]
pub enum MissingSearchIndexDirectoryCreationError {
    #[error("{} exists, but is not a directory", .path.display())]
    NotADirectory { path: PathBuf },

    #[error("unable to create directory {} due to IO error", .directory_path.display())]
    UnableToCreateDirectory {
        directory_path: PathBuf,

        #[source]
        error: io::Error,
    },
}


#[derive(Debug, Error)]
pub enum LoggingConfigurationError {
    #[error(
        "invalid tracing filter of type {} (doesn't parse with EnvFilter): {}",
        .kind,
        .invalid_filter
    )]
    InvalidTracingFilter {
        invalid_filter: String,

        kind: Cow<'static, str>,

        #[source]
        error: tracing_subscriber::filter::ParseError,
    },
}


#[derive(Debug, Error)]
pub enum ConfigurationResolutionError {
    #[error("error while resolving \"logging\" table")]
    LoggingConfigurationError {
        #[from]
        #[source]
        error: LoggingConfigurationError,
    },
}


#[derive(Debug, Error)]
pub enum ConfigurationLoadingError {
    #[error("unable to read configuration file at {}", .path.display())]
    UnableToReadConfigurationFile {
        path: PathBuf,

        #[source]
        error: io::Error,
    },

    #[error("failed to parse the contents of the configuration file as TOML")]
    ParsingError {
        #[from]
        #[source]
        error: toml::de::Error,
    },

    #[error("failed to resolve and validate the contents of the configuration")]
    ResolutionError {
        #[from]
        #[source]
        error: ConfigurationResolutionError,
    },
}
