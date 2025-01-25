mod categories;
mod translations;
use std::{
    fs::File,
    path::{Path, PathBuf},
};

pub use categories::*;
use thiserror::Error;
pub use translations::*;


macro_rules! required_csv_column {
    ($csv_record:expr => $column_index:literal) => {
        $csv_record.get($column_index).ok_or_else(|| {
            SeedSpreadsheetRowError::RequiredColumnIsEmpty {
                column_index: $column_index,
            }
        })?
    };
}

use required_csv_column;

macro_rules! optional_csv_column {
    ($csv_record:expr => $column_index:literal) => {
        match $csv_record.get($column_index) {
            Some(value) => {
                if value.is_empty() {
                    None
                } else {
                    Some(value)
                }
            }
            None => None,
        }
    };
}

use optional_csv_column;


#[derive(Debug, Error)]
pub enum SeedSpreadsheetRowError {
    #[error("encountered unexpected number of columns: {}", .number_of_columns)]
    UnexpectedNumberOfColumns { number_of_columns: usize },

    #[error("column at index {} is required, but found empty", .column_index)]
    RequiredColumnIsEmpty { column_index: usize },

    #[error("uncategorized csv error")]
    CsvError {
        #[from]
        #[source]
        error: csv::Error,
    },
}




#[derive(Debug, Error)]
pub enum SeedSpreadsheetParserError {
    #[error("failed to open CSV file: {}", .csv_file_path.display())]
    UnableToOpen {
        csv_file_path: PathBuf,

        #[source]
        error: csv::Error,
    },
}


pub struct SeedSpreadsheetsParser {
    translations_csv: csv::Reader<File>,
    categories_csv: csv::Reader<File>,
}

impl SeedSpreadsheetsParser {
    pub fn from_csv_file_paths(
        translations_sheet_csv: &Path,
        categories_sheet_csv: &Path,
    ) -> Result<Self, SeedSpreadsheetParserError> {
        let translations_csv = csv::ReaderBuilder::new()
            .flexible(true)
            .has_headers(false)
            .from_path(translations_sheet_csv)
            .map_err(|error| SeedSpreadsheetParserError::UnableToOpen {
                csv_file_path: translations_sheet_csv.to_path_buf(),
                error,
            })?;

        let categories_csv = csv::ReaderBuilder::new()
            .flexible(true)
            .has_headers(false)
            .from_path(categories_sheet_csv)
            .map_err(|error| SeedSpreadsheetParserError::UnableToOpen {
                csv_file_path: categories_sheet_csv.to_path_buf(),
                error,
            })?;

        Ok(Self {
            translations_csv,
            categories_csv,
        })
    }

    pub fn into_iterators(
        self,
    ) -> (
        SeedSpreadsheetTranslationsIter,
        SeedSpreadsheetCategoriesIter,
    ) {
        (
            SeedSpreadsheetTranslationsIter::new(self.translations_csv),
            SeedSpreadsheetCategoriesIter::new(self.categories_csv),
        )
    }
}
