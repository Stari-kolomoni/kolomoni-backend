use std::{collections::HashMap, path::PathBuf};

use miette::{miette, Context, IntoDiagnostic, Result};
use serde::Deserialize;

use crate::traits::ResolvableConfiguration;

#[derive(Deserialize, Debug)]
pub(super) struct UnresolvedBasePathsConfiguration {
    pub(crate) base_data_directory_path: String,
}

#[derive(Debug, Clone)]
pub struct BasePathsConfiguration {
    pub base_data_directory_path: PathBuf,
}

impl ResolvableConfiguration for UnresolvedBasePathsConfiguration {
    type Resolved = BasePathsConfiguration;

    fn resolve(self) -> Result<Self::Resolved> {
        let base_data_directory_path = PathBuf::from(self.base_data_directory_path);

        if base_data_directory_path.exists() && !base_data_directory_path.is_dir() {
            return Err(miette!(
                "Base data directory path exists, but is not a directory!"
            ));
        }

        if !base_data_directory_path.is_dir() {
            std::fs::create_dir_all(&base_data_directory_path)
                .into_diagnostic()
                .wrap_err("Failed to create missing base data directory.")?;
        }


        let base_data_directory_path = dunce::canonicalize(base_data_directory_path)
            .into_diagnostic()
            .wrap_err("Failed to canonicalize base data directory path.")?;


        Ok(BasePathsConfiguration {
            base_data_directory_path,
        })
    }
}


impl BasePathsConfiguration {
    pub fn placeholders(&self) -> HashMap<&'static str, String> {
        let mut placeholders_map = HashMap::with_capacity(1);

        placeholders_map.insert(
            "{BASE_DATA_DIRECTORY}",
            self.base_data_directory_path.to_string_lossy().to_string(),
        );

        placeholders_map
    }
}
