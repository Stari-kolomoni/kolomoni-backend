use std::{sync::Arc, time::Duration};

use cache::{CachedCategory, CachedEnglishWord, CachedSloveneWord, KolomoniEntityCache};
use chrono::{DateTime, Utc};
use kolomoni_configuration::Configuration;
use kolomoni_database::query::{
    self,
    CategoriesQueryOptions,
    EnglishWordsQueryOptions,
    ExpandedEnglishWordInfo,
    ExpandedSloveneWordInfo,
    SloveneWordsQueryOptions,
};
use miette::{miette, Context, IntoDiagnostic, Result};
use sea_orm::{Database, DatabaseConnection};
use tantivy::{
    collector::TopDocs,
    directory::MmapDirectory,
    doc,
    query::FuzzyTermQuery,
    schema::{
        Field,
        IndexRecordOption,
        NumericOptions,
        Schema,
        TextFieldIndexing,
        TextOptions,
        Value,
    },
    Index,
    Term,
};
use tokio::{
    sync::{mpsc, Mutex, RwLock, RwLockWriteGuard},
    task::JoinHandle,
};
use tracing::{debug, error, info, warn};
use uuid::Uuid;


mod cache;

/// Indexed language type.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum IndexedWordLanguage {
    Slovene,
    English,
}

impl IndexedWordLanguage {
    /// Returns the integer internally associated with the language.
    /// Do not use externally, this is only for schema use in the `tantivy` index.
    pub fn id(&self) -> u64 {
        match self {
            IndexedWordLanguage::Slovene => 0,
            IndexedWordLanguage::English => 1,
        }
    }

    /// Given an internal language ID, this function returns
    /// the associated language variant, if any.
    pub fn from_id(id: u64) -> Option<Self> {
        match id {
            0 => Some(IndexedWordLanguage::Slovene),
            1 => Some(IndexedWordLanguage::English),
            _ => None,
        }
    }
}


pub enum SearchResult {
    English(ExpandedEnglishWordInfo),
    Slovene(ExpandedSloveneWordInfo),
}

pub struct SearchResults {
    pub words: Vec<SearchResult>,
}


// TODO Next up: need a handler for create/update and delete operations on words and categories
//      so we can keep the cache and index up-to-date as the changes get made.


#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ChangeEvent {
    EnglishWordCreatedOrUpdated { word_uuid: Uuid },
    EnglishWordRemoved { word_uuid: Uuid },
    SloveneWordCreatedOrUpdated { word_uuid: Uuid },
    SloveneWordRemoved { word_uuid: Uuid },
    CategoryCreatedOrUpdated { category_id: i32 },
    CategoryRemoved { category_id: i32 },
}


pub struct WordIndexChangeHandler {
    // TODO Implement sending these events from application state and implement receiving the messages and updating the index/cache.
    sender: mpsc::Sender<ChangeEvent>,

    receiver_task_handle: Mutex<Option<JoinHandle<Result<()>>>>,

    inner: Arc<RwLock<WordIndexInner>>,
    database: DatabaseConnection,
    schema_fields: WordIndexSchemaFields,
}

impl WordIndexChangeHandler {
    // TODO

    pub(crate) async fn new(
        inner: Arc<RwLock<WordIndexInner>>,
        database: DatabaseConnection,
        schema_fields: WordIndexSchemaFields,
    ) -> Arc<Self> {
        let (sender, receiver) = mpsc::channel(512);

        let arc_self = Arc::new(Self {
            sender,
            receiver_task_handle: Mutex::new(None),
            inner,
            database,
            schema_fields,
        });

        let reciver_loop_task = tokio::spawn(arc_self.clone().receiver_loop(receiver));
        *arc_self.receiver_task_handle.lock().await = Some(reciver_loop_task);

        arc_self
    }

    pub fn sender(&self) -> mpsc::Sender<ChangeEvent> {
        self.sender.clone()
    }

