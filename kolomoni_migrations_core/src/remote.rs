use std::time::Duration;

use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgConnection};

use crate::{errors::RemoteMigrationError, identifier::MigrationIdentifier, sha256::Sha256Hash};


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RemoteMigrationType {
    Sql,
    Rust,
}

impl RemoteMigrationType {
    fn try_from_str(migration_type_name: &str) -> Result<Self, ()> {
        match migration_type_name {
            "sql" => Ok(Self::Sql),
            "rust" => Ok(Self::Rust),
            _ => Err(()),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            RemoteMigrationType::Sql => "sql",
            RemoteMigrationType::Rust => "rust",
        }
    }
}


pub struct RemoteMigrationUp {
    #[allow(dead_code)]
    pub(crate) up_script_type: RemoteMigrationType,

    pub(crate) up_script_sha256_hash: Sha256Hash,
}

pub struct RemoteMigrationDown {
    #[allow(dead_code)]
    pub(crate) down_script_type: RemoteMigrationType,

    pub(crate) down_script_sha256_hash: Sha256Hash,
}


/// A partial, remote-only [`Migration`] information,
/// including applied status in the database.
pub(crate) struct RemoteMigration {
    pub(crate) identifier: MigrationIdentifier,

    pub(crate) up_script: RemoteMigrationUp,

    pub(crate) down_script: Option<RemoteMigrationDown>,

    pub(crate) applied_at: DateTime<Utc>,

    #[allow(dead_code)]
    pub(crate) execution_time: Duration,
}


#[derive(FromRow)]
struct IntermediateRemoteMigration {
    version: i64,

    name: String,

    up_script_type: String,

    up_script_sha256_hash: Vec<u8>,

    down_script_type: Option<String>,

    down_script_sha256_hash: Option<Vec<u8>>,

    applied_at: DateTime<Utc>,

    execution_time_milliseconds: i64,
}

impl IntermediateRemoteMigration {
    fn try_into_remote_migration(self) -> Result<RemoteMigration, RemoteMigrationError> {
        let identifier = MigrationIdentifier::new(self.version, self.name);

        let up_script =
            {
                let up_script_type = RemoteMigrationType::try_from_str(&self.up_script_type)
                    .map_err(|_| RemoteMigrationError::InvalidRow {
                        identifier: identifier.clone(),
                        reason: "invalid up_script_type value, expected \"sql\" or \"rust\"".into(),
                    })?;

                let up_script_sha256_hash = Sha256Hash::try_from_vec(self.up_script_sha256_hash)
                    .map_err(|_| RemoteMigrationError::InvalidRow {
                        identifier: identifier.clone(),
                        reason: "invalid up_sql_sha256_hash field: not a 256-bit hash".into(),
                    })?;

                RemoteMigrationUp {
                    up_script_type,
                    up_script_sha256_hash,
                }
            };

        let down_script = match (
            self.down_script_type,
            self.down_script_sha256_hash,
        ) {
            (None, None) => None,
            (None, Some(_)) => {
                return Err(RemoteMigrationError::InvalidRow {
                    identifier: identifier.clone(),
                    reason: "invalid down_* columns: hash is present, but type is NULL".into(),
                });
            }
            (Some(_), None) => {
                return Err(RemoteMigrationError::InvalidRow {
                    identifier: identifier.clone(),
                    reason: "invalid down_* columns: type is present, but hash is NULL".into(),
                });
            }
            (Some(down_script_type), Some(down_script_sha256_hash)) => {
                let down_script_type = RemoteMigrationType::try_from_str(&down_script_type)
                    .map_err(|_| RemoteMigrationError::InvalidRow {
                        identifier: identifier.clone(),
                        reason: "invalid down_script_type value, expected \"sql\" or \"rust\""
                            .into(),
                    })?;

                let down_script_sha256_hash = Sha256Hash::try_from_vec(down_script_sha256_hash)
                    .map_err(|_| RemoteMigrationError::InvalidRow {
                        identifier: identifier.clone(),
                        reason: "invalid down_sql_sha256_hash field: not a 256-bit hash".into(),
                    })?;

                Some(RemoteMigrationDown {
                    down_script_type,
                    down_script_sha256_hash,
                })
            }
        };


        let execution_time = {
            let millis_u64 = u64::try_from(self.execution_time_milliseconds).map_err(|_| {
                RemoteMigrationError::InvalidRow {
                    identifier: identifier.clone(),
                    reason: "invalid execution_time_milliseconds field: must not be negative".into(),
                }
            })?;

            Duration::from_millis(millis_u64)
        };



        Ok(RemoteMigration {
            identifier,
            up_script,
            down_script,
            applied_at: self.applied_at,
            execution_time,
        })
    }
}


impl RemoteMigration {
    pub async fn load_all_from_database(
        database_connection: &mut PgConnection,
    ) -> Result<Vec<Self>, RemoteMigrationError> {
        let intermediate_remote_migrations: Vec<IntermediateRemoteMigration> = sqlx::query_as(
            r#"
            SELECT
                version, name, up_script_type, up_script_sha256_hash,
                down_script_type, down_script_sha256_hash,
                applied_at, execution_time_milliseconds
            FROM kolomoni.schema_migrations
            ORDER BY version ASC
            "#,
        )
        .fetch_all(&mut *database_connection)
        .await
        .map_err(|error| RemoteMigrationError::QueryFailed { error })?;



        let mut remote_migrations = Vec::with_capacity(intermediate_remote_migrations.len());

        for intermediate_model in intermediate_remote_migrations {
            remote_migrations.push(intermediate_model.try_into_remote_migration()?);
        }

        Ok(remote_migrations)
    }

    #[allow(dead_code)]
    pub async fn load_from_database_by_version(
        database_connection: &mut PgConnection,
        migration_version: i64,
    ) -> Result<Option<Self>, RemoteMigrationError> {
        let optional_intermediate_remote_migration: Option<IntermediateRemoteMigration> =
            sqlx::query_as(
                r#"
            SELECT
                version, name, up_script_type, up_script_sha256_hash,
                down_script_type, down_script_sha256_hash,
                applied_at, execution_time_milliseconds
            FROM kolomoni.schema_migrations
            WHERE version = $1
            LIMIT 1
            "#,
            )
            .bind(migration_version)
            .fetch_optional(&mut *database_connection)
            .await
            .map_err(|error| RemoteMigrationError::QueryFailed { error })?;

        let Some(intermediate_remote_migration) = optional_intermediate_remote_migration else {
            return Ok(None);
        };


        Ok(Some(
            intermediate_remote_migration.try_into_remote_migration()?,
        ))
    }
}
