use std::collections::HashMap;

use chrono::{DateTime, Utc};
use kolomoni_configuration::{Configuration, SearchConfiguration};
use kolomoni_database::{
    entities,
    query::{
        self,
        EnglishWordsQueryOptions,
        ExpandedEnglishWordInfo,
        ExpandedSloveneWordInfo,
        SloveneWordsQueryOptions,
    },
    shared::WordLanguage,
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
use uuid::Uuid;


#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum IndexedWordLanguage {
    Slovene,
    English,
}

impl IndexedWordLanguage {
    pub fn id(&self) -> u64 {
        match self {
            IndexedWordLanguage::Slovene => 0,
            IndexedWordLanguage::English => 1,
        }
    }

    pub fn from_id(id: u64) -> Option<Self> {
        match id {
            0 => Some(IndexedWordLanguage::Slovene),
            1 => Some(IndexedWordLanguage::English),
            _ => None,
        }
    }
}


#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CachedEnglishWord {
    word: entities::word_english::Model,
    categories: Vec<entities::category::Model>,

    // PERF(memory): The following two fields need memory improvements (via a global word cache registry + Arc + sharded lock).
    suggested_translations: Vec<CachedSloveneWord>,
    translations: Vec<CachedSloveneWord>,
}

impl CachedEnglishWord {
    pub fn to_expanded_word_info(self) -> ExpandedEnglishWordInfo {
        let suggested_translations = self
            .suggested_translations
            .into_iter()
            .map(CachedSloveneWord::to_expanded_word_info)
            .collect();

        let translations = self
            .translations
            .into_iter()
            .map(CachedSloveneWord::to_expanded_word_info)
            .collect();


        ExpandedEnglishWordInfo {
            word: self.word,
            categories: self.categories,
            suggested_translations,
            translations,
        }
    }
}

impl From<ExpandedEnglishWordInfo> for CachedEnglishWord {
    fn from(value: ExpandedEnglishWordInfo) -> Self {
        let suggested_translations = value
            .suggested_translations
            .into_iter()
            .map(Into::into)
            .collect();

        let translations = value.translations.into_iter().map(Into::into).collect();


        Self {
            word: value.word,
            categories: value.categories,
            suggested_translations,
            translations,
        }
    }
}


#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CachedSloveneWord {
    word: entities::word_slovene::Model,
    categories: Vec<entities::category::Model>,
}

impl CachedSloveneWord {
    pub fn to_expanded_word_info(self) -> ExpandedSloveneWordInfo {
        ExpandedSloveneWordInfo {
            word: self.word,
            categories: self.categories,
        }
    }
}

impl From<ExpandedSloveneWordInfo> for CachedSloveneWord {
    fn from(value: ExpandedSloveneWordInfo) -> Self {
        Self {
            word: value.word,
            categories: value.categories,
        }
    }
}



#[derive(Clone, PartialEq, Eq, Debug)]
pub enum CachedWord {
    English(CachedEnglishWord),
    Slovene(CachedSloveneWord),
}

impl CachedWord {
    pub fn uuid(&self) -> Uuid {
        match self {
            CachedWord::English(english_word) => english_word.word.word_id,
            CachedWord::Slovene(slovene_word) => slovene_word.word.word_id,
        }
    }
}

impl From<ExpandedEnglishWordInfo> for CachedWord {
    fn from(value: ExpandedEnglishWordInfo) -> Self {
        Self::English(value.into())
    }
}

impl From<ExpandedSloveneWordInfo> for CachedWord {
    fn from(value: ExpandedSloveneWordInfo) -> Self {
        Self::Slovene(value.into())
    }
}




pub struct CachedWordStorage {
    pub words: HashMap<Uuid, CachedWord>,
}

impl CachedWordStorage {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            words: HashMap::new(),
        }
    }

    pub fn put_into_cache(&mut self, word: CachedWord) {
        self.words.insert(word.uuid(), word);
    }

    pub fn word_from_cache(&self, word_uuid: &Uuid) -> Option<&CachedWord> {
        self.words.get(word_uuid)
    }
}



pub(crate) struct WordIndexSchemaFields {
    language: Field,

    uuid: Field,

    lemma: Field,

    disambiguation: Field,

    description: Field,
}



pub struct WordIndex {
    index: Index,

    #[allow(dead_code)]
    schema: Schema,

    fields: WordIndexSchemaFields,

    cache: CachedWordStorage,

    last_modification_time: tokio::sync::Mutex<DateTime<Utc>>,

