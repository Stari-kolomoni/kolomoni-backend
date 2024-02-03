use std::{env::current_dir, path::PathBuf};

use miette::{miette, Context, IntoDiagnostic, Result};


/// Returns the default configuration filepath, which is at
/// `{current directory}/data/configuration.toml`.
pub fn get_default_configuration_file_path() -> Result<PathBuf> {
    let mut configuration_filepath = current_dir()
        .into_diagnostic()
        .wrap_err_with(|| miette!("Could not get the current directory."))?;
    configuration_filepath.push("data/configuration.toml");

    if !configuration_filepath.exists() {
        panic!("Could not find configuration.toml in data directory.");
    }

    Ok(configuration_filepath)
}
