use std::{borrow::Cow, collections::HashSet};

use chrono::{DateTime, Utc};
use crossbeam_channel::Sender;
use entities::{
    CachedCategory,
    CachedEnglishWord,
    CachedEnglishWordMeaning,
    CachedSloveneWord,
    CachedSloveneWordMeaning,
    CachedTranslationRelationship,
};
use futures_util::stream::StreamExt;
use kolomoni_core::ids::{
    CategoryId,
    EnglishWordId,
    EnglishWordMeaningId,
    SloveneWordId,
    SloveneWordMeaningId,
    UserId,
};
use kolomoni_database::{
    entities::{
        category::{CategoryModel, CategoryQuery},
        word_english::{EnglishWordModel, EnglishWordQuery, EnglishWordsQueryOptions},
        word_meaning_english::EnglishWordMeaningModel,
        word_meaning_slovene::SloveneWordMeaningModel,
        word_slovene::{SloveneWordModel, SloveneWordQuery, SloveneWordsQueryOptions},
    },
    DatabaseConnection,
    QueryError,
};
use kolomoni_search_core::SearchIndexModificationMessage;
use store::{EntityStore, EntityStoreEntry, EntityStoreInsertionAction};
use thiserror::Error;
use tracing::{debug, error, info};


pub mod entities;
pub mod store;



#[derive(Debug, Error)]
pub enum WordMeaningEntityError {
    /// Returned when the parent entity of a given entity does not exist.
    ///
    /// For example, this is returned when trying to insert an english word meaning,
    /// but its parent english word is not present in the cache.
    #[error("parent entity does not exist")]
    ParentEntityNotFound,
}

#[derive(Debug, Error)]
pub enum SloveneCategoryError {
    #[error("english word not found in cache")]
    EnglishWordMeaningNotFound,

    #[error("category not found in cache")]
    CategoryNotFound,
}

#[derive(Debug, Error)]
pub enum EnglishCategoryError {
    #[error("slovene word not found in cache")]
    SloveneWordMeaningNotFound,

    #[error("category not found in cache")]
    CategoryNotFound,
}




#[derive(Debug, Error)]
pub enum EnglishWordMeaningRemovalError {
    #[error("the parent english word ({}) does not exist in the cache", .parent_english_word_id)]
    ParentEnglishWordNotFound {
        parent_english_word_id: EnglishWordId,
    },

    #[error(
        "the parent english word ({}) existed, but did not have \
        this english word meaning as one of its linked meanings",
        .parent_english_word_id
    )]
    ParentEnglishWordMisconfigured {
        parent_english_word_id: EnglishWordId,
    },
}



#[derive(Debug, Error)]
pub enum EntityReadError {
    #[error("at least one of the entities does not exist")]
    EntityNotFound,
}


#[derive(Debug, Error)]
pub enum EnglishWordCacheError {
    #[error("at least one of the entities does not exist")]
    EntityNotFound,
}

#[derive(Debug, Error)]
pub enum CacheSeedingError {
    #[error("query error")]
    QueryError {
        #[from]
        #[source]
        error: QueryError,
    },

    #[error("invalid relationship encountered: {}", .details)]
    InvalidRelationship { details: String },
}


#[derive(Debug, Error)]
#[error("invalid cache state: {}", .reason)]
pub struct InvalidCacheError {
    reason: Cow<'static, str>,
}

impl InvalidCacheError {
    pub fn new<R>(reason: R) -> Self
    where
        R: Into<Cow<'static, str>>,
    {
        Self {
            reason: reason.into(),
        }
    }
}


#[derive(Debug, Error)]
pub enum EnglishWordMeaningsLookupError {
    #[error("linked word meaning not founc in cache")]
    WordMeaningNotFoundInCache {
        word_meaning_id: EnglishWordMeaningId,
    },
}


pub struct EntityCache {
    english_words: EntityStore<EnglishWordId, CachedEnglishWord>,

    english_word_meanings: EntityStore<EnglishWordMeaningId, CachedEnglishWordMeaning>,

    slovene_words: EntityStore<SloveneWordId, CachedSloveneWord>,

    slovene_word_meanings: EntityStore<SloveneWordMeaningId, CachedSloveneWordMeaning>,

    categories: EntityStore<CategoryId, CachedCategory>,

    // TODO make sure this is updated in the related functions
    // TODO make sure this is cached when doing the initial seeding
    translation_relationships:
        EntityStore<(EnglishWordMeaningId, SloveneWordMeaningId), CachedTranslationRelationship>,

    search_engine_event_sender: Option<Sender<SearchIndexModificationMessage>>,
}

impl EntityCache {
    pub fn new_empty() -> Self {
        Self {
            english_words: EntityStore::new(),
            english_word_meanings: EntityStore::new(),
            slovene_words: EntityStore::new(),
            slovene_word_meanings: EntityStore::new(),
            categories: EntityStore::new(),
            translation_relationships: EntityStore::new(),
            search_engine_event_sender: None,
        }
    }

    pub fn set_search_engine_event_sender(
        &mut self,
        sender: Sender<SearchIndexModificationMessage>,
    ) {
        self.search_engine_event_sender = Some(sender);
    }

    /*
     * Cache seeding (on startup)
     */

    pub fn clear_cache(&mut self) {
        self.english_words.clear();
        self.english_word_meanings.clear();
        self.slovene_words.clear();
        self.slovene_word_meanings.clear();
        self.categories.clear();
    }

