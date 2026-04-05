use std::{borrow::Cow, path::Path, sync::Arc, time::Duration};

use crossbeam_channel::{Receiver, RecvTimeoutError, SendError, Sender};
use kolomoni_cache::EntityCache;
use kolomoni_core::{
    cancellation::CancellationToken,
    ids::{
        EnglishWordId,
        EnglishWordMeaningId,
        SloveneWordId,
        SloveneWordMeaningId,
        WordId,
        WordMeaningId,
    },
};
use kolomoni_database::entities::{
    word::WordLanguage,
    word_english::EnglishWordModel,
    word_meaning_english::EnglishWordMeaningModel,
    word_meaning_slovene::SloveneWordMeaningModel,
    word_slovene::SloveneWordModel,
};
use kolomoni_search_core::SearchIndexModificationMessage;
use parking_lot::RwLock;
use tantivy::{
    collector::{Count, TopDocs},
    directory::{error::OpenDirectoryError, MmapDirectory},
    query::QueryParser,
    schema::{
        BytesOptions,
        Field,
        IndexRecordOption,
        Schema,
        TextFieldIndexing,
        TextOptions,
        Value,
    },
    tokenizer::TokenizerManager,
    Index,
    IndexReader,
    IndexWriter,
    ReloadPolicy,
    TantivyDocument,
    TantivyError,
    Term,
};
use thiserror::Error;
use tokio::task::JoinHandle;
use tracing::{debug, error, info, trace};
use uuid::Uuid;



#[derive(Debug, Error)]
pub enum SearchIndexModificationError {
    #[error("tantivy error")]
    TantivyError {
        #[from]
        #[source]
        error: TantivyError,
    },

    #[error("inconsistent cache state: {}", .reason)]
    InconsistentCacheError { reason: Cow<'static, str> },
}


#[derive(Debug, Error)]
pub enum SearchError {
    #[error("tantivy error")]
    TantivyError {
        #[from]
        #[source]
        error: TantivyError,
    },

    /// This may happen in edge cases where an index hasn't been updated
    /// after removing a word from the database in time for the search.
    /// This should be rare, if not impossible.
    #[error("unable to find matched word in cache: {}", .word_id)]
    MatchedWordNotFoundInCache { word_id: WordId },

    /// This may happen in edge cases where an index hasn't been updated
    /// after removing a word meaning from the database in time for the search.
    /// This should be rare, if not impossible.
    #[error("unable to find matched word meaning in cache: {}", .word_meaning_id)]
    MatchedWordMeaningNotFoundInCache { word_meaning_id: WordMeaningId },

    /// This may happen in edge cases where an index hasn't been updated in time for the search.
    /// This should be rare, if not impossible.
    #[error(
        "unable to find matched translation relationship in cache: {} with {}",
        .english_word_meaning_id,
        .slovene_word_meaning_id
    )]
    MatchedTranslationRelationshipNotFoundInCache {
        english_word_meaning_id: EnglishWordMeaningId,
        slovene_word_meaning_id: SloveneWordMeaningId,
    },
}



pub struct SearchIndexManagerTaskHandle {
    #[allow(dead_code)]
    receiver_loop_task_join_handle: JoinHandle<()>, // TODO + some sort of cancellation token?
}

impl SearchIndexManagerTaskHandle {
    async fn new(
        index_reader: IndexReader,
        index_writer: IndexWriter,
        index_schema_fields: SearchIndexSchemaFields,
        cache: Arc<RwLock<EntityCache>>,
        cancellation_token: CancellationToken,
    ) -> (Self, Sender<SearchIndexModificationMessage>) {
        let (sender, receiver) = crossbeam_channel::bounded(512);

        let receiver_loop_task = tokio::spawn(Self::run_receiver_loop(
            index_reader,
            index_writer,
            index_schema_fields,
            cache,
            receiver,
            cancellation_token,
        ));

        (
            Self {
                receiver_loop_task_join_handle: receiver_loop_task,
            },
            sender,
        )
    }

