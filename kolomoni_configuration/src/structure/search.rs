use std::path::{Path, PathBuf};

use miette::{miette, Context, IntoDiagnostic, Result};
use serde::Deserialize;

use super::base_paths::BasePathsConfiguration;
use crate::{traits::ResolvableConfigurationWithContext, utilities::replace_placeholders_in_path};

#[derive(Debug, Deserialize)]
pub(super) struct UnresolvedSearchConfiguration {
    pub(super) search_index_directory_path: String,
}

#[derive(Debug, Clone)]
pub struct SearchConfiguration {
    pub search_index_directory_path: PathBuf,
}

impl ResolvableConfigurationWithContext for UnresolvedSearchConfiguration {
    type Resolved = SearchConfiguration;
    type Context = BasePathsConfiguration;


    fn resolve(self, context: Self::Context) -> Result<Self::Resolved> {
        let search_index_directory_path = replace_placeholders_in_path(
            Path::new(&self.search_index_directory_path),
            context.placeholders(),
        );

        if search_index_directory_path.exists() && !search_index_directory_path.is_dir() {
            return Err(miette!(
                "Search index directory path exists, but is not a directory!"
            ));
        }

        if !search_index_directory_path.is_dir() {
            std::fs::create_dir_all(&search_index_directory_path)
                .into_diagnostic()
                .wrap_err("Failed to create missing search index directory.")?;
        }


        Ok(SearchConfiguration {
            search_index_directory_path,
        })
    }
}
