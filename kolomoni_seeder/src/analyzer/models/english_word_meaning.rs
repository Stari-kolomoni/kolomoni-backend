use parking_lot::MappedRwLockReadGuard;
use thiserror::Error;

use super::{
    category::{Category, CategoryResolutionContext, CategoryResolutionError, ResolvedCategory},
    english_word::EnglishWord,
    InternalCategoryId,
    InternalEnglishWordId,
    InternalEnglishWordMeaningId,
    TryToOutputModelWithContext,
    TryToResolvedModelWithContext,
};
use crate::analyzer::{
    clean_up_str,
    insert_only_set::{GrowingKeyedSet, InternalId},
};



pub struct IntermediateEnglishWordMeaning {
    internal_id: InternalEnglishWordMeaningId,

    disambiguation: Option<String>,
    example: Option<String>,
    abbreviation: Option<String>,

    referenced_english_word_by_lemma: String,
    referenced_categories_by_english_name: Vec<String>,
}

impl IntermediateEnglishWordMeaning {
    pub fn from_raw_data(
        raw_referenced_english_word_by_lemma: String,
        raw_disambiguation: Option<String>,
        raw_example: Option<String>,
        raw_abbreviation: Option<String>,
        raw_referenced_categories_by_english_name: Vec<String>,
    ) -> Self {
        let referenced_english_word_by_lemma =
            clean_up_str(&raw_referenced_english_word_by_lemma).to_string();

        let disambiguation = raw_disambiguation.map(|string| clean_up_str(&string).to_string());
        let example = raw_example.map(|string| clean_up_str(&string).to_string());
        let abbreviation = raw_abbreviation.map(|string| clean_up_str(&string).to_string());

        let referenced_categories_by_english_name = raw_referenced_categories_by_english_name
            .into_iter()
            .map(|string| clean_up_str(&string).to_string())
            .collect();

        Self {
            internal_id: InternalEnglishWordMeaningId::generate(),
            disambiguation,
            example,
            abbreviation,
            referenced_english_word_by_lemma,
            referenced_categories_by_english_name,
        }
    }
}

impl InternalId for IntermediateEnglishWordMeaning {
    type InternalId = InternalEnglishWordMeaningId;

    fn internal_id(&self) -> Self::InternalId {
        self.internal_id
    }
}



#[derive(Debug, Clone)]
pub struct EnglishWordMeaning {
    internal_id: InternalEnglishWordMeaningId,

    pub word: InternalEnglishWordId,

    pub disambiguation: Option<String>,
    pub example: Option<String>,
    pub abbreviation: Option<String>,

    pub categories: Vec<InternalCategoryId>,
}

impl InternalId for EnglishWordMeaning {
    type InternalId = InternalEnglishWordMeaningId;

    fn internal_id(&self) -> Self::InternalId {
        self.internal_id
    }
}


pub struct IntermediateEnglishWordMeaningResolutionContext<'c> {
    english_words: &'c GrowingKeyedSet<EnglishWord>,
    categories: &'c GrowingKeyedSet<Category>,
}

impl<'c> IntermediateEnglishWordMeaningResolutionContext<'c> {
    pub fn new(
        english_words: &'c GrowingKeyedSet<EnglishWord>,
        categories: &'c GrowingKeyedSet<Category>,
    ) -> Self {
        Self {
            english_words,
            categories,
        }
    }

    fn english_word_by_lemma(
        &self,
        english_word_lemma: &str,
    ) -> Option<MappedRwLockReadGuard<'_, EnglishWord>> {
        let mut target_english_word_internal_id = None;

        for english_word in self.english_words.read_inner().values() {
            if english_word.lemma == english_word_lemma {
                target_english_word_internal_id = Some(english_word.internal_id());
                break;
            }
        }

        let Some(target_english_word_internal_id) = target_english_word_internal_id else {
            return None;
        };


        self.english_words.get(&target_english_word_internal_id)
    }

    fn category_by_english_name(
        &self,
        english_category_name: &str,
    ) -> Option<MappedRwLockReadGuard<'_, Category>> {
        let mut target_category_internal_id = None;

        for category in self.categories.read_inner().values() {
            if english_category_name == category.english_name {
                target_category_internal_id = Some(category.internal_id());
                break;
            }
        }

        let Some(target_category_internal_id) = target_category_internal_id else {
            return None;
        };

        self.categories.get(&target_category_internal_id)
    }
}


#[derive(Debug, Error)]
pub enum IntermediateEnglishWordMeaningOutputError {
    #[error("no such english word with lemma: \"{}\"", .english_word_lemma)]
    EnglishWordNotFoundByLemma { english_word_lemma: String },

    #[error("no such category with english name: \"{}\"", .category_english_name)]
    CategoryNotFoundByEnglishName { category_english_name: String },
}


