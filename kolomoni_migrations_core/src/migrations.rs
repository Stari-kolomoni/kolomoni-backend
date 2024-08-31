use std::{collections::HashMap, error::Error, fs, future::Future, path::Path, pin::Pin};

use sqlx::{Connection, PgConnection};
use thiserror::Error;

use crate::{
    apply_rust_migration,
    apply_sql_migration,
    configuration::MigrationConfiguration,
    errors::{MigrationApplyError, MigrationRollbackError, RemoteMigrationError},
    identifier::MigrationIdentifier,
    remote::{RemoteMigration, RemoteMigrationType},
    rollback_rust_migration,
    rollback_sql_migration,
    sha256::Sha256Hash,
    DownScriptDetails,
    MigrationStatus,
};




pub async fn connect_to_postgresql_database_by_url(
    database_url: &str,
) -> Result<PgConnection, sqlx::Error> {
    sqlx::PgConnection::connect(database_url).await
}


#[derive(Debug, Error)]
pub enum InitializeMigrationDirectoryError {
    #[error("provided migrations directory path exists, but is not a directory")]
    NotADirectory,

    #[error("unable to create missing migrations directory")]
    UnableToCreate {
        #[source]
        error: std::io::Error,
    },
}

#[derive(Debug, Error)]
pub enum InitializeMigrationTrackingError {
    #[error("unable to connect to database")]
    UnableToConnect {
        #[source]
        error: sqlx::Error,
    },

    #[error("failed to create migration tracking table in database")]
    UnableToCreateTable {
        #[source]
        error: sqlx::Error,
    },
}

#[derive(Debug, Error)]
pub enum StatusError {
    #[error("failed to load migration from database")]
    RemoteMigrationError(
        #[from]
        #[source]
        RemoteMigrationError,
    ),

    #[error(
        "migration exists in the database, but its local \
        (embedded) counterpart cannot be found: {}",
        .identifier
    )]
    MigrationDoesNotExistLocally { identifier: MigrationIdentifier },

    #[error(
        "embedded and remote migration don't match due to different hashes (version {}): \
        {} vs {} (up), {:?} vs {:?} (down)",
        .identifier,
        .remote_up_script_sha256_hash,
        .embedded_up_script_sha256_hash,
        .remote_down_script_sha256_hash,
        .embedded_down_script_sha256_hash
    )]
    HashMismatch {
        identifier: MigrationIdentifier,

        remote_up_script_sha256_hash: Sha256Hash,

        embedded_up_script_sha256_hash: Sha256Hash,

        remote_down_script_sha256_hash: Option<Sha256Hash>,

        embedded_down_script_sha256_hash: Option<Sha256Hash>,
    },

    #[error(
        "embedded and remote migration don't match due to different names \
        being used for the same version: {} and {} are used for version {}",
        .embedded_migration_name,
        .remote_migration_name,
        .version
    )]
    NameMismatch {
        version: i64,

        remote_migration_name: String,

        embedded_migration_name: String,
    },
}



/// Compares the SHA-256 hashes of local (disk) and remote (database)
/// migrations.
///
/// Note that while the down (rollback) script of the migration is optional,
/// if either the local or remote has one, the other one *must* have one
/// with the matching hash as well, otherwise `false` is returned.
fn embedded_and_remote_migration_hashes_match(
    embedded_migration: &EmbeddedMigration<'_>,
    remote_migration: &RemoteMigration,
) -> bool {
    if embedded_migration.up.sha256_hash() != &remote_migration.up_script.up_script_sha256_hash {
        return false;
    }

    match &embedded_migration.down {
        Some(embedded_down_script) => match remote_migration.down_script.as_ref() {
            Some(remote_down_script) => {
                // Both the local and remote migrations have a rollback script,
                // so we compare their hashes.
                &remote_down_script.down_script_sha256_hash == embedded_down_script.sha256_hash()
            }
            None => {
                // Local migration has rollback script, but remote does not.
                // This counts as a mismatch.
                false
            }
        },
        None => {
            if remote_migration.down_script.is_none() {
                // Local migration does not have a rollback script, but remote does.
                // This counts as a mismatch.
                false
            } else {
                // Neither the local nor the remote have a rollback script.
                // This is okay.
                true
            }
        }
    }
}



