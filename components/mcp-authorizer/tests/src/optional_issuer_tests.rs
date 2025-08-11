//! Tests for optional issuer validation

use spin_test_sdk::{
    bindings::{
        fermyon::spin_test_virt::variables,
        fermyon::spin_wasi_virt::http_handler,
        wasi::http::types,
    },
    spin_test,
};

use crate::test_token_utils::{TestKeyPair, TestTokenBuilder};


// Test: Issuer validation when issuer is configured
#[spin_test]
fn test_optional_issuer_validation_when_configured() {
    let key_pair = TestKeyPair::generate();
    
    // Configure JWT provider WITH issuer
    variables::set("mcp_provider_type", "jwt");
    variables::set("mcp_jwt_issuer", "https://expected.issuer.com");
    variables::set("mcp_jwt_audience", "test-api");
    variables::set("mcp_jwt_public_key", &key_pair.public_key_pem());
    variables::set("mcp_gateway_url", "none");
    
    // Test 1: Correct issuer should work
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
    
    let token = key_pair.create_token(
        TestTokenBuilder::new()
            .issuer("https://expected.issuer.com")  // Correct issuer
            .scopes(vec!["read"])
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
    assert_eq!(response.status(), 200, "Token with correct issuer should be accepted");
    
    // Test 2: Wrong issuer should fail
    let token = key_pair.create_token(
        TestTokenBuilder::new()
            .issuer("https://wrong.issuer.com")  // Wrong issuer
            .scopes(vec!["read"])
    );
    
    let headers = types::Headers::new();
    headers.append("authorization", format!("Bearer {}", token).as_bytes()).unwrap();
    let request = types::OutgoingRequest::new(headers);
    request.set_path_with_query(Some("/mcp")).unwrap();
    
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 401, "Token with wrong issuer should be rejected");
}

// Test: Empty issuer config means no validation
#[spin_test]
fn test_optional_issuer_empty_string() {
    let key_pair = TestKeyPair::generate();
    
    // Configure JWT provider with empty issuer string
    variables::set("mcp_provider_type", "jwt");
    variables::set("mcp_jwt_issuer", "");  // Empty string = no validation
    variables::set("mcp_jwt_audience", "test-api");
    variables::set("mcp_jwt_public_key", &key_pair.public_key_pem());
    variables::set("mcp_gateway_url", "none");
    
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
    
    // Token with any issuer should work
    let token = key_pair.create_token(
        TestTokenBuilder::new()
            .issuer("https://some.random.issuer.com")
            .scopes(vec!["read"])
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
    assert_eq!(response.status(), 200, "Token should be accepted when issuer is empty string");
}

// Test: String issuer (non-URL) support
#[spin_test]
fn test_optional_issuer_string_support() {
    let key_pair = TestKeyPair::generate();
    
    // Configure JWT provider with non-URL string issuer
    variables::set("mcp_provider_type", "jwt");
    variables::set("mcp_jwt_issuer", "my-service");  // Non-URL issuer
    variables::set("mcp_jwt_audience", "test-api");
    variables::set("mcp_jwt_public_key", &key_pair.public_key_pem());
    variables::set("mcp_gateway_url", "none");
    
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
    
    // Token with matching string issuer should work
    let token = key_pair.create_token(
        TestTokenBuilder::new()
            .issuer("my-service")  // Matching string issuer
            .scopes(vec!["read"])
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
    assert_eq!(response.status(), 200, "Token with matching string issuer should be accepted");
}