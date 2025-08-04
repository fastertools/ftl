use spin_test_sdk::{
    bindings::wasi::http,
    spin_test,
};
use crate::{ResponseData, test_helpers::*};

#[spin_test]
fn test_cors_preflight_options() {
    setup_default_test_env();
    
    let headers = http::types::Headers::new();
    headers.append("origin", b"https://example.com").unwrap();
    headers.append("access-control-request-method", b"POST").unwrap();
    headers.append("access-control-request-headers", b"content-type").unwrap();
    
    let request = http::types::OutgoingRequest::new(headers);
    request.set_method(&http::types::Method::Options).unwrap();
    request.set_path_with_query(Some("/mcp")).unwrap();
    
    let response = spin_test_sdk::perform_request(request);
    let response_data = ResponseData::from_response(response);
    
    // Should return 200 for OPTIONS
    assert_eq!(response_data.status, 200);
    
    // Check CORS headers
    assert_eq!(
        response_data.find_header("access-control-allow-origin"),
        Some(&b"*".to_vec())
    );
    assert_eq!(
        response_data.find_header("access-control-allow-methods"),
        Some(&b"POST, OPTIONS".to_vec())
    );
    assert_eq!(
        response_data.find_header("access-control-allow-headers"),
        Some(&b"Content-Type".to_vec())
    );
}

#[spin_test]
fn test_cors_headers_on_post_request() {
    setup_default_test_env();
    
    let request_json = create_json_rpc_request("ping", None, Some(serde_json::json!(1)));
    let headers = http::types::Headers::new();
    headers.append("content-type", b"application/json").unwrap();
    headers.append("origin", b"https://example.com").unwrap();
    
    let request = http::types::OutgoingRequest::new(headers);
    request.set_method(&http::types::Method::Post).unwrap();
    request.set_path_with_query(Some("/mcp")).unwrap();
    
    let request_body_bytes = serde_json::to_vec(&request_json).unwrap();
    let body = request.body().unwrap();
    body.write_bytes(&request_body_bytes);
    
    let response = spin_test_sdk::perform_request(request);
    let response_data = ResponseData::from_response(response);
    
    assert_eq!(response_data.status, 200);
    
    // Verify CORS header is present on actual requests
    assert_eq!(
        response_data.find_header("access-control-allow-origin"),
        Some(&b"*".to_vec())
    );
}

#[spin_test]
fn test_cors_headers_on_error_response() {
    setup_default_test_env();
    
    // Send invalid request to trigger error
    let headers = http::types::Headers::new();
    headers.append("content-type", b"application/json").unwrap();
    headers.append("origin", b"https://example.com").unwrap();
    
    let request = http::types::OutgoingRequest::new(headers);
    request.set_method(&http::types::Method::Post).unwrap();
    request.set_path_with_query(Some("/mcp")).unwrap();
    
    let body = request.body().unwrap();
    body.write_bytes(b"invalid json");
    
    let response = spin_test_sdk::perform_request(request);
    let response_data = ResponseData::from_response(response);
    
    assert_eq!(response_data.status, 200);
    
    // CORS headers should be present even on error responses
    assert_eq!(
        response_data.find_header("access-control-allow-origin"),
        Some(&b"*".to_vec())
    );
}

#[spin_test]
fn test_options_request_without_cors_headers() {
    setup_default_test_env();
    
    // OPTIONS request without CORS headers (non-CORS preflight)
    let request = http::types::OutgoingRequest::new(http::types::Headers::new());
    request.set_method(&http::types::Method::Options).unwrap();
    request.set_path_with_query(Some("/mcp")).unwrap();
    
    let response = spin_test_sdk::perform_request(request);
    let response_data = ResponseData::from_response(response);
    
    // Should still return 200 and include CORS headers
    assert_eq!(response_data.status, 200);
    assert!(response_data.find_header("access-control-allow-origin").is_some());
}

#[spin_test]
fn test_non_post_methods_rejected() {
    setup_default_test_env();
    
    // Test GET request
    let request = http::types::OutgoingRequest::new(http::types::Headers::new());
    request.set_method(&http::types::Method::Get).unwrap();
    request.set_path_with_query(Some("/mcp")).unwrap();
    
    let response = spin_test_sdk::perform_request(request);
    let response_data = ResponseData::from_response(response);
    
    assert_eq!(response_data.status, 405);
    assert_eq!(
        response_data.find_header("allow"),
        Some(&b"POST, OPTIONS".to_vec())
    );
    
    // Test PUT request
    let request = http::types::OutgoingRequest::new(http::types::Headers::new());
    request.set_method(&http::types::Method::Put).unwrap();
    request.set_path_with_query(Some("/mcp")).unwrap();
    
    let response = spin_test_sdk::perform_request(request);
    let response_data = ResponseData::from_response(response);
    
    assert_eq!(response_data.status, 405);
    
    // Test DELETE request
    let request = http::types::OutgoingRequest::new(http::types::Headers::new());
    request.set_method(&http::types::Method::Delete).unwrap();
    request.set_path_with_query(Some("/mcp")).unwrap();
    
    let response = spin_test_sdk::perform_request(request);
    let response_data = ResponseData::from_response(response);
    
    assert_eq!(response_data.status, 405);
}