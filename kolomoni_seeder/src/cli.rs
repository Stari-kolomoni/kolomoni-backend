use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};


#[derive(Debug, Args)]
pub struct SeedFromSpreadsheetCommandArguments {
    #[arg(
        long = "input-file-translations",
        help = "Input CSV file with translations defined in a standardised format."
    )]
    pub input_file_path_with_translations: PathBuf,

    #[arg(
        long = "input-file-categories",
        help = "Input CSV file with categories defined in a standardised format."
    )]
    pub input_file_path_with_categories: PathBuf,

    #[arg(
        short = 'u',
        long = "server-host",
        help = "Host (IP or hostname) on which the Stari Kolomoni server is available on, e.g. \"localhost\"."
    )]
    pub server_host_or_ip: String,

    #[arg(
        short = 'p',
        long = "server-port",
        help = "Port on which the Stari Kolomoni server is available on, e.g. \"80\"."
    )]
    pub server_port: usize,

    #[arg(
        short = 't',
        long = "access-token",
        help = "Access token to authenticate with on the Stari Kolomoni server."
    )]
    pub access_token: String,
}


#[derive(Debug, Subcommand)]
pub enum CliCommand {
    #[command(name = "seed-from-spreadsheet")]
    SeedFromSpreadsheetCommand(SeedFromSpreadsheetCommandArguments),
}


#[derive(Debug, Parser)]
pub struct CliArguments {
    #[command(subcommand)]
    pub command: CliCommand,
}
