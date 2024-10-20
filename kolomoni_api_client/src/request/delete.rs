use url::Url;

use super::build_request_url;
use crate::{
    errors::{ClientError, ClientResult},
    response::ServerResponse,
    HttpClient,
};

pub(crate) struct DeleteRequestBuilder<'c, HC, const HAS_URL: bool>
where
    HC: HttpClient,
{
    client: &'c HC,
    url: Option<Result<Url, url::ParseError>>,
}

impl<'c, HC, const HAS_URL: bool> DeleteRequestBuilder<'c, HC, HAS_URL>
where
    HC: HttpClient,
{
    pub(crate) fn new(client: &'c HC) -> DeleteRequestBuilder<'c, HC, false> {
        DeleteRequestBuilder { client, url: None }
    }

    pub(crate) fn endpoint_url<U>(
        self,
        relative_endpoint_url: U,
    ) -> DeleteRequestBuilder<'c, HC, true>
    where
        U: AsRef<str>,
    {
        DeleteRequestBuilder {
            client: self.client,
            url: Some(build_request_url(
                self.client.server(),
                relative_endpoint_url.as_ref(),
            )),
        }
    }
}

impl<'c, HC> DeleteRequestBuilder<'c, HC, true>
where
    HC: HttpClient,
{
    pub(crate) async fn send(self) -> ClientResult<ServerResponse> {
        // PANIC SAFETY: `url` field is `Some` when `HasUrl` const generic is `true`.
        let request_url = match self.url.unwrap() {
            Ok(request_url) => request_url,
            Err(url_parse_error) => {
                return Err(ClientError::UrlPreparationError {
                    error: url_parse_error,
                })
            }
        };

        self.client.delete(request_url).await
    }
}
