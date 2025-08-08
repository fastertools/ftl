//! Tests demonstrating test utilities usage

use spin_test_sdk::{
    bindings::{
        fermyon::spin_test_virt::variables,
        fermyon::spin_wasi_virt::http_handler,
        wasi::http::types,
    },
    spin_test,
};

// Import test utilities from our local module
use crate::test_token_utils::{TestKeyPair, TestTokenBuilder, create_test_token, create_expired_token};

// Test: Using test utilities to create valid JWT tokens
#[spin_test]
fn test_jwt_with_test_utils() {
    
    // Generate a test key pair
    let key_pair = TestKeyPair::generate();
    
    // Configure JWT provider with test public key
    variables::set("mcp_provider_type", "jwt");
    variables::set("mcp_jwt_issuer", "https://test.example.com");
    variables::set("mcp_jwt_public_key", &key_pair.public_key_pem());
    variables::set("mcp_gateway_url", "https://test-gateway.spin.internal");
    
    // Mock gateway
    let response = types::OutgoingResponse::new(types::Headers::new());
    response.set_status_code(200).unwrap();
    let headers = response.headers();
    headers.append("content-type", b"application/json").unwrap();
    
    let body = response.body().unwrap();
    body.write_bytes(b"{\"jsonrpc\":\"2.0\",\"result\":{},\"id\":1}");
    
    http_handler::set_response(
        "https://test-gateway.spin.internal/mcp",
        http_handler::ResponseHandler::Response(response),
    );
    
    // Create a test token with scopes
    let token = create_test_token(&key_pair, vec!["read", "write"]);
    
    // Make request with test token
    let headers = types::Headers::new();
    headers.append("authorization", format!("Bearer {}", token).as_bytes()).unwrap();
    headers.append("content-type", b"application/json").unwrap();
    let request = types::OutgoingRequest::new(headers);
    request.set_path_with_query(Some("/mcp")).unwrap();
    request.set_method(&types::Method::Post).unwrap();
    
    let body = request.body().unwrap();
    body.write_bytes(b"{\"jsonrpc\":\"2.0\",\"method\":\"test\",\"id\":1}");
    
    let response = spin_test_sdk::perform_request(request);
    
    // Should succeed with valid test token
    assert_eq!(response.status(), 200);
}

// Test: Custom token builder with various claims
#[spin_test]
fn test_token_builder_features() {
    use chrono::Duration;
    
    let key_pair = TestKeyPair::generate();
    
    // Configure JWT provider
    variables::set("mcp_provider_type", "jwt");
    variables::set("mcp_jwt_issuer", "https://custom.issuer.com");
    variables::set("mcp_jwt_audience", "https://api.example.com");
    variables::set("mcp_jwt_public_key", &key_pair.public_key_pem());
    variables::set("mcp_gateway_url", "https://test-gateway.spin.internal");
    
    // Mock gateway
    let response = types::OutgoingResponse::new(types::Headers::new());
    response.set_status_code(200).unwrap();
    let headers = response.headers();
    headers.append("content-type", b"application/json").unwrap();
    
    let body = response.body().unwrap();
    body.write_bytes(b"{\"jsonrpc\":\"2.0\",\"result\":{},\"id\":1}");
    
    http_handler::set_response(
        "https://test-gateway.spin.internal/mcp",
        http_handler::ResponseHandler::Response(response),
    );
    
    // Create token with full customization
    let token = key_pair.create_token(
        TestTokenBuilder::new()
            .subject("custom-user-123")
            .issuer("https://custom.issuer.com")
            .audience("https://api.example.com")
            .scopes(vec!["admin", "api", "write"])
            .client_id("my-app")
            .expires_in(Duration::hours(2))
            .kid("test-key-1")
            .claim("department", serde_json::json!("engineering"))
            .claim("role", serde_json::json!("admin"))
    );
    
    // Make request with custom token
    let headers = types::Headers::new();
    headers.append("authorization", format!("Bearer {}", token).as_bytes()).unwrap();
    headers.append("content-type", b"application/json").unwrap();
    let request = types::OutgoingRequest::new(headers);
    request.set_path_with_query(Some("/mcp")).unwrap();
    request.set_method(&types::Method::Post).unwrap();
    
    let body = request.body().unwrap();
    body.write_bytes(b"{\"jsonrpc\":\"2.0\",\"method\":\"test\",\"id\":1}");
    
    let response = spin_test_sdk::perform_request(request);
    
    // Should succeed with custom token
    assert_eq!(response.status(), 200);
}