pub struct MigrationManager {
    embedded_migrations: Vec<EmbeddedMigration<'static>>,
}

impl MigrationManager {
    #[inline]
    pub fn new_embedded(embedded_migrations: Vec<EmbeddedMigration<'static>>) -> Self {
        Self {
            embedded_migrations,
        }
    }

    /// Initializes (i.e. creates) the directory that contains all of the migrations.
    ///
    /// If the `migrations_directory` already exists, this has no effect.
    pub fn initialize_migrations_directory<P>(
        &self,
        migrations_directory: P,
    ) -> Result<(), InitializeMigrationDirectoryError>
    where
        P: AsRef<Path>,
    {
        let migrations_directory_path = migrations_directory.as_ref();

        if migrations_directory_path.exists() {
            if !migrations_directory_path.is_dir() {
                return Err(InitializeMigrationDirectoryError::NotADirectory);
            }

            return Ok(());
        }

        fs::create_dir_all(migrations_directory_path)
            .map_err(|error| InitializeMigrationDirectoryError::UnableToCreate { error })?;

        Ok(())
    }

    /// Initializes the migration tracking table in the database.
    ///
    /// If the table has already been initialized, this has no effect.
    pub async fn initialize_migration_tracking_in_database(
        &self,
        database_connection: &mut PgConnection,
    ) -> Result<(), InitializeMigrationTrackingError> {
        sqlx::query!(
            r#"
            CREATE TABLE IF NOT EXISTS kolomoni.schema_migrations (
                version bigint NOT NULL,
                name text NOT NULL,
                up_script_type text NOT NULL,
                up_script_sha256_hash bytea NOT NULL,
                down_script_type text,
                down_script_sha256_hash bytea,
                applied_at timestamp with time zone NOT NULL,
                execution_time_milliseconds BIGINT NOT NULL,
                CONSTRAINT pk__schema_migrations PRIMARY KEY (version),
                CONSTRAINT check__schema_migrations__up_script_type CHECK (
                    (up_script_type = 'sql') OR (up_script_type = 'rust')
                ),
                CONSTRAINT check__schema_migrations__down_script_type CHECK (
                    (down_script_type = 'sql') OR (down_script_type = 'rust')
                    OR (down_script_type IS NULL)
                ),
                CONSTRAINT check__schema_migrations__up_fields CHECK (
                    (up_script_type IS NULL AND up_script_sha256_hash IS NULL)
                    OR (up_script_type IS NOT NULL AND up_script_sha256_hash IS NOT NULL)
                ),
                CONSTRAINT check__schema_migrations_down_fields CHECK (
                    (down_script_type IS NULL AND down_script_sha256_hash IS NULL)
                    OR (down_script_type IS NOT NULL AND down_script_sha256_hash IS NOT NULL)
                ),
                CONSTRAINT check__schema_migrations__min_execution_time CHECK (execution_time_milliseconds >= -1)
            )
            "#
        )
        .execute(&mut *database_connection)
        .await
        .map_err(|error| InitializeMigrationTrackingError::UnableToCreateTable { error })?;

        Ok(())
    }


    pub fn migrations_without_status(&self) -> &[EmbeddedMigration<'static>] {
        self.embedded_migrations.as_slice()
    }