    async fn receiver_loop(
        self: Arc<Self>,
        mut receiver: mpsc::Receiver<ChangeEvent>,
    ) -> Result<()> {
        loop {
            let next_change_event = match tokio::time::timeout(
                Duration::from_secs(1),
                receiver.recv(),
            )
            .await
            {
                Ok(potential_event) => match potential_event {
                    Some(event) => event,
                    None => {
                        warn!("WordIndexChangeHandler receiver loop is terminating - channel has been closed.");
                        break;
                    }
                },
                Err(_) => continue,
            };


            let update_result = match next_change_event {
                ChangeEvent::EnglishWordCreatedOrUpdated { word_uuid } => {
                    self.on_english_word_created_or_updated(word_uuid).await
                }
                ChangeEvent::EnglishWordRemoved { word_uuid } => {
                    self.on_english_word_removed(word_uuid).await
                }
                ChangeEvent::SloveneWordCreatedOrUpdated { word_uuid } => {
                    self.on_slovene_word_created_or_updated(word_uuid).await
                }
                ChangeEvent::SloveneWordRemoved { word_uuid } => {
                    self.on_slovene_word_removed(word_uuid).await
                }
                ChangeEvent::CategoryCreatedOrUpdated { category_id } => {
                    self.on_category_created_or_updated(category_id).await
                }
                ChangeEvent::CategoryRemoved { category_id } => {
                    self.on_category_removed(category_id).await
                }
            };

            if let Err(update_err) = update_result {
                error!("Failed to update index+cache: {:?}", update_err);
            }
        }

        Ok(())
    }


