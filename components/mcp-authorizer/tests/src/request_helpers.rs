//! Helper functions for creating requests with headers in spin-test
//! 
//! Important: Headers must be set before passing to OutgoingRequest::new()
//! Getting headers from the request and modifying them doesn't work in spin-test.

use spin_test_sdk::bindings::wasi::http::types;

/// Create an OutgoingRequest with authorization header
pub fn create_request_with_auth(auth_token: &str) -> types::OutgoingRequest {
    let headers = types::Headers::new();
    headers.append("authorization", format!("Bearer {}", auth_token).as_bytes()).unwrap();
    types::OutgoingRequest::new(headers)
}

/// Create an OutgoingRequest with authorization and content-type headers
pub fn create_json_request_with_auth(auth_token: &str) -> types::OutgoingRequest {
    let headers = types::Headers::new();
    headers.append("authorization", format!("Bearer {}", auth_token).as_bytes()).unwrap();
    headers.append("content-type", b"application/json").unwrap();
    types::OutgoingRequest::new(headers)
}

/// Create an OutgoingRequest with custom headers
pub fn create_request_with_headers(header_pairs: &[(&str, &[u8])]) -> types::OutgoingRequest {
    let headers = types::Headers::new();
    for (name, value) in header_pairs {
        headers.append(name, value).unwrap();
    }
    types::OutgoingRequest::new(headers)
}