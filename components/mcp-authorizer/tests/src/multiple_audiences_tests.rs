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
    #[serde(flatten)]
    additional: serde_json::Map<String, Value>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum AudienceValue {
    Single(String),
    Multiple(Vec<String>),
}

// Test utility functions
fn generate_rsa_key_pair() -> (RsaPrivateKey, RsaPublicKey) {
    let mut rng = rand::thread_rng();
    let bits = 2048;
    let private_key = RsaPrivateKey::new(&mut rng, bits).expect("failed to generate private key");
    let public_key = RsaPublicKey::from(&private_key);
    (private_key, public_key)
}

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

fn mock_gateway_success() {
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
}

fn create_token(
    private_key: &RsaPrivateKey,
    subject: &str,
    issuer: &str,
    audience: Option<AudienceValue>,
    expires_in_seconds: i64,
) -> String {
    let now = Utc::now();
    let exp = now + Duration::seconds(expires_in_seconds);
    
    let claims = Claims {
        sub: subject.to_string(),
        iss: issuer.to_string(),
        aud: audience,
        exp: exp.timestamp(),
        iat: now.timestamp(),
        scope: None,
        additional: serde_json::Map::new(),
    };

    let header = Header::new(Algorithm::RS256);
    let pem_string = private_key.to_pkcs1_pem(rsa::pkcs1::LineEnding::LF).unwrap();
    let encoding_key = EncodingKey::from_rsa_pem(pem_string.as_bytes()).unwrap();

    jsonwebtoken::encode(&header, &claims, &encoding_key).unwrap()
}

