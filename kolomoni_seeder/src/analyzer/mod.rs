use insert_only_set::{GrowingKeyedSet, InternalId};
use models::{
    category::{
        Category,
        IntermediateCategory,
        IntermediateCategoryOutputError,
        IntermediateCategoryResolutionContext,
    },
    english_word::{EnglishWord, IntermediateEnglishWord},
    english_word_meaning::{
        EnglishWordMeaning,
        IntermediateEnglishWordMeaning,
        IntermediateEnglishWordMeaningOutputError,
        IntermediateEnglishWordMeaningResolutionContext,
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
use parking_lot::MappedRwLockReadGuard;
use thiserror::Error;

use crate::parser::{
    SeedSpreadsheetCategoriesIter,
    SeedSpreadsheetRowError,
    SeedSpreadsheetTranslationsIter,
    SeedSpreadsheetsParser,
};

pub(crate) mod insert_only_set;
pub(crate) mod models;
pub(crate) mod shared;




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
}




#[inline]
fn clean_up_str(string: &str) -> &str {
    string.trim()
}


struct ParsedCategories {
    categories: GrowingKeyedSet<Category>,
}


fn parse_categories(
    categories_iterator: SeedSpreadsheetCategoriesIter,
) -> Result<ParsedCategories, SeedDatasetError> {
    let intermediate_categories = GrowingKeyedSet::new();

    for category_row_result in categories_iterator {
        let category_row = category_row_result?;
        let intermediate_category = IntermediateCategory::from_seed_category_row(category_row);

        let _ = intermediate_categories.insert(intermediate_category);
    }


    // Do a second pass on the categories to assign them
    // the correct references to parent categories.
    let intermediate_category_context =
        IntermediateCategoryResolutionContext::new(&intermediate_categories);


    let final_categories = GrowingKeyedSet::new();

    for intermediate_category in intermediate_categories.read_inner().values() {
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


fn parse_words_and_translations(
    parsed_categories: &ParsedCategories,
    translation_row_iterator: SeedSpreadsheetTranslationsIter,
) -> Result<ParsedWordsAndTranslations, SeedDatasetError> {
    let english_words = GrowingKeyedSet::new();
    let english_word_meanings = GrowingKeyedSet::new();

    let slovene_words = GrowingKeyedSet::new();
    let slovene_word_meanings = GrowingKeyedSet::new();

    let translations = GrowingKeyedSet::new();


    for seed_translations_row_result in translation_row_iterator {
        let seed_translation_row = seed_translations_row_result?;

        /* English word + english word meaning parsing and resolution */
        let intermediate_english_word =
            IntermediateEnglishWord::from_raw_data(seed_translation_row.english_lemma.clone());
        let english_word = intermediate_english_word.to_output_model();

        let _ = english_words.insert(english_word);


        let intermediate_english_word_meaning = IntermediateEnglishWordMeaning::from_raw_data(
            seed_translation_row.english_lemma.clone(),
            seed_translation_row.english_meaning_disamgibuation.clone(),
            seed_translation_row.english_meaning_example.clone(),
            seed_translation_row.english_meaning_abbreviation.clone(),
            seed_translation_row.assigned_categories.clone(),
        );

        let english_word_meaning = intermediate_english_word_meaning
            .try_to_output_model(
                &IntermediateEnglishWordMeaningResolutionContext::new(
                    &english_words,
                    &parsed_categories.categories,
                ),
            )
            .map_err(
                |error| SeedDatasetError::IntermediateEnglishWordMeaningOutputError {
                    english_word_lemma: seed_translation_row.english_lemma.clone(),
                    error,
                },
            )?;

        let english_word_meaning_internal_id = english_word_meaning.internal_id();
        let _ = english_word_meanings.insert(english_word_meaning);


        /* Slovene word + slovene word meaning parsing and resolution */
        let intermediate_slovene_word =
            IntermediateSloveneWord::from_raw_data(seed_translation_row.slovene_lemma.clone());
        let slovene_word = intermediate_slovene_word.to_output_model();

        let _ = slovene_words.insert(slovene_word);


        let intermediate_slovene_word_meaning = IntermediateSloveneWordMeaning::from_raw_data(
            seed_translation_row.slovene_lemma.clone(),
            seed_translation_row.slovene_meaning_disambiguation.clone(),
            seed_translation_row.slovene_meaning_example.clone(),
            seed_translation_row.slovene_meaning_abbreviation.clone(),
            seed_translation_row.assigned_categories.clone(),
        );

        let slovene_word_meaning = intermediate_slovene_word_meaning
            .try_to_output_model(
                &IntermediateSloveneWordMeaningResolutionContext::new(
                    &slovene_words,
                    &parsed_categories.categories,
                ),
            )
            .map_err(
                |error| SeedDatasetError::IntermediateSloveneWordMeaningOutputError {
                    slovene_word_lemma: seed_translation_row.slovene_lemma.clone(),
                    error,
                },
            )?;

        let slovene_word_meaning_internal_id = slovene_word_meaning.internal_id();
        let _ = slovene_word_meanings.insert(slovene_word_meaning);


        /* Translation link parsing */

        let translation = Translation::new(
            english_word_meaning_internal_id,
            slovene_word_meaning_internal_id,
        );

        let _ = translations.insert(translation);
    }


    Ok(ParsedWordsAndTranslations {
        english_words,
        english_word_meanings,
        slovene_words,
        slovene_word_meanings,
        translations,
    })
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

    pub fn category_by_internal_id(
        &self,
        internal_category_id: &InternalCategoryId,
    ) -> Option<MappedRwLockReadGuard<'_, Category>> {
        self.categories.get(internal_category_id)
    }

    pub fn english_word_by_internal_id(
        &self,
        internal_english_word_id: &InternalEnglishWordId,
    ) -> Option<MappedRwLockReadGuard<'_, EnglishWord>> {
        self.english_words.get(internal_english_word_id)
    }

    pub fn english_word_meaning_by_internal_id(
        &self,
        internal_english_word_meaning_id: &InternalEnglishWordMeaningId,
    ) -> Option<MappedRwLockReadGuard<'_, EnglishWordMeaning>> {
        self.english_word_meanings
            .get(internal_english_word_meaning_id)
    }

    pub fn slovene_word_by_internal_id(
        &self,
        internal_slovene_word_id: &InternalSloveneWordId,
    ) -> Option<MappedRwLockReadGuard<'_, SloveneWord>> {
        self.slovene_words.get(internal_slovene_word_id)
    }

    pub fn slovene_word_meaning_by_internal_id(
        &self,
        internal_slovene_word_meaning_id: &InternalSloveneWordMeaningId,
    ) -> Option<MappedRwLockReadGuard<'_, SloveneWordMeaning>> {
        self.slovene_word_meanings
            .get(internal_slovene_word_meaning_id)
    }

    pub fn translation_by_internal_id(
        &self,
        internal_translation_id: &InternalTranslationId,
    ) -> Option<MappedRwLockReadGuard<'_, Translation>> {
        self.translations.get(internal_translation_id)
    }
}
