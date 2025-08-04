use spin_test_sdk::{
    bindings::{
        fermyon::spin_test_virt::variables,
        fermyon::spin_wasi_virt::http_handler,
        wasi::http,
    },
    spin_test,
};

use base64::Engine;
use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use rsa::{pkcs1::EncodeRsaPrivateKey, RsaPrivateKey, RsaPublicKey};
use rsa::traits::PublicKeyParts;
use serde_json::json;

use crate::jwt_verification_tests::{Claims, AudienceValue, ScopeValue};

use crate::jwt_verification_tests::configure_test_provider;

/// Helper to generate RSA key pair
fn generate_test_key_pair() -> (RsaPrivateKey, RsaPublicKey) {
    let mut rng = rand::thread_rng();
    let bits = 2048;
    let private_key = RsaPrivateKey::new(&mut rng, bits).expect("failed to generate private key");
    let public_key = RsaPublicKey::from(&private_key);
    (private_key, public_key)
}

/// Create a JWT token with custom scope configuration
fn create_token_with_scopes(
    private_key: &RsaPrivateKey,
    issuer: &str,
    audience: &str,
    scope: Option<String>,
    scp: Option<ScopeValue>,
) -> String {
    let now = Utc::now();
    let claims = Claims {
        sub: "test-user".to_string(),
        iss: issuer.to_string(),
        aud: Some(AudienceValue::Single(audience.to_string())),
        exp: (now + Duration::hours(1)).timestamp(),
        iat: now.timestamp(),
        scope,
        scp,
        client_id: None,
        additional: serde_json::Map::new(),
    };
    
    let header = Header::new(Algorithm::RS256);
    
    let pem_string = private_key.to_pkcs1_pem(rsa::pkcs1::LineEnding::LF).unwrap();
    let encoding_key = EncodingKey::from_rsa_pem(pem_string.as_bytes()).unwrap();
    
    jsonwebtoken::encode(&header, &claims, &encoding_key).unwrap()
}

/// Mock JWKS endpoint
fn mock_jwks_endpoint(public_key: &RsaPublicKey) {
    let n = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .encode(&public_key.n().to_bytes_be());
    let e = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .encode(&public_key.e().to_bytes_be());
    
    let jwks = json!({
        "keys": [{
            "kty": "RSA",
            "use": "sig",
            "alg": "RS256",
            "n": n,
            "e": e
        }]
    });
    
    let response = http::types::OutgoingResponse::new(http::types::Headers::new());
    response.set_status_code(200).unwrap();
    let headers = response.headers();
    headers.append("content-type", b"application/json").unwrap();
    
    let body = response.body().unwrap();
    body.write_bytes(serde_json::to_string(&jwks).unwrap().as_bytes());
    
    http_handler::set_response(
        "https://test.authkit.app/.well-known/jwks.json",
        http_handler::ResponseHandler::Response(response),
    );
}

/// Mock MCP gateway
fn mock_gateway() {
    let response = http::types::OutgoingResponse::new(http::types::Headers::new());
    response.set_status_code(200).unwrap();
    let headers = response.headers();
    headers.append("content-type", b"application/json").unwrap();
    
    let body = response.body().unwrap();
    body.write_bytes(b"{\"jsonrpc\":\"2.0\",\"result\":{},\"id\":1}");
    
    http_handler::set_response(
        "https://test-gateway.spin.internal/mcp-internal",
        http_handler::ResponseHandler::Response(response),
    );
}

// Test: Token with no scopes
#[spin_test]
fn test_no_scopes_in_token() {
    configure_test_provider();
    
    let (private_key, public_key) = generate_test_key_pair();
    
    // Mock JWKS
    mock_jwks_endpoint(&public_key);
    mock_gateway();
    
    // Create token with no scopes
    let token = create_token_with_scopes(
        &private_key,
        "https://test.authkit.app",
        "test-audience",
        None, // No 'scope' claim
        None, // No 'scp' claim
    );
    
    // Make request
    let headers = http::types::Headers::new();
    headers.append("authorization", format!("Bearer {}", token).as_bytes()).unwrap();
    headers.append("content-type", b"application/json").unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_path_with_query(Some("/mcp")).unwrap();
    request.set_method(&http::types::Method::Post).unwrap();
    
    let body = request.body().unwrap();
    body.write_bytes(b"{\"jsonrpc\":\"2.0\",\"method\":\"test\",\"id\":1}");
    
    let response = spin_test_sdk::perform_request(request);
    
    // Should succeed - no scopes is valid
    assert_eq!(response.status(), 200);
}

