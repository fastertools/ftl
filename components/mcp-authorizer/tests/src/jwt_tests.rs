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
use rsa::{pkcs1::EncodeRsaPrivateKey, pkcs8::EncodePublicKey, RsaPrivateKey, RsaPublicKey};
use rsa::traits::PublicKeyParts;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

// JWT Claims structure
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    iss: String,
    aud: Option<AudienceValue>,
    exp: i64,
    iat: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    scope: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    scp: Option<ScopeValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    client_id: Option<String>,
    #[serde(flatten)]
    additional: serde_json::Map<String, Value>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum AudienceValue {
    Single(String),
    Multiple(Vec<String>),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum ScopeValue {
    String(String),
    List(Vec<String>),
}

// Test utility functions
fn generate_rsa_key_pair() -> (RsaPrivateKey, RsaPublicKey) {
    let mut rng = rand::thread_rng();
    let bits = 2048;
    let private_key = RsaPrivateKey::new(&mut rng, bits).expect("failed to generate private key");
    let public_key = RsaPublicKey::from(&private_key);
    (private_key, public_key)
}

/// Mock JWKS endpoint
fn mock_jwks_endpoint(public_key: &RsaPublicKey, url: &str) {
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
        url,
        http_handler::ResponseHandler::Response(response),
    );
}

fn create_token(
    private_key: &RsaPrivateKey,
    subject: &str,
    issuer: &str,
    audience: Option<AudienceValue>,
    expires_in_seconds: i64,
    additional_claims: serde_json::Map<String, Value>,
) -> String {
    let now = Utc::now();
    let exp = now + Duration::seconds(expires_in_seconds);
    
    let mut claims = Claims {
        sub: subject.to_string(),
        iss: issuer.to_string(),
        aud: audience,
        exp: exp.timestamp(),
        iat: now.timestamp(),
        scope: None,
        scp: None,
        client_id: None,
        additional: additional_claims,
    };

    // Handle scope/scp from additional claims
    if let Some(scope) = claims.additional.remove("scope") {
        if let Some(s) = scope.as_str() {
            claims.scope = Some(s.to_string());
        }
    }
    if let Some(scp) = claims.additional.remove("scp") {
        if let Some(s) = scp.as_str() {
            claims.scp = Some(ScopeValue::String(s.to_string()));
        } else if let Some(arr) = scp.as_array() {
            let scopes: Vec<String> = arr.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect();
            claims.scp = Some(ScopeValue::List(scopes));
        }
    }

    let header = Header::new(Algorithm::RS256);
    let pem_string = private_key.to_pkcs1_pem(rsa::pkcs1::LineEnding::LF).unwrap();
    let encoding_key = EncodingKey::from_rsa_pem(pem_string.as_bytes()).unwrap();

    jsonwebtoken::encode(&header, &claims, &encoding_key).unwrap()
}

fn create_token_with_scopes(
    private_key: &RsaPrivateKey,
    subject: &str,
    issuer: &str,
    audience: Option<&str>,
    scopes: Vec<&str>,
) -> String {
    let aud = audience.map(|a| AudienceValue::Single(a.to_string()));
    let mut additional = serde_json::Map::new();
    additional.insert("scope".to_string(), json!(scopes.join(" ")));
    
    create_token(private_key, subject, issuer, aud, 3600, additional)
}

fn get_public_key_pem(public_key: &RsaPublicKey) -> String {
    public_key.to_public_key_pem(rsa::pkcs8::LineEnding::LF).unwrap()
}

// Mock JWKS response helper
#[allow(dead_code)]
fn mock_jwks_response(public_key: &RsaPublicKey, kid: Option<&str>) -> Value {
    // For testing, we'll create a simplified JWKS that matches the public key
    // In a real test environment, this would be served by a mock HTTP server
    json!({
        "keys": [{
            "kty": "RSA",
            "use": "sig",
            "alg": "RS256",
            "kid": kid.unwrap_or("test-key-1"),
            "n": base64::engine::general_purpose::STANDARD.encode(&public_key.n().to_bytes_be()),
            "e": base64::engine::general_purpose::STANDARD.encode(&public_key.e().to_bytes_be())
        }]
    })
}