impl TryToOutputModelWithContext for IntermediateEnglishWordMeaning {
    type Context<'c> = IntermediateEnglishWordMeaningResolutionContext<'c>;
    type OutputModel = EnglishWordMeaning;
    type Error = IntermediateEnglishWordMeaningOutputError;

    fn try_to_output_model<'a>(
        &self,
        context: &'a Self::Context<'a>,
    ) -> Result<Self::OutputModel, Self::Error> {
        let Some(english_word) =
            context.english_word_by_lemma(&self.referenced_english_word_by_lemma)
        else {
            return Err(
                IntermediateEnglishWordMeaningOutputError::EnglishWordNotFoundByLemma {
                    english_word_lemma: self.referenced_english_word_by_lemma.clone(),
                },
            );
        };


        let mut categories = Vec::with_capacity(self.referenced_categories_by_english_name.len());

        for referenced_category_english_name in &self.referenced_categories_by_english_name {
            let Some(target_category) =
                context.category_by_english_name(&referenced_category_english_name)
            else {
                return Err(
                    IntermediateEnglishWordMeaningOutputError::CategoryNotFoundByEnglishName {
                        category_english_name: referenced_category_english_name.to_owned(),
                    },
                );
            };

            categories.push(target_category.internal_id());
        }


        Ok(Self::OutputModel {
            internal_id: self.internal_id,
            word: english_word.internal_id(),
            disambiguation: self.disambiguation.clone(),
            abbreviation: self.abbreviation.clone(),
            example: self.example.clone(),
            categories,
        })
    }
}



#[derive(Debug, Clone)]
pub struct ResolvedEnglishWordMeaning {
    internal_id: InternalEnglishWordMeaningId,

    pub word: EnglishWord,

    pub disambiguation: Option<String>,
    pub example: Option<String>,
    pub abbreviation: Option<String>,

    pub categories: Vec<ResolvedCategory>,
}



pub struct EnglishWordMeaningResolutionContext<'a> {
    english_words: &'a GrowingKeyedSet<EnglishWord>,
    categories: &'a GrowingKeyedSet<Category>,
}

impl<'a> EnglishWordMeaningResolutionContext<'a> {
    pub fn new(
        english_words: &'a GrowingKeyedSet<EnglishWord>,
        categories: &'a GrowingKeyedSet<Category>,
    ) -> Self {
        Self {
            english_words,
            categories,
        }
    }

    fn english_word_by_internal_id(
        &self,
        internal_english_word_id: &InternalEnglishWordId,
    ) -> Option<MappedRwLockReadGuard<'a, EnglishWord>> {
        self.english_words.get(internal_english_word_id)
    }

    fn category_by_internal_id(
        &self,
        internal_category_id: &InternalCategoryId,
    ) -> Option<MappedRwLockReadGuard<'a, Category>> {
        self.categories.get(internal_category_id)
    }
}



#[derive(Debug, Error)]
pub enum EnglishWordMeaningResolutionError {
    #[error("unable to find english word by internal ID: {}", .internal_english_word_id)]
    EnglishWordNotFound {
        internal_english_word_id: InternalEnglishWordId,
    },

    #[error("unable to find category by internal ID: {}", .internal_category_id)]
    CategoryNotFound {
        internal_category_id: InternalCategoryId,
    },

    #[error("failed to resolve category")]
    CategoryResolutionError {
        #[from]
        #[source]
        error: CategoryResolutionError,
    },
}



impl TryToResolvedModelWithContext for EnglishWordMeaning {
    type ResolvedModel = ResolvedEnglishWordMeaning;
    type Context<'ctx> = EnglishWordMeaningResolutionContext<'ctx>;
    type Error = EnglishWordMeaningResolutionError;

    fn try_to_resolved_model<'ctx>(
        &'ctx self,
        context: &'ctx Self::Context<'ctx>,
    ) -> Result<Self::ResolvedModel, Self::Error> {
        let Some(english_word) = context.english_word_by_internal_id(&self.word) else {
            return Err(
                EnglishWordMeaningResolutionError::EnglishWordNotFound {
                    internal_english_word_id: self.word,
                },
            );
        };


        let mut categories = Vec::with_capacity(self.categories.len());

        for internal_category_id in &self.categories {
            let Some(category) = context.category_by_internal_id(internal_category_id) else {
                return Err(
                    EnglishWordMeaningResolutionError::CategoryNotFound {
                        internal_category_id: internal_category_id.to_owned(),
                    },
                );
            };

            categories.push(
                category.try_to_resolved_model(&CategoryResolutionContext::new(
                    &context.categories,
                ))?,
            );
        }


        Ok(Self::ResolvedModel {
            internal_id: self.internal_id,
            word: english_word.to_owned(),
            disambiguation: self.disambiguation.clone(),
            example: self.example.clone(),
            abbreviation: self.abbreviation.clone(),
            categories,
        })
    }
}
