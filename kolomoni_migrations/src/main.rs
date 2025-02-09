use std::io::ErrorKind;

use clap::Parser;
use cli::{CliArgs, CliCommand};
use commands::{
    down::cli_down,
    generate::cli_generate,
    initialize::cli_initialize,
    status::cli_status,
    up::cli_up,
};
use miette::{Context, IntoDiagnostic, Result};

mod cli;
mod commands;
mod migrations;


/// This function executes [`dotenvy::dotenv`], but does not return an error
/// when no `.env` files are present.
fn load_dotenv_files_if_exist() -> Result<()> {
    let dotenv_result = dotenvy::dotenv();

    if let Err(dotenv_err) = dotenv_result {
        return match dotenv_err {
            dotenvy::Error::Io(error) => match error.kind() {
                ErrorKind::NotFound => Ok(()),
                _ => Err(error)
                    .into_diagnostic()
                    .wrap_err("failed to load any dotenv file"),
            },
            error => {
                return Err(error)
                    .into_diagnostic()
                    .wrap_err("failed to load any dotenv file")
            }
        };
    }

    Ok(())
}


pub fn main() -> Result<()> {
    let cli_args = CliArgs::parse();

    load_dotenv_files_if_exist()?;

    match cli_args.command {
        CliCommand::Initialize(initialize_command_args) => cli_initialize(initialize_command_args),
        CliCommand::Generate(generate_command_args) => cli_generate(generate_command_args),
        CliCommand::Up(up_command_args) => cli_up(up_command_args),
        CliCommand::Down(down_command_args) => cli_down(down_command_args),
        CliCommand::Status(status_command_args) => cli_status(status_command_args),
    }
}


// TODO CLI commands:
// - [DONE, needs a style pass] initialize: creates the migration directory if needed
// - [DONE, needs a style pass] generate: generates a new empty migration (runs initialize automatically if needed)
// - [TODO, medium priority] fresh: drops all tables from the database and reapplies all migrations
//                           (this might be too complicated because we have privileged migrations)
// - [TODO, low priority] refresh: rolls back all migrations, then reapplies all of them
// - [TODO, low priority] reset: rolls back all migrations
// - [DONE, needs a style pass] status: displays the status of all migrations, both applied or not
// - [DONE, needs a style pass] up: applies all pending migrations (or up to a specific version)
// - [DONE, needs a style pass] down: rolls back to a specific database version (migration version)
