use std::{
    ops::{Deref, DerefMut},
    str::FromStr,
    time::Duration,
};

use sqlx::{
    pool::PoolConnection,
    postgres::{PgConnectOptions, PgPoolOptions},
    Acquire,
    PgConnection,
    PgPool,
    Postgres,
    Transaction,
};
use thiserror::Error;


const DEFAULT_APPLICATION_NAME: &str = concat!(
    "stari-kolomoni-backend_v",
    env!("CARGO_PKG_VERSION")
);

const DEFAULT_STATEMENT_CACHE_CAPACITY: usize = 200;

const DEFAULT_IDLE_TIMEOUT: Option<Duration> = None;

const DEFAULT_MAX_CONNECTION_LIFETIME: Option<Duration> = Some(Duration::from_secs(60 * 60 * 24));

const DEFAULT_MIN_POOL_CONNECTIONS: u32 = 1;
const DEFAULT_MAX_POOL_CONNECTIONS: u32 = 10;


#[derive(Debug, Error)]
#[error("failed to parse connection string")]
pub struct DatabaseConnectionOptionsStringError {
    #[from]
    #[source]
    error: sqlx::Error,
}


pub struct DatabaseConnectionOptions {
    connection_options: PgConnectOptions,
}

impl DatabaseConnectionOptions {
    /// Prepare the database connection options from a PostgreSQL connection string.
    ///
    /// If the connection string does not specify an application name,
    /// it is set to `stari-kolomoni-backend_vX.Y.Z`,
    /// (where `vX.Y.Z` is the crate version); see [`DEFAULT_APPLICATION_NAME`].
    ///
    /// The statement cache capacity will be set to `200`, see [`DEFAULT_STATEMENT_CACHE_CAPACITY`].
    ///
    /// # Internals
    /// This uses [`PgConnectOptions`] under the hood.
    pub fn from_connection_string<S>(
        connection_str: S,
    ) -> Result<Self, DatabaseConnectionOptionsStringError>
    where
        S: AsRef<str>,
    {
        let connection_options = PgConnectOptions::from_str(connection_str.as_ref())?
            .statement_cache_capacity(DEFAULT_STATEMENT_CACHE_CAPACITY);

        // Sets the application name to our default, if unset.
        let connection_options = match connection_options.get_application_name() {
            Some(_) => connection_options,
            None => connection_options.application_name(DEFAULT_APPLICATION_NAME),
        };

        Ok(Self { connection_options })
    }

    /// Prepare the database connection options directly from an existing [`PgConnectOptions`].
    ///
    /// The application name will be set to `stari-kolomoni-backend_vX.Y.Z`,
    /// where `vX.Y.Z` is the crate version; see [`DEFAULT_APPLICATION_NAME`].
    ///
    /// The statement cache capacity will be set to `200` by default unless specified,
    /// see [`DEFAULT_STATEMENT_CACHE_CAPACITY`].
    pub fn from_parameters(
        host: &str,
        port: u16,
        username: &str,
        password: Option<&str>,
        database_name: &str,
        statement_cache_capacity: Option<usize>,
    ) -> Self {
        let connection_options = PgConnectOptions::new_without_pgpass()
            .host(host)
            .port(port)
            .username(username)
            .database(database_name)
            .application_name(DEFAULT_APPLICATION_NAME);

        let connection_options = match password {
            Some(password) => connection_options.password(password),
            None => connection_options,
        };

        let connection_options = match statement_cache_capacity {
            Some(custom_statement_cache_capacity) => {
                connection_options.statement_cache_capacity(custom_statement_cache_capacity)
            }
            None => connection_options.statement_cache_capacity(DEFAULT_STATEMENT_CACHE_CAPACITY),
        };


        Self { connection_options }
    }

    pub fn as_pg_connect_options(&self) -> &PgConnectOptions {
        &self.connection_options
    }

    pub fn into_pg_connect_options(self) -> PgConnectOptions {
        self.connection_options
    }
}

