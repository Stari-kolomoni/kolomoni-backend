use entities::{
    CachedCategory,
    CachedEnglishWord,
    CachedEnglishWordMeaning,
    CachedSloveneWord,
    CachedSloveneWordMeaning,
    CategoryCacheKey,
    EnglishWordCacheKey,
    EnglishWordMeaningCacheKey,
    SloveneWordCacheKey,
    SloveneWordMeaningCacheKey,
};
use kolomoni_core::ids::{
    CategoryId,
    EnglishWordId,
    EnglishWordMeaningId,
    SloveneWordId,
    SloveneWordMeaningId,
};
use kolomoni_database::{
    entities::{
        category::CategoryModel,
        word_english::EnglishWordModel,
        word_meaning_english::EnglishWordMeaningModel,
        word_meaning_slovene::SloveneWordMeaningModel,
        word_slovene::SloveneWordModel,
    },
    DatabaseConnection,
};
use store::{EntityStore, EntityStoreEntry};
use thiserror::Error;


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
    DatabaseError { error: sqlx::Error },
}



pub struct KolomoniEntityCache {
    english_words: EntityStore<EnglishWordId, EnglishWordCacheKey, CachedEnglishWord>,

    english_word_meanings:
        EntityStore<EnglishWordMeaningId, EnglishWordMeaningCacheKey, CachedEnglishWordMeaning>,

    slovene_words: EntityStore<SloveneWordId, SloveneWordCacheKey, CachedSloveneWord>,

    slovene_word_meanings:
        EntityStore<SloveneWordMeaningId, SloveneWordMeaningCacheKey, CachedSloveneWordMeaning>,

    categories: EntityStore<CategoryId, CategoryCacheKey, CachedCategory>,
}

impl KolomoniEntityCache {
    pub fn new_empty() -> Self {
        Self {
            english_words: EntityStore::new(),
            english_word_meanings: EntityStore::new(),
            slovene_words: EntityStore::new(),
            slovene_word_meanings: EntityStore::new(),
            categories: EntityStore::new(),
        }
    }

    /*
     * Cache seeding (on startup)
     */

    pub fn clear(&mut self) {
        self.english_words.clear();
        self.english_word_meanings.clear();
        self.slovene_words.clear();
        self.slovene_word_meanings.clear();
        self.categories.clear();
    }

    pub fn clear_and_reseed_cache_from_database(
        &mut self,
        database_connection: &mut DatabaseConnection,
    ) {
        self.clear();

        // Seeding is done in the following order:
        // - categories
        // - words (english and slovene),
        // - their meanings (english and slovene).

        // TODO hmm, but wait, how should we handle cycles? we need some sort of a weak handle (maybe we could just use IDs instead of slotmap keys??)

        // TODO
        todo!();
    }


    /*
     * English word-related methods
     */

    /// This function inserts an english word without any meanings,
    /// or updates the english word if it is already present,
    /// leaving linked meanings untouched.
    pub fn insert_or_update_english_word(
        &mut self,
        english_word: EnglishWordModel,
    ) -> EnglishWordCacheKey {
        match self.english_words.entry(english_word.id()) {
            EntityStoreEntry::Vacant(vacant_store_entry) => vacant_store_entry.insert(
                CachedEnglishWord::new_without_meanings(english_word),
            ),
            EntityStoreEntry::Occupied(mut occupied_store_entry) => {
                occupied_store_entry
                    .get_mut()
                    .update_with_model(english_word);

                occupied_store_entry.key()
            }
        }
    }

    /// This function removes the english word from the cache,
    /// including all of its meanings. If the english word is not present
    /// in the cache, `None` is returned.
    pub fn remove_english_word(
        &mut self,
        english_word_id: EnglishWordId,
    ) -> Option<EnglishWordModel> {
        let english_word_cache_key = self.english_words.get_key(&english_word_id)?;

        let removed_english_word = self.remove_english_word_by_cache_key(
            english_word_cache_key
        )
            // PANIC SAFETY: This can never panic, because the function can only return `None` if 
            // the provided cache key is invalid, but we just obtained the cache key above.
            .expect("failed to remove english word from cache");

        Some(removed_english_word)
    }

    pub fn remove_english_word_by_cache_key(
        &mut self,
        english_word_cache_key: EnglishWordCacheKey,
    ) -> Option<EnglishWordModel> {
        let removed_english_word = self.english_words.remove_by_key(english_word_cache_key)?;

        self.remove_related_english_word_relationships(&removed_english_word);

        Some(removed_english_word.word)
    }