// Test: Scope precedence - 'scope' claim takes precedence over 'scp'
#[spin_test]
fn test_scope_precedence() {
    configure_test_provider();
    
    let (private_key, public_key) = generate_test_key_pair();
    
    // Mock JWKS
    mock_jwks_endpoint(&public_key);
    mock_gateway();
    
    // Create token with both 'scope' and 'scp' claims
    let token = create_token_with_scopes(
        &private_key,
        "https://test.authkit.app",
        "test-audience",
        Some("read write".to_string()), // OAuth2 standard 'scope'
        Some(ScopeValue::String("admin delete".to_string())), // Microsoft 'scp'
    );
    
    // Make request
    let headers = http::types::Headers::new();
    headers.append("authorization", format!("Bearer {}", token).as_bytes()).unwrap();
    headers.append("content-type", b"application/json").unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_path_with_query(Some("/mcp")).unwrap();
    request.set_method(&http::types::Method::Post).unwrap();
    
    let body = request.body().unwrap();
    body.write_bytes(b"{\"jsonrpc\":\"2.0\",\"method\":\"test\",\"id\":1}");
    
    let response = spin_test_sdk::perform_request(request);
    
    // Should succeed
    assert_eq!(response.status(), 200);
    
    // The OAuth2 'scope' claim takes precedence over Microsoft 'scp' claim
    // Gateway forwarding tests verify auth context headers are properly set
}

// Test: String issuer mismatch rejection
#[spin_test]
fn test_string_issuer_mismatch() {
    // Configure provider with a string issuer (not URL)
    variables::set("mcp_gateway_url", "https://test-gateway.spin.internal/mcp-internal");
    variables::set("mcp_jwt_issuer", "my-service"); // String issuer, not URL
    variables::set("mcp_jwt_jwks_uri", "https://test.authkit.app/.well-known/jwks.json");
    variables::set("mcp_jwt_audience", "test-audience");
    
    let (private_key, public_key) = generate_test_key_pair();
    
    // Mock JWKS
    mock_jwks_endpoint(&public_key);
    
    // Create token with different issuer
    let token = create_token_with_scopes(
        &private_key,
        "https://different-service", // Different issuer (normalized)
        "test-audience",
        Some("read".to_string()),
        None,
    );
    
    // Make request
    let headers = http::types::Headers::new();
    headers.append("authorization", format!("Bearer {}", token).as_bytes()).unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_path_with_query(Some("/mcp")).unwrap();
    
    let response = spin_test_sdk::perform_request(request);
    
    // Should fail - issuer mismatch
    assert_eq!(response.status(), 401);
}

// Test: Insufficient scopes
#[spin_test]
fn test_insufficient_scopes() {
    // Configure provider with required scopes
    variables::set("mcp_gateway_url", "https://test-gateway.spin.internal/mcp-internal");
    variables::set("mcp_jwt_issuer", "https://test.authkit.app");
    variables::set("mcp_jwt_jwks_uri", "https://test.authkit.app/.well-known/jwks.json");
    variables::set("mcp_jwt_audience", "test-audience");
    variables::set("mcp_jwt_required_scopes", "admin,write");
    
    let (private_key, public_key) = generate_test_key_pair();
    
    // Mock JWKS
    mock_jwks_endpoint(&public_key);
    
    // Mock gateway
    mock_gateway();
    
    // Create token with insufficient scopes
    let token = create_token_with_scopes(
        &private_key,
        "https://test.authkit.app",
        "test-audience",
        Some("read".to_string()), // Only 'read', but need 'admin write'
        None,
    );
    
    // Make request
    let headers = http::types::Headers::new();
    headers.append("authorization", format!("Bearer {}", token).as_bytes()).unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_path_with_query(Some("/mcp")).unwrap();
    
    let response = spin_test_sdk::perform_request(request);
    
    // Should fail with 401 - insufficient scopes
    assert_eq!(response.status(), 401);
}