    async fn inner_write_lock(&self) -> RwLockWriteGuard<'_, WordIndexInner> {
        self.inner.write().await
    }

    async fn on_english_word_created_or_updated(&self, word_uuid: Uuid) -> Result<()> {
        debug!(
            "Got change event - english word created or updated: {}.",
            word_uuid,
        );

        let mut inner = self.inner_write_lock().await;

        let Some(expanded_word_data) =
            query::EnglishWordQuery::expanded_word_by_uuid(&self.database, word_uuid).await?
        else {
            return Err(miette!(
                "Failed to index+cache english word: word doesn't exist!"
            ));
        };


        let english_word_model = expanded_word_data.word.clone();
        let english_language_index = IndexedWordLanguage::English.id();

        let Some(cached_word_entry) =
            CachedEnglishWord::from_expanded_database_info(expanded_word_data, &inner.cache)
        else {
            return Err(miette!(
                "Failed to index+cache english word: missing related connections."
            ));
        };

        inner.cache.insert_or_update_english_word(cached_word_entry);


        let mut index_writer = inner
            .word_index
            .writer(1024 * 1024 * 16)
            .into_diagnostic()
            .wrap_err("Failed to index+cache english word: failed to initialize index writer.")?;

        index_writer
            .add_document(doc!(
                self.schema_fields.language => english_language_index,
                self.schema_fields.uuid => english_word_model.word_id.to_string(),
                self.schema_fields.lemma => english_word_model.lemma,
                self.schema_fields.disambiguation => english_word_model.disambiguation.unwrap_or_default(),
                self.schema_fields.description => english_word_model.description.unwrap_or_default(),
            ))
            .into_diagnostic()
            .wrap_err("Failed to index+cache english word: failed to add document to index.")?;


        index_writer
            .commit()
            .into_diagnostic()
            .wrap_err("Failed to index+cache english word: failed to commit index change.")?;


        Ok(())
    }

    async fn on_english_word_removed(&self, word_uuid: Uuid) -> Result<()> {
        debug!(
            "Got change event - english word removed: {}.",
            word_uuid,
        );

        let mut inner = self.inner_write_lock().await;


        inner.cache.remove_english_word(word_uuid).map_err(|_| {
            miette!("Failed to remove english word from cache+index: no such word in cache.")
        })?;


        let mut index_writer = inner
            .word_index
            .writer(1024 * 1024 * 16)
            .into_diagnostic()
            .wrap_err(
                "Failed to remove english word from cache+index: failed to initialize index writer.",
            )?;

        index_writer.delete_term(Term::from_field_text(
            self.schema_fields.uuid,
            word_uuid.to_string().as_str(),
        ));

        index_writer.commit().into_diagnostic().wrap_err(
            "Failed to remove english word from cache+index: failed to commit index change.",
        )?;


        Ok(())
    }


    async fn on_slovene_word_created_or_updated(&self, word_uuid: Uuid) -> Result<()> {
        debug!(
            "Got change event - slovene word created or updated: {}.",
            word_uuid,
        );

        let mut inner = self.inner_write_lock().await;

        let Some(expanded_word_data) =
            query::SloveneWordQuery::expanded_word_by_uuid(&self.database, word_uuid).await?
        else {
            return Err(miette!(
                "Failed to index+cache slovene word: word doesn't exist!"
            ));
        };


        let slovene_word_model = expanded_word_data.word.clone();
        let slovene_language_index = IndexedWordLanguage::Slovene.id();

        let Some(cached_word_entry) =
            CachedSloveneWord::from_expanded_database_info(expanded_word_data, &inner.cache)
        else {
            return Err(miette!(
                "Failed to index+cache slovene word: missing related connections."
            ));
        };

        inner.cache.insert_or_update_slovene_word(cached_word_entry);


        let mut index_writer = inner
            .word_index
            .writer(1024 * 1024 * 16)
            .into_diagnostic()
            .wrap_err("Failed to index+cache slovene word: failed to initialize index writer.")?;

        index_writer
            .add_document(doc!(
                self.schema_fields.language => slovene_language_index,
                self.schema_fields.uuid => slovene_word_model.word_id.to_string(),
                self.schema_fields.lemma => slovene_word_model.lemma,
                self.schema_fields.disambiguation => slovene_word_model.disambiguation.unwrap_or_default(),
                self.schema_fields.description => slovene_word_model.description.unwrap_or_default(),
            ))
            .into_diagnostic()
            .wrap_err("Failed to index+cache slovene word: failed to add document to index.")?;


        index_writer
            .commit()
            .into_diagnostic()
            .wrap_err("Failed to index+cache slovene word: failed to commit index change.")?;


        Ok(())
    }

    async fn on_slovene_word_removed(&self, word_uuid: Uuid) -> Result<()> {
        debug!(
            "Got change event - slovene word removed: {}.",
            word_uuid,
        );

        let mut inner = self.inner_write_lock().await;


        inner.cache.remove_slovene_word(word_uuid).map_err(|_| {
            miette!("Failed to remove slovene word from cache+index: no such word in cache.")
        })?;


        let mut index_writer = inner
            .word_index
            .writer(1024 * 1024 * 16)
            .into_diagnostic()
            .wrap_err(
                "Failed to remove slovene word from cache+index: failed to initialize index writer.",
            )?;

        index_writer.delete_term(Term::from_field_text(
            self.schema_fields.uuid,
            word_uuid.to_string().as_str(),
        ));

        index_writer.commit().into_diagnostic().wrap_err(
            "Failed to remove slovene word from cache+index: failed to commit index change.",
        )?;


        Ok(())
    }


    async fn on_category_created_or_updated(&self, category_id: i32) -> Result<()> {
        debug!(
            "Got change event - category created or updated: {}.",
            category_id,
        );

        let mut inner = self.inner_write_lock().await;

        let Some(expanded_word_data) =
            query::CategoryQuery::get_by_id(&self.database, category_id).await?
        else {
            return Err(miette!(
                "Failed to cache category: category doesn't exist!"
            ));
        };

        let cached_category_entry = CachedCategory::from_database_model(expanded_word_data);

        inner.cache.insert_or_update_category(cached_category_entry);


        Ok(())
    }

    async fn on_category_removed(&self, category_id: i32) -> Result<()> {
        debug!(
            "Got change event - category removed: {}.",
            category_id,
        );

        let mut inner = self.inner_write_lock().await;

        inner.cache.remove_category(category_id).map_err(|_| {
            miette!("Failed to remove category from cache: no such category in cache.")
        })?;


        Ok(())
    }
}