    pub fn english_word(&self, english_word_id: EnglishWordId) -> Option<&CachedEnglishWord> {
        self.english_words.get(&english_word_id)
    }


    fn remove_related_english_word_relationships(
        &mut self,
        removed_cached_english_word: &CachedEnglishWord,
    ) {
        for word_meaning_key in &removed_cached_english_word.meanings {
            self.remove_english_word_meaning_by_cache_key(*word_meaning_key)
                // PANIC SAFETY: If a removal fails (i.e. function returns `None`), that would indicate that we weren't diligent
                // with propagating out deleted meanings, which essentially places the cache into an undefined state.
                // We don't want that - we'd rather panic and fix it instead.
                .expect("failed to evict related english word meaning from cache");
        }
    }



    /*
     * English word meaning-related methods
     */

    /// This functions insert an english word meaning without any categories or translations,
    /// or updates an existing english word meaning if it is already present
    /// (leaving categories and translations on it untouched).
    ///
    /// This method will return an error if the parent english is not is not present in the cache.
    pub fn insert_or_update_english_word_meaning(
        &mut self,
        english_word_meaning: EnglishWordMeaningModel,
    ) -> Result<EnglishWordMeaningCacheKey, WordMeaningEntityError> {
        match self
            .english_word_meanings
            .entry(english_word_meaning.word_meaning_id())
        {
            EntityStoreEntry::Vacant(vacant_store_entry) => {
                let parent_word_slotmap_key = self
                    .english_words
                    .get_key(&english_word_meaning.word_id())
                    .ok_or(WordMeaningEntityError::ParentEntityNotFound)?;

                Ok(vacant_store_entry.insert(
                    CachedEnglishWordMeaning::new_without_relationships(
                        parent_word_slotmap_key,
                        english_word_meaning,
                    ),
                ))
            }
            EntityStoreEntry::Occupied(mut occupied_store_entry) => {
                occupied_store_entry
                    .get_mut()
                    .update_with_model(english_word_meaning);

                Ok(occupied_store_entry.key())
            }
        }
    }

    /// Removes the english word meaning from the cache, including
    /// unlinking itself from all of the slovene word meanings it is a translation of.
    /// If the english word meaning is not present in the cache, `None` is returned.
    pub fn remove_english_word_meaning(
        &mut self,
        english_word_meaning_id: &EnglishWordMeaningId,
    ) -> Option<EnglishWordMeaningModel> {
        let english_word_meaning_cache_key = self
            .english_word_meanings
            .get_key(english_word_meaning_id)?;

        let removed_english_word_meaning = self.remove_english_word_meaning_by_cache_key(english_word_meaning_cache_key)
            // PANIC SAFETY: This can never panic, because the function can only return `None` if 
            // the provided cache key is invalid, but we just obtained the cache key above.
            .expect("failed to remove english word meaning from cache");

        Some(removed_english_word_meaning)
    }

    fn remove_english_word_meaning_by_cache_key(
        &mut self,
        english_word_meaning_cache_key: EnglishWordMeaningCacheKey,
    ) -> Option<EnglishWordMeaningModel> {
        let removed_english_word_meaning = self
            .english_word_meanings
            .remove_by_key(english_word_meaning_cache_key)?;

        self.remove_related_english_word_meaning_relationships(
            english_word_meaning_cache_key,
            &removed_english_word_meaning,
        );

        Some(removed_english_word_meaning.word_meaning)
    }