    database: DatabaseConnection,
}

impl WordIndex {
    pub fn new(configuration: &SearchConfiguration, database: DatabaseConnection) -> Result<Self> {
        let mut word_schema_builder = Schema::builder();

        let indexed_word_options = TextOptions::default().set_indexing_options(
            TextFieldIndexing::default().set_index_option(IndexRecordOption::WithFreqsAndPositions),
        );
        let stored_numeric_options = NumericOptions::default().set_stored();
        let stored_word_options = TextOptions::default().set_stored();



        // See [`WordLanguage`] for possible values.
        let word_schema_language =
            word_schema_builder.add_u64_field("language", stored_numeric_options.clone());

        let word_schema_uuid =
            word_schema_builder.add_text_field("uuid", stored_word_options.clone());

        let word_schema_lemma =
            word_schema_builder.add_text_field("lemma", indexed_word_options.clone());

        let word_schema_disambiguation =
            word_schema_builder.add_text_field("disambiguation", indexed_word_options.clone());

        let word_schema_description =
            word_schema_builder.add_text_field("description", indexed_word_options);



        let word_schema = word_schema_builder.build();


        let word_index_directory = MmapDirectory::open(&configuration.search_index_directory_path)
            .into_diagnostic()
            .wrap_err("Failed to initialize MmapDirectory for the search index.")?;

        let word_index = Index::open_or_create(word_index_directory, word_schema.clone())
            .into_diagnostic()
            .wrap_err("Failed to initialize word search index.")?;


        Ok(Self {
            index: word_index,
            schema: word_schema,
            fields: WordIndexSchemaFields {
                uuid: word_schema_uuid,
                language: word_schema_language,
                lemma: word_schema_lemma,
                disambiguation: word_schema_disambiguation,
                description: word_schema_description,
            },
            cache: CachedWordStorage::new(),
            last_modification_time: tokio::sync::Mutex::new(DateTime::<Utc>::MIN_UTC),
            database,
        })
    }