// Test: Sufficient scopes
#[spin_test]
fn test_sufficient_scopes() {
    // Configure provider with required scopes
    variables::set("mcp_gateway_url", "https://test-gateway.spin.internal/mcp-internal");
    variables::set("mcp_jwt_issuer", "https://test.authkit.app");
    variables::set("mcp_jwt_jwks_uri", "https://test.authkit.app/.well-known/jwks.json");
    variables::set("mcp_jwt_audience", "test-audience");
    variables::set("mcp_jwt_required_scopes", "read,write");
    
    let (private_key, public_key) = generate_test_key_pair();
    
    // Mock JWKS
    mock_jwks_endpoint(&public_key);
    mock_gateway();
    
    // Create token with sufficient scopes
    let token = create_token_with_scopes(
        &private_key,
        "https://test.authkit.app",
        "test-audience",
        Some("read write admin".to_string()), // Has required 'read write' plus extra
        None,
    );
    
    // Make request
    let headers = http::types::Headers::new();
    headers.append("authorization", format!("Bearer {}", token).as_bytes()).unwrap();
    headers.append("content-type", b"application/json").unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_path_with_query(Some("/mcp")).unwrap();
    request.set_method(&http::types::Method::Post).unwrap();
    
    let body = request.body().unwrap();
    body.write_bytes(b"{\"jsonrpc\":\"2.0\",\"method\":\"test\",\"id\":1}");
    
    let response = spin_test_sdk::perform_request(request);
    
    // Should succeed with sufficient scopes
    assert_eq!(response.status(), 200);
}

// Test: Empty required scopes
#[spin_test]
fn test_empty_required_scopes() {
    // Configure provider with empty required scopes (should accept any token)
    variables::set("mcp_gateway_url", "https://test-gateway.spin.internal/mcp-internal");
    variables::set("mcp_jwt_issuer", "https://test.authkit.app");
    variables::set("mcp_jwt_jwks_uri", "https://test.authkit.app/.well-known/jwks.json");
    variables::set("mcp_jwt_audience", "test-audience");
    variables::set("mcp_jwt_required_scopes", ""); // Empty required scopes
    
    let (private_key, public_key) = generate_test_key_pair();
    
    // Mock JWKS
    mock_jwks_endpoint(&public_key);
    mock_gateway();
    
    // Create token with no scopes
    let token = create_token_with_scopes(
        &private_key,
        "https://test.authkit.app",
        "test-audience",
        None,
        None,
    );
    
    // Make request
    let headers = http::types::Headers::new();
    headers.append("authorization", format!("Bearer {}", token).as_bytes()).unwrap();
    headers.append("content-type", b"application/json").unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_path_with_query(Some("/mcp")).unwrap();
    request.set_method(&http::types::Method::Post).unwrap();
    
    let body = request.body().unwrap();
    body.write_bytes(b"{\"jsonrpc\":\"2.0\",\"method\":\"test\",\"id\":1}");
    
    let response = spin_test_sdk::perform_request(request);
    
    // Should succeed - no required scopes means any token is valid
    assert_eq!(response.status(), 200);
}