    pub async fn clear_and_reseed_cache_from_database(
        &mut self,
        database_connection: &mut DatabaseConnection,
    ) -> Result<(), CacheSeedingError> {
        info!("Clearing cache before reseeding.");

        self.clear_cache();

        // Seeding is done in the following order:
        // - categories
        // - words (english and slovene),
        // - their meanings (english and slovene).

        // TODO fix this not seeding properly

        // Seed all categories.
        let mut categories = CategoryQuery::get_all_categories(database_connection).await;
        let mut num_categories_cached = 0u64;

        while let Some(category_result) = categories.next().await {
            let category = category_result?;

            self.categories.insert_or_replace(
                category.id,
                CachedCategory::new_without_relationships(category),
            );

            num_categories_cached += 1;
        }

        info!(
            "Finished caching {} categories.",
            num_categories_cached
        );

        drop(categories);


        // Seed english words, including their meanings.
        let mut pending_translation_relationships: HashSet<(
            SloveneWordMeaningId,
            EnglishWordMeaningId,
        )> = HashSet::new();


        let mut english_words_with_meanings_and_translations =
            EnglishWordQuery::get_all_english_words_with_meanings(
                database_connection,
                EnglishWordsQueryOptions::new_without_filters(),
            )
            .await;

        let mut num_english_words_cached = 0u64;
        let mut num_english_word_meanings_cached = 0u64;

        while let Some(english_word_result) =
            english_words_with_meanings_and_translations.next().await
        {
            let english_word_with_meanings = english_word_result?;
            let (english_word, meanings) =
                english_word_with_meanings.into_less_detailed_model_and_meanings();

            let english_word_id = english_word.id();

            let mut cached_english_word = CachedEnglishWord::new_without_meanings(english_word);

            num_english_words_cached += 1;

            for meaning_with_details in meanings {
                let (meaning, categories, translations) =
                    meaning_with_details.into_less_detailed_model_and_individual_details();

                let meaning_id = meaning.id();

                let mut cached_english_word_meaning =
                    CachedEnglishWordMeaning::new_without_relationships(
                        meaning.parent_word_id(),
                        meaning,
                    );

                num_english_word_meanings_cached += 1;


                for category_id in categories {
                    let existing_cached_category = self
                        .categories
                        .get_mut(&category_id)
                        .ok_or_else(|| CacheSeedingError::InvalidRelationship {
                            details: format!(
                                "english word meaning {} is linked to category {}, \
                                but that category does not exist in cache",
                                meaning_id, category_id
                            ),
                        })?;

                    existing_cached_category
                        .add_english_translation_reverse_relationship(meaning_id);

                    cached_english_word_meaning.add_category(category_id);
                }


                for translation in translations {
                    pending_translation_relationships
                        .insert((translation.word_meaning.id(), meaning_id));

                    self.translation_relationships.insert_or_replace(
                        (meaning_id, translation.word_meaning.id()),
                        CachedTranslationRelationship {
                            english_word_meaning_id: meaning_id,
                            slovene_word_meaning_id: translation.word_meaning.id(),
                            translated_at: translation.translated_at,
                            translated_by: translation.translated_by,
                        },
                    );
                }


                self.english_word_meanings
                    .insert_or_replace(meaning_id, cached_english_word_meaning);


                cached_english_word.add_meaning(meaning_id);
            }

            self.english_words
                .insert_or_replace(english_word_id, cached_english_word);
        }

        info!(
            "Finished caching {} english words and {} english word meanings.",
            num_english_words_cached, num_english_word_meanings_cached
        );

        drop(english_words_with_meanings_and_translations);


        // Seed slovene words, including their meanings.
        let mut slovene_words_with_meanings_and_translations =
            SloveneWordQuery::get_all_slovene_words_with_meanings(
                database_connection,
                SloveneWordsQueryOptions::new_without_filters(),
            )
            .await;

        let mut num_slovene_words_cached = 0u64;
        let mut num_slovene_word_meanings_cached = 0u64;

        while let Some(slovene_word_result) =
            slovene_words_with_meanings_and_translations.next().await
        {
            let slovene_word_with_meanings_and_translations = slovene_word_result?;
            let (slovene_word, meanings) =
                slovene_word_with_meanings_and_translations.into_less_detailed_model_and_meanings();

            let slovene_word_id = slovene_word.id();

            let mut cached_slovene_word = CachedSloveneWord::new_without_meanings(slovene_word);

            num_slovene_words_cached += 1;

            for meaning_with_details in meanings {
                let (meaning, categories, translations) =
                    meaning_with_details.into_less_detailed_model_and_individual_details();

                let meaning_id = meaning.id();

                let mut cached_slovene_word_meaning =
                    CachedSloveneWordMeaning::new_without_relationships(
                        meaning.parent_word_id(),
                        meaning,
                    );

                num_slovene_word_meanings_cached += 1;

                for category_id in categories {
                    let existing_cached_category = self
                        .categories
                        .get_mut(&category_id)
                        .ok_or_else(|| CacheSeedingError::InvalidRelationship {
                            details: format!(
                                "slovene word meaning {} is linked to category {}, \
                                but that category does not exist in cache",
                                meaning_id, category_id
                            ),
                        })?;

                    existing_cached_category
                        .add_slovene_translation_reverse_relationship(meaning_id);

                    cached_slovene_word_meaning.add_category(category_id);
                }

                for translation in translations {
                    assert!(pending_translation_relationships
                        .contains(&(meaning_id, translation.word_meaning.id())));

                    assert!(self
                        .translation_relationships
                        .contains(&(translation.word_meaning.id(), meaning_id)));
                }


                self.slovene_word_meanings
                    .insert_or_replace(meaning_id, cached_slovene_word_meaning);

                cached_slovene_word.add_meaning(meaning_id);
            }

            self.slovene_words
                .insert_or_replace(slovene_word_id, cached_slovene_word);
        }

        // DEBUGONLY
        println!(
            "Finished caching {} slovene words and {} slovene word meanings.",
            num_slovene_words_cached, num_slovene_word_meanings_cached
        );
        info!(
            "Finished caching {} slovene words and {} slovene word meanings.",
            num_slovene_words_cached, num_slovene_word_meanings_cached
        );

        drop(slovene_words_with_meanings_and_translations);


        let num_translation_relationships = pending_translation_relationships.len();

        for pending_translation_relationship in pending_translation_relationships {
            let (slovene_word_meaning_id, english_word_meaning_id) =
                pending_translation_relationship;

            let slovene_word_meaning = self.slovene_word_meanings.get_mut(&slovene_word_meaning_id)
                .ok_or_else(|| CacheSeedingError::InvalidRelationship { details: format!(
                    "unable to find slovene word meaning in cache for a given translation relationship: {} - {}",
                    slovene_word_meaning_id,
                    english_word_meaning_id
                ) })?;

            let english_word_meaning = self.english_word_meanings.get_mut(&english_word_meaning_id)
                .ok_or_else(|| CacheSeedingError::InvalidRelationship { details: format!(
                    "unable to find english word meaning in cache for a given translation relationship: {} - {}",
                    slovene_word_meaning_id,
                    english_word_meaning_id
                ) })?;


            slovene_word_meaning.add_translation(english_word_meaning_id);
            english_word_meaning.add_translation(slovene_word_meaning_id);
        }

        info!(
            "Finished caching {} translation relationships.",
            num_translation_relationships
        );


        // PANIC SAFETY: If `check_cache_validity` returns an error just after seeding,
        // that indicates we seeded improperly, so we should just stop and make a fuss at this point,
        // otherwise we'll run into cache errors down the line.
        if let Err(error) = self.check_cache_validity() {
            panic!(
                "failed to seed cache (seeded improperly): {}",
                error.reason
            );
        }

        info!("Cache validated.");


        self.emit_to_search_indexer_if_set(SearchIndexModificationMessage::PerformFullReindex);

        Ok(())
    }

