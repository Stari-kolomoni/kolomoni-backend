use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use errors::{MigrationScanError, MigrationScriptError};
use fs_more::directory::{DirectoryScanDepthLimit, DirectoryScanOptions, DirectoryScanner};
use kolomoni_migrations_core::sha256::Sha256Hash;
use scan::ScannedMigration;
use tracing::warn;

pub(crate) mod errors;
pub(crate) mod scan;




/// Describes a single `*.sql` migration script on disk along with its SHA-256 hash.
#[derive(Clone, Debug)]
pub(crate) struct ScannedSqlMigrationScript {
    /// Contents of the SQL migration script.
    pub sql: String,

    /// SHA-256 hash of the SQL migration script.
    pub sha256_hash: Sha256Hash,
}


/// Describes a single `*.rs` migration script on disk along with its SHA-256 hash.
#[derive(Clone, Debug)]
pub(crate) struct ScannedRustMigrationScript {
    /// Path is relative to the root of the macro crate.
    #[allow(dead_code)]
    pub rs_file_path: PathBuf,

    pub sha256_hash: Sha256Hash,
}




#[derive(Clone, Debug)]
pub enum ScannedMigrationScript {
    Sql(ScannedSqlMigrationScript),
    Rust(ScannedRustMigrationScript),
}

impl ScannedMigrationScript {
    pub(crate) fn load_from_path(script_path: &Path) -> Result<Self, MigrationScriptError> {
        let file_extension = script_path
            .extension()
            .ok_or_else(
                || MigrationScriptError::UnrecognizedFileExtension {
                    file_path: script_path.to_path_buf(),
                },
            )?
            .to_str()
            .ok_or_else(|| MigrationScriptError::FilePathNotUtf8 {
                file_path: script_path.to_path_buf(),
            })?;

        let file_contents = fs::read_to_string(script_path).map_err(|error| {
            MigrationScriptError::UnableToReadFile {
                file_path: script_path.to_path_buf(),
                error,
            }
        })?;

        let file_contents_sha256 = Sha256Hash::calculate(file_contents.as_bytes());


        if file_extension == "sql" {
            Ok(ScannedMigrationScript::Sql(
                ScannedSqlMigrationScript {
                    sql: file_contents,
                    sha256_hash: file_contents_sha256,
                },
            ))
        } else if file_extension == "rs" {
            let macro_crate_relative_file_path =
                pathdiff::diff_paths(script_path, env!("CARGO_MANIFEST_DIR")).unwrap();

            Ok(ScannedMigrationScript::Rust(
                ScannedRustMigrationScript {
                    rs_file_path: macro_crate_relative_file_path,
                    sha256_hash: file_contents_sha256,
                },
            ))
        } else {
            Err(MigrationScriptError::UnrecognizedFileExtension {
                file_path: script_path.to_path_buf(),
            })
        }
    }
}



/// Scans the provided `migrations_directory` for migrations,
/// returning a list of [`LocalMigration`].
///
/// The returned local migrations are sorted by versions in ascending order.
///
/// Since this does not access the database, [`LocalMigration`]s
/// don't have any information about applied state.
pub fn scan_for_migrations(
    migrations_directory: &Path,
) -> Result<Vec<ScannedMigration>, MigrationScanError> {
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
            MigrationScanError::UnableToScanMigrationsDirectory {
                directory_path: migrations_directory.to_path_buf(),
                error,
            }
        })?;

        if !directory_entry.metadata().is_dir() {
            continue;
        }


        let local_migration = ScannedMigration::load_from_directory(directory_entry.path())?;


        if local_migration_versions.contains(&local_migration.identifier.version) {
            return Err(MigrationScanError::MigrationVersionIsNotUnique {
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