    async fn run_receiver_loop(
        _index_reader: IndexReader,
        mut index_writer: IndexWriter,
        index_schema_fields: SearchIndexSchemaFields,
        cache: Arc<RwLock<EntityCache>>,
        message_receiver: Receiver<SearchIndexModificationMessage>,
        cancellation_token: CancellationToken,
    ) {
        loop {
            if cancellation_token.is_cancelled() {
                info!("Acknowledging cancellation token is set, stopping search indexer task.");
                return;
            }

            let next_message = match message_receiver.recv_timeout(Duration::from_millis(500)) {
                Ok(message) => message,
                Err(timeout_error) => match timeout_error {
                    RecvTimeoutError::Timeout => continue,
                    RecvTimeoutError::Disconnected => {
                        return;
                    }
                },
            };

            let is_reindex_message = matches!(
                next_message,
                SearchIndexModificationMessage::PerformFullReindex
            );

            let message_processing_result = match next_message {
                SearchIndexModificationMessage::PerformFullReindex => {
                    Self::clean_and_rebuild_index(&cache, &mut index_writer, &index_schema_fields)
                }
                SearchIndexModificationMessage::EnglishWordMeaningCreatedOrUpdated {
                    english_word_meaning,
                    english_word,
                } => Self::on_english_word_meaning_created_or_updated(
                    &english_word_meaning,
                    &english_word,
                    &mut index_writer,
                    &index_schema_fields,
                ),
                SearchIndexModificationMessage::EnglishWordMeaningRemoved {
                    english_word_meaning_id,
                } => Self::on_english_word_meaning_removed(
                    english_word_meaning_id,
                    &mut index_writer,
                    &index_schema_fields,
                ),
                SearchIndexModificationMessage::SloveneWordMeaningCreatedOrUpdated {
                    slovene_word_meaning,
                    slovene_word,
                } => Self::on_slovene_word_meaning_created_or_updated(
                    &slovene_word_meaning,
                    &slovene_word,
                    &mut index_writer,
                    &index_schema_fields,
                ),
                SearchIndexModificationMessage::SloveneWordMeaningRemoved {
                    slovene_word_meaning_id,
                } => Self::on_slovene_word_meaning_removed(
                    slovene_word_meaning_id,
                    &mut index_writer,
                    &index_schema_fields,
                ),
            };

            if !is_reindex_message {
                debug!("Committing index writer.");
                let commit_result = index_writer.commit();

                if let Err(commit_error) = commit_result {
                    error!(
                        "failed to commit index writer: {:?}",
                        commit_error
                    );
                }
            }

            if let Err(message_processing_error) = message_processing_result {
                error!(
                    "failed to update index: {:?}",
                    message_processing_error
                );
            }
        }
    }