/// A collection of fields present in the [`tantivy`] schema
/// for the Kolomoni word index.
#[derive(Clone, PartialEq, Eq, Debug)]
pub(crate) struct WordIndexSchemaFields {
    language: Field,
    uuid: Field,
    lemma: Field,
    disambiguation: Field,
    description: Field,
}


fn construct_indexing_schema() -> (Schema, WordIndexSchemaFields) {
    let mut word_schema_builder = Schema::builder();

    let indexed_word_options = TextOptions::default().set_indexing_options(
        TextFieldIndexing::default().set_index_option(IndexRecordOption::WithFreqsAndPositions),
    );
    let stored_numeric_options = NumericOptions::default().set_stored();
    let stored_word_options = TextOptions::default().set_stored();



    // See [`WordLanguage`] for possible values.
    let word_schema_language =
        word_schema_builder.add_u64_field("language", stored_numeric_options.clone());

    let word_schema_uuid = word_schema_builder.add_text_field("uuid", stored_word_options.clone());

    let word_schema_lemma =
        word_schema_builder.add_text_field("lemma", indexed_word_options.clone());

    let word_schema_disambiguation =
        word_schema_builder.add_text_field("disambiguation", indexed_word_options.clone());

    let word_schema_description =
        word_schema_builder.add_text_field("description", indexed_word_options);



    let word_schema = word_schema_builder.build();

    (
        word_schema,
        WordIndexSchemaFields {
            uuid: word_schema_uuid,
            language: word_schema_language,
            lemma: word_schema_lemma,
            disambiguation: word_schema_disambiguation,
            description: word_schema_description,
        },
    )
}




async fn clear_index_and_cache(inner: &mut RwLockWriteGuard<'_, WordIndexInner>) -> Result<()> {
    // Clear existing index.
    {
        let mut index_writer = inner
            .word_index
            .writer(50_000_000)
            .into_diagnostic()
            .wrap_err("Failed to initialize index writer for clearing index.")?;

        index_writer
            .delete_all_documents()
            .into_diagnostic()
            .wrap_err("Failed to clear index.")?;
        index_writer
            .commit()
            .into_diagnostic()
            .wrap_err("Failed to commit index clear.")?;
    }

    // Clear existing cache.
    inner.cache.clear();

    Ok(())
}