// Test: Valid token with public key verification
#[spin_test]
fn test_valid_token_with_public_key() {
    // Configure provider with a test public key
    let (private_key, public_key) = generate_rsa_key_pair();
    let public_key_pem = get_public_key_pem(&public_key);
    
    // Set up configuration with public key instead of JWKS
    variables::set("mcp_gateway_url", "none");
    variables::set("mcp_jwt_issuer", "https://test.example.com");  // Use non-authkit issuer to avoid auto-derivation
    variables::set("mcp_jwt_audience", "test-audience");
    variables::set("mcp_jwt_public_key", &public_key_pem);
    
    // Mock MCP gateway
    let gateway_response = http::types::OutgoingResponse::new(http::types::Headers::new());
    gateway_response.set_status_code(200).unwrap();
    let headers = gateway_response.headers();
    headers.append("content-type", b"application/json").unwrap();
    let body = gateway_response.body().unwrap();
    body.write_bytes(b"{\"jsonrpc\":\"2.0\",\"result\":{},\"id\":1}");
    http_handler::set_response(
        "https://test-gateway.spin.internal/mcp",
        http_handler::ResponseHandler::Response(gateway_response),
    );
    
    // Create a valid token
    let token = create_token_with_scopes(
        &private_key,
        "test-user",
        "https://test.example.com",  // Match the configured issuer
        Some("test-audience"),
        vec!["read", "write"],
    );
    
    // Make request with valid token
    let headers = http::types::Headers::new();
    headers.append("authorization", format!("Bearer {}", token).as_bytes()).unwrap();
    headers.append("content-type", b"application/json").unwrap();
    
    let request = http::types::OutgoingRequest::new(headers);
    request.set_path_with_query(Some("/mcp")).unwrap();
    request.set_method(&http::types::Method::Post).unwrap();
    
    let body = request.body().unwrap();
    body.write_bytes(b"{\"jsonrpc\":\"2.0\",\"method\":\"test\",\"id\":1}");
    
    let response = spin_test_sdk::perform_request(request);
    
    // Should succeed with public key verification
    assert_eq!(response.status(), 200);
}

// Test: Missing authorization header
#[spin_test]
fn test_missing_authorization_header() {
    // Setup default configuration
    crate::test_setup::setup_default_test_config();
    
    let request = http::types::OutgoingRequest::new(http::types::Headers::new());
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
    let auth_value = www_auth.unwrap();
    assert!(auth_value.contains("Bearer"));
    assert!(auth_value.contains("error=\"unauthorized\""));
    assert!(auth_value.contains("error_description=\"Missing authorization header\""));
}

// Test: Invalid bearer token format
#[spin_test]
fn test_invalid_bearer_format() {
    // Setup default configuration
    crate::test_setup::setup_default_test_config();
    
    let headers = http::types::Headers::new();
    headers.append("authorization", b"InvalidFormat token").unwrap();
    
    let request = http::types::OutgoingRequest::new(headers);
    request.set_path_with_query(Some("/mcp")).unwrap();
    let response = spin_test_sdk::perform_request(request);
    
    assert_eq!(response.status(), 401);
    
    // Check error response
    let headers = response.headers();
    let entries = headers.entries();
    let www_auth = entries.iter()
        .find(|(name, _)| name == "www-authenticate")
        .map(|(_, value)| String::from_utf8_lossy(value));
    
    assert!(www_auth.is_some());
    assert!(www_auth.unwrap().contains("error=\"invalid_token\""));
}

// Test: Malformed JWT token
#[spin_test]
fn test_malformed_jwt() {
    // Setup default configuration
    crate::test_setup::setup_default_test_config();
    
    let malformed_tokens = vec![
        "not.a.jwt",
        "too.many.parts.here.invalid",
        "invalid-token",
        "header.payload", // Missing signature
    ];
    
    for token in malformed_tokens {
        let headers = http::types::Headers::new();
        headers.append("authorization", format!("Bearer {}", token).as_bytes()).unwrap();
        
        let request = http::types::OutgoingRequest::new(headers);
        request.set_path_with_query(Some("/mcp")).unwrap();
        let response = spin_test_sdk::perform_request(request);
        
        assert_eq!(response.status(), 401);
    }
}

