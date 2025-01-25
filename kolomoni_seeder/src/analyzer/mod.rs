use insert_only_set::{GrowingKeyedSet, InternalId};
use models::{
    category::{
        Category,
        IntermediateCategory,
        IntermediateCategoryOutputContext,
        IntermediateCategoryOutputError,
    },
    english_word::{EnglishWord, IntermediateEnglishWord},
    english_word_meaning::{
        EnglishWordMeaning,
        IntermediateEnglishWordMeaning,
        IntermediateEnglishWordMeaningOutputContext,
        IntermediateEnglishWordMeaningOutputError,
    },
    slovene_word::{IntermediateSloveneWord, SloveneWord},
    slovene_word_meaning::{
        IntermediateSloveneWordMeaning,
        IntermediateSloveneWordMeaningOutputError,
        IntermediateSloveneWordMeaningResolutionContext,
        SloveneWordMeaning,
    },
    translation::Translation,
    InternalCategoryId,
    InternalEnglishWordId,
    InternalEnglishWordMeaningId,
    InternalSloveneWordId,
    InternalSloveneWordMeaningId,
    InternalTranslationId,
    ToOutputModel,
    TryToOutputModelWithContext,
};
use thiserror::Error;

use crate::parser::{
    SeedSpreadsheetCategoriesIter,
    SeedSpreadsheetRowError,
    SeedSpreadsheetTranslationsIter,
    SeedSpreadsheetsParser,
};

pub(crate) mod insert_only_set;
pub(crate) mod models;




#[derive(Debug, Error)]
#[allow(clippy::enum_variant_names)]
pub enum SeedDatasetError {
    #[error("failed while reading seed spreadsheet")]
    SpreadsheetRowError {
        #[source]
        #[from]
        error: SeedSpreadsheetRowError,
    },

    #[error("failed to fully parse category: \"{}\"", .category_english_name)]
    IntermediateCategoryOutputError {
        category_english_name: String,

        #[source]
        error: IntermediateCategoryOutputError,
    },

    #[error("failed to fully parse an english word meaning for {}", .english_word_lemma)]
    IntermediateEnglishWordMeaningOutputError {
        english_word_lemma: String,

        #[source]
        error: IntermediateEnglishWordMeaningOutputError,
    },

    #[error("failed to fully parse a slovene word meaning for {}", .slovene_word_lemma)]
    IntermediateSloveneWordMeaningOutputError {
        slovene_word_lemma: String,

        #[source]
        error: IntermediateSloveneWordMeaningOutputError,
    },

    #[error("word \"{}\" has a duplicate english-slovene word meaning translation", .slovene_word_lemma)]
    DuplicateTranslationEntries { slovene_word_lemma: String },
}




fn clean_up_string<S>(string: S) -> String
where
    S: AsRef<str>,
{
    string.as_ref().trim().to_string()
}

fn clean_up_optional_string<S>(optional_string: Option<S>) -> Option<String>
where
    S: AsRef<str>,
{
    optional_string
        .map(|string| clean_up_string(string))
        .filter(|cleaned_string| !cleaned_string.is_empty())
}



struct ParsedCategories {
    categories: GrowingKeyedSet<Category>,
}


fn parse_categories(
    categories_iterator: SeedSpreadsheetCategoriesIter,
) -> Result<ParsedCategories, SeedDatasetError> {
    let mut intermediate_categories = GrowingKeyedSet::new();

    for category_row_result in categories_iterator {
        let category_row = category_row_result?;
        let intermediate_category = IntermediateCategory::from_seed_category_row(category_row);

        let _ = intermediate_categories.insert(intermediate_category);
    }


    // Do a second pass on the categories to assign them
    // the correct references to parent categories.
    let intermediate_category_context =
        IntermediateCategoryOutputContext::new(&intermediate_categories);

    let mut final_categories = GrowingKeyedSet::new();

    for intermediate_category in intermediate_categories.values() {
        let final_category = intermediate_category
            .try_to_output_model(&intermediate_category_context)
            .map_err(
                |error| SeedDatasetError::IntermediateCategoryOutputError {
                    category_english_name: intermediate_category.english_name().to_string(),
                    error,
                },
            )?;

        let _ = final_categories.insert(final_category);
    }


    Ok(ParsedCategories {
        categories: final_categories,
    })
}


struct ParsedWordsAndTranslations {
    english_words: GrowingKeyedSet<EnglishWord>,
    english_word_meanings: GrowingKeyedSet<EnglishWordMeaning>,