    fn check_cache_validity(&self) -> Result<(), InvalidCacheError> {
        // Validate english words.
        for (_, cached_english_word) in self.english_words.iter() {
            // Check that all associated meanings exist in cache.
            for meaning_id in &cached_english_word.meaning_ids {
                if !self.english_word_meanings.contains(meaning_id) {
                    return Err(InvalidCacheError::new(format!(
                        "english word {} is supposed to have a meaning {}, \
                        but that meaning is not present in the cache",
                        cached_english_word.word.id(),
                        meaning_id
                    )));
                }
            }
        }


        // Validate slovene words.
        for (_, cached_slovene_word) in self.slovene_words.iter() {
            // Check that all associated meanings exist in cache.
            for meaning_id in &cached_slovene_word.meaning_ids {
                if !self.slovene_word_meanings.contains(meaning_id) {
                    return Err(InvalidCacheError::new(format!(
                        "slovene word {} is supposed to have a meaning {}, \
                        but that meaning is not present in the cache",
                        cached_slovene_word.word.id(),
                        meaning_id
                    )));
                }
            }
        }


        // Validate english word meanings.
        for (_, cached_english_word_meaning) in self.english_word_meanings.iter() {
            // Check that the parent word exists.
            if !self
                .english_words
                .contains(&cached_english_word_meaning.parent_word_id)
            {
                return Err(InvalidCacheError::new(format!(
                    "english word meaning {} is supposed to have a parent english word with ID {}, \
                    but that word is not present in the cache",
                    cached_english_word_meaning.word_meaning.id(),
                    cached_english_word_meaning.parent_word_id
                )));
            }

            // Check that associated categories exist.
            for linked_category_id in &cached_english_word_meaning.category_ids {
                if !self.categories.contains(linked_category_id) {
                    return Err(InvalidCacheError::new(format!(
                        "english word meaning {} is supposed to have a linked category with ID {}, \
                        but that category is not present in the cache",
                        cached_english_word_meaning.word_meaning.id(),
                        linked_category_id
                    )));
                }
            }

            // Check that translations exist.
            for translation_id in &cached_english_word_meaning.translation_ids {
                if !self.slovene_word_meanings.contains(translation_id) {
                    return Err(InvalidCacheError::new(format!(
                        "english word meaning {} is supposed to have a slovene translation with ID {}, \
                        but no such slovene word meaning by that ID is present in the cache",
                        cached_english_word_meaning.word_meaning.id(),
                        translation_id
                    )));
                }
            }
        }


        // Validate slovene word meanings.
        for (_, cached_slovene_word_meaning) in self.slovene_word_meanings.iter() {
            // Check that the parent word exists.
            if !self
                .slovene_words
                .contains(&cached_slovene_word_meaning.parent_word_id)
            {
                return Err(InvalidCacheError::new(format!(
                    "slovene word meaning {} is supposed to have a parent slovene word with ID {}, \
                    but that word is not present in the cache",
                    cached_slovene_word_meaning.word_meaning.id(),
                    cached_slovene_word_meaning.parent_word_id
                )));
            }

            // Check that associated categories exist.
            for linked_category_id in &cached_slovene_word_meaning.category_ids {
                if !self.categories.contains(linked_category_id) {
                    return Err(InvalidCacheError::new(format!(
                        "slovene word meaning {} is supposed to have a linked category with ID {}, \
                        but that category is not present in the cache",
                        cached_slovene_word_meaning.word_meaning.id(),
                        linked_category_id
                    )));
                }
            }

            // Check that translations exist.
            for translation_id in &cached_slovene_word_meaning.translation_ids {
                if !self.english_word_meanings.contains(translation_id) {
                    return Err(InvalidCacheError::new(format!(
                        "slovene word meaning {} is supposed to have an english translation with ID {}, \
                        but no such english word meaning by that ID is present in the cache",
                        cached_slovene_word_meaning.word_meaning.id(),
                        translation_id
                    )));
                }
            }
        }


        // Validate categories.
        for (_, cached_category) in self.categories.iter() {
            // Check that all reverse english word meaning relationships are valid.
            for english_word_meaning_id in &cached_category.present_on_english_word_meaning_ids {
                if !self.english_word_meanings.contains(english_word_meaning_id) {
                    return Err(InvalidCacheError::new(format!(
                        "category {} is supposed to be present on the english word meaning with ID {}, \
                        but that word meaning is not present in the cache",
                        cached_category.category.id,
                        english_word_meaning_id
                    )));
                }
            }

            // Check that all reverse slovene word meaning relationships are valid.
            for slovene_word_meaning_id in &cached_category.present_on_slovene_word_meaning_ids {
                if !self.slovene_word_meanings.contains(slovene_word_meaning_id) {
                    return Err(InvalidCacheError::new(format!(
                        "category {} is supposed to be present on the slovene word meaning with ID {}, \
                        but that word meaning is not present in the cache",
                        cached_category.category.id,
                        slovene_word_meaning_id
                    )));
                }
            }
        }


        Ok(())
    }
}

/// Search indexer emit methods.
impl EntityCache {
    fn emit_to_search_indexer_if_set(&self, message: SearchIndexModificationMessage) {
        if let Some(search_indexer_sender) = &self.search_engine_event_sender {
            debug!("Sending event to search indexer.");

            let send_result = search_indexer_sender.send(message);

            if let Err(send_error) = send_result {
                error!(
                    "failed to send message to search indexer: {}",
                    send_error
                );
            }
        }
    }
}


