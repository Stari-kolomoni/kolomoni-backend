use std::{collections::HashMap, path::PathBuf};

use serde::Deserialize;

use crate::{traits::Resolve, MissingBasePathCreationError};


#[derive(Deserialize, Debug)]
pub(super) struct UnresolvedBasePathsConfiguration {
    pub(crate) base_data_directory_path: String,
}



#[derive(Debug, Clone)]
pub struct BasePathsConfiguration {
    pub base_data_directory_path: PathBuf,
}


impl Resolve for UnresolvedBasePathsConfiguration {
    type Resolved = BasePathsConfiguration;

    fn resolve(self) -> Self::Resolved {
        let base_data_directory_path = PathBuf::from(self.base_data_directory_path);

        Self::Resolved {
            base_data_directory_path,
        }
    }
}


impl BasePathsConfiguration {
    pub fn create_base_data_directory_if_missing(&self) -> Result<(), MissingBasePathCreationError> {
        if self.base_data_directory_path.exists() && !self.base_data_directory_path.is_dir() {
            return Err(MissingBasePathCreationError::NotADirectory {
                path: self.base_data_directory_path.clone(),
            });
        }

        std::fs::create_dir_all(&self.base_data_directory_path).map_err(|error| {
            MissingBasePathCreationError::UnableToCreateDirectory {
                directory_path: self.base_data_directory_path.clone(),
                error,
            }
        })
    }

    pub fn placeholders(&self) -> HashMap<&'static str, String> {
        let mut placeholders_map = HashMap::with_capacity(1);

        placeholders_map.insert(
            "{BASE_DATA_DIRECTORY}",
            self.base_data_directory_path.to_string_lossy().to_string(),
        );

        placeholders_map
    }
}
