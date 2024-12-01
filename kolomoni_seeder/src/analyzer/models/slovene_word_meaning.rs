use parking_lot::MappedRwLockReadGuard;
use thiserror::Error;

use super::{
    category::{Category, CategoryResolutionContext, CategoryResolutionError, ResolvedCategory},
    slovene_word::SloveneWord,
    InternalCategoryId,
    InternalSloveneWordId,
    InternalSloveneWordMeaningId,
    TryToOutputModelWithContext,
    TryToResolvedModelWithContext,
};
use crate::analyzer::{
    clean_up_str,
    insert_only_set::{GrowingKeyedSet, InternalId},
};


pub struct IntermediateSloveneWordMeaning {
    internal_id: InternalSloveneWordMeaningId,

    disambiguation: Option<String>,
    example: Option<String>,
    abbreviation: Option<String>,

    referenced_slovene_word_by_lemma: String,
    referenced_categories_by_english_name: Vec<String>,
}

impl IntermediateSloveneWordMeaning {
    pub fn from_raw_data(
        raw_referenced_slovene_word_by_lemma: String,
        raw_disambiguation: Option<String>,
        raw_example: Option<String>,
        raw_abbreviation: Option<String>,
        raw_referenced_categories_by_english_name: Vec<String>,
    ) -> Self {
        let referenced_slovene_word_by_lemma =
            clean_up_str(&raw_referenced_slovene_word_by_lemma).to_string();

        let disambiguation = raw_disambiguation.map(|string| clean_up_str(&string).to_string());
        let example = raw_example.map(|string| clean_up_str(&string).to_string());
        let abbreviation = raw_abbreviation.map(|string| clean_up_str(&string).to_string());

        let referenced_categories_by_english_name = raw_referenced_categories_by_english_name
            .into_iter()
            .map(|string| clean_up_str(&string).to_string())
            .collect();

        Self {
            internal_id: InternalSloveneWordMeaningId::generate(),
            disambiguation,
            example,
            abbreviation,
            referenced_slovene_word_by_lemma,
            referenced_categories_by_english_name,
        }
    }
}

impl InternalId for IntermediateSloveneWordMeaning {
    type InternalId = InternalSloveneWordMeaningId;

    fn internal_id(&self) -> Self::InternalId {
        self.internal_id
    }
}



#[derive(Debug, Clone)]
pub struct SloveneWordMeaning {
    internal_id: InternalSloveneWordMeaningId,

    pub word: InternalSloveneWordId,

    pub disambiguation: Option<String>,
    pub example: Option<String>,
    pub abbreviation: Option<String>,

    pub categories: Vec<InternalCategoryId>,
}

impl InternalId for SloveneWordMeaning {
    type InternalId = InternalSloveneWordMeaningId;

    fn internal_id(&self) -> Self::InternalId {
        self.internal_id
    }
}


pub struct IntermediateSloveneWordMeaningResolutionContext<'c> {
    slovene_words: &'c GrowingKeyedSet<SloveneWord>,
    categories: &'c GrowingKeyedSet<Category>,
}

impl<'c> IntermediateSloveneWordMeaningResolutionContext<'c> {
    pub fn new(
        slovene_words: &'c GrowingKeyedSet<SloveneWord>,
        categories: &'c GrowingKeyedSet<Category>,
    ) -> Self {
        Self {
            slovene_words,
            categories,
        }
    }

    fn slovene_word_by_lemma(
        &self,
        english_word_lemma: &str,
    ) -> Option<MappedRwLockReadGuard<'_, SloveneWord>> {
        let mut target_slovene_word_internal_id = None;

        for english_word in self.slovene_words.read_inner().values() {
            if english_word.lemma == english_word_lemma {
                target_slovene_word_internal_id = Some(english_word.internal_id());
                break;
            }
        }

        let Some(target_slovene_word_internal_id) = target_slovene_word_internal_id else {
            return None;
        };


        self.slovene_words.get(&target_slovene_word_internal_id)
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
pub enum IntermediateSloveneWordMeaningOutputError {
    #[error("no such slovene word with lemma: \"{}\"", .slovene_word_lemma)]
    SloveneWordNotFoundByLemma { slovene_word_lemma: String },