    slovene_words: GrowingKeyedSet<SloveneWord>,
    slovene_word_meanings: GrowingKeyedSet<SloveneWordMeaning>,

    translations: GrowingKeyedSet<Translation>,
}


struct DraftWordsAndTranslations<'c> {
    parsed_categories: &'c ParsedCategories,

    english_words: GrowingKeyedSet<EnglishWord>,
    english_word_meanings: GrowingKeyedSet<EnglishWordMeaning>,

    slovene_words: GrowingKeyedSet<SloveneWord>,
    slovene_word_meanings: GrowingKeyedSet<SloveneWordMeaning>,

    translations: GrowingKeyedSet<Translation>,
}

impl<'c> DraftWordsAndTranslations<'c> {
    pub fn new(parsed_categories: &'c ParsedCategories) -> Self {
        Self {
            parsed_categories,
            english_words: GrowingKeyedSet::new(),
            english_word_meanings: GrowingKeyedSet::new(),
            slovene_words: GrowingKeyedSet::new(),
            slovene_word_meanings: GrowingKeyedSet::new(),
            translations: GrowingKeyedSet::new(),
        }
    }

    pub fn english_word_by_lemma(&self, english_lemma: &str) -> Option<&EnglishWord> {
        self.english_words
            .values()
            .find(|item| item.lemma == english_lemma)
    }

    pub fn slovene_word_by_lemma(&self, slovene_lemma: &str) -> Option<&SloveneWord> {
        self.slovene_words
            .values()
            .find(|item| item.lemma == slovene_lemma)
    }

    pub fn insert_english_word_if_missing(
        &mut self,
        english_lemma: String,
    ) -> InternalEnglishWordId {
        for value in self.english_words.values() {
            if value.lemma == english_lemma {
                return value.internal_id();
            }
        }

        let intermediate_english_word = IntermediateEnglishWord::from_raw_data(english_lemma);
        let english_word = intermediate_english_word.to_output_model();

        let english_word_internal_id = english_word.internal_id();
        let _ = self.english_words.insert(english_word);

        english_word_internal_id
    }

    pub fn try_insert_english_word_meaning_if_missing(
        &mut self,
        referenced_english_word_by_lemma: String,
        description: Option<String>,
        disambiguation: Option<String>,
        example: Option<String>,
        abbreviation: Option<String>,
        referenced_categories_by_slovene_category_name: Vec<String>,
    ) -> Result<InternalEnglishWordMeaningId, IntermediateEnglishWordMeaningOutputError> {
        let Some(referenced_english_word) =
            self.english_word_by_lemma(&referenced_english_word_by_lemma)
        else {
            return Err(
                IntermediateEnglishWordMeaningOutputError::EnglishWordNotFoundByLemma {
                    english_word_lemma: referenced_english_word_by_lemma,
                },
            );
        };

        for value in self.english_word_meanings.values() {
            if value.word_internal_id() == referenced_english_word.internal_id()
                && value.description == description
                && value.disambiguation == disambiguation
                && value.example == example
                && value.abbreviation == abbreviation
            {
                return Ok(value.internal_id());
            }
        }


        let intermediate_english_word_meaning = IntermediateEnglishWordMeaning::from_raw_data(
            referenced_english_word_by_lemma,
            description,
            disambiguation,
            example,
            abbreviation,
            referenced_categories_by_slovene_category_name,
        );

        let english_word_meaning = intermediate_english_word_meaning.try_to_output_model(
            &IntermediateEnglishWordMeaningOutputContext::new(
                &self.english_words,
                &self.parsed_categories.categories,
            ),
        )?;


        let english_word_meaning_internal_id = english_word_meaning.internal_id();
        let _ = self.english_word_meanings.insert(english_word_meaning);

        Ok(english_word_meaning_internal_id)
    }

    pub fn insert_slovene_word_if_missing(
        &mut self,
        slovene_lemma: String,
    ) -> InternalSloveneWordId {
        for value in self.slovene_words.values() {
            if value.lemma == slovene_lemma {
                return value.internal_id();
            }
        }

        let intermediate_slovene_word = IntermediateSloveneWord::from_raw_data(slovene_lemma);
        let slovene_word = intermediate_slovene_word.to_output_model();

        let slovene_word_internal_id = slovene_word.internal_id();
        let _ = self.slovene_words.insert(slovene_word);

        slovene_word_internal_id
    }