    /// # Specifics
    /// The [`IndexWriter`] IS committed (twice) inside this function.
    fn clean_and_rebuild_index(
        cache: &RwLock<EntityCache>,
        index_writer: &mut IndexWriter,
        index_schema_fields: &SearchIndexSchemaFields,
    ) -> Result<(), SearchIndexModificationError> {
        info!("Cleaning and rebuilding search index.");

        index_writer.delete_all_documents()?;
        index_writer.commit()?;

        let locked_cache = cache.write();

        for cached_english_word_meaning in locked_cache.english_word_meanings() {
            trace!(
                "Re-indexing english word meaning: {}",
                cached_english_word_meaning.word_meaning().id()
            );

            let Some(parent_word) = locked_cache
                .english_word(cached_english_word_meaning.word_meaning().parent_word_id())
            else {
                return Err(SearchIndexModificationError::InconsistentCacheError {
                    reason: Cow::Owned(
                        format!(
                            "unable to re-index english word meaning {}, as its parent english word {} is not present in cache",
                            cached_english_word_meaning.word_meaning().id(),
                            cached_english_word_meaning.word_meaning().parent_word_id()
                        )
                    )
                });
            };

            Self::on_english_word_meaning_created_or_updated(
                cached_english_word_meaning.word_meaning(),
                parent_word.word(),
                index_writer,
                index_schema_fields,
            )?;
        }

        info!(
            "Re-indexed {} english word meanings.",
            locked_cache.english_word_meanings().len()
        );


        for cached_slovene_word_meaning in locked_cache.slovene_word_meanings() {
            trace!(
                "Re-indexing slovene word meaning: {}",
                cached_slovene_word_meaning.word_meaning().id()
            );

            let Some(parent_cached_word) = locked_cache
                .slovene_word(cached_slovene_word_meaning.word_meaning().parent_word_id())
            else {
                return Err(SearchIndexModificationError::InconsistentCacheError {
                    reason: Cow::Owned(
                        format!(
                            "unable to re-index slovene word meaning {}, as its parent slovene word {} is not present in cache",
                            cached_slovene_word_meaning.word_meaning().id(),
                            cached_slovene_word_meaning.word_meaning().parent_word_id()
                        )
                    )
                });
            };

            Self::on_slovene_word_meaning_created_or_updated(
                cached_slovene_word_meaning.word_meaning(),
                parent_cached_word.word(),
                index_writer,
                index_schema_fields,
            )?;
        }

        info!(
            "Re-indexed {} slovene word meanings.",
            locked_cache.slovene_word_meanings().len()
        );


        drop(locked_cache);
        index_writer.commit()?;

        info!("Search index rebuilt.");

        Ok(())
    }

    /// # Specifics
    /// The [`IndexWriter`] is NOT committed inside this function.
    ///
    /// # Panics
    /// This function will panic if `english_word` is not `english_word_meaning`'s parent word.
    /// Therefore it is up to the caller to ensure `english_word_meaning` is a meaning of the given `english_word`!
    fn on_english_word_meaning_created_or_updated(
        english_word_meaning: &EnglishWordMeaningModel,
        english_word: &EnglishWordModel,
        index_writer: &mut IndexWriter,
        index_schema_fields: &SearchIndexSchemaFields,
    ) -> Result<(), SearchIndexModificationError> {
        assert_eq!(
            english_word.id(),
            english_word_meaning.parent_word_id(),
            "provided english_word_meaning and english_word do not belong together!"
        );


        // Removes the old english word meaning, if any.
        index_writer.delete_term(Term::from_field_bytes(
            index_schema_fields.word_meaning_id,
            english_word_meaning
                .id()
                .into_uuid()
                .into_bytes()
                .as_slice(),
        ));


        // Inserts the updated or new english word meaning.
        let mut new_document = TantivyDocument::new();

        new_document.add_bytes(
            index_schema_fields.word_meaning_id,
            english_word_meaning.id().into_uuid().as_bytes(),
        );
        new_document.add_bytes(
            index_schema_fields.parent_word_id,
            english_word_meaning.parent_word_id().into_uuid().as_bytes(),
        );

        new_document.add_text(
            index_schema_fields.language_code,
            WordLanguage::English.to_ietf_bcp_47_language_tag(),
        );

        new_document.add_text(index_schema_fields.lemma, english_word.lemma());


        if let Some(disambiguation) = english_word_meaning.disambiguation() {
            new_document.add_text(index_schema_fields.disambiguation, disambiguation);
        }

        if let Some(abbreviation) = english_word_meaning.abbreviation() {
            new_document.add_text(index_schema_fields.abbreviation, abbreviation);
        }

        if let Some(description) = english_word_meaning.description() {
            new_document.add_text(index_schema_fields.description, description);
        }


        index_writer.add_document(new_document)?;

        Ok(())
    }

    /// # Specifics
    /// The [`IndexWriter`] is NOT committed inside this function.
    fn on_english_word_meaning_removed(
        english_word_meaning_id: EnglishWordMeaningId,
        index_writer: &mut IndexWriter,
        index_schema_fields: &SearchIndexSchemaFields,
    ) -> Result<(), SearchIndexModificationError> {
        // Removes the old english word meaning, if any.
        index_writer.delete_term(Term::from_field_bytes(
            index_schema_fields.word_meaning_id,
            english_word_meaning_id.into_uuid().into_bytes().as_slice(),
        ));

        Ok(())
    }

