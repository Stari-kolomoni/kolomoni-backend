use std::{fmt::Display, net::SocketAddr};

use thiserror::Error;
use tracing::warn;
use url::Url;


pub enum ServerHost {
    Ip(SocketAddr),
    DomainName(String),
}

impl Display for ServerHost {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServerHost::Ip(socket_addr) => socket_addr.fmt(f),
            ServerHost::DomainName(domain_name) => domain_name.fmt(f),
        }
    }
}

impl From<SocketAddr> for ServerHost {
    fn from(value: SocketAddr) -> Self {
        Self::Ip(value)
    }
}

impl From<String> for ServerHost {
    fn from(value: String) -> Self {
        Self::DomainName(value)
    }
}

impl From<&str> for ServerHost {
    fn from(value: &str) -> Self {
        Self::DomainName(value.to_string())
    }
}



pub struct ApiServerOptions {
    pub use_https: bool,
}

impl Default for ApiServerOptions {
    fn default() -> Self {
        Self { use_https: true }
    }
}



#[derive(Debug, Error)]
#[error("invalid server base URL: {}", .url)]
pub struct InvalidServerUrl {
    url: String,

    #[source]
    error: url::ParseError,
}


pub struct ApiServer {
    /// Contains only the server url (without any path segments),
    /// e.g. `http://127.0.0.1:8866`.
    server_url: Url,

    /// Contains the base API path as well,
    /// e.g. `http://127.0.0.1:8866/api/v1`.
    full_preconstructed_base_url: Url,
}

impl ApiServer {
    pub fn new_from_server_url<S>(
        server_host: S,
        options: ApiServerOptions,
    ) -> Result<Self, InvalidServerUrl>
    where
        S: Into<ServerHost>,
    {
        let protocol = match options.use_https {
            true => "https",
            false => "http",
        };

        let server_url_string = format!("{}://{}", protocol, server_host.into());
        let server_url = Url::parse(&server_url_string).map_err(|error| InvalidServerUrl {
            url: server_url_string.to_owned(),
            error,
        })?;

        let relative_base_api_path = "/api/v1/";

        let full_preconstructed_base_url =
            server_url
                .join(relative_base_api_path)
                .map_err(|error| InvalidServerUrl {
                    url: format!(
                        "{} (+ \"{}\")",
                        server_url_string, relative_base_api_path
                    ),
                    error,
                })?;

        Ok(Self {
            server_url,
            full_preconstructed_base_url,
        })
    }

    pub fn new_from_server_url_and_base_api_path(
        server_url: &str,
        relative_base_api_path: &str,
    ) -> Result<Self, InvalidServerUrl> {
        let server_url_parsed = Url::parse(server_url).map_err(|error| InvalidServerUrl {
            url: server_url.to_owned(),
            error,
        })?;

        if !(server_url_parsed.path().is_empty() || server_url_parsed.path() == "/") {
            warn!(
                "Provided server_url already contains a path (\"{}\"). This is most likely wrong.",
                server_url_parsed.path()
            );
        }


        let full_preconstructed_base_url =
            server_url_parsed
                .join(relative_base_api_path)
                .map_err(|error| InvalidServerUrl {
                    url: format!(
                        "{} (+ \"{}\")",
                        server_url_parsed, relative_base_api_path
                    ),
                    error,
                })?;


        Ok(Self {
            server_url: server_url_parsed,
            full_preconstructed_base_url,
        })
    }

    pub(crate) fn url_without_base_api_path(&self) -> &Url {
        &self.server_url
    }

    pub(crate) fn url_with_base_api_path(&self) -> &Url {
        &self.full_preconstructed_base_url
    }

    pub(crate) fn is_https(&self) -> bool {
        self.server_url.as_str().starts_with("https")
    }
}
