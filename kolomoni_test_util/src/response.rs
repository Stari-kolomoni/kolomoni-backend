use std::fmt::Debug;

use actix_http::{
    header::{HeaderName, HeaderValue},
    StatusCode,
};
use bytes::Bytes;
use reqwest::{header::HeaderMap, Response};
use serde::Deserialize;

pub struct TestResponse {
    status: StatusCode,
    headers: HeaderMap,
    body_bytes: Bytes,
}

impl TestResponse {
    pub(crate) async fn from_reqwest_response(response: Response) -> Self {
        Self {
            status: response.status(),
            headers: response.headers().to_owned(),
            body_bytes: response
                .bytes()
                .await
                .expect("failed to extract body from response"),
        }
    }

    pub fn assert_status_equals(&self, status_code: StatusCode) {
        assert_eq!(self.status, status_code);
    }

    pub fn assert_header_exists<N>(&self, header_name: N)
    where
        N: Into<HeaderName>,
    {
        let header_name: HeaderName = header_name.into();

        self.headers.get(&header_name).unwrap_or_else(|| {
            panic!(
                "header {} does not exist on response",
                header_name.as_str()
            )
        });
    }

    pub fn assert_header_matches_value<N, V>(&self, header_name: N, header_value: V)
    where
        N: Into<HeaderName>,
        V: Into<HeaderValue>,
    {
        let header_name: HeaderName = header_name.into();
        let expected_header_value: HeaderValue = header_value.into();

        let actual_header_value = self.headers.get(&header_name).unwrap_or_else(|| {
            panic!(
                "header {} does not exist on response",
                header_name.as_str()
            )
        });

        assert_eq!(expected_header_value, actual_header_value);
    }

    pub fn json_body<'de, D>(&'de self) -> D
    where
        D: Deserialize<'de>,
    {
        serde_json::from_slice::<D>(&self.body_bytes).expect("failed to deserialize body as JSON")
    }

    pub fn assert_json_body_matches<'de, D>(&'de self, expected_content: D)
    where
        D: Deserialize<'de> + PartialEq + Eq + Debug,
    {
        let data = self.json_body::<D>();

        assert_eq!(data, expected_content);
    }
}