/// English word and word meaning-related methods.
impl EntityCache {
    /*
     * English word-related methods
     */

    // TODO event emitting

    /// This function inserts an english word without any meanings.
    /// If the english word is already present, it is updated,
    /// leaving linked meanings untouched.
    pub fn insert_or_update_english_word(&mut self, english_word: EnglishWordModel) {
        match self.english_words.entry(english_word.id()) {
            EntityStoreEntry::Vacant(vacant_store_entry) => {
                vacant_store_entry.insert(CachedEnglishWord::new_without_meanings(
                    english_word,
                ));

                // No need to emit anything to the search indexer here.
                // There cannot be any associated meanings yet,
                // we just created a new word without meanings.
            }
            EntityStoreEntry::Occupied(mut occupied_store_entry) => {
                let english_word_copy = english_word.clone();

                occupied_store_entry
                    .get_mut()
                    .update_with_model(english_word);

                let linked_meaning_ids = occupied_store_entry.get().meaning_ids.clone();

                for linked_meaning_id in linked_meaning_ids {
                    let Some(meaning) = self.english_word_meaning(&linked_meaning_id) else {
                        panic!(
                            "invalid cache state: english word meaning {} is linked on english word {}, but doesn't exist in cache",
                            linked_meaning_id,
                            english_word_copy.id(),
                        );
                    };

                    self.emit_to_search_indexer_if_set(
                        SearchIndexModificationMessage::EnglishWordMeaningCreatedOrUpdated {
                            english_word_meaning: meaning.word_meaning().to_owned(),
                            english_word: english_word_copy.clone(),
                        },
                    );
                }
            }
        };
    }

    /// This function removes the english word from the cache,
    /// including all of its meanings. If the english word is not present
    /// in the cache, `None` is returned.
    pub fn remove_english_word(
        &mut self,
        english_word_id: EnglishWordId,
    ) -> Option<EnglishWordModel> {
        let removed_cached_english_word = self.english_words.remove(&english_word_id)?;

        for english_word_meaning_id in &removed_cached_english_word.meaning_ids {
            self.emit_to_search_indexer_if_set(
                SearchIndexModificationMessage::EnglishWordMeaningRemoved {
                    english_word_meaning_id: *english_word_meaning_id,
                },
            );
        }

        self.remove_related_incoming_english_word_relationships(&removed_cached_english_word);

        Some(removed_cached_english_word.word)
    }

    pub fn english_word(&self, english_word_id: EnglishWordId) -> Option<&CachedEnglishWord> {
        self.english_words.get(&english_word_id)
    }

    pub fn english_words(&self) -> Vec<&CachedEnglishWord> {
        self.english_words
            .iter()
            .map(|(_key, value)| value)
            .collect()
    }


    /// Updates or removes the related entities based on the just-removed [`CachedEnglishWord`].
    ///
    /// Concretely, this means the following:
    /// - removing all english word meanings for the given english word from the cache.
    fn remove_related_incoming_english_word_relationships(
        &mut self,
        removed_cached_english_word: &CachedEnglishWord,
    ) {
        for word_meaning_id in &removed_cached_english_word.meaning_ids {
            self.remove_english_word_meaning(word_meaning_id)
                // PANIC SAFETY: If a removal fails (i.e. function returns `None`), that would indicate that we weren't diligent
                // with propagating out deleted meanings, which essentially places the cache into an undefined state.
                // We don't want that - we'd rather panic and fix it instead.
                .expect("failed to evict related english word meaning from cache");
        }
    }



    /*
     * English word meaning-related methods
     */

    /// This functions insert an english word meaning without any categories or translations.
    /// If the english word meaning is already present, it is updated,
    /// leaving categories and translations on it untouched.
    ///
    /// This method will return an error if the parent english is not is not present in the cache.
    pub fn insert_or_update_english_word_meaning(
        &mut self,
        english_word_meaning: EnglishWordMeaningModel,
    ) -> Result<(), WordMeaningEntityError> {
        let english_word_meaning_copy = english_word_meaning.clone();

        match self.english_word_meanings.entry(english_word_meaning.id()) {
            EntityStoreEntry::Vacant(vacant_store_entry) => {
                let parent_english_word = self
                    .english_words
                    .get_mut(&english_word_meaning.parent_word_id())
                    .ok_or(WordMeaningEntityError::ParentEntityNotFound)?;

                parent_english_word.add_meaning(english_word_meaning.id());

                vacant_store_entry.insert(
                    CachedEnglishWordMeaning::new_without_relationships(
                        english_word_meaning.parent_word_id(),
                        english_word_meaning,
                    ),
                );
            }
            EntityStoreEntry::Occupied(mut occupied_store_entry) => {
                occupied_store_entry
                    .get_mut()
                    .update_with_model(english_word_meaning);
            }
        }


        let Some(parent_english_word) =
            self.english_word(english_word_meaning_copy.parent_word_id())
        else {
            panic!(
                "invalid cache state: english word meaning {} has parent english word {}, but the parent doesn't exist in cache",
                english_word_meaning_copy.id(),
                english_word_meaning_copy.parent_word_id(),
            );
        };

        self.emit_to_search_indexer_if_set(
            SearchIndexModificationMessage::EnglishWordMeaningCreatedOrUpdated {
                english_word_meaning: english_word_meaning_copy,
                english_word: parent_english_word.word().to_owned(),
            },
        );

        Ok(())
    }

    pub fn english_word_meaning(
        &self,
        english_word_meaning_id: &EnglishWordMeaningId,
    ) -> Option<&CachedEnglishWordMeaning> {
        self.english_word_meanings.get(english_word_meaning_id)
    }

    pub fn english_word_meanings(&self) -> Vec<&CachedEnglishWordMeaning> {
        self.english_word_meanings
            .iter()
            .map(|(_key, value)| value)
            .collect()
    }

    pub fn meanings_of_english_word(
        &self,
        english_word_id: EnglishWordId,
    ) -> Result<Option<Vec<&CachedEnglishWordMeaning>>, EnglishWordMeaningsLookupError> {
        let Some(english_word) = self.english_word(english_word_id) else {
            return Ok(None);
        };

        let mut meanings = Vec::with_capacity(english_word.meaning_ids.len());
        for meaning_id in &english_word.meaning_ids {
            let Some(meaning) = self.english_word_meaning(meaning_id) else {
                return Err(
                    EnglishWordMeaningsLookupError::WordMeaningNotFoundInCache {
                        word_meaning_id: *meaning_id,
                    },
                );
            };

            meanings.push(meaning);
        }

        Ok(Some(meanings))
    }

