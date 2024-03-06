//! Application-wide state (shared between endpoint functions).

use actix_web::web::Data;
use kolomoni_auth::JsonWebTokenManager;
use kolomoni_configuration::Configuration;
use kolomoni_database::mutation::ArgonHasher;
use kolomoni_search::{ChangeEvent, KolomoniSearchEngine, SearchResults};
use miette::{Context, IntoDiagnostic, Result};
use sea_orm::{prelude::Uuid, DatabaseConnection};
use tokio::sync::mpsc;

use crate::connect_and_set_up_database;


pub struct KolomoniSearchInner {
    pub engine: KolomoniSearchEngine,
    change_sender: mpsc::Sender<ChangeEvent>,
}

impl KolomoniSearchInner {
    #[inline]
    pub async fn search(&self, word_search_query: &str) -> Result<SearchResults> {
        self.engine.search(word_search_query).await
    }

    #[inline]
    pub async fn on_english_word_created_or_updated(&self, word_uuid: Uuid) -> Result<()> {
        self.change_sender
            .send(ChangeEvent::EnglishWordCreatedOrUpdated { word_uuid })
            .await
            .into_diagnostic()
            .wrap_err("Failed to send \"english word created/updated\" event.")
    }

    #[inline]
    pub async fn on_english_word_removed(&self, word_uuid: Uuid) -> Result<()> {
        self.change_sender
            .send(ChangeEvent::EnglishWordRemoved { word_uuid })
            .await
            .into_diagnostic()
            .wrap_err("Failed to send \"english word removed\" event.")
    }


    #[inline]
    pub async fn on_slovene_word_created_or_updated(&self, word_uuid: Uuid) -> Result<()> {
        self.change_sender
            .send(ChangeEvent::SloveneWordCreatedOrUpdated { word_uuid })
            .await
            .into_diagnostic()
            .wrap_err("Failed to send \"slovene word created/updated\" event.")
    }

    #[inline]
    pub async fn on_slovene_word_removed(&self, word_uuid: Uuid) -> Result<()> {
        self.change_sender
            .send(ChangeEvent::SloveneWordRemoved { word_uuid })
            .await
            .into_diagnostic()
            .wrap_err("Failed to send \"slovene word removed\" event.")
    }


    #[inline]
    pub async fn on_category_created_or_updated(&self, category_id: i32) -> Result<()> {
        self.change_sender
            .send(ChangeEvent::CategoryCreatedOrUpdated { category_id })
            .await
            .into_diagnostic()
            .wrap_err("Failed to send \"category created/updated\" event.")
    }

    #[inline]
    pub async fn on_category_removed(&self, category_id: i32) -> Result<()> {
        self.change_sender
            .send(ChangeEvent::CategoryRemoved { category_id })
            .await
            .into_diagnostic()
            .wrap_err("Failed to send \"category removed\" event.")
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
    pub configuration: Configuration,

    /// Password hasher helper struct.
    pub hasher: ArgonHasher,

    /// PostgreSQL database connection.
    pub database: DatabaseConnection,

    /// Authentication token manager (JSON Web Token).
    pub jwt_manager: JsonWebTokenManager,

    pub search: KolomoniSearchInner,
}

impl ApplicationStateInner {
    pub async fn new(configuration: Configuration) -> Result<Self> {
        let hasher = ArgonHasher::new(&configuration)?;
        let database = connect_and_set_up_database(&configuration).await?;
        let jwt_manager = JsonWebTokenManager::new(&configuration.json_web_token.secret);

        let search = {
            let engine = KolomoniSearchEngine::new(&configuration).await?;
            let sender = engine.change_event_sender();

            KolomoniSearchInner {
                engine,
                change_sender: sender,
            }
        };

        Ok(Self {
            configuration,
            hasher,
            database,
            jwt_manager,
            search,
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
