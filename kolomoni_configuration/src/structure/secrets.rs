use serde::Deserialize;

use crate::traits::ResolvableConfiguration;

pub(super) type UnresolvedSecretsConfiguration = SecretsConfiguration;

/// Password hashing-related configuration.
#[derive(Deserialize, Debug, Clone)]
pub struct SecretsConfiguration {
    pub hash_salt: String,
}

impl ResolvableConfiguration for UnresolvedSecretsConfiguration {
    type Resolved = SecretsConfiguration;

    fn resolve(self) -> miette::Result<Self::Resolved> {
        Ok(self)
    }
}