    /// Removes the english word meaning from the cache, including
    /// unlinking itself from all of the slovene word meanings it is a translation of.
    /// If the english word meaning is not present in the cache, `None` is returned.
    pub fn remove_english_word_meaning(
        &mut self,
        english_word_meaning_id: &EnglishWordMeaningId,
    ) -> Option<EnglishWordMeaningModel> {
        let removed_english_word_meaning =
            self.english_word_meanings.remove(english_word_meaning_id)?;


        self.emit_to_search_indexer_if_set(
            SearchIndexModificationMessage::EnglishWordMeaningRemoved {
                english_word_meaning_id: removed_english_word_meaning.word_meaning().id(),
            },
        );

        self.remove_related_english_word_meaning_relationships(&removed_english_word_meaning);

        Some(removed_english_word_meaning.word_meaning)
    }


    fn remove_related_english_word_meaning_relationships(
        &mut self,
        removed_english_word_meaning: &CachedEnglishWordMeaning,
    ) {
        let parent_english_word_id = removed_english_word_meaning.word_meaning.parent_word_id();
        let english_word_meaning_id = removed_english_word_meaning.word_meaning.id();

        let parent_english_word = self
            .english_words
            .get_mut(&parent_english_word_id)
            // PANIC SAFETY: This should never panic, because we should be diligently 
            // updating these relationships internally. If we aren't, this indicates a bug in the code 
            // and this will catch it.
            .expect("failed to obtain parent english word for a just-removed english word meaning");

        parent_english_word.remove_meaning(&english_word_meaning_id);


        for translation_meaning_key in &removed_english_word_meaning.translation_ids {
            let slovene_word_meaning = self.slovene_word_meanings.get_mut(translation_meaning_key)
                // PANIC SAFETY: This should always succeed, because we should be diligently updating
                // these relationships. If we aren't, this indicates a bug in the code and this will catch it.
                .expect("failed to obtain slovene word meaning that is linked as a translation");

            let translation_unlink_success =
                slovene_word_meaning.remove_translation(&english_word_meaning_id);

            // PANIC SAFETY: This should never panic (removing a translation relationship should always succeed),
            // because we should be diligently updating these relationships.
            // If we aren't, this indicates a bug in the code and this will catch it.
            assert!(
                translation_unlink_success,
                "failed to unlink english word meaning as a translation on a slovene word meaning"
            );
        }


        for category_id in &removed_english_word_meaning.category_ids {
            let category = self.categories.get_mut(category_id)
                // PANIC SAFETY: This should always succeed, because we should be diligently updating
                // these relationships. If we aren't, this indicates a bug in the code and this will catch it.
                .expect("failed to obtain category that is linked to a just-removed english word meaning");

            let reverse_category_relationship_unlink_success =
                category.remove_english_translation_reverse_relationship(&english_word_meaning_id);


            // PANIC SAFETY: This should never panic (removing a reverse category relationship should always succeed),
            // because we should be diligently updating these relationships.
            // If we aren't, this indicates a bug in the code and this will catch it.
            assert!(
                reverse_category_relationship_unlink_success,
                "failed to unlink english word meaning from a category's reverse relationship"
            );
        }
    }
}


/// Slovene word and word meaning-related methods.
impl EntityCache {
    /*
     * Slovene word-related methods
     */

    pub fn insert_or_update_slovene_word(&mut self, slovene_word: SloveneWordModel) {
        match self.slovene_words.entry(slovene_word.id()) {
            EntityStoreEntry::Vacant(vacant_store_entry) => {
                vacant_store_entry.insert(CachedSloveneWord::new_without_meanings(
                    slovene_word,
                ));

                // No need to emit anything to the search indexer here.
                // There cannot be any associated meanings yet,
                // we just created a new word without meanings.
            }
            EntityStoreEntry::Occupied(mut occupied_store_entry) => {
                let slovene_word_copy = slovene_word.clone();

                occupied_store_entry
                    .get_mut()
                    .update_with_model(slovene_word);

                let linked_meaning_ids = occupied_store_entry.get().meaning_ids.clone();

                for linked_meaning_id in linked_meaning_ids {
                    let Some(meaning) = self.slovene_word_meaning(&linked_meaning_id) else {
                        panic!(
                            "invalid cache state: slovene word meaning {} is linked on slovene word {}, but doesn't exist in cache",
                            linked_meaning_id,
                            slovene_word_copy.id(),
                        );
                    };

                    self.emit_to_search_indexer_if_set(
                        SearchIndexModificationMessage::SloveneWordMeaningCreatedOrUpdated {
                            slovene_word_meaning: meaning.word_meaning().to_owned(),
                            slovene_word: slovene_word_copy.clone(),
                        },
                    );
                }
            }
        }
    }

    pub fn remove_slovene_word(
        &mut self,
        slovene_word_id: SloveneWordId,
    ) -> Option<SloveneWordModel> {
        let removed_slovene_word = self.slovene_words.remove(&slovene_word_id)?;

        for slovene_word_meaning_id in &removed_slovene_word.meaning_ids {
            self.emit_to_search_indexer_if_set(
                SearchIndexModificationMessage::SloveneWordMeaningRemoved {
                    slovene_word_meaning_id: *slovene_word_meaning_id,
                },
            );
        }

        self.remove_related_slovene_word_relationships(&removed_slovene_word);

        Some(removed_slovene_word.word)
    }

    pub fn slovene_word(&self, slovene_word_id: SloveneWordId) -> Option<&CachedSloveneWord> {
        self.slovene_words.get(&slovene_word_id)
    }

    pub fn slovene_words(&self) -> Vec<&CachedSloveneWord> {
        self.slovene_words
            .iter()
            .map(|(_key, value)| value)
            .collect()
    }


