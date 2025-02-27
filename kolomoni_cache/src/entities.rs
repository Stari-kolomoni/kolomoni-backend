use std::collections::HashSet;

use kolomoni_database::entities::{
    category::CategoryModel,
    word_english::EnglishWordModel,
    word_meaning_english::EnglishWordMeaningModel,
    word_meaning_slovene::SloveneWordMeaningModel,
    word_slovene::SloveneWordModel,
};
use slotmap::new_key_type;


new_key_type! { pub struct EnglishWordCacheKey; }
new_key_type! { pub struct EnglishWordMeaningCacheKey; }
new_key_type! { pub struct SloveneWordCacheKey; }
new_key_type! { pub struct SloveneWordMeaningCacheKey; }
new_key_type! { pub struct CategoryCacheKey; }




pub struct CachedEnglishWord {
    pub word: EnglishWordModel,

    pub meanings: HashSet<EnglishWordMeaningCacheKey>,
}

impl CachedEnglishWord {
    /// Initializes a new cached english word using the provided
    /// english word model. The initialized cached word
    /// has no meanings (must be added manually).
    pub fn new_without_meanings(word: EnglishWordModel) -> Self {
        Self {
            word,
            meanings: HashSet::new(),
        }
    }

    /// Updates the cached english word with the new english word model.
    /// Leaves meanings intact.
    ///
    /// # Panics
    /// This method panics if the provided `word` does not have the same
    /// ID as the previous cached word.
    pub fn update_with_model(&mut self, word: EnglishWordModel) {
        assert_eq!(self.word.id(), word.id());

        self.word = word;
    }

    pub fn add_meaning(
        &mut self,
        english_word_meaning_cache_key: EnglishWordMeaningCacheKey,
    ) -> bool {
        self.meanings.insert(english_word_meaning_cache_key)
    }

    pub fn remove_meaning(
        &mut self,
        english_word_meaning_cache_key: &EnglishWordMeaningCacheKey,
    ) -> bool {
        self.meanings.remove(english_word_meaning_cache_key)
    }
}



pub struct CachedEnglishWordMeaning {
    pub parent_word: EnglishWordCacheKey,

    pub word_meaning: EnglishWordMeaningModel,

    pub categories: HashSet<CategoryCacheKey>,

    pub translations: HashSet<SloveneWordMeaningCacheKey>,
}

impl CachedEnglishWordMeaning {
    pub fn new_without_relationships(
        parent_word: EnglishWordCacheKey,
        word_meaning: EnglishWordMeaningModel,
    ) -> Self {
        Self {
            parent_word,
            word_meaning,
            categories: HashSet::new(),
            translations: HashSet::new(),
        }
    }

    /// Updates the cached english word meaning, leaving categories
    /// and translations as-is.
    ///
    /// # Panics
    /// This method panics if the provided `new_word_meaning` does not have the same
    /// word meaning ID as the previous cached word meaning, or if it now belongs
    /// to a different parent word.
    pub fn update_with_model(&mut self, new_word_meaning: EnglishWordMeaningModel) {
        assert_eq!(
            self.word_meaning.word_id(),
            new_word_meaning.word_id()
        );

        assert_eq!(
            self.word_meaning.word_meaning_id(),
            new_word_meaning.word_meaning_id()
        );

        self.word_meaning = new_word_meaning;
    }

    pub fn add_category(&mut self, category_cache_key: CategoryCacheKey) -> bool {
        self.categories.insert(category_cache_key)
    }

    pub fn remove_category(&mut self, category_cache_key: &CategoryCacheKey) -> bool {
        self.categories.remove(category_cache_key)
    }

    pub fn add_translation(&mut self, translation_cache_key: SloveneWordMeaningCacheKey) -> bool {
        self.translations.insert(translation_cache_key)
    }

    pub fn remove_translation(
        &mut self,
        translation_cache_key: &SloveneWordMeaningCacheKey,
    ) -> bool {
        self.translations.remove(translation_cache_key)
    }
}



pub struct CachedSloveneWord {
    pub word: SloveneWordModel,

    pub meanings: HashSet<SloveneWordMeaningCacheKey>,
}

impl CachedSloveneWord {
    /// Initializes a new cached slovene word using the provided
    /// slovene word model. The initialized cached word
    /// has no meanings (must be added manually).
    pub fn new_without_meanings(word: SloveneWordModel) -> Self {
        Self {
            word,
            meanings: HashSet::new(),
        }
    }

