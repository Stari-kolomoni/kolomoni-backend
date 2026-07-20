use thiserror::Error;

use super::{
    category::Category,
    english_word::EnglishWord,
    InternalCategoryId,
    InternalEnglishWordId,
    InternalEnglishWordMeaningId,
    TryToOutputModelWithContext,
};
use crate::analyzer::{
    clean_up_optional_string,
    clean_up_string,
    insert_only_set::{GrowingKeyedSet, InternalId},
};



pub struct IntermediateEnglishWordMeaning {
    internal_id: InternalEnglishWordMeaningId,

    description: Option<String>,
    disambiguation: Option<String>,
    example: Option<String>,
    abbreviation: Option<String>,

    referenced_english_word_by_lemma: String,
    referenced_categories_by_slovene_name: Vec<String>,
}

impl IntermediateEnglishWordMeaning {
    pub fn from_raw_data(
        raw_referenced_english_word_by_lemma: String,
        raw_description: Option<String>,
        raw_disambiguation: Option<String>,
        raw_example: Option<String>,
        raw_abbreviation: Option<String>,
        raw_referenced_categories_by_slovene_name: Vec<String>,
    ) -> Self {
        let referenced_english_word_by_lemma = clean_up_string(raw_referenced_english_word_by_lemma);

        let description = clean_up_optional_string(raw_description);
        let disambiguation = clean_up_optional_string(raw_disambiguation);
        let example = clean_up_optional_string(raw_example);
        let abbreviation = clean_up_optional_string(raw_abbreviation);

        let referenced_categories_by_slovene_name = raw_referenced_categories_by_slovene_name
            .into_iter()
            .map(clean_up_string)
            .collect();

        Self {
            internal_id: InternalEnglishWordMeaningId::generate(),
            description,
            disambiguation,
            example,
            abbreviation,
            referenced_english_word_by_lemma,
            referenced_categories_by_slovene_name,
        }
    }
}

impl InternalId for IntermediateEnglishWordMeaning {
    type InternalId = InternalEnglishWordMeaningId;

    fn internal_id(&self) -> Self::InternalId {
        self.internal_id
    }
}




pub struct IntermediateEnglishWordMeaningOutputContext<'c> {
    english_words: &'c GrowingKeyedSet<EnglishWord>,
    categories: &'c GrowingKeyedSet<Category>,
}

impl<'c> IntermediateEnglishWordMeaningOutputContext<'c> {
    pub fn new(
        english_words: &'c GrowingKeyedSet<EnglishWord>,
        categories: &'c GrowingKeyedSet<Category>,
    ) -> Self {
        Self {
            english_words,
            categories,
        }
    }

    fn english_word_by_lemma(&self, english_word_lemma: &str) -> Option<&EnglishWord> {
        let mut target_english_word_internal_id = None;

        for english_word in self.english_words.values() {
            if english_word.lemma == english_word_lemma {
                target_english_word_internal_id = Some(english_word.internal_id());
                break;
            }
        }

        let target_english_word_internal_id = target_english_word_internal_id?;
        self.english_words.get(&target_english_word_internal_id)
    }

    fn category_by_slovene_name(&self, slovene_category_name: &str) -> Option<&Category> {
        let mut target_category_internal_id = None;

        for category in self.categories.values() {
            if slovene_category_name == category.full_slovene_name {
                target_category_internal_id = Some(category.internal_id());
                break;
            }
        }

        let target_category_internal_id = target_category_internal_id?;
        self.categories.get(&target_category_internal_id)
    }
}


#[derive(Debug, Error)]
pub enum IntermediateEnglishWordMeaningOutputError {
    #[error("no such english word with lemma: \"{}\"", .english_word_lemma)]
    EnglishWordNotFoundByLemma { english_word_lemma: String },

    #[error("no such category with slovene name: \"{}\"", .category_slovene_name)]
    CategoryNotFoundBySloveneName { category_slovene_name: String },
}


impl TryToOutputModelWithContext for IntermediateEnglishWordMeaning {
    type Context<'c> = IntermediateEnglishWordMeaningOutputContext<'c>;
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


        let mut categories = Vec::with_capacity(self.referenced_categories_by_slovene_name.len());

        for referenced_category_slovene_name in &self.referenced_categories_by_slovene_name {
            let Some(target_category) =
                context.category_by_slovene_name(referenced_category_slovene_name)
            else {
                return Err(
                    IntermediateEnglishWordMeaningOutputError::CategoryNotFoundBySloveneName {
                        category_slovene_name: referenced_category_slovene_name.to_owned(),
                    },
                );
            };

            categories.push(target_category.internal_id());
        }


        Ok(Self::OutputModel {
            internal_id: self.internal_id,
            description: self.description.clone(),
            word: english_word.internal_id(),
            disambiguation: self.disambiguation.clone(),
            abbreviation: self.abbreviation.clone(),
            example: self.example.clone(),
            categories,
        })
    }
}



#[derive(Debug, Clone)]
pub struct EnglishWordMeaning {
    internal_id: InternalEnglishWordMeaningId,

    word: InternalEnglishWordId,

    pub description: Option<String>,
    pub disambiguation: Option<String>,

    // TODO Integrate english word meaning examples into the backend.
    #[allow(dead_code)]
    pub example: Option<String>,

    pub abbreviation: Option<String>,

    categories: Vec<InternalCategoryId>,
}

impl InternalId for EnglishWordMeaning {
    type InternalId = InternalEnglishWordMeaningId;

    fn internal_id(&self) -> Self::InternalId {
        self.internal_id
    }
}


#[derive(Debug, Error)]
#[error("unable to find english word by internal ID: {}", .internal_english_word_id)]
pub struct EnglishWordNotFound {
    internal_english_word_id: InternalEnglishWordId,
}


#[derive(Debug, Error)]
#[error("unable to find category by internal ID: {}", .internal_category_id)]
pub struct CategoryNotFound {
    internal_category_id: InternalCategoryId,
}


impl EnglishWordMeaning {
    pub fn word_internal_id(&self) -> InternalEnglishWordId {
        self.word
    }

    #[allow(dead_code)]
    pub fn word<'s>(
        &self,
        growing_english_word_set: &'s GrowingKeyedSet<EnglishWord>,
    ) -> Result<&'s EnglishWord, EnglishWordNotFound> {
        growing_english_word_set
            .get(&self.word)
            .ok_or(EnglishWordNotFound {
                internal_english_word_id: self.word,
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
