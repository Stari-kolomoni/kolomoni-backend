use thiserror::Error;

use super::{
    category::Category,
    slovene_word::SloveneWord,
    InternalCategoryId,
    InternalSloveneWordId,
    InternalSloveneWordMeaningId,
    TryToOutputModelWithContext,
};
use crate::analyzer::{
    clean_up_optional_string,
    clean_up_string,
    insert_only_set::{GrowingKeyedSet, InternalId},
};


pub struct IntermediateSloveneWordMeaning {
    internal_id: InternalSloveneWordMeaningId,

    description: Option<String>,
    disambiguation: Option<String>,
    example: Option<String>,
    abbreviation: Option<String>,

    referenced_slovene_word_by_lemma: String,
    referenced_categories_by_slovene_name: Vec<String>,
}

impl IntermediateSloveneWordMeaning {
    pub fn from_raw_data(
        raw_referenced_slovene_word_by_lemma: String,
        raw_description: Option<String>,
        raw_disambiguation: Option<String>,
        raw_example: Option<String>,
        raw_abbreviation: Option<String>,
        raw_referenced_categories_by_slovene_name: Vec<String>,
    ) -> Self {
        let referenced_slovene_word_by_lemma =
            clean_up_string(&raw_referenced_slovene_word_by_lemma).to_string();

        let description = clean_up_optional_string(raw_description);
        let disambiguation = clean_up_optional_string(raw_disambiguation);
        let example = clean_up_optional_string(raw_example);
        let abbreviation = clean_up_optional_string(raw_abbreviation);

        let referenced_categories_by_slovene_name = raw_referenced_categories_by_slovene_name
            .into_iter()
            .map(|string| clean_up_string(&string).to_string())
            .collect();

        Self {
            internal_id: InternalSloveneWordMeaningId::generate(),
            description,
            disambiguation,
            example,
            abbreviation,
            referenced_slovene_word_by_lemma,
            referenced_categories_by_slovene_name,
        }
    }
}

impl InternalId for IntermediateSloveneWordMeaning {
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

    fn slovene_word_by_lemma(&self, english_word_lemma: &str) -> Option<&SloveneWord> {
        let mut target_slovene_word_internal_id = None;

        for english_word in self.slovene_words.values() {
            if english_word.lemma == english_word_lemma {
                target_slovene_word_internal_id = Some(english_word.internal_id());
                break;
            }
        }

        let target_slovene_word_internal_id = target_slovene_word_internal_id?;
        self.slovene_words.get(&target_slovene_word_internal_id)
    }

    fn category_by_slovene_name(&self, slovene_category_name: &str) -> Option<&Category> {
        let mut target_category_internal_id = None;

        for category in self.categories.values() {
            if slovene_category_name == category.slovene_name {
                target_category_internal_id = Some(category.internal_id());
                break;
            }
        }

        let target_category_internal_id = target_category_internal_id?;
        self.categories.get(&target_category_internal_id)
    }
}


#[derive(Debug, Error)]
pub enum IntermediateSloveneWordMeaningOutputError {
    #[error("no such slovene word with lemma: \"{}\"", .slovene_word_lemma)]
    SloveneWordNotFoundByLemma { slovene_word_lemma: String },

    #[error("no such category with slovene name: \"{}\"", .category_slovene_name)]
    CategoryNotFoundBySloveneName { category_slovene_name: String },
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


        let mut categories = Vec::with_capacity(self.referenced_categories_by_slovene_name.len());

        for referenced_category_english_name in &self.referenced_categories_by_slovene_name {
            let Some(target_category) =
                context.category_by_slovene_name(referenced_category_english_name)
            else {
                return Err(
                    IntermediateSloveneWordMeaningOutputError::CategoryNotFoundBySloveneName {
                        category_slovene_name: referenced_category_english_name.to_owned(),
                    },
                );
            };

            categories.push(target_category.internal_id());
        }


        Ok(Self::OutputModel {
            internal_id: self.internal_id,
            word: slovene_word.internal_id(),
            description: self.description.clone(),
            disambiguation: self.disambiguation.clone(),
            abbreviation: self.abbreviation.clone(),
            example: self.example.clone(),
            categories,
        })
    }
}




#[derive(Debug, Clone)]
pub struct SloveneWordMeaning {
    internal_id: InternalSloveneWordMeaningId,

    word: InternalSloveneWordId,

    pub description: Option<String>,
    pub disambiguation: Option<String>,
    #[allow(dead_code)]
    pub example: Option<String>,
    pub abbreviation: Option<String>,

    categories: Vec<InternalCategoryId>,
}

impl InternalId for SloveneWordMeaning {
    type InternalId = InternalSloveneWordMeaningId;

    fn internal_id(&self) -> Self::InternalId {
        self.internal_id
    }
}



#[derive(Debug, Error)]
#[error("unable to find slovene word by internal ID: {}", .internal_slovene_word_id)]
pub struct SloveneWordNotFound {
    internal_slovene_word_id: InternalSloveneWordId,
}


#[derive(Debug, Error)]
#[error("unable to find category by internal ID: {}", .internal_category_id)]
pub struct CategoryNotFound {
    internal_category_id: InternalCategoryId,
}

impl SloveneWordMeaning {
    pub fn word_internal_id(&self) -> InternalSloveneWordId {
        self.word
    }

    #[allow(dead_code)]
    pub fn word<'s>(
        &self,
        growing_english_word_set: &'s GrowingKeyedSet<SloveneWord>,
    ) -> Result<&'s SloveneWord, SloveneWordNotFound> {
        growing_english_word_set
            .get(&self.word)
            .ok_or(SloveneWordNotFound {
                internal_slovene_word_id: self.word,
            })
    }

    pub fn categories<'s>(
        &self,
        growing_categories_set: &'s GrowingKeyedSet<Category>,
    ) -> Result<Vec<&'s Category>, CategoryNotFound> {
        let mut categories = Vec::with_capacity(self.categories.len());

        for internal_category_id in &self.categories {
            let category =
                growing_categories_set
                    .get(internal_category_id)
                    .ok_or(CategoryNotFound {
                        internal_category_id: *internal_category_id,
                    })?;

            categories.push(category);
        }

        Ok(categories)
    }
}
