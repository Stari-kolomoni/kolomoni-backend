//! Application-wide state (shared between endpoint functions).

use std::sync::Arc;

use actix_web::web::Data;
use kolomoni_cache::{CacheSeedingError, EntityCache};
use kolomoni_configuration::Configuration;
use kolomoni_core::{
    cancellation::CancellationToken,
    password_hasher::{ArgonHasher, ArgonHasherError},
    token::JsonWebTokenManager,
};
use kolomoni_database::{
    DatabaseConnection,
    DatabaseConnectionAcquireError,
    DatabaseConnectionConnectError,
    DatabaseConnectionOptions,
    DatabaseConnectionPool,
    DatabaseConnectionPoolOptions,
};
use kolomoni_search::{SearchEngine, SearchInitializationError};
use parking_lot::{ArcRwLockReadGuard, ArcRwLockWriteGuard, RwLock};
use thiserror::Error;
use tracing::info;



/*
DEPRECATED remove

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
} */


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


#[inline]
pub fn database_connection_options_from_configuration(
    configuration: &Configuration,
) -> DatabaseConnectionOptions {
    DatabaseConnectionOptions::from_parameters(
        &configuration.database.for_api.host,
        configuration.database.for_api.port,
        &configuration.database.for_api.username,
        configuration.database.for_api.password.as_deref(),
        &configuration.database.for_api.database_name,
        configuration.database.for_api.statement_cache_capacity,
    )
}


#[derive(Debug, Error)]
pub enum ApplicationStateError {
    #[error("failed to initialize password hasher")]
    FailedToInitializePasswordHasher {
        #[from]
        #[source]
        error: ArgonHasherError,
    },

    #[error("failed to seed cache")]
    FailedToSeedCache {
        #[from]
        #[source]
        error: CacheSeedingError,
    },

    #[error("failed to initialize search")]
    FailedToInitializeSearch {
        #[from]
        #[source]
        error: SearchInitializationError,
    },

    #[error("unable to connect to database")]
    UnableToConnectToDatabase {
        #[from]
        #[source]
        error: DatabaseConnectionConnectError,
    },

    #[error("unable to establish first database connection")]
    UnableToObtainInitialDatabaseConnection {
        #[from]
        #[source]
        error: DatabaseConnectionAcquireError,
    },
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
    database_pool: DatabaseConnectionPool,

    /// Authentication token manager (JSON Web Token).
    jwt_manager: JsonWebTokenManager,

    cache: Arc<RwLock<EntityCache>>,

    search: SearchEngine,
}

impl ApplicationStateInner {
    #[allow(clippy::await_holding_lock)]
    pub async fn new(
        configuration: Configuration,
        cancellation_token: CancellationToken,
    ) -> Result<Self, ApplicationStateError> {
        let hasher = ArgonHasher::new(&configuration.secrets.hash_salt)?;


        info!("Parsing database connection options.");
        let database_connection_options =
            database_connection_options_from_configuration(&configuration);

        info!("Connecting to database.");
        let database_pool = DatabaseConnectionPool::connect(
            database_connection_options,
            DatabaseConnectionPoolOptions::default(),
        )
        .await?;


        let jwt_manager = JsonWebTokenManager::new(&configuration.json_web_token.secret);

        info!("Initializing empty cache.");
        let cache = Arc::new(RwLock::new(EntityCache::new_empty()));

        info!("Initializing search engine.");
        let (search, search_management_event_sender) = SearchEngine::new(
            &configuration.search.search_index_directory_path,
            cache.clone(),
            cancellation_token,
        )
        .await?;

        info!("Seeding cache from database (which will trigger a reindex).");
        {
            let mut database_connection = database_pool.acquire_connection().await?;

            // This non-async-aware lock is held over an await point below.
            // This is okay in our situation because, at this point, we are the only ones that could have locked the cache.
            let mut write_locked_cache = cache.write();

            write_locked_cache.set_search_engine_event_sender(search_management_event_sender);

            write_locked_cache
                .clear_and_reseed_cache_from_database(&mut database_connection)
                .await?;

            // // DEBUGONLY
            // // FIXME
            // println!(
            //     "cache reseed result: {:?}",
            //     write_locked_cache
            //         .clear_and_reseed_cache_from_database(&mut database_connection)
            //         .await
            // );
        }
        info!("Finished initial cache seeding.");


        Ok(Self {
            configuration,
            hasher,
            database_pool,
            jwt_manager,
            cache,
            search,
        })
    }

    pub async fn acquire_database_connection(
        &self,
    ) -> Result<DatabaseConnection, DatabaseConnectionAcquireError> {
        self.database_pool.acquire_connection().await
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

    pub fn cache_read(&self) -> ArcRwLockReadGuard<parking_lot::RawRwLock, EntityCache> {
        self.cache.read_arc()
    }

    pub fn cache_write(&self) -> ArcRwLockWriteGuard<parking_lot::RawRwLock, EntityCache> {
        self.cache.write_arc()
    }

    pub fn search_engine(&self) -> &SearchEngine {
        &self.search
    }
}


/// Central application state, wrapped in an actix [`Data`] wrapper-
///
///
/// This enables usage in endpoint functions.
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
///     // state.acquire_database_connection().await?, state.hasher(), ...
///     # todo!();
/// }
/// ```
pub type ApplicationState = Data<ApplicationStateInner>;
