use std::fmt::Debug;

use actix_http::{
    header::{HeaderName, HeaderValue},
    StatusCode,
};
use bytes::Bytes;
use reqwest::{header::HeaderMap, Response};
use serde::Deserialize;

use crate::TestRequestDebugInfo;

#[derive(Debug)]
pub struct TestResponse {
    request_debug_info: TestRequestDebugInfo,

    status: StatusCode,
    headers: HeaderMap,
    body_bytes: Bytes,
}

impl TestResponse {
    pub(crate) async fn new(request_debug_info: TestRequestDebugInfo, response: Response) -> Self {
        Self {
            request_debug_info,
            status: response.status(),
            headers: response.headers().to_owned(),
            body_bytes: response
                .bytes()
                .await
                .expect("failed to extract body from response"),
        }
    }

    fn debug_format_for_panic(&self) -> String {
        format!(
            "\nContext:\n  request={:?}\n  response={{\n    status={},\n    headers={:?},\n    body={}\n  }}",
            self.request_debug_info,
            self.status,
            self.headers,
            String::from_utf8_lossy(&self.body_bytes)
        )
    }

    pub fn assert_status_equals(&self, status_code: StatusCode) {
        assert_eq!(
            self.status,
            status_code,
            "{}",
            self.debug_format_for_panic()
        );
    }

    pub fn assert_header_exists<N>(&self, header_name: N)
    where
        N: Into<HeaderName>,
    {
        let header_name: HeaderName = header_name.into();

        self.headers.get(&header_name).unwrap_or_else(|| {
            panic!(
                "header {} does not exist on response {}",
                header_name.as_str(),
                self.debug_format_for_panic()
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
                "header {} does not exist on response {}",
                header_name.as_str(),
                self.debug_format_for_panic()
            )
        });

        assert_eq!(
            expected_header_value,
            actual_header_value,
            "{}",
            self.debug_format_for_panic()
        );
    }

    pub fn assert_has_json_body<'de, D>(&'de self)
    where
        D: Deserialize<'de>,
    {
        serde_json::from_slice::<D>(&self.body_bytes).unwrap_or_else(|_| {
            panic!(
                "failed to deserialize body as JSON {}",
                self.debug_format_for_panic()
            )
        });
    }

    pub fn json_body<'de, D>(&'de self) -> D
    where
        D: Deserialize<'de>,
    {
        serde_json::from_slice::<D>(&self.body_bytes).unwrap_or_else(|_| {
            panic!(
                "failed to deserialize body as JSON {}",
                self.debug_format_for_panic()
            )
        })
    }

    pub fn assert_json_body_matches<'de, D>(&'de self, expected_content: D)
    where
        D: Deserialize<'de> + PartialEq + Eq + Debug,
    {
        let data = self.json_body::<D>();

        assert_eq!(
            data,
            expected_content,
            "{}",
            self.debug_format_for_panic()
        );
    }
}