// Test: Microsoft-style scp claim support
#[spin_test]
fn test_microsoft_scp_claim() {
    
    let key_pair = TestKeyPair::generate();
    
    // Configure JWT provider
    variables::set("mcp_provider_type", "jwt");
    variables::set("mcp_jwt_issuer", "https://test.microsoft.com");
    variables::set("mcp_jwt_public_key", &key_pair.public_key_pem());
    variables::set("mcp_gateway_url", "https://test-gateway.spin.internal");
    
    // Mock gateway
    let response = types::OutgoingResponse::new(types::Headers::new());
    response.set_status_code(200).unwrap();
    let headers = response.headers();
    headers.append("content-type", b"application/json").unwrap();
    
    let body = response.body().unwrap();
    body.write_bytes(b"{\"jsonrpc\":\"2.0\",\"result\":{},\"id\":1}");
    
    http_handler::set_response(
        "https://test-gateway.spin.internal/mcp",
        http_handler::ResponseHandler::Response(response),
    );
    
    // Test with scp as string
    let token = key_pair.create_token(
        TestTokenBuilder::new()
            .issuer("https://test.microsoft.com")
            .scp_string("user.read mail.read")
    );
    
    let headers = types::Headers::new();
    headers.append("authorization", format!("Bearer {}", token).as_bytes()).unwrap();
    headers.append("content-type", b"application/json").unwrap();
    let request = types::OutgoingRequest::new(headers);
    request.set_path_with_query(Some("/mcp")).unwrap();
    request.set_method(&types::Method::Post).unwrap();
    
    let body = request.body().unwrap();
    body.write_bytes(b"{\"jsonrpc\":\"2.0\",\"method\":\"test\",\"id\":1}");
    
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 200);
    
    // Re-mock gateway for next test
    let response = types::OutgoingResponse::new(types::Headers::new());
    response.set_status_code(200).unwrap();
    let headers = response.headers();
    headers.append("content-type", b"application/json").unwrap();
    
    let body = response.body().unwrap();
    body.write_bytes(b"{\"jsonrpc\":\"2.0\",\"result\":{},\"id\":1}");
    
    http_handler::set_response(
        "https://test-gateway.spin.internal/mcp",
        http_handler::ResponseHandler::Response(response),
    );
    
    // Test with scp as array
    let token = key_pair.create_token(
        TestTokenBuilder::new()
            .issuer("https://test.microsoft.com")
            .scp_array(vec!["user.read", "mail.read"])
    );
    
    let headers = types::Headers::new();
    headers.append("authorization", format!("Bearer {}", token).as_bytes()).unwrap();
    headers.append("content-type", b"application/json").unwrap();
    let request = types::OutgoingRequest::new(headers);
    request.set_path_with_query(Some("/mcp")).unwrap();
    request.set_method(&types::Method::Post).unwrap();
    
    let body = request.body().unwrap();
    body.write_bytes(b"{\"jsonrpc\":\"2.0\",\"method\":\"test\",\"id\":1}");
    
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 200);
}

// Test: Expired token creation
#[spin_test]
fn test_expired_token_creation() {
    
    let key_pair = TestKeyPair::generate();
    
    // Configure JWT provider
    variables::set("mcp_provider_type", "jwt");
    variables::set("mcp_jwt_issuer", "https://test.example.com");
    variables::set("mcp_jwt_public_key", &key_pair.public_key_pem());
    variables::set("mcp_gateway_url", "https://test-gateway.spin.internal");
    
    // Create an expired token
    let token = create_expired_token(&key_pair);
    
    // Make request with expired token
    let headers = types::Headers::new();
    headers.append("authorization", format!("Bearer {}", token).as_bytes()).unwrap();
    let request = types::OutgoingRequest::new(headers);
    request.set_path_with_query(Some("/mcp")).unwrap();
    
    let response = spin_test_sdk::perform_request(request);
    
    // Should fail with 401
    assert_eq!(response.status(), 401);
}