    #[error("no such category with english name: \"{}\"", .category_english_name)]
    CategoryNotFoundByEnglishName { category_english_name: String },
}


impl TryToOutputModelWithContext for IntermediateSloveneWordMeaning {
    type Context<'c> = IntermediateSloveneWordMeaningResolutionContext<'c>;
    type OutputModel = SloveneWordMeaning;
    type Error = IntermediateSloveneWordMeaningOutputError;

    fn try_to_output_model<'a>(
        &self,
        context: &'a Self::Context<'a>,
    ) -> Result<Self::OutputModel, Self::Error> {
        let Some(slovene_word) =
            context.slovene_word_by_lemma(&self.referenced_slovene_word_by_lemma)
        else {
            return Err(
                IntermediateSloveneWordMeaningOutputError::SloveneWordNotFoundByLemma {
                    slovene_word_lemma: self.referenced_slovene_word_by_lemma.clone(),
                },
            );
        };


        let mut categories = Vec::with_capacity(self.referenced_categories_by_english_name.len());

        for referenced_category_english_name in &self.referenced_categories_by_english_name {
            let Some(target_category) =
                context.category_by_english_name(&referenced_category_english_name)
            else {
                return Err(
                    IntermediateSloveneWordMeaningOutputError::CategoryNotFoundByEnglishName {
                        category_english_name: referenced_category_english_name.to_owned(),
                    },
                );
            };

            categories.push(target_category.internal_id());
        }


        Ok(Self::OutputModel {
            internal_id: self.internal_id,
            word: slovene_word.internal_id(),
            disambiguation: self.disambiguation.clone(),
            abbreviation: self.abbreviation.clone(),
            example: self.example.clone(),
            categories,
        })
    }
}




#[derive(Debug, Clone)]
pub struct ResolvedSloveneWordMeaning {
    internal_id: InternalSloveneWordMeaningId,

    pub word: SloveneWord,

    pub disambiguation: Option<String>,
    pub example: Option<String>,
    pub abbreviation: Option<String>,

    pub categories: Vec<ResolvedCategory>,
}


pub struct SloveneWordMeaningResolutionContext<'a> {
    slovene_words: &'a GrowingKeyedSet<SloveneWord>,
    categories: &'a GrowingKeyedSet<Category>,
}

impl<'a> SloveneWordMeaningResolutionContext<'a> {
    pub fn new(
        slovene_words: &'a GrowingKeyedSet<SloveneWord>,
        categories: &'a GrowingKeyedSet<Category>,
    ) -> Self {
        Self {
            slovene_words,
            categories,
        }
    }

    fn slovene_word_by_internal_id(
        &self,
        internal_slovene_word_id: &InternalSloveneWordId,
    ) -> Option<MappedRwLockReadGuard<'a, SloveneWord>> {
        self.slovene_words.get(internal_slovene_word_id)
    }

    fn category_by_internal_id(
        &self,
        internal_category_id: &InternalCategoryId,
    ) -> Option<MappedRwLockReadGuard<'a, Category>> {
        self.categories.get(internal_category_id)
    }
}


#[derive(Debug, Error)]
pub enum SloveneWordMeaningResolutionError {
    #[error("unable to find slovene word by internal ID: {}", .internal_slovene_word_id)]
    SloveneWordNotFound {
        internal_slovene_word_id: InternalSloveneWordId,
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


impl TryToResolvedModelWithContext for SloveneWordMeaning {
    type ResolvedModel = ResolvedSloveneWordMeaning;
    type Context<'ctx> = SloveneWordMeaningResolutionContext<'ctx>;
    type Error = SloveneWordMeaningResolutionError;

    fn try_to_resolved_model<'ctx>(
        &'ctx self,
        context: &'ctx Self::Context<'ctx>,
    ) -> Result<Self::ResolvedModel, Self::Error> {
        let Some(slovene_word) = context.slovene_word_by_internal_id(&self.word) else {
            return Err(
                SloveneWordMeaningResolutionError::SloveneWordNotFound {
                    internal_slovene_word_id: self.word,
                },
            );
        };


        let mut categories = Vec::with_capacity(self.categories.len());

        for internal_category_id in &self.categories {
            let Some(category) = context.category_by_internal_id(internal_category_id) else {
                return Err(
                    SloveneWordMeaningResolutionError::CategoryNotFound {
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
            word: slovene_word.to_owned(),
            disambiguation: self.disambiguation.clone(),
            abbreviation: self.abbreviation.clone(),
            example: self.example.clone(),
            categories,
        })
    }
}