/*
/// Reindex and cache words that have been modified since the last call
/// to [`refresh_modified_words`] or [`initialize_with_fresh_words`].
async fn refresh_modified_entities(index_inner: &Arc<RwLock<WordIndexInner>>) -> Result<()> {
    let mut inner = index_inner.write().await;

    let last_entity_modification_time = inner.last_entity_modification_time;


    let updated_categories = query::CategoryQuery::all(
        &self.database,
        CategoriesQueryOptions {
            only_categories_modified_after: Some(last_entity_modification_time),
        },
    )
    .await?;

    let updated_english_words = query::EnglishWordQuery::all_words_expanded(
        &self.database,
        EnglishWordsQueryOptions {
            only_words_modified_after: Some(last_entity_modification_time),
        },
    )
    .await?;

    let updated_slovene_words = query::SloveneWordQuery::all_words_expanded(
        &self.database,
        SloveneWordsQueryOptions {
            only_words_modified_after: Some(last_entity_modification_time),
        },
    )
    .await?;


    if updated_categories.is_empty()
        && updated_english_words.is_empty()
        && updated_slovene_words.is_empty()
    {
        return Ok(());
    }


    // Calculate updated last modification datetime so we can query only
    // modifications since that time the next time we refresh the index.
    let updated_last_modification_time = updated_english_words
        .iter()
        .map(|info| info.word.last_modified_at)
        .chain(updated_slovene_words.iter().map(|info| info.word.last_modified_at))
        .chain(updated_categories.iter().map(|info| info.last_modified_at))
        .max()
        // PANIC SAFETY: We checked above that at least one Vec is not empty.
        .unwrap()
        .to_utc();


    let mut index_writer = inner
        .word_index
        .writer(50_000_000)
        .into_diagnostic()
        .wrap_err("Failed to initialize index writer for incremental update.")?;


    for updated_category in updated_categories {
        inner
            .cache
            .insert_or_update_category(CachedCategory::from_database_model(
                updated_category.clone(),
            ));
    }

    for slovene_word_info in updated_slovene_words {
        let slovene_word = slovene_word_info.word.clone();
        let ietf_language_tag = WordLanguage::Slovene.to_ietf_language_tag();


        let cached_word_entry =
            CachedSloveneWord::from_expanded_database_info(slovene_word_info, &inner.cache).expect(
                "failed to convert expanded slovene word into a cached word on incremental update",
            );

        inner.cache.insert_or_update_slovene_word(cached_word_entry);


        index_writer
            .add_document(doc!(
                self.schema_fields.language => ietf_language_tag,
                self.schema_fields.uuid => slovene_word.word_id.to_string(),
                self.schema_fields.lemma => slovene_word.lemma,
                self.schema_fields.disambiguation => slovene_word.disambiguation.unwrap_or_default(),
                self.schema_fields.description => slovene_word.description.unwrap_or_default(),
            ))
            .into_diagnostic()
            .wrap_err("Failed to add slovene word to tantivy index for incremental update.")?;
    }

    for english_word_info in updated_english_words {
        let english_word = english_word_info.word.clone();
        let ietf_language_tag = WordLanguage::English.to_ietf_language_tag();


        let cached_word_entry =
            CachedEnglishWord::from_expanded_database_info(english_word_info, &inner.cache).expect(
                "failed to convert expanded english word into a cached word on incremental update",
            );

        inner.cache.insert_or_update_english_word(cached_word_entry);


        index_writer
            .add_document(doc!(
                self.schema_fields.language => ietf_language_tag,
                self.schema_fields.uuid => english_word.word_id.to_string(),
                self.schema_fields.lemma => english_word.lemma,
                self.schema_fields.disambiguation => english_word.disambiguation.unwrap_or_default(),
                self.schema_fields.description => english_word.description.unwrap_or_default(),
            ))
            .into_diagnostic()
            .wrap_err("Failed to add english word to tantivy index for incremental update.")?;
    }


    index_writer
        .commit()
        .into_diagnostic()
        .wrap_err("Failed to commit index for incremental update.")?;


    inner.last_entity_modification_time = updated_last_modification_time;

    Ok(())
} */




pub(crate) struct WordIndexInner {
    word_index: Index,

    cache: KolomoniEntityCache,

    last_entity_modification_time: DateTime<Utc>,
}


/// A search engine implementation for Stari Kolomoni.
///
/// Allows fuzzy matching with a maximum Levenshtein distance of 2.
pub struct KolomoniSearchEngine {
    change_handler: Arc<WordIndexChangeHandler>,

    #[allow(dead_code)]
    schema: Schema,

    schema_fields: WordIndexSchemaFields,

    inner: Arc<RwLock<WordIndexInner>>,

    database: DatabaseConnection,
}

impl KolomoniSearchEngine {
    /// Initialize a new word cache and index.
    ///
    /// Will reuse an existing [`tantivy`] disk index if present.
    pub async fn new(configuration: &Configuration) -> Result<Self> {
        let database = Database::connect(format!(
            "postgres://{}:{}@{}:{}/{}",
            configuration.database.username,
            configuration.database.password,
            configuration.database.host,
            configuration.database.port,
            configuration.database.database_name,
        ))
        .await
        .into_diagnostic()
        .wrap_err("Could not initialize connection to PostgreSQL database.")?;


        let (word_schema, schema_fields) = construct_indexing_schema();

        let word_index_directory =
            MmapDirectory::open(&configuration.search.search_index_directory_path)
                .into_diagnostic()
                .wrap_err("Failed to initialize MmapDirectory for the search index.")?;

        let word_index = Index::open_or_create(word_index_directory, word_schema.clone())
            .into_diagnostic()
            .wrap_err("Failed to initialize word search index.")?;


        let inner = WordIndexInner {
            word_index,
            cache: KolomoniEntityCache::new(),
            last_entity_modification_time: DateTime::<Utc>::MIN_UTC,
        };

        let arc_rwlock_inner = Arc::new(RwLock::new(inner));
        let arc_rwlock_inner_clone = arc_rwlock_inner.clone();


        let change_handler = WordIndexChangeHandler::new(
            arc_rwlock_inner_clone,
            database.clone(),
            schema_fields.clone(),
        )
        .await;

        Ok(Self {
            change_handler,
            inner: arc_rwlock_inner,
            schema: word_schema,
            schema_fields,
            database,
        })
    }