    pub async fn migrations_with_status<'m>(
        &'m self,
        database_connection: &mut PgConnection,
    ) -> Result<Vec<ConsolidatedMigration<'m>>, StatusError> {
        let mut embedded_migrations_by_version: HashMap<i64, &EmbeddedMigration<'static>> = self
            .embedded_migrations
            .iter()
            .map(|migration| (migration.identifier.version, migration))
            .collect::<HashMap<_, _>>();


        let remote_migrations = RemoteMigration::load_all_from_database(database_connection).await?;


        let mut consolidated_migrations = Vec::with_capacity(self.embedded_migrations.len());

        // The following checks are performed:
        // - migrations that have been applied must also exist locally (embedded, in this case),
        // - the hashes of applied migration should match the local migrations,
        // - the version-name pairs must be consistent between the database and local migrations.
        for remote_migration in remote_migrations {
            let Some(corresponding_embedded_migration) =
                embedded_migrations_by_version.remove(&remote_migration.identifier.version)
            else {
                return Err(StatusError::MigrationDoesNotExistLocally {
                    identifier: remote_migration.identifier.clone(),
                });
            };

            if corresponding_embedded_migration.identifier.name != remote_migration.identifier.name {
                return Err(StatusError::NameMismatch {
                    version: remote_migration.identifier.version,
                    remote_migration_name: remote_migration.identifier.name.clone(),
                    embedded_migration_name: corresponding_embedded_migration
                        .identifier
                        .name
                        .clone(),
                });
            }

            if !embedded_and_remote_migration_hashes_match(
                corresponding_embedded_migration,
                &remote_migration,
            ) {
                return Err(StatusError::HashMismatch {
                    identifier: remote_migration.identifier.clone(),
                    remote_up_script_sha256_hash: remote_migration
                        .up_script
                        .up_script_sha256_hash
                        .clone(),
                    embedded_up_script_sha256_hash: corresponding_embedded_migration
                        .up
                        .sha256_hash()
                        .to_owned(),
                    remote_down_script_sha256_hash: remote_migration
                        .down_script
                        .as_ref()
                        .map(|down| down.down_script_sha256_hash.clone()),
                    embedded_down_script_sha256_hash: corresponding_embedded_migration
                        .down
                        .as_ref()
                        .map(|down| down.sha256_hash().to_owned()),
                });
            }


            consolidated_migrations.push(ConsolidatedMigration {
                migration: corresponding_embedded_migration,
                status: MigrationStatus::Applied {
                    at: remote_migration.applied_at,
                },
            })
        }


        for remaining_local_migration in embedded_migrations_by_version.into_values() {
            consolidated_migrations.push(ConsolidatedMigration {
                migration: remaining_local_migration,
                status: MigrationStatus::Pending,
            });
        }


        consolidated_migrations.sort_unstable_by_key(|migration| migration.identifier().version);

        Ok(consolidated_migrations)
    }

    /*
    pub async fn up(
        &self,
        database_connection: &mut PgConnection,
    ) -> Result<(), MigrationApplyError> {
        todo!();
    }

    pub async fn down(
        &self,
        database_connection: &mut PgConnection,
    ) -> Result<(), MigrationRollbackError> {
        todo!();
    } */


    // TODO functions like status, up, down, ...
}


pub struct ConsolidatedMigration<'c> {
    migration: &'c EmbeddedMigration<'c>,

    status: MigrationStatus,
}

impl<'c> ConsolidatedMigration<'c> {
    pub fn identifier(&self) -> &MigrationIdentifier {
        &self.migration.identifier
    }

    pub fn status(&self) -> &MigrationStatus {
        &self.status
    }

    pub fn has_rollback_script(&self) -> bool {
        self.migration.down.is_some()
    }

    pub async fn execute_up(
        &self,
        database_connection: &mut PgConnection,
    ) -> Result<(), MigrationApplyError> {
        self.migration
            .execute_up(
                database_connection,
                self.migration.configuration.up.run_inside_transaction,
            )
            .await
    }

    pub async fn execute_down(
        &self,
        database_connection: &mut PgConnection,
    ) -> Result<(), MigrationRollbackError> {
        self.migration
            .execute_down(
                database_connection,
                self.migration.configuration.down.run_inside_transaction,
            )
            .await
    }
}



pub struct EmbeddedMigration<'c> {
    identifier: MigrationIdentifier,

    configuration: MigrationConfiguration,

    up: EmbeddedMigrationScript<'c, MigrationApplyError>,

    down: Option<EmbeddedMigrationScript<'c, MigrationRollbackError>>,
}

impl<'c> EmbeddedMigration<'c> {
    #[inline]
    pub fn new(
        identifier: MigrationIdentifier,
        configuration: MigrationConfiguration,
        up: EmbeddedMigrationScript<'c, MigrationApplyError>,
        down: Option<EmbeddedMigrationScript<'c, MigrationRollbackError>>,
    ) -> Self {
        Self {
            identifier,
            configuration,
            up,
            down,
        }
    }

    pub fn identifier(&self) -> &MigrationIdentifier {
        &self.identifier
    }

