use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

use chrono::{DateTime, Utc};
use fs_more::directory::{DirectoryScanDepthLimit, DirectoryScanOptions, DirectoryScanner};
use local::{configuration::MigrationConfiguration, LocalMigration};
use remote::RemoteMigration;
use sqlx::{query, Acquire, Executor, PgConnection};
use tracing::warn;

use crate::{
    errors::{LocalMigrationError, MigrationApplyError, MigrationError, MigrationRollbackError},
    sha256::Sha256Hash,
};

pub mod identifier;
pub mod local;
pub mod remote;
pub use identifier::*;



/// Descibes a migration's status: either pending or applied (at some moment in time).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MigrationStatus {
    /// The migration is pending, which means its details reside in the migrations directory
    /// on disk, but it hasn't yet been applied to the database.
    Pending,

    /// The migration has already been applied to the database
    Applied {
        /// When the migration had been applied.
        at: DateTime<Utc>,
    },
}




/// Describes a single `*.sql` migration script on disk along with its SHA-256 hash.
#[derive(Clone, Debug)]
pub struct MigrationScript {
    /// Contents of the SQL migration script.
    pub sql: String,

    /// SHA-256 hash of the SQL migration script.
    pub sha256_hash: Sha256Hash,
}


async fn apply_migration(
    database_connection: &mut PgConnection,
    migration: &Migration,
) -> Result<(), MigrationApplyError> {
    let applied_at = Utc::now();

    // Executes the entire up.sql script of the migration.
    database_connection
        .execute(migration.up.sql.as_str())
        .await
        .map_err(|error| MigrationApplyError::FailedToExecuteQuery { error })?;


    // Inserta a new row in the schema_migrations table that corresponds
    // to the migration that was just applied.
    query(
        "INSERT INTO kolomoni.schema_migrations \
                (version, name, up_sql_sha256_hash, down_sql_sha256_hash, applied_at)
                VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(migration.identifer.version)
    .bind(migration.identifer.name.as_str())
    .bind(migration.up.sha256_hash.as_slice())
    .bind(
        migration
            .down
            .as_ref()
            .map(|down| down.sha256_hash.as_slice()),
    )
    .bind(applied_at)
    .execute(database_connection)
    .await
    .map_err(|error| MigrationApplyError::FailedToExecuteQuery { error })?;


    Ok(())
}


async fn rollback_migration(
    database_connection: &mut PgConnection,
    migration: &Migration,
) -> Result<(), MigrationRollbackError> {
    let Some(migration_down) = migration.down.as_ref() else {
        return Err(MigrationRollbackError::MigrationCannotBeRolledBack);
    };


    // Executes the entire down.sql script of the migration.
    database_connection
        .execute(migration_down.sql.as_str())
        .await
        .map_err(|error| MigrationRollbackError::FailedToExecuteQuery { error })?;


    // Deletes the corresponding row in the schema_migrations table that tracked
    // the migration being applied.
    query("DELETE FROM kolomoni.schema_migrations WHERE version = $1")
        .bind(migration.identifer.version)
        .execute(database_connection)
        .await
        .map_err(|error| MigrationRollbackError::FailedToExecuteQuery { error })?;


    Ok(())
}



/// A consolidated and verified migration (see [`get_migrations_with_status`]),
/// using information from both the migrations on disk and from the database.
pub struct Migration {
    /// Uniquely identifies (by `version`) a single migration.
    pub(crate) identifer: MigrationIdentifier,

    /// Contains migration-specific configuration.
    pub(crate) configuration: MigrationConfiguration,

    /// The current state of the migration (applied or unapplied).
    pub(crate) status: MigrationStatus,

    /// The SQL script that can be (or has been) executed to migrate the database.
    pub(crate) up: MigrationScript,

    /// If set, this is the SQL script that can be executed to roll back the migration.
    pub(crate) down: Option<MigrationScript>,
}


impl Migration {
    /// Applies the migration to the database.
    ///
    /// This also takes care of updating the `schema_migrations` table
    /// (i.e. inserting a new migration row to mark it as finished).
    pub async fn apply(
        &self,
        database_connection: &mut PgConnection,
    ) -> Result<(), MigrationApplyError> {
        if self.configuration.up.run_inside_transaction {
            let mut transaction = database_connection
                .begin()
                .await
                .map_err(|error| MigrationApplyError::FailedToPerformTransaction { error })?;

            apply_migration(&mut *transaction, &self).await?;

            transaction
                .commit()
                .await
                .map_err(|error| MigrationApplyError::FailedToPerformTransaction { error })?;
        } else {
            apply_migration(database_connection, &self).await?;
        }

        Ok(())
    }

    /// Rolls back the migration, if possible for the given migration.
    ///
    /// If rollback is not available for the given migration, a
    /// [`MigrationRollbackError::MigrationCannotBeRolledBack`] will be returned.
    ///
    /// This also takes care of updating the `schema_migrations` table
    /// (i.e. deleting the relevant migration row).
    pub async fn rollback(
        &self,
        database_connection: &mut PgConnection,
    ) -> Result<(), MigrationRollbackError> {
        if self.configuration.down.run_inside_transaction {
            let mut transaction = database_connection
                .begin()
                .await
                .map_err(|error| MigrationRollbackError::FailedToPerformTransaction { error })?;

            rollback_migration(&mut *transaction, &self).await?;

            transaction
                .commit()
                .await
                .map_err(|error| MigrationRollbackError::FailedToPerformTransaction { error })?;
        } else {
            rollback_migration(database_connection, &self).await?;
        }

        Ok(())
    }
}



/// Scans the provided `migrations_directory` for migrations,
/// returning a list of [`LocalMigration`].
///
/// The returned local migrations are sorted by versions in ascending order.
///
/// Since this does not access the database, [`LocalMigration`]s
/// don't have any information about applied state.
pub fn load_local_migrations(
    migrations_directory: &Path,
) -> Result<Vec<LocalMigration>, LocalMigrationError> {
    let mut local_migrations = Vec::new();

    let mut local_migration_versions = HashSet::new();
    let mut local_migration_names = HashSet::new();


    let migrations_directory_scanner = DirectoryScanner::new(
        migrations_directory,
        DirectoryScanOptions {
            follow_base_directory_symbolic_link: false,
            follow_symbolic_links: false,
            yield_base_directory: false,
            maximum_scan_depth: DirectoryScanDepthLimit::Limited { maximum_depth: 0 },
        },
    );

    for directory_entry_result in migrations_directory_scanner {
        let directory_entry = directory_entry_result.map_err(|error| {
            LocalMigrationError::UnableToScanMigrationsDirectory {
                directory_path: migrations_directory.to_path_buf(),
                error,
            }
        })?;

        if !directory_entry.metadata().is_dir() {
            continue;
        }


        let local_migration = LocalMigration::load_from_directory(directory_entry.path())?;


        if local_migration_versions.contains(&local_migration.identifier.version) {
            return Err(LocalMigrationError::MigrationVersionIsNotUnique {
                version: local_migration.identifier.version,
            });
        }

        if local_migration_names.contains(&local_migration.identifier.name) {
            warn!(
                "Non-unique migration name: \"{}\" is used in more than one migration.",
                local_migration.identifier.name
            );
        }


        local_migration_versions.insert(local_migration.identifier.version);
        local_migration_names.insert(local_migration.identifier.name.clone());

        local_migrations.push(local_migration);
    }


    local_migrations.sort_unstable_by_key(|migration| migration.identifier.version);


    Ok(local_migrations)
}



/// Retrieves and consolidates both the local migrations (located in the `migrations_directory` directory)
/// as well as migrations that are noted down in the database.
///
/// It also verifies the SHA-256 hashes of migration scripts to ensure consistency. This means
/// that changing a migration script after applying it will cause a [`MigrationError::RemoteAndLocalMigrationHasDifferentHash`].
pub async fn load_and_validate_migrations_with_status(
    migrations_directory: &Path,
    database_connection: &mut PgConnection,
) -> Result<Vec<Migration>, MigrationError> {
    let mut consolidated_migrations = Vec::new();


    // Step 1: load all migrations from the "migrations" directory.
    let local_migrations = load_local_migrations(migrations_directory)?;

    let mut local_migrations_by_identifier = local_migrations
        .iter()
        .map(|local_migration| {
            (
                local_migration.identifier.clone(),
                local_migration,
            )
        })
        .collect::<HashMap<_, _>>();



    // Step 2: load all migrations from the database.
    let remote_migrations = RemoteMigration::load_all_from_database(database_connection).await?;


    // Step 3: error if there exist migrations that were applied but are not on disk
    // Step 4: ensure the hashes still match with the local migration scripts
    // Step 6: consolidate migrations into final [`Migration`] structs
    for remote_migration in remote_migrations {
        let Some(corresponding_local_migration) =
            local_migrations_by_identifier.remove(&remote_migration.identifier)
        else {
            return Err(MigrationError::MigrationNoLongerExistsLocally {
                identifier: remote_migration.identifier.clone(),
            });
        };


        // Check for hash mismatches.
        if !local_and_remote_migration_hashes_match(
            &corresponding_local_migration,
            &remote_migration,
        ) {
            return Err(
                MigrationError::RemoteAndLocalMigrationHasDifferentHash {
                    identifier: remote_migration.identifier.clone(),
                    remote_up_sql_sha256_hash: remote_migration.up_sql_sha256_hash,
                    local_up_sql_sha256_hash: corresponding_local_migration.up.sha256_hash.clone(),
                    remote_down_sql_sha256_hash: remote_migration.down_sql_sha256_hash,
                    local_down_sql_sha256_hash: corresponding_local_migration
                        .down
                        .as_ref()
                        .map(|down| down.sha256_hash.clone()),
                },
            );
        }



        consolidated_migrations.push(Migration {
            identifer: corresponding_local_migration.identifier.clone(),
            configuration: corresponding_local_migration.configuration.clone(),
            status: MigrationStatus::Applied {
                at: remote_migration.applied_at,
            },
            up: corresponding_local_migration.up.clone(),
            down: corresponding_local_migration.down.clone(),
        });
    }


    for remaining_local_migration in local_migrations_by_identifier.into_values() {
        consolidated_migrations.push(Migration {
            identifer: remaining_local_migration.identifier.clone(),
            configuration: remaining_local_migration.configuration.clone(),
            status: MigrationStatus::Pending,
            up: remaining_local_migration.up.clone(),
            down: remaining_local_migration.down.clone(),
        })
    }


    consolidated_migrations.sort_unstable_by_key(|migration| migration.identifer.version);


    Ok(consolidated_migrations)
}



/// Compares the SHA-256 hashes of local (disk) and remote (database)
/// migrations.
///
/// Note that while the down (rollback) script of the migration is optional,
/// if either the local or remote has one, the other one *must* have one
/// with the matching hash as well, otherwise `false` is returned.
fn local_and_remote_migration_hashes_match(
    local_migration: &LocalMigration,
    remote_migration: &RemoteMigration,
) -> bool {
    if local_migration.up.sha256_hash != remote_migration.up_sql_sha256_hash {
        return false;
    }

    match &local_migration.down {
        Some(local_down_script) => match &remote_migration.down_sql_sha256_hash {
            Some(remote_down_script_sha256_hash) => {
                // Both the local and remote migrations have a rollback script,
                // so we compare their hashes.
                remote_down_script_sha256_hash == &local_down_script.sha256_hash
            }
            None => {
                // Local migration has rollback script, but remote does not.
                // This counts as a mismatch.
                false
            }
        },
        None => {
            if remote_migration.down_sql_sha256_hash.is_none() {
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
