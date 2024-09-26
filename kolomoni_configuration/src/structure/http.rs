use serde::Deserialize;

use crate::traits::Resolve;


pub(crate) type UnresolvedHttpConfiguration = HttpConfiguration;

/// Actix HTTP server-related configuration.
#[derive(Deserialize, Debug, Clone)]
pub struct HttpConfiguration {
    /// Host to bind the HTTP server to.
    pub host: String,

    /// Port to bind the HTTP server to.
    pub port: usize,
}

impl Resolve for UnresolvedHttpConfiguration {
    type Resolved = HttpConfiguration;

    fn resolve(self) -> Self::Resolved {
        self
    }
}