impl FromStr for DatabaseConnectionOptions {
    type Err = DatabaseConnectionOptionsStringError;

    /// Initialize the database connection options from a PostgreSQL connection string.
    ///
    /// If the connection string does not specify an application name,
    /// it is set to `stari-kolomoni-backend_vX.Y.Z`,
    /// (where `vX.Y.Z` is the crate version); see [`DEFAULT_APPLICATION_NAME`].
    ///
    /// This uses [`PgConnectOptions`] under the hood.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_connection_string(s)
    }
}



pub struct DatabaseConnectionPoolOptions {
    /// # Default value
    /// The default idle connection timeout is `None`, i.e. no limit; see [`DEFAULT_IDLE_TIMEOUT`].
    /// Database connection lifetime still has an upper bound, see the [`max_connection_lifetime`] field.
    ///
    ///
    /// [`max_connection_lifetime`]: Self::max_connection_lifetime
    pub idle_timeout: Option<Duration>,

    /// `None` indicates no maximum connection lifetime
    /// (i.e. fully persistent database connections).
    /// **It is recommended to set this to `Some(Duration)`, even if a very long one**;
    /// see [`sqlx::postgres::PgPoolOptions::max_lifetime`] for the rationale
    /// (tl;dr: potential database memory leaks).
    ///
    /// # Default value
    /// The default maximum connection lifetime is one day, see [`DEFAULT_MAX_CONNECTION_LIFETIME`].
    pub max_connection_lifetime: Option<Duration>,

    /// # Default value
    /// The default minimum number of connections in the connection pool is `1`,
    /// see [`DEFAULT_MIN_POOL_CONNECTIONS`].
    pub min_pool_connections: u32,

    /// # Default value
    /// The default maximum number of connections in the connection pool is `10`,
    /// see [`DEFAULT_MAX_POOL_CONNECTIONS`].
    pub max_pool_connections: u32,
}

impl Default for DatabaseConnectionPoolOptions {
    fn default() -> Self {
        Self {
            idle_timeout: DEFAULT_IDLE_TIMEOUT,
            max_connection_lifetime: DEFAULT_MAX_CONNECTION_LIFETIME,
            min_pool_connections: DEFAULT_MIN_POOL_CONNECTIONS,
            max_pool_connections: DEFAULT_MAX_POOL_CONNECTIONS,
        }
    }
}




#[derive(Debug, Error)]
#[error("failed to connect to the database")]
pub struct DatabaseConnectionConnectError {
    #[from]
    #[source]
    error: sqlx::Error,
}

impl DatabaseConnectionConnectError {
    pub fn into_inner(self) -> sqlx::Error {
        self.error
    }
}


#[derive(Debug, Error)]
#[error("failed to acquire connection from database connection pool")]
pub struct DatabaseConnectionAcquireError {
    #[from]
    #[source]
    error: sqlx::Error,
}

impl DatabaseConnectionAcquireError {
    pub fn into_inner(self) -> sqlx::Error {
        self.error
    }
}


pub struct DatabaseConnectionPool {
    pool: PgPool,
}

impl DatabaseConnectionPool {
    pub async fn connect(
        connection_options: DatabaseConnectionOptions,
        pool_options: DatabaseConnectionPoolOptions,
    ) -> Result<Self, DatabaseConnectionConnectError> {
        let pool = PgPoolOptions::new()
            .idle_timeout(pool_options.idle_timeout)
            .max_lifetime(pool_options.max_connection_lifetime)
            .min_connections(pool_options.min_pool_connections)
            .max_connections(pool_options.max_pool_connections)
            .test_before_acquire(true)
            .connect_with(connection_options.into_pg_connect_options())
            .await?;

        Ok(Self { pool })
    }

    pub async fn acquire_connection(
        &self,
    ) -> Result<DatabaseConnection, DatabaseConnectionAcquireError> {
        DatabaseConnection::acquire_from_pool(&self.pool).await
    }
}


