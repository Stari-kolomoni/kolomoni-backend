//! Application-wide state (shared between endpoint functions).

use std::{
    ops::{Deref, DerefMut},
    time::Duration,
};

use actix_web::web::Data;
use kolomoni_configuration::{Configuration, ForApiDatabaseConfiguration};
use kolomoni_core::{
    password_hasher::{ArgonHasher, ArgonHasherError},
    token::JsonWebTokenManager,
};
use sqlx::{
    pool::PoolConnection,
    postgres::{PgConnectOptions, PgPoolOptions},
    Acquire,
    PgConnection,
    PgPool,
    Pool,
    Postgres,
    Transaction,
};
use thiserror::Error;



pub async fn establish_database_connection_pool(
    database_configuration: &ForApiDatabaseConfiguration,
) -> Result<PgPool, sqlx::Error> {
    let mut connection_options = PgConnectOptions::new_without_pgpass()
        .application_name(&format!(
            "stari-kolomoni-backend-api_v{}",
            env!("CARGO_PKG_VERSION")
        ))
        .statement_cache_capacity(
            database_configuration
                .statement_cache_capacity
                .unwrap_or(200),
        )
        .host(&database_configuration.host)
        .port(database_configuration.port)
        .username(&database_configuration.username)
        .database(&database_configuration.database_name);

    if let Some(password) = &database_configuration.password {
        connection_options = connection_options.password(password.as_str());
    }


    PgPoolOptions::new()
        .idle_timeout(Some(Duration::from_secs(60 * 20)))
        .max_lifetime(Some(Duration::from_secs(60 * 60)))
        .min_connections(1)
        .max_connections(10)
        .test_before_acquire(true)
        .connect_with(connection_options)
        .await
}


// TODO needs to be reworked to be more general (a cache layer), then connect search into it, or maybe even setup this whole thing to be decoupled by using db triggers or something
/*
/// A dictionary search engine.
///
/// Handles searching, seeding and incrementally updating the internal index and cache.
pub struct KolomoniSearch {
    pub engine: KolomoniSearchEngine,
    change_sender: mpsc::Sender<ChangeEvent>,
}

impl KolomoniSearch {
    /// Run a fuzzy word search with the given `word_search_query`.
    /// Returns a list of both slovene and english search results.
    #[inline]
    pub async fn search(&self, word_search_query: &str) -> Result<SearchResults> {
        self.engine.search(word_search_query).await
    }

    /// Signals to the search indexer that an english word has been created or updated.
    ///
    /// This method does not block unless the communication channel is full (which is unlikely).
    /// The indexing (and caching) of the created or updated word will be performed in
    /// a separate async task as soon as the receiver can pick it up, which will very likely be in
    /// less than a second after sending.
    #[inline]
    pub async fn signal_english_word_created_or_updated(&self, word_uuid: Uuid) -> Result<()> {
        self.change_sender
            .send(ChangeEvent::EnglishWordCreatedOrUpdated { word_uuid })
            .await
            .into_diagnostic()
            .wrap_err("Failed to send \"english word created/updated\" event.")
    }

    /// Signals to the search indexer that an english word has been removed from the database.
    ///
    /// This method does not block unless the communication channel is full (which is unlikely).
    /// Removal from index and cache will be performed in a separate async task as soon
    /// as the receiver can pick it up, which will very likely be in less than a second after sending.
    #[inline]
    pub async fn signal_english_word_removed(&self, word_uuid: Uuid) -> Result<()> {
        self.change_sender
            .send(ChangeEvent::EnglishWordRemoved { word_uuid })
            .await
            .into_diagnostic()
            .wrap_err("Failed to send \"english word removed\" event.")
    }


    /// Signals to the search indexer that a slovene word has been created or updated.
    ///
    /// This method does not block unless the communication channel is full (which is unlikely).
    /// The indexing (and caching) of the created or updated word will be performed in
    /// a separate async task as soon as the receiver can pick it up, which will very likely be in
    /// less than a second after sending.
    #[inline]
    pub async fn signal_slovene_word_created_or_updated(&self, word_uuid: Uuid) -> Result<()> {
        self.change_sender
            .send(ChangeEvent::SloveneWordCreatedOrUpdated { word_uuid })
            .await
            .into_diagnostic()
            .wrap_err("Failed to send \"slovene word created/updated\" event.")
    }

    /// Signals to the search indexer that a slovene word has been removed from the database.
    ///
    /// This method does not block unless the communication channel is full (which is unlikely).
    /// Removal from index and cache will be performed in a separate async task as soon
    /// as the receiver can pick it up, which will very likely be in less than a second after sending.
    #[inline]
    pub async fn signal_slovene_word_removed(&self, word_uuid: Uuid) -> Result<()> {
        self.change_sender
            .send(ChangeEvent::SloveneWordRemoved { word_uuid })
            .await
            .into_diagnostic()
            .wrap_err("Failed to send \"slovene word removed\" event.")
    }


    /// Signals to the search indexer that a category has been created or updated.
    ///
    /// This method does not block unless the communication channel is full (which is unlikely).
    /// The indexing (and caching) of the created or updated category will be performed in
    /// a separate async task as soon as the receiver can pick it up, which will very likely be in
    /// less than a second after sending.
    #[inline]
    pub async fn signal_category_created_or_updated(&self, category_id: i32) -> Result<()> {
        self.change_sender
            .send(ChangeEvent::CategoryCreatedOrUpdated { category_id })
            .await
            .into_diagnostic()
            .wrap_err("Failed to send \"category created/updated\" event.")
    }

    /// Signals to the search indexer that a category has been removed from the database.
    ///
    /// This method does not block unless the communication channel is full (which is unlikely).
    /// Removal from index and cache will be performed in a separate async task as soon
    /// as the receiver can pick it up, which will very likely be in less than a second after sending.
    #[inline]
    pub async fn signal_category_removed(&self, category_id: i32) -> Result<()> {
        self.change_sender
            .send(ChangeEvent::CategoryRemoved { category_id })
            .await
            .into_diagnostic()
            .wrap_err("Failed to send \"category removed\" event.")
    }
} */


