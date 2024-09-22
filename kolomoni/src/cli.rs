//! Command-line interface definitions for the server binary.

use std::path::PathBuf;

use clap::Parser;


/// Server command-line arguments.
#[derive(Parser)]
#[command(
    name = "stari-kolomoni-backend",
    author,
    about = "API server for the Stari Kolomoni open translation project.",
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

    #[arg(
        long = "apply-pending-migrations",
        action = ArgAction::SetTrue,
        help = "On startup, apply any pending database migrations."
    )]
    pub apply_pending_migrations: bool,
}
