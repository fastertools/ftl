//! Tests for tenant validation in private mode

use spin_test_sdk::{
    bindings::{
        fermyon::spin_test_virt::variables,
        fermyon::spin_wasi_virt::http_handler,
        wasi::http::types,
    },
    spin_test,
};

use crate::test_token_utils::{TestKeyPair, TestTokenBuilder};

/// Test tenant validation when mcp_tenant_id is set
#[spin_test]
fn test_tenant_validation_with_org_id() {
    // Set up test key pair
    let key_pair = TestKeyPair::generate();
    
    // Configure provider with public key
    variables::set("mcp_gateway_url", "https://test-gateway.spin.internal/mcp-internal");
    variables::set("mcp_jwt_issuer", "https://test.authkit.app");
    variables::set("mcp_jwt_public_key", &key_pair.public_key_pem());
    
    // Set tenant ID (simulating private mode)
    variables::set("mcp_tenant_id", "org_12345");
    
    // Mock gateway response
    let gateway_response = types::OutgoingResponse::new(types::Headers::new());
    gateway_response.set_status_code(200).unwrap();
    let headers = gateway_response.headers();
    headers.append("content-type", b"application/json").unwrap();
    let body = gateway_response.body().unwrap();
    body.write_bytes(b"{\"jsonrpc\":\"2.0\",\"result\":{},\"id\":1}");
    http_handler::set_response(
        "https://test-gateway.spin.internal/mcp-internal",
        http_handler::ResponseHandler::Response(gateway_response),
    );
    
    // Create token with matching org_id
    let token = key_pair.create_token(
        TestTokenBuilder::new()
            .issuer("https://test.authkit.app")
            .subject("user_abc")
            .claim("org_id", serde_json::json!("org_12345"))
    );
    
    let headers = types::Headers::new();
    headers.append("authorization", format!("Bearer {}", token).as_bytes()).unwrap();
    let request = types::OutgoingRequest::new(headers);
    request.set_path_with_query(Some("/mcp")).unwrap();
    
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 200, "Should accept token with matching org_id");
}

/// Test tenant validation falls back to sub when no org_id
#[spin_test]
fn test_tenant_validation_with_sub_fallback() {
    // Set up test key pair
    let key_pair = TestKeyPair::generate();
    
    // Configure provider with public key
    variables::set("mcp_gateway_url", "https://test-gateway.spin.internal/mcp-internal");
    variables::set("mcp_jwt_issuer", "https://test.authkit.app");
    variables::set("mcp_jwt_public_key", &key_pair.public_key_pem());
    
    // Set tenant ID to a user ID (simulating private mode for individual)
    variables::set("mcp_tenant_id", "user_12345");
    
    // Mock gateway response
    let gateway_response = types::OutgoingResponse::new(types::Headers::new());
    gateway_response.set_status_code(200).unwrap();
    let headers = gateway_response.headers();
    headers.append("content-type", b"application/json").unwrap();
    let body = gateway_response.body().unwrap();
    body.write_bytes(b"{\"jsonrpc\":\"2.0\",\"result\":{},\"id\":1}");
    http_handler::set_response(
        "https://test-gateway.spin.internal/mcp-internal",
        http_handler::ResponseHandler::Response(gateway_response),
    );
    
    // Create token without org_id
    let token = key_pair.create_token(
        TestTokenBuilder::new()
            .issuer("https://test.authkit.app")
            .subject("user_12345")
    );
    
    let headers = types::Headers::new();
    headers.append("authorization", format!("Bearer {}", token).as_bytes()).unwrap();
    let request = types::OutgoingRequest::new(headers);
    request.set_path_with_query(Some("/mcp")).unwrap();
    
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 200, "Should accept token with matching sub when no org_id");
}

/// Test tenant validation rejects mismatched org_id
#[spin_test]
fn test_tenant_validation_rejects_wrong_org() {
    // Set up test key pair
    let key_pair = TestKeyPair::generate();
    
    // Configure provider with public key
    variables::set("mcp_gateway_url", "https://test-gateway.spin.internal/mcp-internal");
    variables::set("mcp_jwt_issuer", "https://test.authkit.app");
    variables::set("mcp_jwt_public_key", &key_pair.public_key_pem());
    
    // Set tenant ID (simulating private mode)
    variables::set("mcp_tenant_id", "org_12345");
    
    // Create token with different org_id
    let token = key_pair.create_token(
        TestTokenBuilder::new()
            .issuer("https://test.authkit.app")
            .subject("user_abc")
            .claim("org_id", serde_json::json!("org_99999"))
    );
    
    let headers = types::Headers::new();
    headers.append("authorization", format!("Bearer {}", token).as_bytes()).unwrap();
    let request = types::OutgoingRequest::new(headers);
    request.set_path_with_query(Some("/mcp")).unwrap();
    
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 401, "Should reject token with mismatched org_id");
    
    // Check error message
    let body = response.body().unwrap_or_default();
    let body_str = String::from_utf8_lossy(&body);
    assert!(body_str.contains("invalid tenant"), "Error should mention tenant mismatch");
}

