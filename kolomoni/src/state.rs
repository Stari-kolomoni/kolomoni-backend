//! Application-wide state (shared between endpoint functions).

use actix_web::web::Data;
use kolomoni_auth::{ArgonHasher, ArgonHasherError, JsonWebTokenManager};
use kolomoni_configuration::Configuration;
use sqlx::PgPool;
use thiserror::Error;

use crate::establish_database_connection_pool;




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
    pub configuration: Configuration,

    /// Password hasher helper struct.
    pub hasher: ArgonHasher,

    /// PostgreSQL database connection pool.
    pub database_pool: PgPool,

    /// Authentication token manager (JSON Web Token).
    pub jwt_manager: JsonWebTokenManager,
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