    /// # Specifics
    /// The [`IndexWriter`] is NOT committed inside this function.
    ///
    /// # Panics
    /// This function will panic if `slovene_word` is not `slovene_word_meaning`'s parent word.
    /// Therefore it is up to the caller to ensure `slovene_word_meaning` is a meaning of the given `slovene_word`!
    fn on_slovene_word_meaning_created_or_updated(
        slovene_word_meaning: &SloveneWordMeaningModel,
        slovene_word: &SloveneWordModel,
        index_writer: &mut IndexWriter,
        index_schema_fields: &SearchIndexSchemaFields,
    ) -> Result<(), SearchIndexModificationError> {
        assert_eq!(
            slovene_word.id(),
            slovene_word_meaning.parent_word_id(),
            "provided slovene_word_meaning and slovene_word do not belong together!"
        );


        // Removes the old english word meaning, if any.
        index_writer.delete_term(Term::from_field_bytes(
            index_schema_fields.word_meaning_id,
            slovene_word_meaning
                .id()
                .into_uuid()
                .into_bytes()
                .as_slice(),
        ));


        // Inserts the updated or new english word meaning.
        let mut new_document = TantivyDocument::new();

        new_document.add_bytes(
            index_schema_fields.word_meaning_id,
            slovene_word_meaning.id().into_uuid().as_bytes(),
        );
        new_document.add_bytes(
            index_schema_fields.parent_word_id,
            slovene_word_meaning.parent_word_id().into_uuid().as_bytes(),
        );

        new_document.add_text(
            index_schema_fields.language_code,
            WordLanguage::Slovene.to_ietf_bcp_47_language_tag(),
        );

        new_document.add_text(index_schema_fields.lemma, slovene_word.lemma());


        if let Some(disambiguation) = slovene_word_meaning.disambiguation() {
            new_document.add_text(index_schema_fields.disambiguation, disambiguation);
        }

        if let Some(abbreviation) = slovene_word_meaning.abbreviation() {
            new_document.add_text(index_schema_fields.abbreviation, abbreviation);
        }

        if let Some(description) = slovene_word_meaning.description() {
            new_document.add_text(index_schema_fields.description, description);
        }


        index_writer.add_document(new_document)?;

        Ok(())
    }

    /// # Specifics
    /// The [`IndexWriter`] is NOT committed inside this function.
    fn on_slovene_word_meaning_removed(
        slovene_word_meaning_id: SloveneWordMeaningId,
        index_writer: &mut IndexWriter,
        index_schema_fields: &SearchIndexSchemaFields,
    ) -> Result<(), SearchIndexModificationError> {
        // Removes the old slovene word meaning, if any.
        index_writer.delete_term(Term::from_field_bytes(
            index_schema_fields.word_meaning_id,
            slovene_word_meaning_id.into_uuid().into_bytes().as_slice(),
        ));

        Ok(())
    }
}




/// A collection of fields present in the [`tantivy`] schema
/// for the Kolomoni word meaning index.
#[derive(Clone, PartialEq, Eq, Debug)]
pub(crate) struct SearchIndexSchemaFields {
    word_meaning_id: Field,
    parent_word_id: Field,
    language_code: Field,
    lemma: Field,
    disambiguation: Field,
    abbreviation: Field,
    description: Field,
}

