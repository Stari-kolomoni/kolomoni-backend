use std::path::PathBuf;

use clap::Parser;

/// Command-line arguments for the backend.
#[derive(Parser)]
#[command(
    name = "stari-kolomoni-backend",
    author,
    about = "Backend API for the Stari Kolomoni open translation project.",
    version
)]
pub struct CLIArgs {
    /// This is the path to the configuration file to use.
    /// If unspecified, this defaults to `./data/configuration.toml`.
    #[arg(
        short = 'c',
        long = "configurationFilePath",
        help = "Path to the configuration file to use. Defaults to ./data/configuration.toml"
    )]
    pub configuration_file_path: Option<PathBuf>,
}