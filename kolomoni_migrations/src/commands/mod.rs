use std::{
    borrow::Cow,
    env::{self, VarError},
};

use miette::{miette, Result};

pub(crate) mod down;
pub(crate) mod generate;
pub(crate) mod initialize;
pub(crate) mod up;


pub(crate) fn get_database_url_with_env_fallback<'a, S>(
    optional_database_url: Option<&'a S>,
    fallback_environment_variable_name: &str,
) -> Result<Option<Cow<'a, str>>>
where
    S: AsRef<str>,
{
    if let Some(database_url_override) = optional_database_url {
        return Ok(Some(Cow::from(database_url_override.as_ref())));
    }

    match env::var(fallback_environment_variable_name) {
        Ok(database_url_from_env) => Ok(Some(Cow::from(database_url_from_env))),
        Err(error) => match error {
            VarError::NotPresent => Ok(None),
            VarError::NotUnicode(_) => Err(miette!(
                "the {} environment variable is not valid Unicode",
                fallback_environment_variable_name
            )),
        },
    }
}
