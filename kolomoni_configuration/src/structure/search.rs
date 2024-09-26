use std::path::{Path, PathBuf};

use serde::Deserialize;

use super::base_paths::BasePathsConfiguration;
use crate::{
    traits::ResolveWithContext,
    utilities::replace_placeholders_in_path,
    MissingSearchIndexDirectoryCreationError,
};


#[derive(Debug, Deserialize)]
pub(super) struct UnresolvedSearchConfiguration {
    pub(super) search_index_directory_path: String,
}

#[derive(Debug, Clone)]
pub struct SearchConfiguration {
    pub search_index_directory_path: PathBuf,
}


impl<'r> ResolveWithContext<'r> for UnresolvedSearchConfiguration {
    type Resolved = SearchConfiguration;
    type Context = &'r BasePathsConfiguration;

    fn resolve_with_context(self, context: Self::Context) -> Self::Resolved {
        let search_index_directory_path = replace_placeholders_in_path(
            Path::new(&self.search_index_directory_path),
            context.placeholders(),
        );

        Self::Resolved {
            search_index_directory_path,
        }
    }
}

impl SearchConfiguration {
    pub fn create_search_index_directory_if_missing(
        &self,
    ) -> Result<(), MissingSearchIndexDirectoryCreationError> {
        if self.search_index_directory_path.exists() && !self.search_index_directory_path.is_dir() {
            return Err(
                MissingSearchIndexDirectoryCreationError::NotADirectory {
                    path: self.search_index_directory_path.clone(),
                },
            );
        }

        std::fs::create_dir_all(&self.search_index_directory_path).map_err(|error| {
            MissingSearchIndexDirectoryCreationError::UnableToCreateDirectory {
                directory_path: self.search_index_directory_path.clone(),
                error,
            }
        })
    }
}