    pub fn configuration(&self) -> &MigrationConfiguration {
        &self.configuration
    }

    pub async fn execute_up(
        &self,
        database_connection: &mut PgConnection,
        run_in_transaction: bool,
    ) -> Result<(), MigrationApplyError> {
        let down_details = match self.down.as_ref() {
            Some(down) => Some(DownScriptDetails {
                sha256_hash: down.sha256_hash(),
                r#type: down.remote_migration_type(),
            }),
            None => todo!(),
        };

        match &self.up {
            EmbeddedMigrationScript::Sql(sql_up) => {
                apply_sql_migration(
                    database_connection,
                    run_in_transaction,
                    &self.identifier,
                    &sql_up.sql,
                    &sql_up.sha256_hash,
                    down_details,
                )
                .await
            }
            EmbeddedMigrationScript::Rust(rust_up) => {
                apply_rust_migration(
                    database_connection,
                    run_in_transaction,
                    &self.identifier,
                    &rust_up.callback,
                    &rust_up.sha256_hash,
                    down_details,
                )
                .await
            }
        }
    }

    pub async fn execute_down(
        &self,
        database_connection: &mut PgConnection,
        run_in_transaction: bool,
    ) -> Result<(), MigrationRollbackError> {
        match self.down.as_ref() {
            Some(down) => match down {
                EmbeddedMigrationScript::Sql(sql_down) => {
                    rollback_sql_migration(
                        database_connection,
                        run_in_transaction,
                        &self.identifier,
                        &sql_down.sql,
                    )
                    .await
                }
                EmbeddedMigrationScript::Rust(rust_down) => {
                    rollback_rust_migration(
                        database_connection,
                        run_in_transaction,
                        &self.identifier,
                        &rust_down.callback,
                    )
                    .await
                }
            },
            None => Err(MigrationRollbackError::RollbackUndefined),
        }
    }
}


pub enum EmbeddedMigrationScript<'c, E>
where
    E: Error,
{
    Sql(ConcreteSqlMigrationScript),
    Rust(ConcreteRustMigrationScript<'c, E>),
}

impl<'c, E> EmbeddedMigrationScript<'c, E>
where
    E: Error,
{
    pub fn new_sql<S>(sql: S, sql_sha256_hash: Sha256Hash) -> Self
    where
        S: Into<String>,
    {
        let sql: String = sql.into();

        Self::Sql(ConcreteSqlMigrationScript {
            sql,
            sha256_hash: sql_sha256_hash,
        })
    }

    pub fn new_rust<F>(migration_function: F, file_sha256_hash: Sha256Hash) -> Self
    where
        F: for<'a> Fn(&'a mut PgConnection) -> Pin<Box<dyn Future<Output = Result<(), E>> + 'a>>
            + 'c,
    {
        let boxed_fn = Box::new(migration_function);

        Self::Rust(ConcreteRustMigrationScript {
            callback: boxed_fn,
            sha256_hash: file_sha256_hash,
        })
    }

    pub fn sha256_hash(&self) -> &Sha256Hash {
        match self {
            EmbeddedMigrationScript::Sql(sql) => &sql.sha256_hash,
            EmbeddedMigrationScript::Rust(rust) => &rust.sha256_hash,
        }
    }

    pub fn remote_migration_type(&self) -> RemoteMigrationType {
        match self {
            EmbeddedMigrationScript::Sql(_) => RemoteMigrationType::Sql,
            EmbeddedMigrationScript::Rust(_) => RemoteMigrationType::Rust,
        }
    }
}



//pub enum ConcreteSqlMigrationScriptKind {
//Up,
//Down,
//}


pub struct ConcreteSqlMigrationScript {
    sql: String,
    sha256_hash: Sha256Hash,
    // kind: ConcreteSqlMigrationScriptKind,
}

pub type BoxedMigrationFn<'c, E> = Box<
    dyn for<'a> Fn(&'a mut PgConnection) -> Pin<Box<dyn Future<Output = Result<(), E>> + 'a>> + 'c,
>;

pub struct ConcreteRustMigrationScript<'c, E: Error> {
    callback: BoxedMigrationFn<'c, E>,
    sha256_hash: Sha256Hash,
}
