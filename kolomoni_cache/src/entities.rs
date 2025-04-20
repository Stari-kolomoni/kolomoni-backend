use std::collections::HashSet;

use kolomoni_core::ids::{
    CategoryId,
    EnglishWordId,
    EnglishWordMeaningId,
    SloveneWordId,
    SloveneWordMeaningId,
};
use kolomoni_database::entities::{
    category::CategoryModel,
    word_english::EnglishWordModel,
    word_meaning_english::EnglishWordMeaningModel,
    word_meaning_slovene::SloveneWordMeaningModel,
    word_slovene::SloveneWordModel,
};




#[derive(Clone)]
pub struct CachedEnglishWord {
    /// English word.
    pub(crate) word: EnglishWordModel,

    pub(crate) meaning_ids: HashSet<EnglishWordMeaningId>,
}

impl CachedEnglishWord {
    /// Initializes a new cached english word using the provided
    /// english word model. The initialized cached word
    /// has no meanings (must be added manually).
    pub(crate) fn new_without_meanings(word: EnglishWordModel) -> Self {
        Self {
            word,
            meaning_ids: HashSet::new(),
        }
    }

    pub fn word(&self) -> &EnglishWordModel {
        &self.word
    }

    /// Updates the cached english word with the new english word model.
    /// Leaves meanings intact.
    ///
    /// # Panics
    /// This method panics if the provided `word` does not have the same
    /// ID as the previous cached word.
    pub(crate) fn update_with_model(&mut self, word: EnglishWordModel) {
        assert_eq!(self.word.id(), word.id());

        self.word = word;
    }

    pub(crate) fn add_meaning(&mut self, english_word_meaning_id: EnglishWordMeaningId) -> bool {
        self.meaning_ids.insert(english_word_meaning_id)
    }

    pub(crate) fn remove_meaning(&mut self, english_word_meaning_id: &EnglishWordMeaningId) -> bool {
        self.meaning_ids.remove(english_word_meaning_id)
    }
}




#[derive(Clone)]
pub struct CachedEnglishWordMeaning {
    pub(crate) parent_word_id: EnglishWordId,

    pub(crate) word_meaning: EnglishWordMeaningModel,

    pub(crate) category_ids: HashSet<CategoryId>,

    pub(crate) translation_ids: HashSet<SloveneWordMeaningId>,
}

impl CachedEnglishWordMeaning {
    pub(crate) fn new_without_relationships(
        parent_word_id: EnglishWordId,
        word_meaning: EnglishWordMeaningModel,
    ) -> Self {
        Self {
            parent_word_id,
            word_meaning,
            category_ids: HashSet::new(),
            translation_ids: HashSet::new(),
        }
    }

    pub fn word_meaning(&self) -> &EnglishWordMeaningModel {
        &self.word_meaning
    }

    /// Updates the cached english word meaning, leaving categories
    /// and translations as-is.
    ///
    /// # Panics
    /// This method panics if the provided `new_word_meaning` does not have the same
    /// word meaning ID as the previous cached word meaning, or if it now belongs
    /// to a different parent word.
    pub(crate) fn update_with_model(&mut self, new_word_meaning: EnglishWordMeaningModel) {
        assert_eq!(
            self.word_meaning.parent_word_id(),
            new_word_meaning.parent_word_id()
        );

        assert_eq!(self.word_meaning.id(), new_word_meaning.id());

        self.word_meaning = new_word_meaning;
    }

    pub(crate) fn add_category(&mut self, category_id: CategoryId) -> bool {
        self.category_ids.insert(category_id)
    }

    pub(crate) fn remove_category(&mut self, category_id: &CategoryId) -> bool {
        self.category_ids.remove(category_id)
    }

    pub(crate) fn add_translation(&mut self, translation_id: SloveneWordMeaningId) -> bool {
        self.translation_ids.insert(translation_id)
    }

    pub(crate) fn remove_translation(&mut self, translation_id: &SloveneWordMeaningId) -> bool {
        self.translation_ids.remove(translation_id)
    }
}



#[derive(Clone)]
pub struct CachedSloveneWord {
    pub(crate) word: SloveneWordModel,

    pub(crate) meaning_ids: HashSet<SloveneWordMeaningId>,
}

impl CachedSloveneWord {
    /// Initializes a new cached slovene word using the provided
    /// slovene word model. The initialized cached word
    /// has no meanings (must be added manually).
    pub(crate) fn new_without_meanings(word: SloveneWordModel) -> Self {
        Self {
            word,
            meaning_ids: HashSet::new(),
        }
    }

    pub fn word(&self) -> &SloveneWordModel {
        &self.word
    }

