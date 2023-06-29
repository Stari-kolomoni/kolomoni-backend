use std::path::Path;

use anyhow::{anyhow, Context, Result};

/// Returns the default configuration filepath at `./data/configuration.toml`.
pub fn get_default_configuration_file_path() -> Result<String> {
    let default_configuration_file_path = Path::new("./data/configuration.toml");
    if !default_configuration_file_path.exists() {
        return Err(anyhow!("No configuration file at default path."));
    }

    let configuration_filepath = dunce::canonicalize(default_configuration_file_path)
        .with_context(|| "Could not canonicalize the configuration file path.")?;

    Ok(configuration_filepath.to_string_lossy().to_string())
}
