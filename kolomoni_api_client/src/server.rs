use std::{fmt::Display, net::SocketAddr};

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


pub struct ApiServer {
    base_api_url: String,
}

impl ApiServer {
    pub fn new<S>(server_host: S, options: ApiServerOptions) -> Self
    where
        S: Into<ServerHost>,
    {
        let protocol = match options.use_https {
            true => "https",
            false => "http",
        };

        Self {
            base_api_url: format!("{}://{}/api/v1", protocol, server_host.into()),
        }
    }

    pub(crate) fn base_url(&self) -> &str {
        &self.base_api_url
    }
}
