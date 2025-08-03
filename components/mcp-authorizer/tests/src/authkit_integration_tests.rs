//! Tests for WorkOS AuthKit integration

use spin_test_sdk::{
    bindings::{
        fermyon::spin_test_virt::variables,
        fermyon::spin_wasi_virt::http_handler,
        wasi::http::types,
    },
    spin_test,
};
use serde_json::json;

// Test: AuthKit auto-derives JWKS URI
#[spin_test]
fn test_authkit_jwks_auto_derivation() {
    // Configure with AuthKit issuer only - JWKS should be auto-derived
    variables::set("mcp_provider_type", "jwt");
    variables::set("mcp_jwt_issuer", "https://test-project.authkit.app");
    // DO NOT set mcp_jwt_jwks_uri - it should be auto-derived
    variables::set("mcp_gateway_url", "https://test-gateway.spin.internal/mcp-internal");
    
    // Test that metadata endpoint works with auto-derived JWKS
    let request = types::OutgoingRequest::new(types::Headers::new());
    request.set_path_with_query(Some("/.well-known/oauth-authorization-server")).unwrap();
    let response = spin_test_sdk::perform_request(request);
    
    assert_eq!(response.status(), 200);
    
    // Get response body
    let body = response.body().unwrap_or_else(|_| Vec::new());
    let metadata: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    // Verify AuthKit-specific metadata
    assert_eq!(metadata["issuer"], "https://test-project.authkit.app");
    assert_eq!(metadata["jwks_uri"], "https://test-project.authkit.app/oauth2/jwks");
    assert_eq!(metadata["authorization_endpoint"], "https://test-project.authkit.app/oauth2/authorize");
    assert_eq!(metadata["token_endpoint"], "https://test-project.authkit.app/oauth2/token");
    assert_eq!(metadata["registration_endpoint"], "https://test-project.authkit.app/oauth2/register");
}

// Test: OAuth protected resource metadata for AuthKit
#[spin_test]
fn test_authkit_protected_resource_metadata() {
    // Configure with AuthKit issuer
    variables::set("mcp_provider_type", "jwt");
    variables::set("mcp_jwt_issuer", "https://test-project.authkit.app");
    variables::set("mcp_gateway_url", "https://test-gateway.spin.internal/mcp-internal");
    
    // Request protected resource metadata
    let headers = types::Headers::new();
    headers.append("host", b"mcp.example.com").unwrap();
    let request = types::OutgoingRequest::new(headers);
    request.set_path_with_query(Some("/.well-known/oauth-protected-resource")).unwrap();
    let response = spin_test_sdk::perform_request(request);
    
    assert_eq!(response.status(), 200);
    
    // Get response body
    let body = response.body().unwrap_or_else(|_| Vec::new());
    let metadata: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    // Verify protected resource metadata
    assert_eq!(metadata["resource"], "https://mcp.example.com");
    assert_eq!(metadata["authorization_servers"][0], "https://test-project.authkit.app");
    assert_eq!(metadata["bearer_methods_supported"], json!(["header"]));
}

// Test: OpenID configuration for AuthKit
#[spin_test]
fn test_authkit_openid_configuration() {
    // Configure with AuthKit issuer
    variables::set("mcp_provider_type", "jwt");
    variables::set("mcp_jwt_issuer", "https://test-project.authkit.app");
    variables::set("mcp_gateway_url", "https://test-gateway.spin.internal/mcp-internal");
    
    // Request OpenID configuration
    let request = types::OutgoingRequest::new(types::Headers::new());
    request.set_path_with_query(Some("/.well-known/openid-configuration")).unwrap();
    let response = spin_test_sdk::perform_request(request);
    
    assert_eq!(response.status(), 200);
    
    // Get response body
    let body = response.body().unwrap_or_else(|_| Vec::new());
    let metadata: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    // Verify OpenID configuration
    assert_eq!(metadata["issuer"], "https://test-project.authkit.app");
    assert!(metadata["scopes_supported"].as_array().unwrap().contains(&serde_json::json!("openid")));
    assert!(metadata["response_types_supported"].as_array().unwrap().contains(&serde_json::json!("code")));
    assert!(metadata["code_challenge_methods_supported"].as_array().unwrap().contains(&serde_json::json!("S256")));
}

// Test: WorkOS.com domain also gets AuthKit treatment
#[spin_test]
fn test_workos_domain_support() {
    // Configure with workos.com domain
    variables::set("mcp_provider_type", "jwt");
    variables::set("mcp_jwt_issuer", "https://api.workos.com");
    variables::set("mcp_gateway_url", "https://test-gateway.spin.internal/mcp-internal");
    
    // Test that metadata endpoint works
    let request = types::OutgoingRequest::new(types::Headers::new());
    request.set_path_with_query(Some("/.well-known/oauth-authorization-server")).unwrap();
    let response = spin_test_sdk::perform_request(request);
    
    assert_eq!(response.status(), 200);
    
    // Get response body
    let body = response.body().unwrap_or_else(|_| Vec::new());
    let metadata: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    // Verify JWKS auto-derivation
    assert_eq!(metadata["jwks_uri"], "https://api.workos.com/oauth2/jwks");
}

// Test: Non-AuthKit domains don't get special treatment
#[spin_test]
fn test_non_authkit_domain() {
    // Configure with non-AuthKit issuer
    variables::set("mcp_provider_type", "jwt");
    variables::set("mcp_jwt_issuer", "https://auth.example.com");
    variables::set("mcp_jwt_jwks_uri", "https://auth.example.com/jwks");
    variables::set("mcp_gateway_url", "https://test-gateway.spin.internal/mcp-internal");
    
    // Request authorization server metadata
    let request = types::OutgoingRequest::new(types::Headers::new());
    request.set_path_with_query(Some("/.well-known/oauth-authorization-server")).unwrap();
    let response = spin_test_sdk::perform_request(request);
    
    assert_eq!(response.status(), 200);
    
    // Get response body
    let body = response.body().unwrap_or_else(|_| Vec::new());
    let metadata: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    // Verify it doesn't have AuthKit-specific endpoints
    assert_eq!(metadata["issuer"], "https://auth.example.com");
    assert!(metadata["registration_endpoint"].is_null());
}

// Test: AuthKit token validation with correct issuer
#[spin_test]
fn test_authkit_token_validation() {
    use crate::test_token_utils::{TestKeyPair, TestTokenBuilder};
    
    let key_pair = TestKeyPair::generate();
    
    // Configure with AuthKit issuer
    variables::set("mcp_provider_type", "jwt");
    variables::set("mcp_jwt_issuer", "https://test-project.authkit.app");
    variables::set("mcp_jwt_public_key", &key_pair.public_key_pem());
    variables::set("mcp_gateway_url", "https://test-gateway.spin.internal/mcp-internal");
    
    // Mock gateway
    let response = types::OutgoingResponse::new(types::Headers::new());
    response.set_status_code(200).unwrap();
    let headers = response.headers();
    headers.append("content-type", b"application/json").unwrap();
    
    let body = response.body().unwrap();
    body.write_bytes(b"{\"jsonrpc\":\"2.0\",\"result\":{},\"id\":1}");
    
    http_handler::set_response(
        "https://test-gateway.spin.internal/mcp-internal",
        http_handler::ResponseHandler::Response(response),
    );
    
    // Create token with AuthKit issuer
    let token = key_pair.create_token(
        TestTokenBuilder::new()
            .issuer("https://test-project.authkit.app")
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
    
    // Should succeed with valid AuthKit token
    assert_eq!(response.status(), 200);
}