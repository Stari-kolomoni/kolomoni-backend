use std::{fs::File, iter::FusedIterator};

use csv::StringRecord;

use super::{optional_csv_column, required_csv_column, SeedSpreadsheetRowError};




pub struct SeedTranslationRow {
    pub english_lemma: String,
    pub english_meaning_disamgibuation: Option<String>,
    pub english_meaning_example: Option<String>,
    pub english_meaning_abbreviation: Option<String>,
    pub english_meaning_description: Option<String>,

    pub slovene_lemma: String,
    pub slovene_meaning_disambiguation: Option<String>,
    pub slovene_meaning_example: Option<String>,
    pub slovene_meaning_abbreviation: Option<String>,
    pub slovene_meaning_description: Option<String>,

    pub assigned_categories: Vec<String>,
}

impl SeedTranslationRow {
    fn try_from_csv_row(csv_record: &StringRecord) -> Result<Self, SeedSpreadsheetRowError> {
        // Order of columns:
        // - english word: lemma
        // - english word meaning: disambiguation
        // - english word meaning: example
        // - english word meaning: abbreviation
        // - english word meaning: description
        // - slovene word: lemma
        // - slovene word meaning: disambiguation
        // - slovene word meaning: example
        // - slovene word meaning: abbreviation
        // - slovene word meaning: description
        // - (3x) categories assigned to the meanings

        // We'd expect at least a slovene translation lemma to be present (the 6th column).
        // Google Drive CSV downloads seem to truncate the commas after the last non-empty column.
        if csv_record.len() < 6 {
            return Err(
                SeedSpreadsheetRowError::UnexpectedNumberOfColumns {
                    number_of_columns: csv_record.len(),
                },
            );
        }


        let english_lemma = required_csv_column!(csv_record => 0);

        let english_meaning_disamgibuation = optional_csv_column!(csv_record => 1);
        let english_meaning_example = optional_csv_column!(csv_record => 2);
        let english_meaning_abbreviation = optional_csv_column!(csv_record => 3);
        let english_meaning_description = optional_csv_column!(csv_record => 4);

        let slovene_lemma = required_csv_column!(csv_record => 5);

        let slovene_meaning_disambiguation = optional_csv_column!(csv_record => 6);
        let slovene_meaning_example = optional_csv_column!(csv_record => 7);
        let slovene_meaning_abbreviation = optional_csv_column!(csv_record => 8);
        let slovene_meaning_description = optional_csv_column!(csv_record => 9);


        let mut assigned_categories = Vec::new();
        for possible_category_column_index in 10usize..=12usize {
            if let Some(category_name) = csv_record.get(possible_category_column_index) {
                assigned_categories.push(category_name.to_owned());
            }
        }


        Ok(Self {
            english_lemma: english_lemma.to_owned(),
            english_meaning_disamgibuation: english_meaning_disamgibuation.map(str::to_owned),
            english_meaning_example: english_meaning_example.map(str::to_owned),
            english_meaning_abbreviation: english_meaning_abbreviation.map(str::to_owned),
            english_meaning_description: english_meaning_description.map(str::to_owned),
            slovene_lemma: slovene_lemma.to_owned(),
            slovene_meaning_disambiguation: slovene_meaning_disambiguation.map(str::to_owned),
            slovene_meaning_example: slovene_meaning_example.map(str::to_owned),
            slovene_meaning_abbreviation: slovene_meaning_abbreviation.map(str::to_owned),
            slovene_meaning_description: slovene_meaning_description.map(str::to_owned),
            assigned_categories,
        })
    }
}




pub struct SeedSpreadsheetTranslationsIter {
    translations_csv: csv::Reader<File>,
    csv_string_record: StringRecord,
}

impl SeedSpreadsheetTranslationsIter {
    pub(super) fn new(translations_csv: csv::Reader<File>) -> Self {
        Self {
            translations_csv,
            csv_string_record: StringRecord::new(),
        }
    }
}

impl Iterator for SeedSpreadsheetTranslationsIter {
    type Item = Result<SeedTranslationRow, SeedSpreadsheetRowError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.translations_csv.is_done() {
            return None;
        }


        match self
            .translations_csv
            .read_record(&mut self.csv_string_record)
        {
            Ok(success) => {
                if !success {
                    return None;
                }
            }
            Err(error) => {
                return Some(Err(SeedSpreadsheetRowError::CsvError { error }));
            }
        };

        Some(SeedTranslationRow::try_from_csv_row(
            &self.csv_string_record,
        ))
    }
}

impl FusedIterator for SeedSpreadsheetTranslationsIter {}