fn build_indexing_schema() -> (Schema, SearchIndexSchemaFields) {
    let mut schema_builder = Schema::builder();

    let stored_bytes_field_options = BytesOptions::default().set_stored();

    let word_meaning_id_field = schema_builder.add_bytes_field(
        "word_meaning_id",
        stored_bytes_field_options.clone(),
    );
    let word_meaning_parent_word_id_field =
        schema_builder.add_bytes_field("parent_word_id", stored_bytes_field_options);

    let stored_text_field_options = TextOptions::default().set_stored();
    let word_meaning_language_field =
        schema_builder.add_text_field("language_code", stored_text_field_options);

    let indexing_options = TextFieldIndexing::default()
        .set_index_option(IndexRecordOption::WithFreqsAndPositions)
        .set_tokenizer("en_stem")
        .set_fieldnorms(true);

    let indexed_text_field_options = TextOptions::default()
        .set_stored()
        .set_indexing_options(indexing_options);


    let word_meaning_lemma_field =
        schema_builder.add_text_field("lemma", indexed_text_field_options.clone());

    let word_meaning_disambiguation_field = schema_builder.add_text_field(
        "disambiguation",
        indexed_text_field_options.clone(),
    );

    let word_meaning_description_field =
        schema_builder.add_text_field("description", indexed_text_field_options.clone());

    let word_meaning_abbreviation_field =
        schema_builder.add_text_field("abbreviation", indexed_text_field_options);


    let schema = schema_builder.build();

    (
        schema,
        SearchIndexSchemaFields {
            word_meaning_id: word_meaning_id_field,
            parent_word_id: word_meaning_parent_word_id_field,
            language_code: word_meaning_language_field,
            lemma: word_meaning_lemma_field,
            disambiguation: word_meaning_disambiguation_field,
            description: word_meaning_description_field,
            abbreviation: word_meaning_abbreviation_field,
        },
    )
}


#[derive(Debug, Error)]
pub enum SearchInitializationError {
    #[error("failed to open index directory")]
    IndexDirectoryError {
        #[from]
        #[source]
        error: OpenDirectoryError,
    },

    #[error("failed to initialize tantivy")]
    TantivyError {
        #[from]
        #[source]
        error: tantivy::error::TantivyError,
    },

    #[error("failed to send initial message")]
    ChannelError {
        #[from]
        #[source]
        error: SendError<SearchIndexModificationMessage>,
    },
}



/// A single shallow search result. The data here is
/// shallow in the sense that only the result score,
/// word language, word ID and word meaning ID are stored.
///
/// To turn these results into a more structured and full
/// representation that can be used to return search results
/// through the API, see [`generate_detailed_search_results`].
pub enum WordMeaningSearchResult {
    English {
        result_score: f32,
        word_id: EnglishWordId,
        word_meaning_id: EnglishWordMeaningId,
    },
    Slovene {
        result_score: f32,
        word_id: SloveneWordId,
        word_meaning_id: SloveneWordMeaningId,
    },
}

impl WordMeaningSearchResult {
    pub fn result_score(&self) -> &f32 {
        match self {
            WordMeaningSearchResult::English { result_score, .. } => result_score,
            WordMeaningSearchResult::Slovene { result_score, .. } => result_score,
        }
    }
}


enum IntermediateWordMeaningSearchResult {
    English {
        result_score: f32,
        word_id: EnglishWordId,
        word_lemma: String,
        word_meaning_id: EnglishWordMeaningId,
    },
    Slovene {
        result_score: f32,
        word_id: SloveneWordId,
        word_lemma: String,
        word_meaning_id: SloveneWordMeaningId,
    },
}

impl IntermediateWordMeaningSearchResult {
    fn lemma(&self) -> &str {
        match self {
            Self::English { word_lemma, .. } => word_lemma,
            Self::Slovene { word_lemma, .. } => word_lemma,
        }
    }

    fn score(&self) -> &f32 {
        match self {
            Self::English { result_score, .. } => result_score,
            Self::Slovene { result_score, .. } => result_score,
        }
    }

    fn score_mut(&mut self) -> &mut f32 {
        match self {
            Self::English { result_score, .. } => result_score,
            Self::Slovene { result_score, .. } => result_score,
        }
    }

    fn into_public_enum(self) -> WordMeaningSearchResult {
        match self {
            Self::English {
                result_score,
                word_id,
                word_meaning_id,
                ..
            } => WordMeaningSearchResult::English {
                result_score,
                word_id,
                word_meaning_id,
            },
            Self::Slovene {
                result_score,
                word_id,
                word_meaning_id,
                ..
            } => WordMeaningSearchResult::Slovene {
                result_score,
                word_id,
                word_meaning_id,
            },
        }
    }
}


