use std::{borrow::Cow, path::PathBuf};

use serde::Deserialize;
use tracing_subscriber::EnvFilter;

use crate::{traits::TryResolve, LoggingConfigurationError, MissingLoggingDirectoryCreationError};


#[derive(Deserialize, Clone, Debug)]
pub(super) struct UnresolvedLoggingConfiguration {
    console_output_level_filter: String,

    log_file_output_level_filter: String,

    log_file_output_directory: String,
}

#[derive(Clone, Debug)]
pub struct LoggingConfiguration {
    pub console_output_level_filter: String,

    pub log_file_output_level_filter: String,

    pub log_file_output_directory: PathBuf,
}


impl TryResolve for UnresolvedLoggingConfiguration {
    type Resolved = LoggingConfiguration;
    type Error = LoggingConfigurationError;

    fn try_resolve(self) -> Result<Self::Resolved, Self::Error> {
        // Validate the file and console level filters.
        EnvFilter::try_new(&self.console_output_level_filter).map_err(|error| {
            LoggingConfigurationError::InvalidTracingFilter {
                invalid_filter: self.console_output_level_filter.clone(),
                kind: Cow::from("console output"),
                error,
            }
        })?;

        EnvFilter::try_new(&self.log_file_output_level_filter).map_err(|error| {
            LoggingConfigurationError::InvalidTracingFilter {
                invalid_filter: self.log_file_output_level_filter.clone(),
                kind: Cow::from("log file output"),
                error,
            }
        })?;


        let log_file_output_directory = PathBuf::from(self.log_file_output_directory);

        Ok(Self::Resolved {
            console_output_level_filter: self.console_output_level_filter,
            log_file_output_level_filter: self.log_file_output_level_filter,
            log_file_output_directory,
        })
    }
}


impl LoggingConfiguration {
    pub fn create_logging_directory_if_missing(
        &self,
    ) -> Result<(), MissingLoggingDirectoryCreationError> {
        if self.log_file_output_directory.exists() && !self.log_file_output_directory.is_dir() {
            return Err(
                MissingLoggingDirectoryCreationError::NotADirectory {
                    path: self.log_file_output_directory.clone(),
                },
            );
        }

        std::fs::create_dir_all(&self.log_file_output_directory).map_err(|error| {
            MissingLoggingDirectoryCreationError::UnableToCreateDirectory {
                directory_path: self.log_file_output_directory.clone(),
                error,
            }
        })
    }

    pub fn console_output_level_filter(&self) -> EnvFilter {
        // PANIC SAFETY: This is safe because we checked that the input is valid in `try_resolve`.
        EnvFilter::try_new(&self.console_output_level_filter).unwrap()
    }

    pub fn log_file_output_level_filter(&self) -> EnvFilter {
        // PANIC SAFETY: This is safe because we checked that the input is valid in `try_resolve`.
        EnvFilter::try_new(&self.log_file_output_level_filter).unwrap()
    }
}
