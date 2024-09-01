use std::{path::PathBuf, str::FromStr};

use clap::{ArgAction, Args, Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "kolomoni_migrations",
    author,
    about = "Stari Kolomoni database migrations CLI.",
    version
)]
pub struct CliArgs {
    #[command(subcommand)]
    pub command: CliCommand,
}



#[derive(Subcommand)]
pub enum CliCommand {
    #[command(
        name = "initialize",
        about = "Initializes the directory containing migrations and prepares the database for migrations."
    )]
    Initialize(InitializeCommandArguments),

    #[command(name = "generate", about = "Generates a new empty migration.")]
    Generate(GenerateCommandArguments),

    #[command(
        name = "up",
        about = "Applies pending migrations to upgrade the database to the specified schema version."
    )]
    Up(UpCommandArguments),

    #[command(
        name = "down",
        about = "Rolls back applied migrations to revert the database to the specified schema version.\
                Note that in general, this is a destructive action."
    )]
    Down(DownCommandArguments),
    // Status(StatusCommandArguments),
    // TODO and so on
}


#[derive(Args)]
pub struct InitializeCommandArguments {
    #[arg(
        long = "migrations-directory",
        short = 'm',
        help = "Path to the to-be-created \"migrations\" directory that will contain all database migrations \
                for Stari Kolomoni. This should generally be set to \"./kolomoni_migrations/migrations\" \
                (relative to the repository root)."
    )]
    pub migrations_directory_path: PathBuf,

    #[arg(
        long = "database-url",
        short = 'd',
        help = "URL of the PostgreSQL database to use. If unspecified, we'll attempt to use \
                the DATABASE_URL environment variable. If neither this option nor DATABASE_URL are available, \
                an error will be returned"
    )]
    pub database_url: Option<String>,
}


#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GeneratedScriptType {
    Sql,
    Rust,
}

impl FromStr for GeneratedScriptType {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "sql" => Ok(Self::Sql),
            "rust" => Ok(Self::Rust),
            _ => Err("expected either \"sql\" or \"rust\""),
        }
    }
}


#[derive(Args)]
pub struct GenerateCommandArguments {
    #[arg(
        long = "migrations-directory",
        short = 'm',
        help = "Path to the to-be-created \"migrations\" directory that will contain all database migrations \
                for Stari Kolomoni. This should generally be set to \"./kolomoni_migrations/migrations\" \
                (relative to the repository root)."
    )]
    pub migrations_directory_path: PathBuf,

    #[arg(
        long = "migration-version",
        short = 'v',
        help = "Specifies the version of the new migration. If left empty, we'll automatically generate \
                an increment based on the last migration version."
    )]
    pub migration_version: Option<i64>,

    #[arg(
        long = "migration-name",
        short = 'n',
        help = "Name to associate with the new migration."
    )]
    pub migration_name: String,

    #[arg(
        long = "no-configuration-file",
        action = ArgAction::SetTrue,
        help = "If set, this will result in the action not generating a default migration.toml file. \
                This means the new migration will get the defaults (but you may still create the file \
                manually later if you wish)."
    )]
    pub no_configuration_file: bool,

    #[arg(
        long = "up-script-type",
        help = "This option sets the type of migration script to generate, i.e. \"sql\" (generates up.sql) \
                or \"rust\" (generates up.rs + mod.rs). Defaults to SQL if unspecified."
    )]
    pub up_script_type: Option<GeneratedScriptType>,

    #[arg(
        long = "rollback-script-type",
        help = "This option sets the type of rollback script to generate, i.e. \"sql\" (generates down.sql) \
                or \"rust\" (generates down.rs + mod.rs). Defaults to SQL if unspecified. \
                If --no-rollback is set, this option has no effect."
    )]
    pub rollback_script_type: Option<GeneratedScriptType>,

    #[arg(
        long = "no-rollback",
        action = ArgAction::SetTrue,
        help = "If set, this will result in not generating the down.sql script, meaning the new migration \
                will not be reversible."
    )]
    pub no_rollback: bool,
}



#[derive(Args)]
pub struct UpCommandArguments {
    #[arg(
        long = "migrations-directory",
        short = 'm',
        help = "Path to the to-be-created \"migrations\" directory that will contain all database migrations \
                for Stari Kolomoni. This should generally be set to \"./kolomoni_migrations/migrations\" \
                (relative to the repository root)."
    )]
    pub migrations_directory_path: PathBuf,

    #[arg(
        long = "database-url",
        short = 'd',
        help = "URL of the PostgreSQL database to use. If unspecified, we'll attempt to use \
                the DATABASE_URL environment variable. If neither this option nor DATABASE_URL are available, \
                an error will be returned"
    )]
    pub database_url: Option<String>,

    #[arg(
        long = "migrate-to-version",
        short = 'v',
        help = "Specifies the version of the database to migrate to. The version should match a defined migration\
                and must be greater than the currently applied version. If unspecified, it defaults to the newest version."
    )]
    pub migrate_to_version: Option<i64>,
}



#[derive(Args)]
pub struct DownCommandArguments {
    #[arg(
        long = "migrations-directory",
        short = 'm',
        help = "Path to the to-be-created \"migrations\" directory that will contain all database migrations \
                for Stari Kolomoni. This should generally be set to \"./kolomoni_migrations/migrations\" \
                (relative to the repository root)."
    )]
    pub migrations_directory_path: PathBuf,

    #[arg(
        long = "database-url",
        short = 'd',
        help = "URL of the PostgreSQL database to use. If unspecified, we'll attempt to use \
                the DATABASE_URL environment variable. If neither this option nor DATABASE_URL are available, \
                an error will be returned"
    )]
    pub database_url: Option<String>,

    #[arg(
        long = "rollback-to-version",
        short = 'v',
        help = "Specifies the version of the database to rollback to. The version should match a defined migration \
                and must be smaller than the currently applied version. If you want to perform a rollback all the \
                way to the beginning, set this to 0."
    )]
    pub rollback_to_version: i64,
}