    pub fn try_insert_slovene_word_meaning_if_missing(
        &mut self,
        referenced_slovene_word_by_lemma: String,
        description: Option<String>,
        disambiguation: Option<String>,
        example: Option<String>,
        abbreviation: Option<String>,
        referenced_categories_by_slovene_category_name: Vec<String>,
    ) -> Result<InternalSloveneWordMeaningId, IntermediateSloveneWordMeaningOutputError> {
        let Some(referenced_slovene_word) =
            self.slovene_word_by_lemma(&referenced_slovene_word_by_lemma)
        else {
            return Err(
                IntermediateSloveneWordMeaningOutputError::SloveneWordNotFoundByLemma {
                    slovene_word_lemma: referenced_slovene_word_by_lemma,
                },
            );
        };

        for value in self.slovene_word_meanings.values() {
            if value.word_internal_id() == referenced_slovene_word.internal_id()
                && value.description == description
                && value.disambiguation == disambiguation
                && value.example == example
                && value.abbreviation == abbreviation
            {
                return Ok(value.internal_id());
            }
        }


        let intermediate_slovene_word_meaning = IntermediateSloveneWordMeaning::from_raw_data(
            referenced_slovene_word_by_lemma,
            description,
            disambiguation,
            example,
            abbreviation,
            referenced_categories_by_slovene_category_name,
        );

        let slovene_word_meaning = intermediate_slovene_word_meaning.try_to_output_model(
            &IntermediateSloveneWordMeaningResolutionContext::new(
                &self.slovene_words,
                &self.parsed_categories.categories,
            ),
        )?;


        let slovene_word_meaning_internal_id = slovene_word_meaning.internal_id();
        let _ = self.slovene_word_meanings.insert(slovene_word_meaning);

        Ok(slovene_word_meaning_internal_id)
    }

    pub fn try_insert_translation(
        &mut self,
        english_word_meaning_internal_id: InternalEnglishWordMeaningId,
        slovene_word_meaning_internal_id: InternalSloveneWordMeaningId,
    ) -> Result<(), ()> {
        for translation in self.translations.values() {
            if translation.english_word_meaning == english_word_meaning_internal_id
                && translation.slovene_word_meaning == slovene_word_meaning_internal_id
            {
                return Err(());
            }
        }

        let translation = Translation::new(
            english_word_meaning_internal_id,
            slovene_word_meaning_internal_id,
        );

        let _ = self.translations.insert(translation);
        Ok(())
    }

    pub fn into_parsed_words_and_translations(self) -> ParsedWordsAndTranslations {
        ParsedWordsAndTranslations {
            english_words: self.english_words,
            english_word_meanings: self.english_word_meanings,
            slovene_words: self.slovene_words,
            slovene_word_meanings: self.slovene_word_meanings,
            translations: self.translations,
        }
    }
}


fn parse_words_and_translations(
    parsed_categories: &ParsedCategories,
    translation_row_iterator: SeedSpreadsheetTranslationsIter,
) -> Result<ParsedWordsAndTranslations, SeedDatasetError> {
    let mut draft = DraftWordsAndTranslations::new(parsed_categories);


    for seed_translations_row_result in translation_row_iterator {
        let seed_translation_row = seed_translations_row_result?;

        /* English word + english word meaning parsing and resolution */
        let _ = draft.insert_english_word_if_missing(seed_translation_row.english_lemma.clone());

        let english_word_meaning_internal_id = draft
            .try_insert_english_word_meaning_if_missing(
                seed_translation_row.english_lemma.clone(),
                seed_translation_row.english_meaning_description.clone(),
                seed_translation_row.english_meaning_disamgibuation.clone(),
                seed_translation_row.english_meaning_example.clone(),
                seed_translation_row.english_meaning_abbreviation.clone(),
                seed_translation_row.assigned_categories.clone(),
            )
            .map_err(
                |error| SeedDatasetError::IntermediateEnglishWordMeaningOutputError {
                    english_word_lemma: seed_translation_row.english_lemma.clone(),
                    error,
                },
            )?;


        /* Slovene word + slovene word meaning parsing and resolution */
        let _ = draft.insert_slovene_word_if_missing(seed_translation_row.slovene_lemma.clone());

        let slovene_word_meaning_internal_id = draft
            .try_insert_slovene_word_meaning_if_missing(
                seed_translation_row.slovene_lemma.clone(),
                seed_translation_row.slovene_meaning_description.clone(),
                seed_translation_row.slovene_meaning_disambiguation.clone(),
                seed_translation_row.slovene_meaning_example.clone(),
                seed_translation_row.slovene_meaning_abbreviation.clone(),
                seed_translation_row.assigned_categories.clone(),
            )
            .map_err(
                |error| SeedDatasetError::IntermediateSloveneWordMeaningOutputError {
                    slovene_word_lemma: seed_translation_row.slovene_lemma.clone(),
                    error,
                },
            )?;


        /* Translation link parsing */

        draft
            .try_insert_translation(
                english_word_meaning_internal_id,
                slovene_word_meaning_internal_id,
            )
            .map_err(
                |_| SeedDatasetError::DuplicateTranslationEntries {
                    slovene_word_lemma: seed_translation_row.slovene_lemma.clone(),
                },
            )?;
    }


    Ok(draft.into_parsed_words_and_translations())
}


