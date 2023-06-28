use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[command(
    name = "stari-kolomoni-backend",
    author,
    about = "Backend API for the Stari Kolomoni open translation project.",
    version
)]
pub struct CLIArgs {
    #[arg(
        short = 'c',
        long = "configurationFilePath",
        help = "Path to the configuration file to use. Defaults to ./data/configuration.toml"
    )]
    pub configuration_file_path: Option<PathBuf>,
}