pub struct SearchResults {
    pub word_meanings: Vec<WordMeaningSearchResult>,
}


/// Given mutable access to a [`IntermediateWordMeaningSearchResult`], this function applies a modification to its `search_score`
/// field based on the accuracy of the match.
///
/// For example, this will boost a search result's score if the match is exactly perfect,
/// and partially boost it if the query is a substring of the lemma (based on the ratio of the match).
fn rescore_search_result(search_query: &str, result: &mut IntermediateWordMeaningSearchResult) {
    const EXACT_LEMMA_MATCH_MULTIPLIER: f32 = 1.4;

    // This means the maximum multiplication for a partial match is `1.25`
    // (and the minimum multiplication is just above `1.0`).
    const PARTIAL_LEMMA_MATCH_RATIO_DIVISOR: f32 = 4.0;

    let result_lemma = result.lemma();

    if search_query == result_lemma {
        *result.score_mut() *= EXACT_LEMMA_MATCH_MULTIPLIER;
    } else if result_lemma.contains(search_query) {
        // Always in range `[0, 1]`.
        let match_ratio_of_entire_lemma = (search_query.len() as f32) / (result_lemma.len() as f32);

        *result.score_mut() *=
            1.0 + (match_ratio_of_entire_lemma / PARTIAL_LEMMA_MATCH_RATIO_DIVISOR);
    }
}



pub struct SearchEngine {
    #[allow(dead_code)]
    background_task_handle: SearchIndexManagerTaskHandle,

    index_reader: IndexReader,
    index_fields: SearchIndexSchemaFields,
    index_query_parser: QueryParser,
}


impl SearchEngine {
    pub async fn new(
        index_directory_path: &Path,
        cache: Arc<RwLock<EntityCache>>,
        cancellation_token: CancellationToken,
    ) -> Result<(Self, Sender<SearchIndexModificationMessage>), SearchInitializationError> {
        let (schema, schema_fields) = build_indexing_schema();

        let index_directory = MmapDirectory::open(index_directory_path)?;

        let index = Index::builder()
            .schema(schema.clone())
            .tokenizers(TokenizerManager::default())
            .open_or_create(index_directory)?;


        let primary_index_reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommitWithDelay)
            .try_into()?;