#[derive(Debug, Error)]
pub enum ApplicationStateError {
    #[error("failed to initialize password hasher")]
    FailedToInitializePasswordHasher {
        #[from]
        #[source]
        error: ArgonHasherError,
    },

    #[error("unable to connect to database")]
    UnableToConnectToDatabase {
        #[from]
        #[source]
        error: sqlx::Error,
    },
}



pub struct DatabaseConnection {
    connection: PoolConnection<Postgres>,
}

impl DatabaseConnection {
    async fn acquire_from_pool(postgres_pool: &Pool<Postgres>) -> Result<Self, sqlx::Error> {
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




/// Central application state.
///
/// Use [`ApplicationState`] instead as it already wraps this struct
/// in [`actix_web::web::Data`]!
///
/// If you need mutable state, opt for internal mutability as the struct
/// is internally essentially wrapped in an `Arc` by actix.
/// For more information about mutable state, see
/// <https://actix.rs/docs/application#shared-mutable-state>.
pub struct ApplicationStateInner {
    /// The configuration that this server was loaded with.
    #[allow(unused)]
    configuration: Configuration,

    /// Password hasher helper struct.
    hasher: ArgonHasher,

    /// PostgreSQL database connection pool.
    database_pool: PgPool,

    /// Authentication token manager (JSON Web Token).
    jwt_manager: JsonWebTokenManager,
    // TODO
    // pub search: KolomoniSearch,
}

impl ApplicationStateInner {
    pub async fn new(configuration: Configuration) -> Result<Self, ApplicationStateError> {
        let hasher = ArgonHasher::new(&configuration.secrets.hash_salt)?;

        let database_pool =
            establish_database_connection_pool(&configuration.database.for_api).await?;

        let jwt_manager = JsonWebTokenManager::new(&configuration.json_web_token.secret);

        /*
        let search = {
            let engine = KolomoniSearchEngine::new(&configuration).await?;
            let sender = engine.change_event_sender();

            KolomoniSearch {
                engine,
                change_sender: sender,
            }
        }; */

        Ok(Self {
            configuration,
            hasher,
            database_pool,
            jwt_manager,
            // search,
        })
    }

    pub async fn acquire_database_connection(&self) -> Result<DatabaseConnection, sqlx::Error> {
        DatabaseConnection::acquire_from_pool(&self.database_pool).await
    }

    #[allow(dead_code)]
    pub fn configuration(&self) -> &Configuration {
        &self.configuration
    }

    pub fn hasher(&self) -> &ArgonHasher {
        &self.hasher
    }

    pub fn jwt_manager(&self) -> &JsonWebTokenManager {
        &self.jwt_manager
    }
}


/// Central application state, wrapped in an actix [`Data`] wrapper-
///
///
/// This enables usage in endpoint functions.///
/// See <https://actix.rs/docs/application#state> for more information.
///
/// # Examples
/// ```no_run
/// # use actix_web::{post, web};
/// # use kolomoni::api::errors::EndpointResult;
/// # use kolomoni::state::ApplicationState;
/// #[post("")]
/// pub async fn some_endpoint(
///     state: ApplicationState,
/// ) -> EndpointResult {
///     // state.database, state.configuration, ...
///     # todo!();
/// }
/// ```
pub type ApplicationState = Data<ApplicationStateInner>;