    /// Updates the cached english word with the new english word model.
    /// Leaves meanings intact.
    ///
    /// # Panics
    /// This method panics if the provided `word` does not have the same
    /// ID as the previous cached word.
    pub(crate) fn update_with_model(&mut self, word: SloveneWordModel) {
        assert_eq!(self.word.id(), word.id());

        self.word = word;
    }

    pub(crate) fn add_meaning(&mut self, slovene_word_meaning_id: SloveneWordMeaningId) -> bool {
        self.meaning_ids.insert(slovene_word_meaning_id)
    }

    pub(crate) fn remove_meaning(&mut self, slovene_word_meaning_id: &SloveneWordMeaningId) -> bool {
        self.meaning_ids.remove(slovene_word_meaning_id)
    }
}



#[derive(Clone)]
pub struct CachedSloveneWordMeaning {
    pub(crate) parent_word_id: SloveneWordId,

    pub(crate) word_meaning: SloveneWordMeaningModel,

    pub(crate) category_ids: HashSet<CategoryId>,

    pub(crate) translation_ids: HashSet<EnglishWordMeaningId>,
}

impl CachedSloveneWordMeaning {
    pub(crate) fn new_without_relationships(
        parent_word_id: SloveneWordId,
        word_meaning: SloveneWordMeaningModel,
    ) -> Self {
        Self {
            parent_word_id,
            word_meaning,
            category_ids: HashSet::new(),
            translation_ids: HashSet::new(),
        }
    }

    pub fn word_meaning(&self) -> &SloveneWordMeaningModel {
        &self.word_meaning
    }

    /// Updates the cached slovene word meaning, leaving categories
    /// and translations as-is.
    ///
    /// # Panics
    /// This method panics if the provided `new_word_meaning` does not have the same
    /// word meaning ID as the previous cached word meaning, or if it now belongs
    /// to a different parent word.
    pub(crate) fn update_with_model(&mut self, new_word_meaning: SloveneWordMeaningModel) {
        assert_eq!(
            self.word_meaning.parent_word_id(),
            new_word_meaning.parent_word_id()
        );

        assert_eq!(self.word_meaning.id(), new_word_meaning.id());

        self.word_meaning = new_word_meaning;
    }

    pub(crate) fn add_category(&mut self, category_id: CategoryId) -> bool {
        self.category_ids.insert(category_id)
    }

    pub(crate) fn remove_category(&mut self, category_id: &CategoryId) -> bool {
        self.category_ids.remove(category_id)
    }

    pub(crate) fn add_translation(&mut self, translation_id: EnglishWordMeaningId) -> bool {
        self.translation_ids.insert(translation_id)
    }

    pub(crate) fn remove_translation(&mut self, translation_id: &EnglishWordMeaningId) -> bool {
        self.translation_ids.remove(translation_id)
    }
}




#[derive(Clone, Debug)]
pub struct CachedCategory {
    pub(crate) category: CategoryModel,

    pub(crate) present_on_english_word_meaning_ids: HashSet<EnglishWordMeaningId>,

    pub(crate) present_on_slovene_word_meaning_ids: HashSet<SloveneWordMeaningId>,
}

impl CachedCategory {
    pub(crate) fn new_without_relationships(category_model: CategoryModel) -> Self {
        Self {
            category: category_model,
            present_on_english_word_meaning_ids: HashSet::new(),
            present_on_slovene_word_meaning_ids: HashSet::new(),
        }
    }

    pub(crate) fn update_with_model(&mut self, new_category_model: CategoryModel) {
        assert_eq!(self.category.id, new_category_model.id);

        self.category = new_category_model;
    }

    pub(crate) fn add_english_translation_reverse_relationship(
        &mut self,
        english_word_meaning_id: EnglishWordMeaningId,
    ) -> bool {
        self.present_on_english_word_meaning_ids
            .insert(english_word_meaning_id)
    }

    pub(crate) fn remove_english_translation_reverse_relationship(
        &mut self,
        english_word_meaning_id: &EnglishWordMeaningId,
    ) -> bool {
        self.present_on_english_word_meaning_ids
            .remove(english_word_meaning_id)
    }

    pub(crate) fn add_slovene_translation_reverse_relationship(
        &mut self,
        slovene_word_meaning_id: SloveneWordMeaningId,
    ) -> bool {
        self.present_on_slovene_word_meaning_ids
            .insert(slovene_word_meaning_id)
    }

    pub(crate) fn remove_slovene_translation_reverse_relationship(
        &mut self,
        slovene_word_meaning_id: &SloveneWordMeaningId,
    ) -> bool {
        self.present_on_slovene_word_meaning_ids
            .remove(slovene_word_meaning_id)
    }
}