    fn remove_related_english_word_meaning_relationships(
        &mut self,
        removed_english_word_meaning_cache_key: EnglishWordMeaningCacheKey,
        removed_english_word_meaning: &CachedEnglishWordMeaning,
    ) {
        for translation_meaning_key in &removed_english_word_meaning.translations {
            let slovene_word_meaning = self.slovene_word_meanings.get_mut_by_key(*translation_meaning_key)
                // PANIC SAFETY: This should always succeed, because we should be diligently updating
                // these relationships. If we aren't, this indicates a bug in the code and this will catch it.
                .expect("failed to obtain slovene word meaning that is linked as a translation");

            let translation_unlink_success =
                slovene_word_meaning.remove_translation(&removed_english_word_meaning_cache_key);

            // PANIC SAFETY: This should never panic (removing a translation relationship should always succeed),
            // because we should be diligently updating these relationships.
            // If we aren't, this indicates a bug in the code and this will catch it.
            assert!(
                translation_unlink_success,
                "failed to unlink english word meaning as a translation on a slovene word meaning"
            );
        }


        for category_key in &removed_english_word_meaning.categories {
            let category = self.categories.get_mut_by_key(*category_key)
                // PANIC SAFETY: This should always succeed, because we should be diligently updating
                // these relationships. If we aren't, this indicates a bug in the code and this will catch it.
                .expect("failed to obtain category that is linked to a just-removed english word meaning");

            let reverse_category_relationship_unlink_success = category
                .remove_english_translation_reverse_relationship(
                    &removed_english_word_meaning_cache_key,
                );


            // PANIC SAFETY: This should never panic (removing a reverse category relationship should always succeed),
            // because we should be diligently updating these relationships.
            // If we aren't, this indicates a bug in the code and this will catch it.
            assert!(
                reverse_category_relationship_unlink_success,
                "failed to unlink english word meaning from a category's reverse relationship"
            );
        }
    }



    /*
     * Slovene word-related methods
     */

    pub fn insert_or_update_slovene_word(
        &mut self,
        slovene_word: SloveneWordModel,
    ) -> SloveneWordCacheKey {
        match self.slovene_words.entry(slovene_word.id()) {
            EntityStoreEntry::Vacant(vacant_store_entry) => vacant_store_entry.insert(
                CachedSloveneWord::new_without_meanings(slovene_word),
            ),
            EntityStoreEntry::Occupied(mut occupied_store_entry) => {
                occupied_store_entry
                    .get_mut()
                    .update_with_model(slovene_word);

                occupied_store_entry.key()
            }
        }
    }

    pub fn remove_slovene_word(
        &mut self,
        slovene_word_id: SloveneWordId,
    ) -> Option<SloveneWordModel> {
        let slovene_word_cache_key = self.slovene_words.get_key(&slovene_word_id)?;

        let removed_slovene_word = self.remove_slovene_word_by_cache_key(slovene_word_cache_key)
            // PANIC SAFETY: This can never panic, because the function can only return `None` if 
            // the provided cache key is invalid, but we just obtained the cache key above.
            .expect("failed to remove slovene word from cache");

        Some(removed_slovene_word)
    }

    pub fn remove_slovene_word_by_cache_key(
        &mut self,
        slovene_word_cache_key: SloveneWordCacheKey,
    ) -> Option<SloveneWordModel> {
        let removed_slovene_word = self.slovene_words.remove_by_key(slovene_word_cache_key)?;

        self.remove_related_slovene_word_relationships(&removed_slovene_word);

        Some(removed_slovene_word.word)
    }

    pub fn slovene_word(&self, slovene_word_id: SloveneWordId) -> Option<&CachedSloveneWord> {
        self.slovene_words.get(&slovene_word_id)
    }