pub struct SeedDataset {
    categories: GrowingKeyedSet<Category>,

    english_words: GrowingKeyedSet<EnglishWord>,
    english_word_meanings: GrowingKeyedSet<EnglishWordMeaning>,

    slovene_words: GrowingKeyedSet<SloveneWord>,
    slovene_word_meanings: GrowingKeyedSet<SloveneWordMeaning>,

    translations: GrowingKeyedSet<Translation>,
}

impl SeedDataset {
    pub fn parse(spreadsheet_parser: SeedSpreadsheetsParser) -> Result<Self, SeedDatasetError> {
        let (translation_iterator, categories_iterator) = spreadsheet_parser.into_iterators();


        let parsed_categories = parse_categories(categories_iterator)?;

        let parsed_words_and_translations =
            parse_words_and_translations(&parsed_categories, translation_iterator)?;


        Ok(Self {
            categories: parsed_categories.categories,
            english_words: parsed_words_and_translations.english_words,
            english_word_meanings: parsed_words_and_translations.english_word_meanings,
            slovene_words: parsed_words_and_translations.slovene_words,
            slovene_word_meanings: parsed_words_and_translations.slovene_word_meanings,
            translations: parsed_words_and_translations.translations,
        })
    }

    pub fn categories(&self) -> &GrowingKeyedSet<Category> {
        &self.categories
    }

    pub fn english_words(&self) -> &GrowingKeyedSet<EnglishWord> {
        &self.english_words
    }

    pub fn english_word_meanings(&self) -> &GrowingKeyedSet<EnglishWordMeaning> {
        &self.english_word_meanings
    }

    pub fn slovene_words(&self) -> &GrowingKeyedSet<SloveneWord> {
        &self.slovene_words
    }

    pub fn slovene_word_meanings(&self) -> &GrowingKeyedSet<SloveneWordMeaning> {
        &self.slovene_word_meanings
    }

    pub fn translations(&self) -> &GrowingKeyedSet<Translation> {
        &self.translations
    }

    #[allow(dead_code)]
    pub fn category_by_internal_id<'a>(
        &'a self,
        internal_category_id: &'_ InternalCategoryId,
    ) -> Option<&'a Category> {
        self.categories.get(internal_category_id)
    }

    #[allow(dead_code)]
    pub fn english_word_by_internal_id<'a>(
        &'a self,
        internal_english_word_id: &'_ InternalEnglishWordId,
    ) -> Option<&'a EnglishWord> {
        self.english_words.get(internal_english_word_id)
    }

    #[allow(dead_code)]
    pub fn english_word_meaning_by_internal_id<'a>(
        &'a self,
        internal_english_word_meaning_id: &'_ InternalEnglishWordMeaningId,
    ) -> Option<&'a EnglishWordMeaning> {
        self.english_word_meanings
            .get(internal_english_word_meaning_id)
    }

    #[allow(dead_code)]
    pub fn slovene_word_by_internal_id<'a>(
        &'a self,
        internal_slovene_word_id: &'_ InternalSloveneWordId,
    ) -> Option<&'a SloveneWord> {
        self.slovene_words.get(internal_slovene_word_id)
    }

    #[allow(dead_code)]
    pub fn slovene_word_meaning_by_internal_id<'a>(
        &'a self,
        internal_slovene_word_meaning_id: &'_ InternalSloveneWordMeaningId,
    ) -> Option<&'a SloveneWordMeaning> {
        self.slovene_word_meanings
            .get(internal_slovene_word_meaning_id)
    }

    #[allow(dead_code)]
    pub fn translation_by_internal_id<'a>(
        &'a self,
        internal_translation_id: &'_ InternalTranslationId,
    ) -> Option<&'a Translation> {
        self.translations.get(internal_translation_id)
    }
}
