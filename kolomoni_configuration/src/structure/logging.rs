use std::path::PathBuf;

use miette::{miette, Context, IntoDiagnostic, Result};
use serde::Deserialize;
use tracing_subscriber::EnvFilter;

use crate::traits::ResolvableConfiguration;

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

impl ResolvableConfiguration for UnresolvedLoggingConfiguration {
    type Resolved = LoggingConfiguration;

    fn resolve(self) -> Result<Self::Resolved> {
        // Validate the file and console level filters.
        EnvFilter::try_new(&self.console_output_level_filter)
            .into_diagnostic()
            .wrap_err_with(|| miette!("Failed to parse field console_output_level_filter"))?;

        EnvFilter::try_new(&self.log_file_output_level_filter)
            .into_diagnostic()
            .wrap_err_with(|| miette!("Failed to parse field log_file_output_level_filter"))?;

        let log_file_output_directory = PathBuf::from(self.log_file_output_directory);

        Ok(Self::Resolved {
            console_output_level_filter: self.console_output_level_filter,
            log_file_output_level_filter: self.log_file_output_level_filter,
            log_file_output_directory,
        })
    }
}

impl LoggingConfiguration {
    pub fn console_output_level_filter(&self) -> EnvFilter {
        // PANIC SAFETY: This is safe because we checked that the input is valid in `resolve`.
        EnvFilter::try_new(&self.console_output_level_filter).unwrap()
    }

    pub fn log_file_output_level_filter(&self) -> EnvFilter {
        // PANIC SAFETY: This is safe because we checked that the input is valid in `resolve`.
        EnvFilter::try_new(&self.log_file_output_level_filter).unwrap()
    }
}