    pub fn change_event_sender(&self) -> mpsc::Sender<ChangeEvent> {
        self.change_handler.sender()
    }

    /// Returns matching english and slovene words for the given search query.
    /// Does not perform any database lookups, and instead relies on the index and cache being up-to-date.
    pub async fn search(&self, word_search_query: &str) -> Result<SearchResults> {
        let inner = self.inner.read().await;


        let reader = inner
            .word_index
            .reader()
            .into_diagnostic()
            .wrap_err("Failed to initialize word index reader.")?;

        let searcher = reader.searcher();


        let search_term = Term::from_field_text(self.schema_fields.lemma, word_search_query);
        let search_query = FuzzyTermQuery::new(search_term, 2, true);

        let search_results = searcher
            .search(&search_query, &TopDocs::with_limit(6))
            .into_diagnostic()
            .wrap_err("Failed to search word index.")?;


        let mut resulting_words = Vec::new();
        for (_score, doc_address) in search_results {
            let document = searcher
                .doc(doc_address)
                .into_diagnostic()
                .wrap_err("Failed to retrieve search result.")?;


            let word_language = {
                let word_language_value = document
                    .get_first(self.schema_fields.language)
                    .ok_or_else(|| miette!("BUG: Failed to look up word language after search."))?;

                let Value::U64(word_language_index) = word_language_value else {
                    return Err(miette!(
                        "BUG: Failed to extract word language index after search: {:?}.",
                        word_language_value
                    ));
                };

                IndexedWordLanguage::from_id(*word_language_index).ok_or_else(|| {
                    miette!(
                        "BUG: Invalid word language index: {}",
                        word_language_index
                    )
                })?
            };

            let word_uuid = {
                let word_uuid_value = document
                    .get_first(self.schema_fields.uuid)
                    .ok_or_else(|| miette!("BUG: Failed to look up word UUID after search."))?;

                let Value::Str(word_uuid_string) = word_uuid_value else {
                    return Err(miette!(
                        "BUG: Failed to extract word UUID after search: {:?}.",
                        word_uuid_value
                    ));
                };

                Uuid::try_parse(word_uuid_string)
                    .into_diagnostic()
                    .wrap_err("BUG: Failed to convert string to UUID after search.")?
            };



            let matching_word = match word_language {
                IndexedWordLanguage::Slovene => inner
                    .cache
                    .slovene_word(word_uuid)
                    .map(SearchResult::Slovene),
                IndexedWordLanguage::English => inner
                    .cache
                    .english_word(word_uuid)
                    .map(SearchResult::English),
            };

            if let Some(matching_word) = matching_word {
                resulting_words.push(matching_word);
            } else {
                warn!(
                    word_uuid = %word_uuid,
                    word_language = ?word_language,
                    "Failed to look up word in search cache."
                );
            }
        }


        Ok(SearchResults {
            words: resulting_words,
        })
    }


