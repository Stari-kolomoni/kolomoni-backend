use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgConnection};

use super::MigrationIdentifier;
use crate::{errors::RemoteMigrationError, sha256::Sha256Hash};



/// A partial, remote-only [`Migration`] information,
/// including applied status in the database.
pub struct RemoteMigration {
    pub(crate) identifier: MigrationIdentifier,

    pub(crate) up_sql_sha256_hash: Sha256Hash,

    pub(crate) down_sql_sha256_hash: Option<Sha256Hash>,

    pub(crate) applied_at: DateTime<Utc>,
}


#[derive(FromRow)]
struct IntermediateRemoteMigration {
    version: i64,

    name: String,

    up_sql_sha256_hash: Vec<u8>,

    down_sql_sha256_hash: Option<Vec<u8>>,

    applied_at: DateTime<Utc>,
}

impl RemoteMigration {
    pub async fn load_all_from_database(
        database_connection: &mut PgConnection,
    ) -> Result<Vec<Self>, RemoteMigrationError> {
        let intermediate_remote_migrations: Vec<IntermediateRemoteMigration> = sqlx::query_as(
            r#"
            SELECT version, name, up_sql_sha256_hash, down_sql_sha256_hash, applied_at
            FROM kolomoni.schema_migrations
            ORDER BY version ASC
            "#,
        )
        .fetch_all(&mut *database_connection)
        .await
        .map_err(|error| RemoteMigrationError::UnableToAccessDatabase { error })?;



        let mut remote_migrations = Vec::with_capacity(intermediate_remote_migrations.len());

        for intermediate_model in intermediate_remote_migrations {
            let migration_identifier = MigrationIdentifier::new(
                intermediate_model.version,
                intermediate_model.name,
            );


            let up_sql_sha256_hash = Sha256Hash::try_from_vec(intermediate_model.up_sql_sha256_hash)
                .map_err(|_| RemoteMigrationError::InvalidRow {
                    identifier: migration_identifier.clone(),
                    reason: "invalid up_sql_sha256_hash field: not a 256-bit hash".into(),
                })?;

            let down_sql_sha256_hash = if let Some(down_sql_sha256_hash) =
                intermediate_model.down_sql_sha256_hash
            {
                Some(
                    Sha256Hash::try_from_vec(down_sql_sha256_hash).map_err(|_| {
                        RemoteMigrationError::InvalidRow {
                            identifier: migration_identifier.clone(),
                            reason: "invalid down_sql_sha256_hash field: not a 256-bit hash".into(),
                        }
                    })?,
                )
            } else {
                None
            };

            remote_migrations.push(Self {
                identifier: migration_identifier,
                up_sql_sha256_hash,
                down_sql_sha256_hash,
                applied_at: intermediate_model.applied_at,
            });
        }


        Ok(remote_migrations)
    }

    pub async fn load_from_database_by_version(
        database_connection: &mut PgConnection,
        migration_version: i64,
    ) -> Result<Option<Self>, RemoteMigrationError> {
        let optional_intermediate_remote_migration: Option<IntermediateRemoteMigration> =
            sqlx::query_as(
                r#"
            SELECT version, name, up_sql_sha256_hash, down_sql_sha256_hash, applied_at
            FROM kolomoni.schema_migrations
            WHERE version = $1
            LIMIT 1
            "#,
            )
            .bind(migration_version)
            .fetch_optional(&mut *database_connection)
            .await
            .map_err(|error| RemoteMigrationError::UnableToAccessDatabase { error })?;

        let Some(intermediate_remote_migratio) = optional_intermediate_remote_migration else {
            return Ok(None);
        };


        let migration_identifier = MigrationIdentifier::new(
            intermediate_remote_migratio.version,
            intermediate_remote_migratio.name,
        );

        let up_sql_sha256_hash = Sha256Hash::try_from_vec(
            intermediate_remote_migratio.up_sql_sha256_hash,
        )
        .map_err(|_| RemoteMigrationError::InvalidRow {
            identifier: migration_identifier.clone(),
            reason: "invalid up_sql_sha256_hash field: not a 256-bit hash".into(),
        })?;

        let down_sql_sha256_hash =
            if let Some(down_sql_sha256_hash) = intermediate_remote_migratio.down_sql_sha256_hash {
                Some(
                    Sha256Hash::try_from_vec(down_sql_sha256_hash).map_err(|_| {
                        RemoteMigrationError::InvalidRow {
                            identifier: migration_identifier.clone(),
                            reason: "invalid down_sql_sha256_hash field: not a 256-bit hash".into(),
                        }
                    })?,
                )
            } else {
                None
            };


        Ok(Some(Self {
            identifier: migration_identifier,
            up_sql_sha256_hash,
            down_sql_sha256_hash,
            applied_at: intermediate_remote_migratio.applied_at,
        }))
    }
}