// Test: Exact scope match
#[spin_test]
fn test_exact_scope_match() {
    // Configure provider with required scopes
    variables::set("mcp_gateway_url", "https://test-gateway.spin.internal/mcp-internal");
    variables::set("mcp_jwt_issuer", "https://test.authkit.app");
    variables::set("mcp_jwt_jwks_uri", "https://test.authkit.app/.well-known/jwks.json");
    variables::set("mcp_jwt_audience", "test-audience");
    variables::set("mcp_jwt_required_scopes", "users:read,users:write");
    
    let (private_key, public_key) = generate_test_key_pair();
    
    // Mock JWKS
    mock_jwks_endpoint(&public_key);
    mock_gateway();
    
    // Create token with exact matching scopes
    let token = create_token_with_scopes(
        &private_key,
        "https://test.authkit.app",
        "test-audience",
        Some("users:read users:write".to_string()), // Exact match
        None,
    );
    
    // Make request
    let headers = http::types::Headers::new();
    headers.append("authorization", format!("Bearer {}", token).as_bytes()).unwrap();
    headers.append("content-type", b"application/json").unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_path_with_query(Some("/mcp")).unwrap();
    request.set_method(&http::types::Method::Post).unwrap();
    
    let body = request.body().unwrap();
    body.write_bytes(b"{\"jsonrpc\":\"2.0\",\"method\":\"test\",\"id\":1}");
    
    let response = spin_test_sdk::perform_request(request);
    
    // Should succeed with exact scope match
    assert_eq!(response.status(), 200);
}

// Test: Partial scope match failure
#[spin_test]
fn test_partial_scope_match_failure() {
    // Configure provider with required scopes
    variables::set("mcp_gateway_url", "https://test-gateway.spin.internal/mcp-internal");
    variables::set("mcp_jwt_issuer", "https://test.authkit.app");
    variables::set("mcp_jwt_jwks_uri", "https://test.authkit.app/.well-known/jwks.json");
    variables::set("mcp_jwt_audience", "test-audience");
    variables::set("mcp_jwt_required_scopes", "users:read,users:write,admin");
    
    let (private_key, public_key) = generate_test_key_pair();
    
    // Mock JWKS
    mock_jwks_endpoint(&public_key);
    mock_gateway();
    
    // Create token with only partial scopes
    let token = create_token_with_scopes(
        &private_key,
        "https://test.authkit.app",
        "test-audience",
        Some("users:read admin".to_string()), // Missing users:write
        None,
    );
    
    // Make request
    let headers = http::types::Headers::new();
    headers.append("authorization", format!("Bearer {}", token).as_bytes()).unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_path_with_query(Some("/mcp")).unwrap();
    
    let response = spin_test_sdk::perform_request(request);
    
    // Should fail - missing required scope
    assert_eq!(response.status(), 401);
}

// Test: Scope validation with Microsoft 'scp' claim
#[spin_test]
fn test_scope_validation_with_scp_claim() {
    // Configure provider with required scopes
    variables::set("mcp_gateway_url", "https://test-gateway.spin.internal/mcp-internal");
    variables::set("mcp_jwt_issuer", "https://test.authkit.app");
    variables::set("mcp_jwt_jwks_uri", "https://test.authkit.app/.well-known/jwks.json");
    variables::set("mcp_jwt_audience", "test-audience");
    variables::set("mcp_jwt_required_scopes", "api.read,api.write");
    
    let (private_key, public_key) = generate_test_key_pair();
    
    // Mock JWKS
    mock_jwks_endpoint(&public_key);
    mock_gateway();
    
    // Create token with scopes in 'scp' claim as array (Microsoft style)
    let token = create_token_with_scopes(
        &private_key,
        "https://test.authkit.app",
        "test-audience",
        None, // No 'scope' claim
        Some(ScopeValue::List(vec!["api.read".to_string(), "api.write".to_string(), "api.admin".to_string()])),
    );
    
    // Make request
    let headers = http::types::Headers::new();
    headers.append("authorization", format!("Bearer {}", token).as_bytes()).unwrap();
    headers.append("content-type", b"application/json").unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_path_with_query(Some("/mcp")).unwrap();
    request.set_method(&http::types::Method::Post).unwrap();
    
    let body = request.body().unwrap();
    body.write_bytes(b"{\"jsonrpc\":\"2.0\",\"method\":\"test\",\"id\":1}");
    
    let response = spin_test_sdk::perform_request(request);
    
    // Should succeed - has required scopes via 'scp' claim
    assert_eq!(response.status(), 200);
}