    /// Clear the index and cache and refresh their contents from a full database scan.
    pub async fn initialize_with_fresh_entries(&mut self) -> Result<()> {
        let mut inner = self.inner.write().await;

        info!("Clearing any existing search index and cache.");

        clear_index_and_cache(&mut inner).await?;

        info!("Initializing search index and cache with fresh entries from the database.");


        let all_categories =
            query::CategoryQuery::all(&self.database, CategoriesQueryOptions::default()).await?;

        let all_english_words = query::EnglishWordQuery::all_words_expanded(
            &self.database,
            EnglishWordsQueryOptions::default(),
        )
        .await?;

        let all_slovene_words = query::SloveneWordQuery::all_words_expanded(
            &self.database,
            SloveneWordsQueryOptions::default(),
        )
        .await?;


        if all_categories.is_empty() && all_english_words.is_empty() && all_slovene_words.is_empty()
        {
            info!("Search index and cache generated - no entries.");
            return Ok(());
        }


        let all_categories_count = all_categories.len();
        let all_english_words_count = all_english_words.len();
        let all_slovene_words_count = all_slovene_words.len();


        // Calculate latest modification datetime so we can query only
        // modifications since that time the next time we refresh the index.
        let last_modification_time = all_english_words
            .iter()
            .map(|info| info.word.last_modified_at)
            .chain(all_slovene_words.iter().map(|info| info.word.last_modified_at))
            .chain(all_categories.iter().map(|info| info.last_modified_at))
            .max()
            // PANIC SAFETY: We checked above that at least one Vec is not empty.
            .unwrap()
            .to_utc();


        let mut index_writer = inner
            .word_index
            .writer(50_000_000)
            .into_diagnostic()
            .wrap_err("Failed to initialize index writer.")?;


        debug!(
            "Inserting {} categories into cache.",
            all_categories_count
        );

        for category in all_categories {
            inner
                .cache
                .insert_or_update_category(CachedCategory::from_database_model(category));
        }


        debug!(
            "Inserting {} slovene words into cache and index.",
            all_slovene_words_count
        );

        for slovene_word_info in all_slovene_words {
            let slovene_word = slovene_word_info.word.clone();
            let slovene_language_index = IndexedWordLanguage::Slovene.id();

            // TODO If (or when) the database will start to contain more complex references
            //      (e.g. english words linked to other english words), we'll need to modify this approach:
            //      we'll create a queue and try to insert entities into the cache until no elements are remaining.
            let cached_word_entry =
                CachedSloveneWord::from_expanded_database_info(slovene_word_info, &inner.cache)
                    .expect("failed to convert expanded slovene word info into a cached word");

            inner.cache.insert_or_update_slovene_word(cached_word_entry);


            index_writer
            .add_document(doc!(
                self.schema_fields.language => slovene_language_index,
                self.schema_fields.uuid => slovene_word.word_id.to_string(),
                self.schema_fields.lemma => slovene_word.lemma,
                self.schema_fields.disambiguation => slovene_word.disambiguation.unwrap_or_default(),
                self.schema_fields.description => slovene_word.description.unwrap_or_default(),
            ))
            .into_diagnostic()
            .wrap_err("Failed to add slovene word to tantivy index.")?;
        }


        debug!(
            "Inserting {} english words into cache and index.",
            all_english_words_count
        );

        for english_word_info in all_english_words {
            let english_word = english_word_info.word.clone();
            let english_language_index = IndexedWordLanguage::English.id();


            let cached_word_entry =
                CachedEnglishWord::from_expanded_database_info(english_word_info, &inner.cache)
                    .expect("failed to convert expanded english word info into a cached word");

            inner.cache.insert_or_update_english_word(cached_word_entry);


            index_writer
            .add_document(doc!(
                self.schema_fields.language => english_language_index,
                self.schema_fields.uuid => english_word.word_id.to_string(),
                self.schema_fields.lemma => english_word.lemma,
                self.schema_fields.disambiguation => english_word.disambiguation.unwrap_or_default(),
                self.schema_fields.description => english_word.description.unwrap_or_default(),
            ))
            .into_diagnostic()
            .wrap_err("Failed to add english word to tantivy index.")?;
        }


        index_writer
            .commit()
            .into_diagnostic()
            .wrap_err("Failed to commit index.")?;


        inner.last_entity_modification_time = last_modification_time;


        info!(
            "Search index and cache generated - {} slovene words, {} english words, {} categories.",
            all_slovene_words_count, all_english_words_count, all_categories_count
        );

        Ok(())
    }
}