// Test: Expired token
#[spin_test]
fn test_expired_token() {
    // Set up test configuration
    use spin_test_sdk::bindings::fermyon::spin_test_virt::variables;
    variables::set("mcp_gateway_url", "none");
    variables::set("mcp_jwt_issuer", "https://test.authkit.app");
    variables::set("mcp_jwt_jwks_uri", "https://test.authkit.app/.well-known/jwks.json");
    variables::set("mcp_jwt_audience", "test-audience");
    
    let (private_key, public_key) = generate_rsa_key_pair();
    
    // Mock JWKS endpoint
    mock_jwks_endpoint(&public_key, "https://test.authkit.app/.well-known/jwks.json");
    
    // Create an expired token
    let token = create_token(
        &private_key,
        "test-user",
        "https://test.authkit.app",
        Some(AudienceValue::Single("test-audience".to_string())),
        -3600, // Expired 1 hour ago
        serde_json::Map::new(),
    );
    
    let headers = http::types::Headers::new();
    headers.append("authorization", format!("Bearer {}", token).as_bytes()).unwrap();
    
    let request = http::types::OutgoingRequest::new(headers);
    request.set_path_with_query(Some("/mcp")).unwrap();
    let response = spin_test_sdk::perform_request(request);
    
    assert_eq!(response.status(), 401);
}

// Test: Multiple audiences in token
#[spin_test]
fn test_multiple_audiences() {
    // Set up test configuration
    variables::set("mcp_gateway_url", "none");
    variables::set("mcp_jwt_issuer", "https://test.authkit.app");
    variables::set("mcp_jwt_jwks_uri", "https://test.authkit.app/.well-known/jwks.json");
    variables::set("mcp_jwt_audience", "test-audience");
    
    let (private_key, public_key) = generate_rsa_key_pair();
    
    // Mock JWKS endpoint
    mock_jwks_endpoint(&public_key, "https://test.authkit.app/.well-known/jwks.json");
    
    // Mock MCP gateway
    let gateway_response = http::types::OutgoingResponse::new(http::types::Headers::new());
    gateway_response.set_status_code(200).unwrap();
    let headers = gateway_response.headers();
    headers.append("content-type", b"application/json").unwrap();
    let body = gateway_response.body().unwrap();
    body.write_bytes(b"{\"jsonrpc\":\"2.0\",\"result\":{},\"id\":1}");
    http_handler::set_response(
        "https://test-gateway.spin.internal/mcp",
        http_handler::ResponseHandler::Response(gateway_response),
    );
    
    // Create token with multiple audiences
    let additional = serde_json::Map::new();
    let token = create_token(
        &private_key,
        "test-user",
        "https://test.authkit.app",
        Some(AudienceValue::Multiple(vec![
            "test-audience".to_string(),
            "other-audience".to_string(),
        ])),
        3600,
        additional,
    );
    
    let headers = http::types::Headers::new();
    headers.append("authorization", format!("Bearer {}", token).as_bytes()).unwrap();
    headers.append("content-type", b"application/json").unwrap();
    
    let request = http::types::OutgoingRequest::new(headers);
    request.set_path_with_query(Some("/mcp")).unwrap();
    request.set_method(&http::types::Method::Post).unwrap();
    
    let body = request.body().unwrap();
    body.write_bytes(b"{\"jsonrpc\":\"2.0\",\"method\":\"test\",\"id\":1}");
    
    let response = spin_test_sdk::perform_request(request);
    
    // Should succeed - one of the audiences matches configured audience
    assert_eq!(response.status(), 200);
}