/// Test tenant validation rejects mismatched sub when used as fallback
#[spin_test]
fn test_tenant_validation_rejects_wrong_sub() {
    // Set up test key pair
    let key_pair = TestKeyPair::generate();
    
    // Configure provider with public key
    variables::set("mcp_gateway_url", "https://test-gateway.spin.internal/mcp-internal");
    variables::set("mcp_jwt_issuer", "https://test.authkit.app");
    variables::set("mcp_jwt_public_key", &key_pair.public_key_pem());
    
    // Set tenant ID to a user ID
    variables::set("mcp_tenant_id", "user_12345");
    
    // Create token with different sub and no org_id
    let token = key_pair.create_token(
        TestTokenBuilder::new()
            .issuer("https://test.authkit.app")
            .subject("user_99999")
    );
    
    let headers = types::Headers::new();
    headers.append("authorization", format!("Bearer {}", token).as_bytes()).unwrap();
    let request = types::OutgoingRequest::new(headers);
    request.set_path_with_query(Some("/mcp")).unwrap();
    
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 401, "Should reject token with mismatched sub");
}

/// Test no tenant validation when mcp_tenant_id is not set (custom mode)
#[spin_test]
fn test_no_tenant_validation_when_not_configured() {
    // Set up test key pair
    let key_pair = TestKeyPair::generate();
    
    // Configure provider with public key
    variables::set("mcp_gateway_url", "https://test-gateway.spin.internal/mcp-internal");
    variables::set("mcp_jwt_issuer", "https://test.authkit.app");
    variables::set("mcp_jwt_public_key", &key_pair.public_key_pem());
    
    // Don't set mcp_tenant_id (simulating custom mode)
    variables::set("mcp_tenant_id", "");
    
    // Mock gateway response
    let gateway_response = types::OutgoingResponse::new(types::Headers::new());
    gateway_response.set_status_code(200).unwrap();
    let headers = gateway_response.headers();
    headers.append("content-type", b"application/json").unwrap();
    let body = gateway_response.body().unwrap();
    body.write_bytes(b"{\"jsonrpc\":\"2.0\",\"result\":{},\"id\":1}");
    http_handler::set_response(
        "https://test-gateway.spin.internal/mcp-internal",
        http_handler::ResponseHandler::Response(gateway_response),
    );
    
    // Create token with any org_id
    let token = key_pair.create_token(
        TestTokenBuilder::new()
            .issuer("https://test.authkit.app")
            .subject("user_abc")
            .claim("org_id", serde_json::json!("org_any"))
    );
    
    let headers = types::Headers::new();
    headers.append("authorization", format!("Bearer {}", token).as_bytes()).unwrap();
    let request = types::OutgoingRequest::new(headers);
    request.set_path_with_query(Some("/mcp")).unwrap();
    
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 200, "Should accept any token when tenant validation is disabled");
}

/// Test static token provider with org_id
#[spin_test]
fn test_static_token_with_org_id() {
    variables::set("mcp_gateway_url", "https://test-gateway.spin.internal/mcp-internal");
    variables::set("mcp_provider_type", "static");
    variables::set("mcp_tenant_id", "org_static");
    
    // Format: token:client_id:sub:scopes:org_id (when no expiration)
    variables::set("mcp_static_tokens", "static-token-1:client1:user1:read,write:org_static");
    
    // Mock gateway response
    let gateway_response = types::OutgoingResponse::new(types::Headers::new());
    gateway_response.set_status_code(200).unwrap();
    let headers = gateway_response.headers();
    headers.append("content-type", b"application/json").unwrap();
    let body = gateway_response.body().unwrap();
    body.write_bytes(b"{\"jsonrpc\":\"2.0\",\"result\":{},\"id\":1}");
    http_handler::set_response(
        "https://test-gateway.spin.internal/mcp-internal",
        http_handler::ResponseHandler::Response(gateway_response),
    );
    
    let headers = types::Headers::new();
    headers.append("authorization", b"Bearer static-token-1").unwrap();
    let request = types::OutgoingRequest::new(headers);
    request.set_path_with_query(Some("/mcp")).unwrap();
    
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 200, "Should accept static token with matching org_id");
}

/// Test static token rejected when org_id doesn't match
#[spin_test]
fn test_static_token_wrong_org_id() {
    variables::set("mcp_gateway_url", "https://test-gateway.spin.internal/mcp-internal");
    variables::set("mcp_provider_type", "static");
    variables::set("mcp_tenant_id", "org_expected");
    
    // Token has different org_id
    variables::set("mcp_static_tokens", "static-token-2:client2:user2:read:org_different");
    
    let headers = types::Headers::new();
    headers.append("authorization", b"Bearer static-token-2").unwrap();
    let request = types::OutgoingRequest::new(headers);
    request.set_path_with_query(Some("/mcp")).unwrap();
    
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 401, "Should reject static token with wrong org_id");
}