// Test: Multiple configured audiences - accept token with single matching audience
#[spin_test]
fn test_multiple_configured_audiences_single_token_audience_match() {
    // Configure with multiple audiences (comma-separated)
    variables::set("mcp_gateway_url", "none");
    variables::set("mcp_jwt_issuer", "https://test.authkit.app");
    variables::set("mcp_jwt_jwks_uri", "https://test.authkit.app/.well-known/jwks.json");
    variables::set("mcp_jwt_audience", "api-1,api-2,api-3");
    
    let (private_key, public_key) = generate_rsa_key_pair();
    
    // Mock JWKS endpoint
    mock_jwks_endpoint(&public_key, "https://test.authkit.app/.well-known/jwks.json");
    mock_gateway_success();
    
    // Create token with single audience that matches one of the configured audiences
    let token = create_token(
        &private_key,
        "test-user",
        "https://test.authkit.app",
        Some(AudienceValue::Single("api-2".to_string())), // Matches second configured audience
        3600,
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
    
    // Should succeed - token audience matches one of the configured audiences
    assert_eq!(response.status(), 200, "Token with matching audience should be accepted");
}

// Test: Multiple configured audiences - reject token with non-matching audience
#[spin_test]
fn test_multiple_configured_audiences_single_token_audience_no_match() {
    // Configure with multiple audiences
    variables::set("mcp_gateway_url", "none");
    variables::set("mcp_jwt_issuer", "https://test.authkit.app");
    variables::set("mcp_jwt_jwks_uri", "https://test.authkit.app/.well-known/jwks.json");
    variables::set("mcp_jwt_audience", "api-1,api-2,api-3");
    
    let (private_key, public_key) = generate_rsa_key_pair();
    
    // Mock JWKS endpoint
    mock_jwks_endpoint(&public_key, "https://test.authkit.app/.well-known/jwks.json");
    
    // Create token with audience that doesn't match any configured audience
    let token = create_token(
        &private_key,
        "test-user",
        "https://test.authkit.app",
        Some(AudienceValue::Single("different-api".to_string())), // Does not match any configured audience
        3600,
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
    
    // Should fail - token audience doesn't match any configured audience
    assert_eq!(response.status(), 401, "Token with non-matching audience should be rejected");
}

// Test: Multiple configured audiences - accept token with multiple audiences where one matches
#[spin_test]
fn test_multiple_configured_audiences_multiple_token_audiences_partial_match() {
    // Configure with multiple audiences
    variables::set("mcp_gateway_url", "none");
    variables::set("mcp_jwt_issuer", "https://test.authkit.app");
    variables::set("mcp_jwt_jwks_uri", "https://test.authkit.app/.well-known/jwks.json");
    variables::set("mcp_jwt_audience", "api-1,api-2,api-3");
    
    let (private_key, public_key) = generate_rsa_key_pair();
    
    // Mock JWKS endpoint
    mock_jwks_endpoint(&public_key, "https://test.authkit.app/.well-known/jwks.json");
    mock_gateway_success();
    
    // Create token with multiple audiences, one of which matches a configured audience
    let token = create_token(
        &private_key,
        "test-user",
        "https://test.authkit.app",
        Some(AudienceValue::Multiple(vec![
            "different-api".to_string(),
            "api-2".to_string(), // This one matches
            "another-api".to_string(),
        ])),
        3600,
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
    
    // Should succeed - at least one token audience matches a configured audience
    assert_eq!(response.status(), 200, "Token with at least one matching audience should be accepted");
}

// Test: Multiple configured audiences - reject token with multiple audiences where none match
#[spin_test]
fn test_multiple_configured_audiences_multiple_token_audiences_no_match() {
    // Configure with multiple audiences
    variables::set("mcp_gateway_url", "none");
    variables::set("mcp_jwt_issuer", "https://test.authkit.app");
    variables::set("mcp_jwt_jwks_uri", "https://test.authkit.app/.well-known/jwks.json");
    variables::set("mcp_jwt_audience", "api-1,api-2,api-3");
    
    let (private_key, public_key) = generate_rsa_key_pair();
    
    // Mock JWKS endpoint
    mock_jwks_endpoint(&public_key, "https://test.authkit.app/.well-known/jwks.json");
    
    // Create token with multiple audiences, none of which match any configured audience
    let token = create_token(
        &private_key,
        "test-user",
        "https://test.authkit.app",
        Some(AudienceValue::Multiple(vec![
            "different-api".to_string(),
            "another-api".to_string(),
            "yet-another-api".to_string(),
        ])),
        3600,
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
    
    // Should fail - no token audience matches any configured audience
    assert_eq!(response.status(), 401, "Token with no matching audiences should be rejected");
}

// Test: Empty audiences after splitting comma-separated string
#[spin_test]
fn test_multiple_configured_audiences_with_empty_values() {
    // Configure with audiences that include empty values (e.g., trailing comma)
    variables::set("mcp_gateway_url", "none");
    variables::set("mcp_jwt_issuer", "https://test.authkit.app");
    variables::set("mcp_jwt_jwks_uri", "https://test.authkit.app/.well-known/jwks.json");
    variables::set("mcp_jwt_audience", "api-1,,api-2,"); // Contains empty values
    
    let (private_key, public_key) = generate_rsa_key_pair();
    
    // Mock JWKS endpoint
    mock_jwks_endpoint(&public_key, "https://test.authkit.app/.well-known/jwks.json");
    mock_gateway_success();
    
    // Create token with audience that matches one of the non-empty configured audiences
    let token = create_token(
        &private_key,
        "test-user",
        "https://test.authkit.app",
        Some(AudienceValue::Single("api-2".to_string())),
        3600,
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
    
    // Should succeed - empty values are filtered out during parsing
    assert_eq!(response.status(), 200, "Empty audience values should be filtered out");
}

// Test: Whitespace handling in comma-separated audiences
#[spin_test]
fn test_multiple_configured_audiences_with_whitespace() {
    // Configure with audiences that have whitespace
    variables::set("mcp_gateway_url", "none");
    variables::set("mcp_jwt_issuer", "https://test.authkit.app");
    variables::set("mcp_jwt_jwks_uri", "https://test.authkit.app/.well-known/jwks.json");
    variables::set("mcp_jwt_audience", " api-1 , api-2 , api-3 "); // Whitespace around values
    
    let (private_key, public_key) = generate_rsa_key_pair();
    
    // Mock JWKS endpoint
    mock_jwks_endpoint(&public_key, "https://test.authkit.app/.well-known/jwks.json");
    mock_gateway_success();
    
    // Create token with trimmed audience value
    let token = create_token(
        &private_key,
        "test-user",
        "https://test.authkit.app",
        Some(AudienceValue::Single("api-2".to_string())), // No whitespace
        3600,
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
    
    // Should succeed - whitespace should be trimmed during parsing
    assert_eq!(response.status(), 200, "Whitespace in audience configuration should be trimmed");
}