    fn remove_related_slovene_word_relationships(
        &mut self,
        removed_cached_slovene_word: &CachedSloveneWord,
    ) {
        for word_meaning_id in &removed_cached_slovene_word.meaning_ids {
            self.remove_slovene_word_meaning(word_meaning_id)
                // PANIC SAFETY: If a removal fails (i.e. function returns `None`), that would indicate that we weren't diligent
                // with propagating out deleted meanings, which essentially places the cache into an undefined state.
                // We don't want that - we'd rather panic and fix it instead.
                .expect("failed to evict related slovene word meaning from cache");
        }
    }



    /*
     * Slovene word meaning-related methods
     */

    pub fn insert_or_update_slovene_word_meaning(
        &mut self,
        slovene_word_meaning: SloveneWordMeaningModel,
    ) -> Result<(), WordMeaningEntityError> {
        let slovene_word_meaning_copy = slovene_word_meaning.clone();

        match self.slovene_word_meanings.entry(slovene_word_meaning.id()) {
            EntityStoreEntry::Vacant(vacant_store_entry) => {
                let parent_slovene_word = self
                    .slovene_words
                    .get_mut(&slovene_word_meaning.parent_word_id())
                    .ok_or(WordMeaningEntityError::ParentEntityNotFound)?;

                parent_slovene_word.add_meaning(slovene_word_meaning.id());

                vacant_store_entry.insert(
                    CachedSloveneWordMeaning::new_without_relationships(
                        slovene_word_meaning.parent_word_id(),
                        slovene_word_meaning,
                    ),
                );
            }
            EntityStoreEntry::Occupied(mut occupied_store_entry) => {
                occupied_store_entry
                    .get_mut()
                    .update_with_model(slovene_word_meaning);
            }
        }

        let Some(parent_slovene_word) =
            self.slovene_word(slovene_word_meaning_copy.parent_word_id())
        else {
            panic!(
                "invalid cache state: slovene word meaning {} has parent slovene word {}, but the parent doesn't exist in cache",
                slovene_word_meaning_copy.id(),
                slovene_word_meaning_copy.parent_word_id(),
            );
        };

        self.emit_to_search_indexer_if_set(
            SearchIndexModificationMessage::SloveneWordMeaningCreatedOrUpdated {
                slovene_word_meaning: slovene_word_meaning_copy,
                slovene_word: parent_slovene_word.word().to_owned(),
            },
        );

        Ok(())
    }

    pub fn slovene_word_meaning(
        &self,
        slovene_word_meaning_id: &SloveneWordMeaningId,
    ) -> Option<&CachedSloveneWordMeaning> {
        self.slovene_word_meanings.get(slovene_word_meaning_id)
    }

    pub fn slovene_word_meanings(&self) -> Vec<&CachedSloveneWordMeaning> {
        self.slovene_word_meanings
            .iter()
            .map(|(_key, value)| value)
            .collect()
    }

    pub fn remove_slovene_word_meaning(
        &mut self,
        slovene_word_meaning_id: &SloveneWordMeaningId,
    ) -> Option<SloveneWordMeaningModel> {
        let removed_slovene_word_meaning =
            self.slovene_word_meanings.remove(slovene_word_meaning_id)?;

        self.emit_to_search_indexer_if_set(
            SearchIndexModificationMessage::SloveneWordMeaningRemoved {
                slovene_word_meaning_id: removed_slovene_word_meaning.word_meaning().id(),
            },
        );

        self.remove_related_slovene_word_meaning_relationships(&removed_slovene_word_meaning);

        Some(removed_slovene_word_meaning.word_meaning)
    }


    fn remove_related_slovene_word_meaning_relationships(
        &mut self,
        removed_slovene_word_meaning: &CachedSloveneWordMeaning,
    ) {
        let parent_slovene_word_id = removed_slovene_word_meaning.word_meaning.parent_word_id();
        let slovene_word_meaning_id = removed_slovene_word_meaning.word_meaning.id();


        let parent_slovene_word = self.slovene_words
            .get_mut(&parent_slovene_word_id)
            // PANIC SAFETY: This should never panic, because we should be diligently 
            // updating these relationships internally. If we aren't, this indicates a bug in the code 
            // and this will catch it.
            .expect("failed to obtain parent slovene word for a just-removed english word meaning");

        parent_slovene_word.remove_meaning(&slovene_word_meaning_id);


        for translation_meaning_id in &removed_slovene_word_meaning.translation_ids {
            let english_word_meaning = self.english_word_meanings.get_mut(translation_meaning_id)
                // PANIC SAFETY: This should always succeed, because we should be diligently updating
                // these relationships. If we aren't, this indicates a bug in the code and this will catch it.
                .expect("failed to obtain english word meaning that is linked as a translation");

            let translation_unlink_success =
                english_word_meaning.remove_translation(&slovene_word_meaning_id);

            // PANIC SAFETY: This should never panic (removing a translation relationship should always succeed),
            // because we should be diligently updating these relationships.
            // If we aren't, this indicates a bug in the code and this will catch it.
            assert!(
                translation_unlink_success,
                "failed to unlink slovene word meaning as a translation on an english word meaning"
            );
        }

        for category_id in &removed_slovene_word_meaning.category_ids {
            let category = self.categories.get_mut(category_id)
                // PANIC SAFETY: This should always succeed, because we should be diligently updating
                // these relationships. If we aren't, this indicates a bug in the code and this will catch it.
                .expect("failed to obtain category that is linked to a just-removed slovene word meaning");

            let reverse_category_relationship_unlink_success =
                category.remove_slovene_translation_reverse_relationship(&slovene_word_meaning_id);


            // PANIC SAFETY: This should never panic (removing a reverse category relationship should always succeed),
            // because we should be diligently updating these relationships.
            // If we aren't, this indicates a bug in the code and this will catch it.
            assert!(
                reverse_category_relationship_unlink_success,
                "failed to unlink slovene word meaning from a category's reverse relationship"
            );
        }
    }
}


/// Translation-related methods.
impl EntityCache {
    /*
     * Translation linking and unlinking methods
     */