// Test: Multiple audiences
#[spin_test]
fn test_token_utils_multiple_audiences() {
    
    let key_pair = TestKeyPair::generate();
    
    // Configure JWT provider with specific audience
    variables::set("mcp_provider_type", "jwt");
    variables::set("mcp_jwt_issuer", "https://test.example.com");
    variables::set("mcp_jwt_audience", "https://api.example.com");
    variables::set("mcp_jwt_public_key", &key_pair.public_key_pem());
    variables::set("mcp_gateway_url", "https://test-gateway.spin.internal");
    
    // Mock gateway
    let response = types::OutgoingResponse::new(types::Headers::new());
    response.set_status_code(200).unwrap();
    let headers = response.headers();
    headers.append("content-type", b"application/json").unwrap();
    
    let body = response.body().unwrap();
    body.write_bytes(b"{\"jsonrpc\":\"2.0\",\"result\":{},\"id\":1}");
    
    http_handler::set_response(
        "https://test-gateway.spin.internal/mcp",
        http_handler::ResponseHandler::Response(response),
    );
    
    // Create token with multiple audiences
    let token = key_pair.create_token(
        TestTokenBuilder::new()
            .issuer("https://test.example.com")
            .audiences(vec![
                "https://api.example.com".to_string(),
                "https://other.example.com".to_string(),
            ])
            .scopes(vec!["read"])
    );
    
    // Make request
    let headers = types::Headers::new();
    headers.append("authorization", format!("Bearer {}", token).as_bytes()).unwrap();
    headers.append("content-type", b"application/json").unwrap();
    let request = types::OutgoingRequest::new(headers);
    request.set_path_with_query(Some("/mcp")).unwrap();
    request.set_method(&types::Method::Post).unwrap();
    
    let body = request.body().unwrap();
    body.write_bytes(b"{\"jsonrpc\":\"2.0\",\"method\":\"test\",\"id\":1}");
    
    let response = spin_test_sdk::perform_request(request);
    
    // Should succeed as one of the audiences matches
    assert_eq!(response.status(), 200);
}

// Test: Combining test utils with scope validation
#[spin_test]
fn test_utils_with_scope_validation() {
    
    let key_pair = TestKeyPair::generate();
    
    // Configure JWT provider with required scopes
    variables::set("mcp_provider_type", "jwt");
    variables::set("mcp_jwt_issuer", "https://test.example.com");
    variables::set("mcp_jwt_public_key", &key_pair.public_key_pem());
    variables::set("mcp_jwt_required_scopes", "admin,write");
    variables::set("mcp_gateway_url", "https://test-gateway.spin.internal");
    
    // Mock gateway
    let response = types::OutgoingResponse::new(types::Headers::new());
    response.set_status_code(200).unwrap();
    let headers = response.headers();
    headers.append("content-type", b"application/json").unwrap();
    
    let body = response.body().unwrap();
    body.write_bytes(b"{\"jsonrpc\":\"2.0\",\"result\":{},\"id\":1}");
    
    http_handler::set_response(
        "https://test-gateway.spin.internal/mcp",
        http_handler::ResponseHandler::Response(response),
    );
    
    // Create token with all required scopes
    let token = key_pair.create_token(
        TestTokenBuilder::new()
            .issuer("https://test.example.com")
            .scopes(vec!["admin", "write", "read"])  // Has all required scopes
    );
    
    let headers = types::Headers::new();
    headers.append("authorization", format!("Bearer {}", token).as_bytes()).unwrap();
    headers.append("content-type", b"application/json").unwrap();
    let request = types::OutgoingRequest::new(headers);
    request.set_path_with_query(Some("/mcp")).unwrap();
    request.set_method(&types::Method::Post).unwrap();
    
    let body = request.body().unwrap();
    body.write_bytes(b"{\"jsonrpc\":\"2.0\",\"method\":\"test\",\"id\":1}");
    
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 200);
    
    // Re-mock gateway for second test
    let response = types::OutgoingResponse::new(types::Headers::new());
    response.set_status_code(200).unwrap();
    let headers = response.headers();
    headers.append("content-type", b"application/json").unwrap();
    
    let body = response.body().unwrap();
    body.write_bytes(b"{\"jsonrpc\":\"2.0\",\"result\":{},\"id\":1}");
    
    http_handler::set_response(
        "https://test-gateway.spin.internal/mcp",
        http_handler::ResponseHandler::Response(response),
    );
    
    // Test with missing required scope
    let token = key_pair.create_token(
        TestTokenBuilder::new()
            .issuer("https://test.example.com")
            .scopes(vec!["read", "write"])  // Missing "admin" scope
    );
    
    let headers = types::Headers::new();
    headers.append("authorization", format!("Bearer {}", token).as_bytes()).unwrap();
    let request = types::OutgoingRequest::new(headers);
    request.set_path_with_query(Some("/mcp")).unwrap();
    
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 401);
}