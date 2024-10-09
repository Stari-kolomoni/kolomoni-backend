use std::{
    borrow::Cow,
    env::{self, VarError},
    path::PathBuf,
    str::FromStr,
};

use clap::{ArgAction, Args, Parser, Subcommand};
use sqlx::postgres::PgConnectOptions;
use thiserror::Error;

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
}


#[derive(Args)]
pub struct InitializeCommandArguments {
    #[arg(
        long = "migrations-directory",
        short = 'm',
        help = "Path to the to-be-created migrations directory that will contain all database migrations \
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


#[derive(Debug, Error)]
#[error("environment variable {} is not valid Unicode", .variable_name)]
pub struct EnvValueNotUnicode {
    variable_name: String,
}

pub(crate) fn get_string_with_env_fallback<'a, S>(
    optional_str: Option<&'a S>,
    fallback_environment_variable_name: &str,
) -> Result<Option<Cow<'a, str>>, EnvValueNotUnicode>
where
    S: AsRef<str>,
{
    if let Some(specified_value) = optional_str {
        return Ok(Some(Cow::from(specified_value.as_ref())));
    }

    match env::var(fallback_environment_variable_name) {
        Ok(database_url_from_env) => Ok(Some(Cow::from(database_url_from_env))),
        Err(error) => match error {
            VarError::NotPresent => Ok(None),
            VarError::NotUnicode(_) => Err(EnvValueNotUnicode {
                variable_name: fallback_environment_variable_name.to_string(),
            }),
        },
    }
}


#[derive(Debug, Error)]
pub enum DatabaseConnectionArgsError {
    #[error("invalid database URL format")]
    InvalidDatabaseUrlFormat {
        #[source]
        error: sqlx::Error,
    },
}


#[derive(Args)]
pub struct DatabaseConnectionArgs {
    #[arg(
        long = "database-url-for-normal-user",
        help = "PostgreSQL connection URL for the normal user \
                (see run_as_privileged_user option for individual migrations). \
                If unspecified, we'll attempt to use the KOLOMONI_MIGRATIONS_DATABASE_URL_NORMAL_USER environment variable. \
                If you attempt to perform a migration that would use this user \
                — i.e. for migrations with `run_as_privileged_user = false` — an error will be shown. \
                If no migration needs to use the normal user, it is valid to not specify it."
    )]
    pub database_url_for_normal_user: Option<String>,

    #[arg(
        long = "database-url-for-privileged-user",
        help = "PostgreSQL connection URL for the privileged user \
                (see run_as_privileged_user option for individual migrations). \
                If unspecified, we'll attempt to use the KOLOMONI_MIGRATIONS_DATABASE_URL_PRIVILEGED_USER environment variable. \
                If you attempt to perform a migration that would use this user \
                — i.e. for migrations with `run_as_privileged_user = true` — an error will be shown. \
                If no migration needs to use the normal user, it is valid to not specify it."
    )]
    pub database_url_for_privileged_user: Option<String>,
}

impl DatabaseConnectionArgs {
    /// # Panic
    /// Panics when the `KOLOMONI_MIGRATIONS_DATABASE_URL_NORMAL_USER`
    /// environment variable is not valid Unicode.
    pub fn database_connection_options_for_normal_user(
        &self,
    ) -> Result<Option<PgConnectOptions>, DatabaseConnectionArgsError> {
        let potential_normal_user_db_url = get_string_with_env_fallback(
            self.database_url_for_normal_user.as_ref(),
            "KOLOMONI_MIGRATIONS_DATABASE_URL_NORMAL_USER",
        )
        // PANIC INFO: Documented on the function.
        .expect("environment variable is not valid unicode");

        let Some(normal_user_db_url) = potential_normal_user_db_url else {
            return Ok(None);
        };


        let normal_user_db_connect_options = PgConnectOptions::from_str(&normal_user_db_url)
            .map_err(|error| DatabaseConnectionArgsError::InvalidDatabaseUrlFormat { error })?;

        Ok(Some(normal_user_db_connect_options))
    }

    /// # Panic
    /// Panics when the `KOLOMONI_MIGRATIONS_DATABASE_URL_PRIVILEGED_USER`
    /// environment variable is not valid Unicode.
    pub fn database_connection_options_for_privileged_user(
        &self,
    ) -> Result<Option<PgConnectOptions>, DatabaseConnectionArgsError> {
        let potential_privileged_user_db_url = get_string_with_env_fallback(
            self.database_url_for_normal_user.as_ref(),
            "KOLOMONI_MIGRATIONS_DATABASE_URL_PRIVILEGED_USER",
        )
        // PANIC INFO: Documented on the function.
        .expect("environment variable is not valid unicode");

        let Some(privileged_user_db_url) = potential_privileged_user_db_url else {
            return Ok(None);
        };


        let privileged_user_db_connect_options = PgConnectOptions::from_str(&privileged_user_db_url)
            .map_err(|error| DatabaseConnectionArgsError::InvalidDatabaseUrlFormat { error })?;

        Ok(Some(privileged_user_db_connect_options))
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
    #[command(flatten)]
    pub database: DatabaseConnectionArgs,

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
    #[command(flatten)]
    pub database: DatabaseConnectionArgs,

    #[arg(
        long = "rollback-to-version",
        short = 'v',
        help = "Specifies the version of the database to rollback to. The version should match a defined migration \
                and must be smaller than the currently applied version. If you want to perform a rollback all the \
                way to the beginning, set this to 0."
    )]
    pub rollback_to_version: i64,
}
