use serde::Deserialize;

use crate::traits::Resolve;


pub(crate) type UnresolvedJsonWebTokenConfiguration = JsonWebTokenConfiguration;


/// JSON Web Token-related configuration.
#[derive(Deserialize, Debug, Clone)]
pub struct JsonWebTokenConfiguration {
    pub secret: String,
}

impl Resolve for UnresolvedJsonWebTokenConfiguration {
    type Resolved = JsonWebTokenConfiguration;

    fn resolve(self) -> Self::Resolved {
        self
    }
}