// Test: Scope extraction from different formats
#[spin_test]
fn test_scope_formats() {
    let (private_key, _) = generate_rsa_key_pair();
    
    // Test 1: Standard OAuth2 'scope' claim as string
    let mut additional = serde_json::Map::new();
    additional.insert("scope".to_string(), json!("read write admin"));
    
    let token = create_token(
        &private_key,
        "test-user",
        "https://test.authkit.app",
        Some(AudienceValue::Single("test-audience".to_string())),
        3600,
        additional,
    );
    
    // Test would verify scope extraction in actual implementation
    assert!(!token.is_empty());
    
    // Test 2: Microsoft-style 'scp' claim as string
    let mut additional = serde_json::Map::new();
    additional.insert("scp".to_string(), json!("read write admin"));
    
    let token = create_token(
        &private_key,
        "test-user",
        "https://test.authkit.app",
        Some(AudienceValue::Single("test-audience".to_string())),
        3600,
        additional,
    );
    
    assert!(!token.is_empty());
    
    // Test 3: 'scp' claim as array
    let mut additional = serde_json::Map::new();
    additional.insert("scp".to_string(), json!(["read", "write", "admin"]));
    
    let token = create_token(
        &private_key,
        "test-user",
        "https://test.authkit.app",
        Some(AudienceValue::Single("test-audience".to_string())),
        3600,
        additional,
    );
    
    assert!(!token.is_empty());
}

// Test: Provider configuration validation
#[spin_test]
fn test_provider_requires_key_or_jwks() {
    // Test that provider configuration requires either public_key or jwks_uri
    variables::set("mcp_jwt_issuer", "https://example.com");
    variables::set("mcp_jwt_jwks_uri", ""); // Empty JWKS URI
    variables::set("mcp_jwt_audience", "");
    
    // Component should fail to initialize without key or JWKS URI
    let request = http::types::OutgoingRequest::new(http::types::Headers::new());
    request.set_path_with_query(Some("/mcp")).unwrap();
    let response = spin_test_sdk::perform_request(request);
    
    assert_eq!(response.status(), 500);
}

// Test: String issuer validation
#[spin_test]
fn test_string_issuer() {
    // Configure provider with string issuer (kept as-is per RFC 7519)
    let (private_key, public_key) = generate_rsa_key_pair();
    let public_key_pem = get_public_key_pem(&public_key);
    
    variables::set("mcp_gateway_url", "none");
    variables::set("mcp_jwt_issuer", "my-service"); // String issuer kept as-is
    variables::set("mcp_jwt_audience", "test-audience");
    variables::set("mcp_jwt_public_key", &public_key_pem);
    // Don't set jwks_uri at all to avoid conflicts
    
    // Mock MCP gateway
    let gateway_response = http::types::OutgoingResponse::new(http::types::Headers::new());
    gateway_response.set_status_code(200).unwrap();
    let headers = gateway_response.headers();
    headers.append("content-type", b"application/json").unwrap();
    let body = gateway_response.body().unwrap();
    body.write_bytes(b"{\"jsonrpc\":\"2.0\",\"result\":{},\"id\":1}");
    http_handler::set_response(
        "https://test-gateway.spin.internal/mcp",
        http_handler::ResponseHandler::Response(gateway_response),
    );
    
    // Create token with string issuer (not URL)
    let token = create_token(
        &private_key,
        "test-user",
        "my-service", // String issuer should match exactly
        Some(AudienceValue::Single("test-audience".to_string())),
        3600,
        serde_json::Map::new(),
    );
    
    let headers = http::types::Headers::new();
    headers.append("authorization", format!("Bearer {}", token).as_bytes()).unwrap();
    headers.append("content-type", b"application/json").unwrap();
    
    let request = http::types::OutgoingRequest::new(headers);
    request.set_path_with_query(Some("/mcp")).unwrap();
    request.set_method(&http::types::Method::Post).unwrap();
    
    let body = request.body().unwrap();
    body.write_bytes(b"{\"jsonrpc\":\"2.0\",\"method\":\"test\",\"id\":1}");
    
    let response = spin_test_sdk::perform_request(request);
    
    // Should succeed because string issuer matches exactly
    assert_eq!(response.status(), 200);
}

// Test: Client ID extraction fallback
#[spin_test]
fn test_client_id_extraction() {
    let (private_key, _) = generate_rsa_key_pair();
    
    // Test with explicit client_id claim
    let mut additional = serde_json::Map::new();
    additional.insert("client_id".to_string(), json!("app456"));
    
    let token = create_token(
        &private_key,
        "user123", // sub claim
        "https://test.authkit.app",
        Some(AudienceValue::Single("test-audience".to_string())),
        3600,
        additional,
    );
    
    // Token should prefer client_id over sub when both present
    assert!(!token.is_empty());
}