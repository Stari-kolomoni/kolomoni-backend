use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};



/// Returns the default configuration filepath, which is at
/// `{current directory}/data/configuration.toml`.
pub fn get_default_configuration_file_path() -> PathBuf {
    PathBuf::from("./data/configuration.toml")
}

#[must_use = "function returns the modified path"]
pub fn replace_placeholders_in_path(
    original_path: &Path,
    placeholders: HashMap<&'static str, String>,
) -> PathBuf {
    let mut path_string = original_path.to_string_lossy().to_string();

    for (key, value) in placeholders.into_iter() {
        path_string = path_string.replace(key, &value);
    }

    PathBuf::from(path_string)
}