pub struct DatabaseConnection {
    connection: PoolConnection<Postgres>,
}

impl DatabaseConnection {
    async fn acquire_from_pool(
        postgres_pool: &PgPool,
    ) -> Result<Self, DatabaseConnectionAcquireError> {
        let connection = postgres_pool.acquire().await?;

        Ok(Self { connection })
    }

    #[inline]
    #[allow(dead_code)]
    pub fn into_inner(self) -> PoolConnection<Postgres> {
        self.connection
    }

    #[inline]
    pub fn transaction(&mut self) -> DatabaseTransactionBuilder<'_> {
        DatabaseTransactionBuilder::new(&mut self.connection)
    }
}

impl AsRef<PgConnection> for DatabaseConnection {
    fn as_ref(&self) -> &PgConnection {
        &self.connection
    }
}

impl AsMut<PgConnection> for DatabaseConnection {
    fn as_mut(&mut self) -> &mut PgConnection {
        &mut self.connection
    }
}

impl Deref for DatabaseConnection {
    type Target = PgConnection;

    fn deref(&self) -> &Self::Target {
        self.connection.deref()
    }
}

impl DerefMut for DatabaseConnection {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.connection.deref_mut()
    }
}


/// PostgreSQL transaction isolation level,
/// see <https://www.postgresql.org/docs/current/sql-set-transaction.html>.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TransactionIsolationLevel {
    ReadCommitted,
    RepeatableRead,
    Serializable,
}

/// PostgreSQL transaction access mode (read/write or read-only),
/// see <https://www.postgresql.org/docs/current/sql-set-transaction.html>.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TransactionAccessMode {
    ReadWrite,
    ReadOnly,
}


pub struct DatabaseTransactionBuilder<'c> {
    connection: &'c mut PoolConnection<Postgres>,
    isolation_level: Option<TransactionIsolationLevel>,
    access_mode: Option<TransactionAccessMode>,
}

impl<'c> DatabaseTransactionBuilder<'c> {
    #[inline]
    fn new(connection: &'c mut PoolConnection<Postgres>) -> Self {
        Self {
            connection,
            isolation_level: None,
            access_mode: None,
        }
    }
}

impl<'c> DatabaseTransactionBuilder<'c> {
    #[allow(dead_code)]
    #[inline]
    fn isolation_level(self, isolation_level: TransactionIsolationLevel) -> Self {
        Self {
            connection: self.connection,
            access_mode: self.access_mode,
            isolation_level: Some(isolation_level),
        }
    }

    #[allow(dead_code)]
    #[inline]
    pub fn isolation_level_read_committed(self) -> Self {
        self.isolation_level(TransactionIsolationLevel::ReadCommitted)
    }

    #[allow(dead_code)]
    #[inline]
    pub fn isolation_level_repeatable_read(self) -> Self {
        self.isolation_level(TransactionIsolationLevel::RepeatableRead)
    }

    #[allow(dead_code)]
    #[inline]
    pub fn isolation_level_serializable(self) -> Self {
        self.isolation_level(TransactionIsolationLevel::Serializable)
    }

    #[allow(dead_code)]
    #[inline]
    fn access_mode(self, access_mode: TransactionAccessMode) -> Self {
        Self {
            connection: self.connection,
            isolation_level: self.isolation_level,
            access_mode: Some(access_mode),
        }
    }

    #[allow(dead_code)]
    #[inline]
    pub fn access_mode_read_write(self) -> Self {
        self.access_mode(TransactionAccessMode::ReadWrite)
    }

    #[allow(dead_code)]
    #[inline]
    pub fn access_mode_read_only(self) -> Self {
        self.access_mode(TransactionAccessMode::ReadOnly)
    }

    pub async fn begin(self) -> Result<DatabaseTransaction<'c>, sqlx::Error> {
        let mut transaction = self.connection.begin().await?;

