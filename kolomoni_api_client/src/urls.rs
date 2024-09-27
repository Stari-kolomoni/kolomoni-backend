use url::Url;

use crate::server::ApiServer;

pub(crate) fn build_request_url(server: &ApiServer, endpoint: &str) -> Result<Url, url::ParseError> {
    if !endpoint.starts_with('/') {
        Url::parse(&format!("{}/{}", server.base_url(), endpoint))
    } else {
        Url::parse(&format!("{}{}", server.base_url(), endpoint))
    }
}
