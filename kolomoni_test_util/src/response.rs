use kolomoni_api_client::response::ServerResponse;
use reqwest::{header::HeaderValue, StatusCode};


pub trait AssertableServerResponse {
    fn assert_status_equals(&self, status_code: StatusCode);

    fn assert_header_exists(&self, header_name: &str);

    fn assert_header_matches(&self, header_name: &str, header_value: HeaderValue);
}


fn format_debug_info_for_panic(server_response: &ServerResponse) -> String {
    let status = server_response.status();
    let headers = server_response.headers().to_owned();

    format!(
        "Context {{\n  \
          status={}\n  \
          headers={:?}\n\
        }}",
        status, headers
    )
}


impl AssertableServerResponse for ServerResponse {
    fn assert_status_equals(&self, status_code: StatusCode) {
        assert_eq!(
            self.status(),
            status_code,
            "{}",
            format_debug_info_for_panic(self)
        );
    }

    fn assert_header_exists(&self, header_name: &str) {
        let has_header = self.headers().contains_key(header_name);

        assert!(
            has_header,
            "Header {} is not present on response.\n{}",
            header_name,
            format_debug_info_for_panic(self)
        );
    }

    fn assert_header_matches(&self, header_name: &str, header_value: HeaderValue) {
        self.assert_header_exists(header_name);

        let actual_header_value = self
            .headers()
            .get(header_name)
            .expect("expected the header to exist");

        assert_eq!(
            header_value,
            actual_header_value,
            "Expected header {} differs from the actual value.\n\
            Expected: {}. Actual: {}.\n\
            {}",
            header_name,
            header_value.to_str().unwrap_or("[?non-ASCII?]"),
            actual_header_value.to_str().unwrap_or("[?non-ASCII?]"),
            format_debug_info_for_panic(self)
        );
    }
}