        // We need to do this large match to get sqlx's compile-time checks.
        if self.isolation_level.is_some() || self.access_mode.is_some() {
            let query = match (self.isolation_level, self.access_mode) {
                (None, Some(access_mode)) => match access_mode {
                    TransactionAccessMode::ReadWrite => {
                        sqlx::query!("SET TRANSACTION READ WRITE")
                    }
                    TransactionAccessMode::ReadOnly => {
                        sqlx::query!("SET TRANSACTION READ ONLY")
                    }
                },
                (Some(isolation_level), None) => match isolation_level {
                    TransactionIsolationLevel::ReadCommitted => {
                        sqlx::query!("SET TRANSACTION ISOLATION LEVEL READ COMMITTED")
                    }
                    TransactionIsolationLevel::RepeatableRead => {
                        sqlx::query!("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ")
                    }
                    TransactionIsolationLevel::Serializable => {
                        sqlx::query!("SET TRANSACTION ISOLATION LEVEL SERIALIZABLE")
                    }
                },
                (Some(isolation_level), Some(access_mode)) => match access_mode {
                    TransactionAccessMode::ReadWrite => match isolation_level {
                        TransactionIsolationLevel::ReadCommitted => {
                            sqlx::query!(
                                "SET TRANSACTION ISOLATION LEVEL READ COMMITTED, READ WRITE"
                            )
                        }
                        TransactionIsolationLevel::RepeatableRead => {
                            sqlx::query!(
                                "SET TRANSACTION ISOLATION LEVEL REPEATABLE READ, READ WRITE"
                            )
                        }
                        TransactionIsolationLevel::Serializable => {
                            sqlx::query!("SET TRANSACTION ISOLATION LEVEL SERIALIZABLE, READ WRITE")
                        }
                    },
                    TransactionAccessMode::ReadOnly => match isolation_level {
                        TransactionIsolationLevel::ReadCommitted => {
                            sqlx::query!("SET TRANSACTION ISOLATION LEVEL READ COMMITTED, READ ONLY")
                        }
                        TransactionIsolationLevel::RepeatableRead => {
                            sqlx::query!(
                                "SET TRANSACTION ISOLATION LEVEL REPEATABLE READ, READ ONLY"
                            )
                        }
                        TransactionIsolationLevel::Serializable => {
                            sqlx::query!("SET TRANSACTION ISOLATION LEVEL SERIALIZABLE, READ ONLY")
                        }
                    },
                },
                (None, None) => unreachable!(
                    "either isolation_level or access_mode must be Some here due to an earlier if statement"
                ),
            };

            query.execute(&mut *transaction).await?;
        }


        Ok(DatabaseTransaction::new(transaction))
    }
}



pub struct DatabaseTransaction<'c> {
    transaction: Transaction<'c, Postgres>,
}

impl<'c> DatabaseTransaction<'c> {
    #[inline]
    fn new(transaction: Transaction<'c, Postgres>) -> Self {
        Self { transaction }
    }

    #[inline]
    pub async fn commit(self) -> Result<(), sqlx::Error> {
        self.transaction.commit().await
    }

    #[inline]
    pub async fn rollback(self) -> Result<(), sqlx::Error> {
        self.transaction.rollback().await
    }

    #[allow(dead_code)]
    #[inline]
    pub fn into_inner(self) -> Transaction<'c, Postgres> {
        self.transaction
    }
}



impl<'c> AsRef<PgConnection> for DatabaseTransaction<'c> {
    fn as_ref(&self) -> &PgConnection {
        &self.transaction
    }
}

impl<'c> AsMut<PgConnection> for DatabaseTransaction<'c> {
    fn as_mut(&mut self) -> &mut PgConnection {
        &mut self.transaction
    }
}

impl<'c> Deref for DatabaseTransaction<'c> {
    type Target = PgConnection;

    fn deref(&self) -> &Self::Target {
        self.transaction.deref()
    }
}

impl<'c> DerefMut for DatabaseTransaction<'c> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.transaction.deref_mut()
    }
}
