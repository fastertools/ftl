//! Tests for static token provider

use spin_test_sdk::{
    bindings::{
        fermyon::spin_test_virt::variables,
        fermyon::spin_wasi_virt::http_handler,
        wasi::http::types,
    },
    spin_test,
};

// Test: Basic static token authentication
#[spin_test]
fn test_static_token_auth() {
    // Configure static provider
    variables::set("mcp_provider_type", "static");
    variables::set("mcp_static_tokens", "dev-token:dev-app:dev-user:read,write");
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
    
    // Make request with static token
    let headers = types::Headers::new();
    headers.append("authorization", b"Bearer dev-token").unwrap();
    headers.append("content-type", b"application/json").unwrap();
    let request = types::OutgoingRequest::new(headers);
    request.set_path_with_query(Some("/mcp")).unwrap();
    request.set_method(&types::Method::Post).unwrap();
    
    let body = request.body().unwrap();
    body.write_bytes(b"{\"jsonrpc\":\"2.0\",\"method\":\"test\",\"id\":1}");
    
    let response = spin_test_sdk::perform_request(request);
    
    // Should succeed with valid static token
    assert_eq!(response.status(), 200);
}

// Test: Invalid static token
#[spin_test]
fn test_invalid_static_token() {
    // Configure static provider
    variables::set("mcp_provider_type", "static");
    variables::set("mcp_static_tokens", "dev-token:dev-app:dev-user:read,write");
    variables::set("mcp_gateway_url", "none");
    
    // Make request with invalid token
    let headers = types::Headers::new();
    headers.append("authorization", b"Bearer wrong-token").unwrap();
    let request = types::OutgoingRequest::new(headers);
    request.set_path_with_query(Some("/mcp")).unwrap();
    
    let response = spin_test_sdk::perform_request(request);
    
    // Should fail with 401
    assert_eq!(response.status(), 401);
}

// Test: Static token with required scopes
#[spin_test]
fn test_static_token_required_scopes() {
    // Configure static provider with required scopes
    variables::set("mcp_provider_type", "static");
    variables::set("mcp_static_tokens", "admin-token:admin-app:admin:admin,write;user-token:user-app:user:read");
    variables::set("mcp_jwt_required_scopes", "admin");
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
    
    // Test with admin token (has required scope)
    let headers = types::Headers::new();
    headers.append("authorization", b"Bearer admin-token").unwrap();
    headers.append("content-type", b"application/json").unwrap();
    let request = types::OutgoingRequest::new(headers);
    request.set_path_with_query(Some("/mcp")).unwrap();
    request.set_method(&types::Method::Post).unwrap();
    
    let body = request.body().unwrap();
    body.write_bytes(b"{\"jsonrpc\":\"2.0\",\"method\":\"test\",\"id\":1}");
    
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 200);
    
    // Test with user token (lacks required scope)
    let headers = types::Headers::new();
    headers.append("authorization", b"Bearer user-token").unwrap();
    let request = types::OutgoingRequest::new(headers);
    request.set_path_with_query(Some("/mcp")).unwrap();
    
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 401);
}

// Test: Multiple static tokens
#[spin_test]
fn test_multiple_static_tokens() {
    // Configure multiple tokens
    variables::set("mcp_provider_type", "static");
    variables::set("mcp_static_tokens", 
        "token1:app1:user1:read;token2:app2:user2:write;token3:app3:user3:read,write");
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
    
    // Test each token
    for token in ["token1", "token2", "token3"] {
        // Re-mock gateway for each iteration (spin test framework limitation)
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
        
        let headers = types::Headers::new();
        headers.append("authorization", format!("Bearer {}", token).as_bytes()).unwrap();
        headers.append("content-type", b"application/json").unwrap();
        let request = types::OutgoingRequest::new(headers);
        request.set_path_with_query(Some("/mcp")).unwrap();
        request.set_method(&types::Method::Post).unwrap();
        
        let body = request.body().unwrap();
        body.write_bytes(b"{\"jsonrpc\":\"2.0\",\"method\":\"test\",\"id\":1}");
        
        let response = spin_test_sdk::perform_request(request);
        assert_eq!(response.status(), 200, "Token {} should be valid", token);
    }
}

// Test: Static token with expiration
#[spin_test]
fn test_static_token_expiration() {
    use chrono::Utc;
    
    let future_exp = (Utc::now().timestamp() + 3600).to_string();
    let past_exp = (Utc::now().timestamp() - 3600).to_string();
    
    // Configure tokens with expiration
    variables::set("mcp_provider_type", "static");
    variables::set("mcp_static_tokens", 
        &format!("valid-token:app:user:read:{};expired-token:app:user:read:{}", future_exp, past_exp));
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
    
    // Test valid token
    let headers = types::Headers::new();
    headers.append("authorization", b"Bearer valid-token").unwrap();
    headers.append("content-type", b"application/json").unwrap();
    let request = types::OutgoingRequest::new(headers);
    request.set_path_with_query(Some("/mcp")).unwrap();
    request.set_method(&types::Method::Post).unwrap();
    
    let body = request.body().unwrap();
    body.write_bytes(b"{\"jsonrpc\":\"2.0\",\"method\":\"test\",\"id\":1}");
    
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 200);
    
    // Test expired token
    let headers = types::Headers::new();
    headers.append("authorization", b"Bearer expired-token").unwrap();
    let request = types::OutgoingRequest::new(headers);
    request.set_path_with_query(Some("/mcp")).unwrap();
    
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 401);
}