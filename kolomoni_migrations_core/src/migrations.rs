use std::{collections::HashMap, error::Error, fs, future::Future, path::Path, pin::Pin};

use sqlx::{postgres::PgConnectOptions, ConnectOptions, PgConnection};

use crate::{
    apply_rust_migration,
    apply_sql_migration,
    configuration::MigrationConfiguration,
    context::MigrationContext,
    create_migration_tracking_table_if_needed,
    errors::{
        InitializeMigrationDirectoryError,
        InitializeMigrationTrackingError,
        MigrationApplyError,
        MigrationRollbackError,
        RemoteMigrationError,
        StatusError,
    },
    identifier::MigrationIdentifier,
    remote::{RemoteMigration, RemoteMigrationType},
    rollback_rust_migration,
    rollback_sql_migration,
    sha256::Sha256Hash,
    DownScriptDetails,
    MigrationStatus,
};


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SegmentedHashMatch {
    Neither,
    OnlyUp,
    OnlyDown,
    Both,
}

impl SegmentedHashMatch {
    pub fn up_matches(self) -> bool {
        matches!(self, Self::OnlyUp | Self::Both)
    }

    pub fn down_matches(self) -> bool {
        matches!(self, Self::OnlyDown | Self::Both)
    }

    pub fn fully_matches(self) -> bool {
        matches!(self, Self::Both)
    }
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
) -> SegmentedHashMatch {
    let up_matches =
        embedded_migration.up.sha256_hash() == &remote_migration.up_script.up_script_sha256_hash;

    let down_matches = match &embedded_migration.down {
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
    };

    match (up_matches, down_matches) {
        (true, true) => SegmentedHashMatch::Both,
        (true, false) => SegmentedHashMatch::OnlyUp,
        (false, true) => SegmentedHashMatch::OnlyDown,
        (false, false) => SegmentedHashMatch::Neither,
    }
}



#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MigrationsWithStatusOptions {
    pub require_up_hashes_match: bool,
    pub require_down_hashes_match: bool,
}

impl Default for MigrationsWithStatusOptions {
    fn default() -> Self {
        Self {
            require_up_hashes_match: true,
            require_down_hashes_match: true,
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
    pub async fn initialize_migration_tracking_in_database_if_needed(
        database_connection: &mut PgConnection,
    ) -> Result<(), InitializeMigrationTrackingError> {
        create_migration_tracking_table_if_needed(database_connection)
            .await
            .map_err(|error| InitializeMigrationTrackingError::UnableToCreateTable { error })
    }

    pub async fn migration_tracking_table_exists(
        database_connection: &mut PgConnection,
    ) -> Result<bool, sqlx::Error> {
        let schema_migrations_table_exists: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS (
                SELECT FROM information_schema.tables 
                WHERE  table_schema = 'migrations'
                AND    table_name   = 'schema_migrations'
            )"#,
        )
        .fetch_one(database_connection)
        .await?;

        Ok(schema_migrations_table_exists)
    }


    pub fn migrations_without_status(&self) -> &[EmbeddedMigration<'static>] {
        self.embedded_migrations.as_slice()
    }

    /// Returns all embedded migrations marked as pending.
    fn get_consolidated_migrations_marked_as_pending(&self) -> Vec<ConsolidatedMigration<'_>> {
        self.embedded_migrations
            .iter()
            .map(|migration| ConsolidatedMigration {
                migration,
                status: MigrationStatus::Pending,
            })
            .collect()
    }

    /// Unlike [`Self::migrations_with_status`], this method expects a [`PgConnectOptions`] instead of the live connection.
    /// This is because when a connection cannot be established, or when the migration tracking table is not
    /// present in the database, the returned migrations will all be marked as pending.
    pub async fn migrations_with_status_with_fallback<'m>(
        &'m self,
        connection_options: Option<&PgConnectOptions>,
        options: MigrationsWithStatusOptions,
    ) -> Result<Vec<ConsolidatedMigration<'m>>, StatusError> {
        let Some(connection_options) = connection_options else {
            // When a connection fails, we will return all migrations as pending.
            return Ok(self.get_consolidated_migrations_marked_as_pending());
        };

        let Ok(mut connection) = connection_options.connect().await else {
            // When a connection fails, we will return all migrations as pending.
            return Ok(self.get_consolidated_migrations_marked_as_pending());
        };

        if !Self::migration_tracking_table_exists(&mut connection)
            .await
            .map_err(|error| {
                StatusError::RemoteMigrationError(RemoteMigrationError::QueryFailed { error })
            })?
        {
            // When the migration table is not present, we will return all migrations as pending.
            return Ok(self.get_consolidated_migrations_marked_as_pending());
        }


        self.migrations_with_status(&mut connection, options).await
    }


    pub async fn migrations_with_status<'m>(
        &'m self,
        database_connection: &mut PgConnection,
        options: MigrationsWithStatusOptions,
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


            let hash_match_info = embedded_and_remote_migration_hashes_match(
                corresponding_embedded_migration,
                &remote_migration,
            );

            if (options.require_up_hashes_match && !hash_match_info.up_matches())
                || (options.require_down_hashes_match && !hash_match_info.down_matches())
            {
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

    pub fn configuration(&self) -> &MigrationConfiguration {
        &self.migration.configuration
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
        let down_details = self.down.as_ref().map(|down| DownScriptDetails {
            sha256_hash: down.sha256_hash(),
            r#type: down.remote_migration_type(),
        });

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
        F: for<'a> Fn(MigrationContext<'a>) -> Pin<Box<dyn Future<Output = Result<(), E>> + 'a>>
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
    dyn for<'a> Fn(MigrationContext<'a>) -> Pin<Box<dyn Future<Output = Result<(), E>> + 'a>> + 'c,
>;

pub struct ConcreteRustMigrationScript<'c, E: Error> {
    callback: BoxedMigrationFn<'c, E>,
    sha256_hash: Sha256Hash,
}