    /// Updates the cached english word with the new english word model.
    /// Leaves meanings intact.
    ///
    /// # Panics
    /// This method panics if the provided `word` does not have the same
    /// ID as the previous cached word.
    pub fn update_with_model(&mut self, word: SloveneWordModel) {
        assert_eq!(self.word.id(), word.id());

        self.word = word;
    }

    pub fn add_meaning(
        &mut self,
        slovene_word_meaning_cache_key: SloveneWordMeaningCacheKey,
    ) -> bool {
        self.meanings.insert(slovene_word_meaning_cache_key)
    }

    pub fn remove_meaning(
        &mut self,
        slovene_word_meaning_cache_key: &SloveneWordMeaningCacheKey,
    ) -> bool {
        self.meanings.remove(slovene_word_meaning_cache_key)
    }
}



pub struct CachedSloveneWordMeaning {
    pub parent_word: SloveneWordCacheKey,

    pub word_meaning: SloveneWordMeaningModel,

    pub categories: HashSet<CategoryCacheKey>,

    pub translations: HashSet<EnglishWordMeaningCacheKey>,
}

impl CachedSloveneWordMeaning {
    pub fn new_without_relationships(
        parent_word: SloveneWordCacheKey,
        word_meaning: SloveneWordMeaningModel,
    ) -> Self {
        Self {
            parent_word,
            word_meaning,
            categories: HashSet::new(),
            translations: HashSet::new(),
        }
    }

    /// Updates the cached slovene word meaning, leaving categories
    /// and translations as-is.
    ///
    /// # Panics
    /// This method panics if the provided `new_word_meaning` does not have the same
    /// word meaning ID as the previous cached word meaning, or if it now belongs
    /// to a different parent word.
    pub fn update_with_model(&mut self, new_word_meaning: SloveneWordMeaningModel) {
        assert_eq!(
            self.word_meaning.word_id(),
            new_word_meaning.word_id()
        );

        assert_eq!(
            self.word_meaning.word_meaning_id(),
            new_word_meaning.word_meaning_id()
        );

        self.word_meaning = new_word_meaning;
    }

    pub fn add_category(&mut self, category_cache_key: CategoryCacheKey) -> bool {
        self.categories.insert(category_cache_key)
    }

    pub fn remove_category(&mut self, category_cache_key: &CategoryCacheKey) -> bool {
        self.categories.remove(category_cache_key)
    }

    pub fn add_translation(&mut self, translation_cache_key: EnglishWordMeaningCacheKey) -> bool {
        self.translations.insert(translation_cache_key)
    }

    pub fn remove_translation(
        &mut self,
        translation_cache_key: &EnglishWordMeaningCacheKey,
    ) -> bool {
        self.translations.remove(translation_cache_key)
    }
}




pub struct CachedCategory {
    pub category: CategoryModel,

    pub present_on_english_word_meanings: HashSet<EnglishWordMeaningCacheKey>,

    pub present_on_slovene_word_meanings: HashSet<SloveneWordMeaningCacheKey>,
}

impl CachedCategory {
    pub fn new_without_relationships(category_model: CategoryModel) -> Self {
        Self {
            category: category_model,
            present_on_english_word_meanings: HashSet::new(),
            present_on_slovene_word_meanings: HashSet::new(),
        }
    }

    pub fn update_with_model(&mut self, new_category_model: CategoryModel) {
        assert_eq!(self.category.id, new_category_model.id);

        self.category = new_category_model;
    }

    pub fn add_english_translation_reverse_relationship(
        &mut self,
        english_word_meaning_cache_key: EnglishWordMeaningCacheKey,
    ) -> bool {
        self.present_on_english_word_meanings
            .insert(english_word_meaning_cache_key)
    }

    pub fn remove_english_translation_reverse_relationship(
        &mut self,
        english_word_meaning_cache_key: &EnglishWordMeaningCacheKey,
    ) -> bool {
        self.present_on_english_word_meanings
            .remove(english_word_meaning_cache_key)
    }

    pub fn add_slovene_translation_reverse_relationship(
        &mut self,
        slovene_word_meaning_cache_key: SloveneWordMeaningCacheKey,
    ) -> bool {
        self.present_on_slovene_word_meanings
            .insert(slovene_word_meaning_cache_key)
    }

    pub fn remove_slovene_translation_reverse_relationship(
        &mut self,
        slovene_word_meaning_cache_key: &SloveneWordMeaningCacheKey,
    ) -> bool {
        self.present_on_slovene_word_meanings
            .remove(slovene_word_meaning_cache_key)
    }
}
