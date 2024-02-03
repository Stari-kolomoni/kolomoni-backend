use serde::Deserialize;

use crate::traits::ResolvableConfiguration;

pub(crate) type UnresolvedJsonWebTokenConfiguration = JsonWebTokenConfiguration;


/// JSON Web Token-related configuration.
#[derive(Deserialize, Debug, Clone)]
pub struct JsonWebTokenConfiguration {
    pub secret: String,
}

impl ResolvableConfiguration for UnresolvedJsonWebTokenConfiguration {
    type Resolved = JsonWebTokenConfiguration;

    fn resolve(self) -> miette::Result<Self::Resolved> {
        Ok(self)
    }
}