    fn remove_related_slovene_word_relationships(
        &mut self,
        removed_cached_slovene_word: &CachedSloveneWord,
    ) {
        for word_meaning_key in &removed_cached_slovene_word.meanings {
            self.remove_slovene_word_meaning_by_cache_key(*word_meaning_key)
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
    ) -> Result<SloveneWordMeaningCacheKey, WordMeaningEntityError> {
        match self
            .slovene_word_meanings
            .entry(slovene_word_meaning.word_meaning_id())
        {
            EntityStoreEntry::Vacant(vacant_store_entry) => {
                let parent_word_cache_key = self
                    .slovene_words
                    .get_key(&slovene_word_meaning.word_id())
                    .ok_or(WordMeaningEntityError::ParentEntityNotFound)?;

                Ok(vacant_store_entry.insert(
                    CachedSloveneWordMeaning::new_without_relationships(
                        parent_word_cache_key,
                        slovene_word_meaning,
                    ),
                ))
            }
            EntityStoreEntry::Occupied(mut occupied_store_entry) => {
                occupied_store_entry
                    .get_mut()
                    .update_with_model(slovene_word_meaning);

                Ok(occupied_store_entry.key())
            }
        }
    }

    pub fn remove_slovene_word_meaning(
        &mut self,
        slovene_word_meaning_id: &SloveneWordMeaningId,
    ) -> Option<SloveneWordMeaningModel> {
        let slovene_word_meaning_cache_key = self
            .slovene_word_meanings
            .get_key(slovene_word_meaning_id)?;


        let removed_slovene_word_meaning = self.remove_slovene_word_meaning_by_cache_key(slovene_word_meaning_cache_key)
            // PANIC SAFETY: This can never panic, because the function can only return `None` if 
            // the provided cache key is invalid, but we just obtained the cache key above.
            .expect("failed to remove slovene word meaning from cache");

        Some(removed_slovene_word_meaning)
    }

    pub fn remove_slovene_word_meaning_by_cache_key(
        &mut self,
        slovene_word_meaning_cache_key: SloveneWordMeaningCacheKey,
    ) -> Option<SloveneWordMeaningModel> {
        let removed_slovene_word_meaning = self
            .slovene_word_meanings
            .remove_by_key(slovene_word_meaning_cache_key)?;

        self.remove_related_slovene_word_meaning_relationships(
            slovene_word_meaning_cache_key,
            &removed_slovene_word_meaning,
        );

        Some(removed_slovene_word_meaning.word_meaning)
    }


    fn remove_related_slovene_word_meaning_relationships(
        &mut self,
        removed_slovene_word_meaning_cache_key: SloveneWordMeaningCacheKey,
        removed_slovene_word_meaning: &CachedSloveneWordMeaning,
    ) {
        for translation_meaning_key in &removed_slovene_word_meaning.translations {
            let english_word_meaning = self.english_word_meanings.get_mut_by_key(*translation_meaning_key)
                // PANIC SAFETY: This should always succeed, because we should be diligently updating
                // these relationships. If we aren't, this indicates a bug in the code and this will catch it.
                .expect("failed to obtain english word meaning that is linked as a translation");

            let translation_unlink_success =
                english_word_meaning.remove_translation(&removed_slovene_word_meaning_cache_key);

            // PANIC SAFETY: This should never panic (removing a translation relationship should always succeed),
            // because we should be diligently updating these relationships.
            // If we aren't, this indicates a bug in the code and this will catch it.
            assert!(
                translation_unlink_success,
                "failed to unlink slovene word meaning as a translation on an english word meaning"
            );
        }

        for category_key in &removed_slovene_word_meaning.categories {
            let category = self.categories.get_mut_by_key(*category_key)
                // PANIC SAFETY: This should always succeed, because we should be diligently updating
                // these relationships. If we aren't, this indicates a bug in the code and this will catch it.
                .expect("failed to obtain category that is linked to a just-removed slovene word meaning");

            let reverse_category_relationship_unlink_success = category
                .remove_slovene_translation_reverse_relationship(
                    &removed_slovene_word_meaning_cache_key,
                );


            // PANIC SAFETY: This should never panic (removing a reverse category relationship should always succeed),
            // because we should be diligently updating these relationships.
            // If we aren't, this indicates a bug in the code and this will catch it.
            assert!(
                reverse_category_relationship_unlink_success,
                "failed to unlink slovene word meaning from a category's reverse relationship"
            );
        }
    }



    /*
     * Translation linking and unlinking methods
     */

    /// Establishes a translation relationship between the given slovene and english word meaning.
    ///
    /// If either (or both) if the IDs provided (`slovene_word_meaning_id` and `english_word_meaning_id`) refer
    /// to meanings that aren't present in cache, this method will return an error.
    ///
    /// If the translation relationship is already present, this call will have no effect, and `Ok(())` will be returned.
    pub fn link_slovene_and_english_word_meanings_as_translations(
        &mut self,
        slovene_word_meaning_id: SloveneWordMeaningId,
        english_word_meaning_id: EnglishWordMeaningId,
    ) -> Result<(), EntityReadError> {
        let slovene_word_meaning = self
            .slovene_word_meanings
            .get_mut_and_key(&slovene_word_meaning_id);

        let english_word_meaning = self
            .english_word_meanings
            .get_mut_and_key(&english_word_meaning_id);


        let (
            (slovene_word_meaning, slovene_word_meaning_key),
            (english_word_meaning, english_word_meaning_key),
        ) = match (slovene_word_meaning, english_word_meaning) {
            (Some(slovene_word_meaning), Some(english_word_meaning)) => {
                (slovene_word_meaning, english_word_meaning)
            }
            _ => {
                return Err(EntityReadError::EntityNotFound);
            }
        };


        let english_translation_add_result =
            slovene_word_meaning.add_translation(english_word_meaning_key);

        // PANIC SAFETY: If adding fails, that would indicate we weren't diligent
        // with propagating our deleted translation relationships and such, which essentially
        // places the cache into an undefined state. We don't want that - we'd rather panic and fix it instead.
        assert!(
            english_translation_add_result,
            "failed to insert english translation link into slovene word meaning"
        );


        let slovene_translation_add_result =
            english_word_meaning.add_translation(slovene_word_meaning_key);

        // PANIC SAFETY: If adding fails, that would indicate we weren't diligent
        // with propagating our deleted translation relationships and such, which essentially
        // places the cache into an undefined state. We don't want that - we'd rather panic and fix it instead.
        assert!(
            slovene_translation_add_result,
            "failed to insert slovene translation link into english word meaning"
        );


        Ok(())
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
        let slovene_word_meaning = self
            .slovene_word_meanings
            .get_mut_and_key(&slovene_word_meaning_id);

        let english_word_meaning = self
            .english_word_meanings
            .get_mut_and_key(&english_word_meaning_id);


        let (
            (slovene_word_meaning, slovene_word_meaning_key),
            (english_word_meaning, english_word_meaning_key),
        ) = match (slovene_word_meaning, english_word_meaning) {
            (Some(slovene_word_meaning), Some(english_word_meaning)) => {
                (slovene_word_meaning, english_word_meaning)
            }
            _ => {
                return Err(EntityReadError::EntityNotFound);
            }
        };


        let english_translation_removal_result =
            slovene_word_meaning.remove_translation(&english_word_meaning_key);

        // PANIC SAFETY: If a removal fails, that would indicate we weren't diligent
        // with propagating our deleted translation relationships and such, which essentially
        // places the cache into an undefined state. We don't want that - we'd rather panic and fix it instead.
        assert!(
            english_translation_removal_result,
            "failed to remove english translation link from slovene word meaning"
        );


        let slovene_translation_removal_result =
            english_word_meaning.remove_translation(&slovene_word_meaning_key);

        // PANIC SAFETY: If a removal fails, that would indicate we weren't diligent
        // with propagating our deleted translation relationships and such, which essentially
        // places the cache into an undefined state. We don't want that - we'd rather panic and fix it instead.
        assert!(
            slovene_translation_removal_result,
            "failed to remove slovene translation link from english word meaning"
        );


        Ok(())
    }



    /*
     * Category model-related methods
     */

    pub fn insert_or_update_category(&mut self, category: CategoryModel) -> CategoryCacheKey {
        match self.categories.entry(category.id) {
            EntityStoreEntry::Vacant(vacant_store_entry) => vacant_store_entry.insert(
                CachedCategory::new_without_relationships(category),
            ),
            EntityStoreEntry::Occupied(mut occupied_store_entry) => {
                occupied_store_entry.get_mut().update_with_model(category);

                occupied_store_entry.key()
            }
        }
    }

    pub fn remove_category(&mut self, category_id: CategoryId) -> Option<CategoryModel> {
        let category_cache_key = self.categories.get_key(&category_id)?;

        let removed_category = self.remove_category_by_cache_key(category_cache_key)
            // PANIC SAFETY: This can never panic, because the function can only return `None` if 
            // the provided cache key is invalid, but we just obtained the cache key above.
            .expect("failed to remove category from cache");

        Some(removed_category)
    }

    pub fn remove_category_by_cache_key(
        &mut self,
        category_cache_key: CategoryCacheKey,
    ) -> Option<CategoryModel> {
        let removed_category = self.categories.remove_by_key(category_cache_key)?;

        self.remove_related_category_relationships(category_cache_key, &removed_category);

        Some(removed_category.category)
    }


    pub fn category(&self, category_id: CategoryId) -> Option<&CachedCategory> {
        self.categories.get(&category_id)
    }


    fn remove_related_category_relationships(
        &mut self,
        removed_cached_category_cache_key: CategoryCacheKey,
        removed_cached_category: &CachedCategory,
    ) {
        for english_word_meaning_key in &removed_cached_category.present_on_english_word_meanings {
            let english_word_meaning = self.english_word_meanings.get_mut_by_key(*english_word_meaning_key)
                // PANIC SAFETY: This should always succeed, because we should be diligently updating
                // these relationships. If we aren't, this indicates a bug in the code and this will catch it.
                .expect("failed to obtain english word meaning that is reverse linked to a category");


            let category_link_removal_result =
                english_word_meaning.remove_category(&removed_cached_category_cache_key);

            // PANIC SAFETY: This should always succeed (the category should always be present on the word meaning prior to removal),
            // because we should be diligently updating these relationships.
            // If we aren't, this indicates a bug in the code and this will catch it.
            assert!(
                category_link_removal_result,
                "failed to unlink category from an english word meaning"
            );
        }

        for slovene_word_meaning_key in &removed_cached_category.present_on_slovene_word_meanings {
            let slovene_word_meaning = self.slovene_word_meanings.get_mut_by_key(*slovene_word_meaning_key)
                // PANIC SAFETY: This should always succeed, because we should be diligently updating
                // these relationships. If we aren't, this indicates a bug in the code and this will catch it.
                .expect("failed to obtain slovene word meaning that is reverse linked to a category");


            let category_link_removal_result =
                slovene_word_meaning.remove_category(&removed_cached_category_cache_key);

            // PANIC SAFETY: This should always succeed (the category should always be present on the word meaning prior to removal),
            // because we should be diligently updating these relationships.
            // If we aren't, this indicates a bug in the code and this will catch it.
            assert!(
                category_link_removal_result,
                "failed to unlink category from a slovene word meaning"
            );
        }
    }



    /*
     * (Internal) category-to-word-meaning linking and unlinking methods
     */

    /// Unlinks (untracks) an english word meaning from the given category. This method
    /// must be called when an english word meaning is removed or the category in question is
    /// removed from it.
    ///
    /// If `target_english_word_meaning_id` does not point to an english word meaning in cache
    /// or if the meaning is not actually present on the category, `false` is returned.
    ///
    /// Importantly, the caller is responsible for upholding the other end of this relationship
    /// (removing or updating the english word meaning cache itself, this method only updates the
    /// reverse relationship on the category entity itself).
    ///
    ///
    /// # Panics
    /// This method panics if the provided `category_cache_key` points to a non-existing category.
    #[deprecated]
    pub(crate) fn unlink_english_word_meaning_from_category_reverse_relationship(
        &mut self,
        category_cache_key: CategoryCacheKey,
        target_english_word_meaning_id: EnglishWordMeaningId,
    ) -> bool {
        let Some(cached_category) = self.categories.get_mut_by_key(category_cache_key) else {
            // PANIC SAFETY: As noted in the method documentation, it is up to the caller to ensure
            // the provided category cache key points to a valid category.
            panic!("failed to obtain cached category: cache key points to a non-existent category");
        };


        let Some(target_english_word_meaning_cache_key) = self
            .english_word_meanings
            .get_key(&target_english_word_meaning_id)
        else {
            return false;
        };



        let previous_number_of_meanings = cached_category.present_on_english_word_meanings.len();

        cached_category
            .present_on_english_word_meanings
            .retain(|other_english_word_meaning| {
                other_english_word_meaning != &target_english_word_meaning_cache_key
            });

        let updated_number_of_meanings = cached_category.present_on_english_word_meanings.len();

        previous_number_of_meanings > updated_number_of_meanings
    }

    /// Links (tracks) an english word meaning with the given category. This method
    /// must be called when an english word meaning is linked with this category.
    /// If the category exists (and the word meaning has been linked or is already linked),
    /// the method returns `true`; otherwise it returns `false`.
    ///
    /// Importantly, the caller is responsible for upholding the other end of this relationship
    /// (adding or updating the english word meaning cache itself).
    ///
    /// # Panics
    /// This method panics if the provided `category_id` does not point to any cached category.
    #[deprecated]
    pub(crate) fn link_english_word_meaning_to_category_reverse_relationship(
        &mut self,
        category_id: CategoryId,
        target_english_word_meaning_cache_key: EnglishWordMeaningCacheKey,
    ) {
        let Some(cached_category) = self.categories.get_mut(&category_id) else {
            // PANIC SAFETY: As noted in the method documentation, it is up to the caller to ensure
            // the provided category cache key points to a valid category.
            panic!("failed to obtain cached category: cache key points to a non-existent category");
        };

        cached_category
            .present_on_english_word_meanings
            .insert(target_english_word_meaning_cache_key);
    }
    // TODO
}
