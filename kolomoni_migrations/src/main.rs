use clap::Parser;
use cli::{CliArgs, CliCommand};
use commands::{down::cli_down, generate::cli_generate, initialize::cli_initialize, up::cli_up};
use miette::{Context, IntoDiagnostic, Result};

mod cli;
mod commands;
mod migrations;


pub fn main() -> Result<()> {
    let cli_args = CliArgs::parse();

    dotenvy::dotenv()
        .into_diagnostic()
        .wrap_err("failed to load any dotenv file")?;


    match cli_args.command {
        CliCommand::Initialize(initialize_command_args) => cli_initialize(initialize_command_args),
        CliCommand::Generate(generate_command_args) => cli_generate(generate_command_args),
        CliCommand::Up(up_command_args) => cli_up(up_command_args),
        CliCommand::Down(down_command_args) => cli_down(down_command_args),
    }
}




// TODO Required CLI commands:
// - [DONE, needs a style pass] initialize: creates the migration directory if needed
// - [DONE, needs a style pass] generate: generates a new empty migration (runs initialize automatically if needed)
// - [PENDING, medium priority] fresh: drops all tables from the database and reapplies all migrations
// - [PENDING, low priority] refresh: rolls back all migrations, then reapplies all of them
// - [PENDING, low priority] reset: rolls back all migrations
// - [PENDING, high priority] status: displays the status of all migrations, both applied or not
// - [DONE, needs a style pass] up: applies all pending migrations (or up to a specific version)
// - [DONE, needs a style pass] down: rolls back to a specific database version (migration version)
