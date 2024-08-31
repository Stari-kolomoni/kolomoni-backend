use std::{
    fs,
    path::{Path, PathBuf},
};

use chrono::{DateTime, SecondsFormat, Utc};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens, TokenStreamExt};
use serde::{Deserialize, Serialize};
use thiserror::Error;



/// An error that can ocurr when loading migration configuration files.
#[derive(Error, Debug)]
pub enum MigrationConfigurationError {
    #[error(
        "migration configuration file \"{}\" does not exist",
        .file_path.display()
    )]
    NotFound { file_path: PathBuf },

    #[error(
        "migration configuration file \"{}\" is not a file",
        .file_path.display()
    )]
    NotAFile { file_path: PathBuf },

    #[error(
        "migration configuration file \"{}\" could not be read",
        .file_path.display()
    )]
    UnableToReadFile {
        file_path: PathBuf,

        #[source]
        error: std::io::Error,
    },

    #[error(
        "migration configuration file \"{}\" could not be parsed as TOML",
        .file_path.display()
    )]
    UnableToParseContents {
        file_path: PathBuf,

        #[source]
        error: Box<toml::de::Error>,
    },
}


/// Configuration that impacts the up.sql migration script.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct MigrationUpConfiguration {
    /// Certain statements, such as CREATE INDEX CONCURRENTLY, cannot be run inside a transaction.
    /// Using those statements in your migration script will require you to run the migration
    /// without a transaction.
    pub run_inside_transaction: bool,
}

impl Default for MigrationUpConfiguration {
    fn default() -> Self {
        Self {
            run_inside_transaction: true,
        }
    }
}

impl ToTokens for MigrationUpConfiguration {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let run_inside_transaction = self.run_inside_transaction;

        tokens.append_all(quote! {
            kolomoni_migrations_core::configuration::MigrationUpConfiguration {
                run_inside_transaction: #run_inside_transaction
            }
        });
    }
}



/// Configuration that impacts the down.sql rollback script.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct MigrationDownConfiguration {
    /// Certain statements, such as CREATE INDEX CONCURRENTLY, cannot be run inside a transaction.
    /// Using those statements in your migration script will require you to run the migration
    /// without a transaction.
    pub run_inside_transaction: bool,
}

impl Default for MigrationDownConfiguration {
    fn default() -> Self {
        Self {
            run_inside_transaction: true,
        }
    }
}

impl ToTokens for MigrationDownConfiguration {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let run_inside_transaction = self.run_inside_transaction;

        tokens.append_all(quote! {
            kolomoni_migrations_core::configuration::MigrationDownConfiguration {
                run_inside_transaction: #run_inside_transaction
            }
        });
    }
}



/// Migration configuration, generally loaded from `migration.toml`
/// inside a single migration's directory.
///
/// Note that it is impossible to modify the version or name of the migration in
/// this configuration file *by design*. Both the version and name are
/// *always* parsed from the parent directory name for consistency.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct MigrationConfiguration {
    pub up: MigrationUpConfiguration,

    pub down: MigrationDownConfiguration,
}


impl MigrationConfiguration {
    pub const fn file_name_in_migration_directory() -> &'static str {
        "migration.toml"
    }

    pub fn load_from_directory<P>(
        migration_directory_path: P,
    ) -> Result<Option<Self>, MigrationConfigurationError>
    where
        P: AsRef<Path>,
    {
        let configuration_file_path = migration_directory_path
            .as_ref()
            .join(Self::file_name_in_migration_directory());

        if !configuration_file_path.exists() {
            return Ok(None);
        } else if configuration_file_path.exists() && !configuration_file_path.is_file() {
            return Err(MigrationConfigurationError::NotAFile {
                file_path: configuration_file_path.to_path_buf(),
            });
        }


        let configuration_contents =
            fs::read_to_string(&configuration_file_path).map_err(|error| {
                MigrationConfigurationError::UnableToReadFile {
                    file_path: configuration_file_path.to_path_buf(),
                    error,
                }
            })?;


        let config: MigrationConfiguration =
            toml::from_str(&configuration_contents).map_err(|error| {
                MigrationConfigurationError::UnableToParseContents {
                    file_path: configuration_file_path.to_path_buf(),
                    error: Box::new(error),
                }
            })?;


        Ok(Some(config))
    }

    #[allow(dead_code)]
    pub(crate) fn load_from_str(
        migration_configuration_toml_str: &str,
    ) -> Result<Self, toml::de::Error> {
        toml::from_str(migration_configuration_toml_str)
    }

    pub fn generate_template(version: i64, name: &str, created_on: DateTime<Utc>) -> String {
        let created_on_formatted = created_on.to_rfc3339_opts(SecondsFormat::Secs, true);

        format!(
            r#"###
# Migration
#   version {:04}
#   name    {}
#
# Created on: {}
###
#
# Note that it is impossible to modify the version or name of the migration in 
# this configuration file *by design*. Both the version and name are 
# *always* parsed from the parent directory name for consistency.
#


##
# Configuration that impacts the up.sql migration script.
##
[up]
# Certain statements, such as CREATE INDEX CONCURRENTLY, cannot be run inside a transaction.
# Using those statements in your migration script will require you to run the migration
# without a transaction.
run_inside_transaction = true



##
# Configuration that impacts the down.sql rollback script.
##
[down]
# Certain statements, such as CREATE INDEX CONCURRENTLY, cannot be run inside a transaction.
# Using those statements in your migration script will require you to run the migration
# without a transaction.
run_inside_transaction = true
"#,
            version, name, created_on_formatted
        )
    }
}

impl ToTokens for MigrationConfiguration {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let up_field_tokens = self.up.to_token_stream();
        let down_field_tokens = self.down.to_token_stream();

        tokens.append_all(quote! {
            kolomoni_migrations_core::configuration::MigrationConfiguration {
                up: #up_field_tokens,
                down: #down_field_tokens
            }
        });
    }
}



#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn template_generator_creates_valid_toml() {
        let template_string =
            MigrationConfiguration::generate_template(1, "hello-world", DateTime::<Utc>::MIN_UTC);

        let parsed_migration_config =
            MigrationConfiguration::load_from_str(&template_string).unwrap();

        assert_eq!(
            parsed_migration_config,
            MigrationConfiguration::default()
        );
    }
}
