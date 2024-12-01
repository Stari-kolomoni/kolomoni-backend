use std::{fs::File, iter::FusedIterator};

use csv::StringRecord;

use super::SeedSpreadsheetRowError;
use crate::parser::{optional_csv_column, required_csv_column};


pub struct SeedCategoryRow {
    pub english_name: String,
    pub slovene_name: String,
    pub english_description: Option<String>,
    pub slovene_description: Option<String>,
    pub parent_category_english_name: Option<String>,
}

impl SeedCategoryRow {
    fn try_from_csv_row(csv_record: &StringRecord) -> Result<Self, SeedSpreadsheetRowError> {
        // Order of columns:
        // - english name
        // - slovene name
        // - english description (optional)
        // - slovene description (optional)
        // - parent category (by english name, optional)


        // At least the english and slovene name must be provided.
        if csv_record.len() < 2 {
            return Err(
                SeedSpreadsheetRowError::UnexpectedNumberOfColumns {
                    number_of_columns: csv_record.len(),
                },
            );
        }


        let english_name = required_csv_column!(csv_record => 0);
        let slovene_name = required_csv_column!(csv_record => 1);

        let english_description = optional_csv_column!(csv_record => 2);
        let slovene_description = optional_csv_column!(csv_record => 3);

        let parent_category_english_name = optional_csv_column!(csv_record => 4);


        Ok(Self {
            english_name: english_name.to_owned(),
            slovene_name: slovene_name.to_owned(),
            english_description: english_description.map(str::to_owned),
            slovene_description: slovene_description.map(str::to_owned),
            parent_category_english_name: parent_category_english_name.map(str::to_owned),
        })
    }
}



pub struct SeedSpreadsheetCategoriesIter {
    categories_csv: csv::Reader<File>,
    csv_string_record: StringRecord,
}


impl SeedSpreadsheetCategoriesIter {
    pub(super) fn new(categories_csv: csv::Reader<File>) -> Self {
        Self {
            categories_csv,
            csv_string_record: StringRecord::new(),
        }
    }
}

impl Iterator for SeedSpreadsheetCategoriesIter {
    type Item = Result<SeedCategoryRow, SeedSpreadsheetRowError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.categories_csv.is_done() {
            return None;
        }


        match self.categories_csv.read_record(&mut self.csv_string_record) {
            Ok(success) => {
                if !success {
                    return None;
                }
            }
            Err(error) => {
                return Some(Err(SeedSpreadsheetRowError::CsvError { error }));
            }
        };

        Some(SeedCategoryRow::try_from_csv_row(
            &self.csv_string_record,
        ))
    }
}

impl FusedIterator for SeedSpreadsheetCategoriesIter {}