    pub async fn initialize_with_fresh_words(&mut self) -> Result<()> {
        let mut locked_last_modification_time = self.last_modification_time.lock().await;

        // Clear existing index.
        {
            let mut index_writer = self
                .index
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


        if all_english_words.is_empty() && all_slovene_words.is_empty() {
            return Ok(());
        }


        // Calculate latest modification datetime so we can query only
        // modifications since that time the next time we refresh the index.
        let last_modification_time = all_english_words
            .iter()
            .map(|info| info.word.last_modified_at)
            .chain(all_slovene_words.iter().map(|info| info.word.last_modified_at))
            .max()
            // PANIC SAFETY: We checked above that at least one Vec is not empty.
            .unwrap()
            .to_utc();


        let mut index_writer = self
            .index
            .writer(50_000_000)
            .into_diagnostic()
            .wrap_err("Failed to initialize index writer.")?;

        for english_word_info in all_english_words {
            self.cache.put_into_cache(english_word_info.clone().into());

            let english_word = english_word_info.word;
            let ietf_language_tag = WordLanguage::English.to_ietf_language_tag();

            index_writer
                .add_document(doc!(
                    self.fields.language => ietf_language_tag,
                    self.fields.uuid => english_word.word_id.to_string(),
                    self.fields.lemma => english_word.lemma,
                    self.fields.disambiguation => english_word.disambiguation.unwrap_or_default(),
                    self.fields.description => english_word.description.unwrap_or_default(),
                ))
                .into_diagnostic()
                .wrap_err("Failed to add english word to tantivy index.")?;
        }

        for slovene_word_info in all_slovene_words {
            self.cache.put_into_cache(slovene_word_info.clone().into());

            let slovene_word = slovene_word_info.word;
            let ietf_language_tag = WordLanguage::Slovene.to_ietf_language_tag();

            index_writer
                .add_document(doc!(
                    self.fields.language => ietf_language_tag,
                    self.fields.uuid => slovene_word.word_id.to_string(),
                    self.fields.lemma => slovene_word.lemma,
                    self.fields.disambiguation => slovene_word.disambiguation.unwrap_or_default(),
                    self.fields.description => slovene_word.description.unwrap_or_default(),
                ))
                .into_diagnostic()
                .wrap_err("Failed to add slovene word to tantivy index.")?;
        }


        index_writer
            .commit()
            .into_diagnostic()
            .wrap_err("Failed to commit index.")?;


        *locked_last_modification_time = last_modification_time;
        drop(locked_last_modification_time);

        Ok(())
    }

    pub async fn refresh_modified_words(&mut self) -> Result<()> {
        let mut locked_last_modification_time = self.last_modification_time.lock().await;


        let all_english_words = query::EnglishWordQuery::all_words_expanded(
            &self.database,
            EnglishWordsQueryOptions {
                only_words_modified_after: Some(*locked_last_modification_time),
            },
        )
        .await?;

        let all_slovene_words = query::SloveneWordQuery::all_words_expanded(
            &self.database,
            SloveneWordsQueryOptions {
                only_words_modified_after: Some(*locked_last_modification_time),
            },
        )
        .await?;


        if all_english_words.is_empty() && all_slovene_words.is_empty() {
            return Ok(());
        }


        // Calculate updated last modification datetime so we can query only
        // modifications since that time the next time we refresh the index.
        let last_modification_time = all_english_words
            .iter()
            .map(|info| info.word.last_modified_at)
            .chain(all_slovene_words.iter().map(|info| info.word.last_modified_at))
            .max()
            // PANIC SAFETY: We checked above that at least one Vec is not empty.
            .unwrap()
            .to_utc();


        let mut index_writer = self
            .index
            .writer(50_000_000)
            .into_diagnostic()
            .wrap_err("Failed to initialize index writer for incremental update.")?;

        for english_word_info in all_english_words {
            self.cache.put_into_cache(english_word_info.clone().into());

            let english_word = english_word_info.word;
            let ietf_language_tag = WordLanguage::English.to_ietf_language_tag();

            index_writer
                .add_document(doc!(
                    self.fields.language => ietf_language_tag,
                    self.fields.uuid => english_word.word_id.to_string(),
                    self.fields.lemma => english_word.lemma,
                    self.fields.disambiguation => english_word.disambiguation.unwrap_or_default(),
                    self.fields.description => english_word.description.unwrap_or_default(),
                ))
                .into_diagnostic()
                .wrap_err("Failed to add english word to tantivy index for incremental update.")?;
        }

        for slovene_word_info in all_slovene_words {
            self.cache.put_into_cache(slovene_word_info.clone().into());

            let slovene_word = slovene_word_info.word;
            let ietf_language_tag = WordLanguage::Slovene.to_ietf_language_tag();

            index_writer
                .add_document(doc!(
                    self.fields.language => ietf_language_tag,
                    self.fields.uuid => slovene_word.word_id.to_string(),
                    self.fields.lemma => slovene_word.lemma,
                    self.fields.disambiguation => slovene_word.disambiguation.unwrap_or_default(),
                    self.fields.description => slovene_word.description.unwrap_or_default(),
                ))
                .into_diagnostic()
                .wrap_err("Failed to add slovene word to tantivy index for incremental update.")?;
        }


        index_writer
            .commit()
            .into_diagnostic()
            .wrap_err("Failed to commit index for incremental update.")?;


        *locked_last_modification_time = last_modification_time;
        drop(locked_last_modification_time);

        Ok(())
    }

    pub fn search(&self, word_search_query: &str) -> Result<Vec<CachedWord>> {
        let reader = self
            .index
            .reader()
            .into_diagnostic()
            .wrap_err("Failed to initialize word index reader.")?;

        let searcher = reader.searcher();


        let search_term = Term::from_field_text(self.fields.lemma, word_search_query);
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


            let word_uuid_value = document
                .get_first(self.fields.uuid)
                .ok_or_else(|| miette!("BUG: Failed to look up word UUID after search."))?;

            let Value::Str(word_uuid_string) = word_uuid_value else {
                return Err(miette!(
                    "BUG: Failed to extract word UUID after search."
                ));
            };

            let word_uuid = Uuid::try_parse(word_uuid_string)
                .into_diagnostic()
                .wrap_err("BUG: Failed to convert string to UUID after search.")?;


            let word_from_cache = self
                .cache
                .word_from_cache(&word_uuid)
                .ok_or_else(|| miette!("BUG: No word in cache even though search looked it up!"))?;

            resulting_words.push(word_from_cache.to_owned());
        }

        Ok(resulting_words)
    }
}


pub struct KolomoniSearchEngine {
    pub word_index: WordIndex,
}

impl KolomoniSearchEngine {
    #[allow(clippy::new_without_default)]
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

        let word_index = WordIndex::new(&configuration.search, database.clone())
            .wrap_err("Failed to initialize word indexing.")?;


        Ok(Self { word_index })
    }

    // TODO writers, readers + ways of signaling that a specific word needs to be refreshed with data from the database
}