    /// Establishes a translation relationship between the given slovene and english word meaning.
    ///
    /// If either of the IDs provided (`slovene_word_meaning_id` and `english_word_meaning_id`) refer
    /// to meanings that aren't present in cache, this method will return an error.
    ///
    /// If the translation relationship is already present, this call will update the `translated_at` and `translated_by` properties.
    pub fn link_slovene_and_english_word_meanings_as_translations(
        &mut self,
        slovene_word_meaning_id: SloveneWordMeaningId,
        english_word_meaning_id: EnglishWordMeaningId,
        translated_at: DateTime<Utc>,
        translated_by: Option<UserId>,
    ) -> Result<(), EntityReadError> {
        let optional_slovene_word_meaning =
            self.slovene_word_meanings.get_mut(&slovene_word_meaning_id);

        let optional_english_word_meaning =
            self.english_word_meanings.get_mut(&english_word_meaning_id);


        let (slovene_word_meaning, english_word_meaning) = match (
            optional_slovene_word_meaning,
            optional_english_word_meaning,
        ) {
            (Some(slovene_word_meaning), Some(english_word_meaning)) => {
                (slovene_word_meaning, english_word_meaning)
            }
            _ => {
                return Err(EntityReadError::EntityNotFound);
            }
        };


        if let Some(existing_translation_relationship) = self
            .translation_relationships
            .get_mut(&(english_word_meaning_id, slovene_word_meaning_id))
        {
            existing_translation_relationship.translated_at = translated_at;
            existing_translation_relationship.translated_by = translated_by;

            return Ok(());
        };


        let english_translation_add_result =
            slovene_word_meaning.add_translation(english_word_meaning_id);

        // PANIC SAFETY: If adding fails, that would indicate we weren't diligent
        // with propagating our deleted translation relationships and such, which essentially
        // places the cache into an undefined state. We don't want that - we'd rather panic and fix it instead.
        assert!(
            english_translation_add_result,
            "failed to insert english translation link into slovene word meaning"
        );


        let slovene_translation_add_result =
            english_word_meaning.add_translation(slovene_word_meaning_id);

        // PANIC SAFETY: If adding fails, that would indicate we weren't diligent
        // with propagating our deleted translation relationships and such, which essentially
        // places the cache into an undefined state. We don't want that - we'd rather panic and fix it instead.
        assert!(
            slovene_translation_add_result,
            "failed to insert slovene translation link into english word meaning"
        );


        let relationship_insertion_result = self.translation_relationships.insert_or_replace(
            (english_word_meaning_id, slovene_word_meaning_id),
            CachedTranslationRelationship {
                english_word_meaning_id,
                slovene_word_meaning_id,
                translated_at,
                translated_by,
            },
        );

        // PANIC SAFETY: If the relationship existed before, the `translation_relationships.get_mut` call above would
        // be entered and return early. This is just a sanity check that things are proceeding as expected.
        assert!(matches!(
            relationship_insertion_result,
            EntityStoreInsertionAction::Inserted
        ));


        Ok(())
    }

    pub fn translations_for_english_word_meaning<'a>(
        &'a self,
        english_word_meaning_id: EnglishWordMeaningId,
    ) -> Option<Vec<&'a CachedTranslationRelationship>> {
        let cached_english_word_meaning = self.english_word_meaning(&english_word_meaning_id)?;

        let mut translation_relationships: Vec<&'a CachedTranslationRelationship> =
            Vec::with_capacity(cached_english_word_meaning.translation_ids.len());

        for slovene_word_meaning_id in &cached_english_word_meaning.translation_ids {
            let Some(translation_relationship) = self
                .translation_relationships
                .get(&(english_word_meaning_id, *slovene_word_meaning_id))
            else {
                error!(
                    "Failed to look up translation relationship with {} for english word meaning {}.",
                    slovene_word_meaning_id,
                    english_word_meaning_id
                );

                continue;
            };

            translation_relationships.push(translation_relationship);
        }

        Some(translation_relationships)
    }

    pub fn translations_for_slovene_word_meaning<'a>(
        &'a self,
        slovene_word_meaning_id: SloveneWordMeaningId,
    ) -> Option<Vec<&'a CachedTranslationRelationship>> {
        let cached_slovene_word_meaning = self.slovene_word_meaning(&slovene_word_meaning_id)?;

        let mut translation_relationships: Vec<&'a CachedTranslationRelationship> =
            Vec::with_capacity(cached_slovene_word_meaning.translation_ids.len());

        for english_word_meaning_id in &cached_slovene_word_meaning.translation_ids {
            let Some(translation_relationship) = self
                .translation_relationships
                .get(&(*english_word_meaning_id, slovene_word_meaning_id))
            else {
                error!(
                    "Failed to look up translation relationship with {} for slovene word meaning {}.",
                    english_word_meaning_id,
                    slovene_word_meaning_id
                );

                continue;
            };

            translation_relationships.push(translation_relationship);
        }

        Some(translation_relationships)
    }

    pub fn translation_by_id(
        &self,
        english_word_meaning_id: EnglishWordMeaningId,
        slovene_word_meaning_id: SloveneWordMeaningId,
    ) -> Option<&CachedTranslationRelationship> {
        self.translation_relationships
            .get(&(english_word_meaning_id, slovene_word_meaning_id))
    }

    /// Removes a translation relationship between the given slovene and english word meaning.
    ///
    /// If either (or both) if the IDs provided (`slovene_word_meaning_id` and `english_word_meaning_id`) refer
    /// to meanings that aren't present in cache, this method will return an error.
    ///
    /// If the translation relationship does not exist, this call will have no effect, and `Ok(())` will be returned.
    pub fn unlink_slovene_and_english_word_meanings_as_translations(
        &mut self,
        slovene_word_meaning_id: SloveneWordMeaningId,
        english_word_meaning_id: EnglishWordMeaningId,
    ) -> Result<(), EntityReadError> {
        let optional_slovene_word_meaning =
            self.slovene_word_meanings.get_mut(&slovene_word_meaning_id);

        let optional_english_word_meaning =
            self.english_word_meanings.get_mut(&english_word_meaning_id);


        let (slovene_word_meaning, english_word_meaning) = match (
            optional_slovene_word_meaning,
            optional_english_word_meaning,
        ) {
            (Some(slovene_word_meaning), Some(english_word_meaning)) => {
                (slovene_word_meaning, english_word_meaning)
            }
            _ => {
                return Err(EntityReadError::EntityNotFound);
            }
        };


        let Some(_) = self
            .translation_relationships
            .remove(&(english_word_meaning_id, slovene_word_meaning_id))
        else {
            return Err(EntityReadError::EntityNotFound);
        };


        let english_translation_removal_result =
            slovene_word_meaning.remove_translation(&english_word_meaning_id);

        // PANIC SAFETY: If a removal fails, that would indicate we weren't diligent
        // with propagating our deleted translation relationships and such, which essentially
        // places the cache into an undefined state. We don't want that - we'd rather panic and fix it instead.
        assert!(
            english_translation_removal_result,
            "failed to remove english translation link from slovene word meaning"
        );


        let slovene_translation_removal_result =
            english_word_meaning.remove_translation(&slovene_word_meaning_id);

        // PANIC SAFETY: If a removal fails, that would indicate we weren't diligent
        // with propagating our deleted translation relationships and such, which essentially
        // places the cache into an undefined state. We don't want that - we'd rather panic and fix it instead.
        assert!(
            slovene_translation_removal_result,
            "failed to remove slovene translation link from english word meaning"
        );


        Ok(())
    }
}


