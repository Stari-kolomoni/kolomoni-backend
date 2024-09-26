use serde::Deserialize;

use crate::traits::Resolve;


pub(super) type UnresolvedSecretsConfiguration = SecretsConfiguration;

/// Password hashing-related configuration.
#[derive(Deserialize, Debug, Clone)]
pub struct SecretsConfiguration {
    pub hash_salt: String,
}

impl Resolve for UnresolvedSecretsConfiguration {
    type Resolved = SecretsConfiguration;

    fn resolve(self) -> Self::Resolved {
        self
    }
}
