use spin_test_sdk::{
    bindings::{
        fermyon::spin_test_virt::variables,
        wasi::http
    },
    spin_test,
};
use serde_json::Value;

// Test error response format for missing token
#[spin_test]
fn test_missing_token_error_format() {
    let request = http::types::OutgoingRequest::new(http::types::Headers::new());
    request.set_path_with_query(Some("/mcp")).unwrap();
    let response = spin_test_sdk::perform_request(request);
    
    assert_eq!(response.status(), 401);
    
    // Check WWW-Authenticate header format
    let headers = response.headers();
    let entries = headers.entries();
    let www_auth = entries.iter()
        .find(|(name, _)| name == "www-authenticate")
        .map(|(_, value)| String::from_utf8_lossy(value));
    
    assert!(www_auth.is_some());
    let auth_header = www_auth.unwrap();
    
    // Should contain Bearer scheme with error details
    assert!(auth_header.starts_with("Bearer"));
    assert!(auth_header.contains("error=\"unauthorized\""));
    assert!(auth_header.contains("error_description=\"Missing authorization header\""));
    
    // Check response body
    let body = crate::read_body(&response);
    if !body.is_empty() {
        let json: Result<Value, _> = serde_json::from_slice(&body);
        if let Ok(json) = json {
            assert_eq!(json["error"], "unauthorized");
            assert_eq!(json["error_description"], "Missing authorization header");
        }
    }
}

// Test error response for invalid token
#[spin_test]
fn test_invalid_token_error_format() {
    let headers = http::types::Headers::new();
    headers.append("authorization", b"Bearer invalid.token.here").unwrap();
    
    let request = http::types::OutgoingRequest::new(headers);
    request.set_path_with_query(Some("/mcp")).unwrap();
    let response = spin_test_sdk::perform_request(request);
    
    assert_eq!(response.status(), 401);
    
    // Check WWW-Authenticate header
    let headers = response.headers();
    let entries = headers.entries();
    let www_auth = entries.iter()
        .find(|(name, _)| name == "www-authenticate")
        .map(|(_, value)| String::from_utf8_lossy(value));
    
    assert!(www_auth.is_some());
    let auth_header = www_auth.unwrap();
    
    assert!(auth_header.contains("error=\"invalid_token\""));
}

// Test error response includes resource metadata URL when host is present
#[spin_test]
fn test_error_includes_resource_metadata_url() {
    let headers = http::types::Headers::new();
    headers.append("host", b"api.example.com").unwrap();
    
    let request = http::types::OutgoingRequest::new(headers);
    request.set_path_with_query(Some("/mcp")).unwrap();
    let response = spin_test_sdk::perform_request(request);
    
    assert_eq!(response.status(), 401);
    
    // Check WWW-Authenticate header includes resource metadata
    let headers = response.headers();
    let entries = headers.entries();
    let www_auth = entries.iter()
        .find(|(name, _)| name == "www-authenticate")
        .map(|(_, value)| String::from_utf8_lossy(value));
    
    assert!(www_auth.is_some());
    let auth_header = www_auth.unwrap();
    
    assert!(auth_header.contains("resource_metadata=\"https://api.example.com/.well-known/oauth-protected-resource\""));
}

// Test error response without host header
#[spin_test]
fn test_error_without_host() {
    let request = http::types::OutgoingRequest::new(http::types::Headers::new());
    request.set_path_with_query(Some("/mcp")).unwrap();
    let response = spin_test_sdk::perform_request(request);
    
    assert_eq!(response.status(), 401);
    
    // Check WWW-Authenticate header doesn't include resource metadata
    let headers = response.headers();
    let entries = headers.entries();
    let www_auth = entries.iter()
        .find(|(name, _)| name == "www-authenticate")
        .map(|(_, value)| String::from_utf8_lossy(value));
    
    assert!(www_auth.is_some());
    let auth_header = www_auth.unwrap();
    
    // Should not contain resource_metadata without host
    assert!(!auth_header.contains("resource_metadata="));
}

// Test JSON error response content type
#[spin_test]
fn test_error_json_content_type() {
    let request = http::types::OutgoingRequest::new(http::types::Headers::new());
    request.set_path_with_query(Some("/mcp")).unwrap();
    let response = spin_test_sdk::perform_request(request);
    
    assert_eq!(response.status(), 401);
    
    // Check content type
    let headers = response.headers();
    let entries = headers.entries();
    let content_type = entries.iter()
        .find(|(name, _)| name == "content-type")
        .map(|(_, value)| String::from_utf8_lossy(value));
    
    assert!(content_type.is_some());
    assert!(content_type.unwrap().contains("application/json"));
}

// Test internal server error format
#[spin_test]
fn test_internal_error_format() {
    // Configure without required issuer to trigger internal error
    // Clear the issuer to trigger error
    variables::set("mcp_jwt_issuer", "");
    
    let request = http::types::OutgoingRequest::new(http::types::Headers::new());
    request.set_path_with_query(Some("/mcp")).unwrap();
    let response = spin_test_sdk::perform_request(request);
    
    // Should return 500 for configuration error
    assert_eq!(response.status(), 500);
}

// Test malformed authorization header
#[spin_test]
fn test_malformed_auth_header() {
    let test_cases = vec![
        "NotBearer token",
        "Bearer",  // Missing token
        "Bearer  ", // Empty token
        "Token abc123", // Wrong scheme
    ];
    
    for auth_value in test_cases {
        let headers = http::types::Headers::new();
        headers.append("authorization", auth_value.as_bytes()).unwrap();
        
        let request = http::types::OutgoingRequest::new(headers);
        request.set_path_with_query(Some("/mcp")).unwrap();
        let response = spin_test_sdk::perform_request(request);
        
        assert_eq!(response.status(), 401);
    }
}

// Test trace ID propagation in error responses
#[spin_test]
fn test_error_trace_id_propagation() {
    let headers = http::types::Headers::new();
    headers.append("x-trace-id", b"error-trace-123").unwrap();
    
    let request = http::types::OutgoingRequest::new(headers);
    request.set_path_with_query(Some("/mcp")).unwrap();
    let response = spin_test_sdk::perform_request(request);
    
    assert_eq!(response.status(), 401);
    
    // Check trace ID is preserved in response
    let response_headers = response.headers();
    let entries = response_headers.entries();
    let trace_id = entries.iter()
        .find(|(name, _)| name == "x-trace-id")
        .map(|(_, value)| String::from_utf8_lossy(value));
    
    assert_eq!(trace_id, Some("error-trace-123".into()));
}

// Test various invalid bearer tokens
#[spin_test]
fn test_various_invalid_tokens() {
    let invalid_tokens = vec![
        "not.a.jwt",
        "too.many.parts.here.invalid",
        "invalid-token",
        "", // Empty token
        "header.payload", // Missing signature
        "header.payload.signature.extra", // Too many parts
    ];
    
    for token in invalid_tokens {
        let headers = http::types::Headers::new();
        headers.append("authorization", format!("Bearer {}", token).as_bytes()).unwrap();
        
        let request = http::types::OutgoingRequest::new(headers);
        request.set_path_with_query(Some("/mcp")).unwrap();
        let response = spin_test_sdk::perform_request(request);
        
        assert_eq!(response.status(), 401);
        
        // Check for invalid_token error
        let headers = response.headers();
        let entries = headers.entries();
        let www_auth = entries.iter()
            .find(|(name, _)| name == "www-authenticate")
            .map(|(_, value)| String::from_utf8_lossy(value));
        
        assert!(www_auth.is_some());
        assert!(www_auth.unwrap().contains("error=\"invalid_token\""));
    }
}