        let background_index_reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::Manual)
            .try_into()?;

        let background_index_writer = index.writer(20_000_000)?;


        let mut query_parser = QueryParser::for_index(
            &index,
            vec![
                schema_fields.lemma,
                schema_fields.disambiguation,
                schema_fields.abbreviation,
                schema_fields.description,
            ],
        );

        query_parser.set_field_boost(schema_fields.lemma, 4.0);
        query_parser.set_field_boost(schema_fields.disambiguation, 1.1);
        query_parser.set_field_boost(schema_fields.abbreviation, 1.1);

        query_parser.set_field_fuzzy(schema_fields.lemma, false, 2, true);
        query_parser.set_field_fuzzy(schema_fields.disambiguation, false, 1, true);
        query_parser.set_field_fuzzy(schema_fields.abbreviation, false, 1, true);


        let (background_task_handle, background_task_sender) = SearchIndexManagerTaskHandle::new(
            background_index_reader,
            background_index_writer,
            schema_fields.clone(),
            cache.clone(),
            cancellation_token,
        )
        .await;

        background_task_sender.send(SearchIndexModificationMessage::PerformFullReindex)?;

        Ok((
            Self {
                background_task_handle,
                index_reader: primary_index_reader,
                index_fields: schema_fields,
                index_query_parser: query_parser,
            },
            background_task_sender,
        ))
    }

    pub fn search(&self, query: &str) -> Result<SearchResults, SearchError> {
        const SEARCH_RESULTS_LIMIT: usize = 8;

        let searcher = self.index_reader.searcher();


        let (parsed_query, _query_parse_errors) = self.index_query_parser.parse_query_lenient(query);

        let (_document_count, top_documents) = searcher.search(
            &parsed_query,
            &(Count, TopDocs::with_limit(SEARCH_RESULTS_LIMIT)),
        )?;


        let mut search_results = Vec::with_capacity(SEARCH_RESULTS_LIMIT);

        for (score, document_address) in top_documents {
            let document: TantivyDocument = searcher.doc(document_address)?;


            let word_language = {
                let Some(word_language_value) = document.get_first(self.index_fields.language_code)
                else {
                    panic!("invalid indexed result: missing language_code field");
                };

                let Some(word_language_code) = word_language_value.as_str() else {
                    panic!("invalid indexed result: expected language_code to be a string");
                };

                let Some(language) = WordLanguage::from_ietf_bcp_47_language_tag(word_language_code)
                else {
                    panic!(
                        "invalid indexed result: expected a valid IETF BCP 47 language tag, got {}",
                        word_language_code
                    );
                };

                language
            };

            let word_id = {
                let Some(word_id_value) = document.get_first(self.index_fields.parent_word_id)
                else {
                    panic!("invalid indexed result: missing parent_word_id field");
                };

                let Some(word_id_bytes) = word_id_value.as_bytes() else {
                    panic!("invalid indexed result: expected parent_word_id to be bytes");
                };

                let Ok(word_id_16_bytes) = word_id_bytes.try_into() else {
                    panic!(
                        "invalid indexed result: expected parent_word_id to be precisely 16 bytes"
                    );
                };

                let raw_word_uuid = Uuid::from_bytes_ref(word_id_16_bytes);

                WordId::new(*raw_word_uuid)
            };

            let word_meaning_id = {
                let Some(word_meaning_id_value) =
                    document.get_first(self.index_fields.word_meaning_id)
                else {
                    panic!("invalid indexed result: missing word_meaning_id field");
                };

                let Some(word_meaning_id_bytes) = word_meaning_id_value.as_bytes() else {
                    panic!("invalid indexed result: expected word_meaning_id to be bytes");
                };

                let Ok(word_meaning_id_16_bytes) = word_meaning_id_bytes.try_into() else {
                    panic!(
                        "invalid indexed result: expected word_meaning_id to be precisely 16 bytes"
                    );
                };

                let raw_word_meaning_uuid = Uuid::from_bytes_ref(word_meaning_id_16_bytes);

                WordMeaningId::new(*raw_word_meaning_uuid)
            };

            let word_lemma = {
                let Some(lemma_value) = document.get_first(self.index_fields.lemma) else {
                    panic!("invalid indexed result: missing lemma field");
                };

                let Some(lemma_str) = lemma_value.as_str() else {
                    panic!("invalid indexed result: expected lemma field to be a string");
                };

                lemma_str.to_owned()
            };


            let mut search_result = match word_language {
                WordLanguage::Slovene => IntermediateWordMeaningSearchResult::Slovene {
                    result_score: score,
                    word_id: word_id.downcast_to_slovene_word_id_unchecked(),
                    word_meaning_id: word_meaning_id.downcast_to_slovene_word_meaning_id_unchecked(),
                    word_lemma,
                },
                WordLanguage::English => IntermediateWordMeaningSearchResult::English {
                    result_score: score,
                    word_id: word_id.downcast_to_english_word_id_unchecked(),
                    word_meaning_id: word_meaning_id.downcast_to_english_word_meaning_id_unchecked(),
                    word_lemma,
                },
            };

            rescore_search_result(query, &mut search_result);

            search_results.push(search_result);
        }


        search_results.sort_unstable_by(|first, second| {
            let first_score = first.score();
            let second_score = second.score();

            first_score.total_cmp(second_score).reverse()
        });


        let transformed_search_results = search_results
            .into_iter()
            .map(|internal_result| internal_result.into_public_enum())
            .collect();

        Ok(SearchResults {
            word_meanings: transformed_search_results,
        })
    }
}