/// Category-related methods.
impl EntityCache {
    /*
     * Category model-related methods
     */

    pub fn insert_or_update_category(&mut self, category: CategoryModel) {
        match self.categories.entry(category.id) {
            EntityStoreEntry::Vacant(vacant_store_entry) => {
                vacant_store_entry.insert(CachedCategory::new_without_relationships(
                    category,
                ));
            }
            EntityStoreEntry::Occupied(mut occupied_store_entry) => {
                occupied_store_entry.get_mut().update_with_model(category);
            }
        }
    }

    pub fn remove_category(&mut self, category_id: CategoryId) -> Option<CategoryModel> {
        let removed_category = self.categories.remove(&category_id)?;

        self.remove_related_category_relationships(&removed_category);

        Some(removed_category.category)
    }

    pub fn category(&self, category_id: CategoryId) -> Option<&CachedCategory> {
        self.categories.get(&category_id)
    }


    fn remove_related_category_relationships(&mut self, removed_cached_category: &CachedCategory) {
        let category_id = removed_cached_category.category.id;

        for english_word_meaning_id in &removed_cached_category.present_on_english_word_meaning_ids {
            let english_word_meaning = self.english_word_meanings.get_mut(english_word_meaning_id)
                // PANIC SAFETY: This should always succeed, because we should be diligently updating
                // these relationships. If we aren't, this indicates a bug in the code and this will catch it.
                .expect("failed to obtain english word meaning that is reverse linked to a category");


            let category_link_removal_result = english_word_meaning.remove_category(&category_id);

            // PANIC SAFETY: This should always succeed (the category should always be present on the word meaning prior to removal),
            // because we should be diligently updating these relationships.
            // If we aren't, this indicates a bug in the code and this will catch it.
            assert!(
                category_link_removal_result,
                "failed to unlink category from an english word meaning"
            );
        }

        for slovene_word_meaning_id in &removed_cached_category.present_on_slovene_word_meaning_ids {
            let slovene_word_meaning = self.slovene_word_meanings.get_mut(slovene_word_meaning_id)
                // PANIC SAFETY: This should always succeed, because we should be diligently updating
                // these relationships. If we aren't, this indicates a bug in the code and this will catch it.
                .expect("failed to obtain slovene word meaning that is reverse linked to a category");


            let category_link_removal_result = slovene_word_meaning.remove_category(&category_id);

            // PANIC SAFETY: This should always succeed (the category should always be present on the word meaning prior to removal),
            // because we should be diligently updating these relationships.
            // If we aren't, this indicates a bug in the code and this will catch it.
            assert!(
                category_link_removal_result,
                "failed to unlink category from a slovene word meaning"
            );
        }
    }


    pub fn link_category_to_english_word_meaning(
        &mut self,
        english_word_meaning_id: EnglishWordMeaningId,
        category_id: CategoryId,
    ) -> Result<(), EnglishCategoryError> {
        let english_word_meaning = self
            .english_word_meanings
            .get_mut(&english_word_meaning_id)
            .ok_or(EnglishCategoryError::SloveneWordMeaningNotFound)?;

        let category = self
            .categories
            .get_mut(&category_id)
            .ok_or(EnglishCategoryError::CategoryNotFound)?;


        english_word_meaning.add_category(category_id);
        category.add_english_translation_reverse_relationship(english_word_meaning_id);

        Ok(())
    }

    pub fn unlink_category_from_english_word_meaning(
        &mut self,
        english_word_meaning_id: &EnglishWordMeaningId,
        category_id: &CategoryId,
    ) -> Result<(), EnglishCategoryError> {
        let english_word_meaning = self
            .english_word_meanings
            .get_mut(english_word_meaning_id)
            .ok_or(EnglishCategoryError::SloveneWordMeaningNotFound)?;

        let category = self
            .categories
            .get_mut(category_id)
            .ok_or(EnglishCategoryError::CategoryNotFound)?;


        english_word_meaning.remove_category(category_id);
        category.remove_english_translation_reverse_relationship(english_word_meaning_id);

        Ok(())
    }

    pub fn link_category_to_slovene_word_meaning(
        &mut self,
        slovene_word_meaning_id: SloveneWordMeaningId,
        category_id: CategoryId,
    ) -> Result<(), SloveneCategoryError> {
        let slovene_word_meaning = self
            .slovene_word_meanings
            .get_mut(&slovene_word_meaning_id)
            .ok_or(SloveneCategoryError::EnglishWordMeaningNotFound)?;

        let category = self
            .categories
            .get_mut(&category_id)
            .ok_or(SloveneCategoryError::CategoryNotFound)?;


        slovene_word_meaning.add_category(category_id);
        category.add_slovene_translation_reverse_relationship(slovene_word_meaning_id);

        Ok(())
    }

    pub fn unlink_category_from_slovene_word_meaning(
        &mut self,
        slovene_word_meaning_id: &SloveneWordMeaningId,
        category_id: &CategoryId,
    ) -> Result<(), SloveneCategoryError> {
        let english_word_meaning = self
            .slovene_word_meanings
            .get_mut(slovene_word_meaning_id)
            .ok_or(SloveneCategoryError::EnglishWordMeaningNotFound)?;

        let category = self
            .categories
            .get_mut(category_id)
            .ok_or(SloveneCategoryError::CategoryNotFound)?;


        english_word_meaning.remove_category(category_id);
        category.remove_slovene_translation_reverse_relationship(slovene_word_meaning_id);

        Ok(())
    }
}
