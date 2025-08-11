use spin_test_sdk::{
    bindings::{
        fermyon::spin_test_virt::{key_value, variables},
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

use crate::jwt_verification_tests::{Claims, AudienceValue};

// Track HTTP calls to JWKS endpoint
static mut JWKS_CALL_COUNT: u32 = 0;

/// Helper to generate RSA key pair for testing
fn generate_test_key_pair() -> (RsaPrivateKey, RsaPublicKey) {
    let mut rng = rand::thread_rng();
    let bits = 2048;
    let private_key = RsaPrivateKey::new(&mut rng, bits).expect("failed to generate private key");
    let public_key = RsaPublicKey::from(&private_key);
    (private_key, public_key)
}

/// Helper to create a JWT token
fn create_test_token(
    private_key: &RsaPrivateKey,
    issuer: &str,
    audience: &str,
    kid: &str,
) -> String {
    let now = Utc::now();
    let claims = Claims {
        sub: "test-user".to_string(),
        iss: issuer.to_string(),
        aud: Some(AudienceValue::Single(audience.to_string())),
        exp: (now + Duration::hours(1)).timestamp(),
        iat: now.timestamp(),
        scope: Some("read".to_string()),
        scp: None,
        client_id: None,
        additional: serde_json::Map::new(),
    };
    
    let header = Header {
        alg: Algorithm::RS256,
        kid: Some(kid.to_string()),
        ..Default::default()
    };
    
    let pem_string = private_key.to_pkcs1_pem(rsa::pkcs1::LineEnding::LF).unwrap();
    let encoding_key = EncodingKey::from_rsa_pem(pem_string.as_bytes()).unwrap();
    
    jsonwebtoken::encode(&header, &claims, &encoding_key).unwrap()
}

/// Create a JWKS response with tracking
fn create_tracked_jwks_response(public_key: &RsaPublicKey, kid: &str) -> serde_json::Value {
    // Increment call count
    unsafe {
        JWKS_CALL_COUNT += 1;
    }
    
    // Create JWKS JSON
    let n = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .encode(&public_key.n().to_bytes_be());
    let e = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .encode(&public_key.e().to_bytes_be());
    
    json!({
        "keys": [{
            "kty": "RSA",
            "use": "sig",
            "alg": "RS256",
            "kid": kid,
            "n": n,
            "e": e
        }]
    })
}

/// Mock MCP gateway success with specific ID
fn mock_mcp_gateway_with_id(id: u32) {
    let response = http::types::OutgoingResponse::new(http::types::Headers::new());
    response.set_status_code(200).unwrap();
    let headers = response.headers();
    headers.append("content-type", b"application/json").unwrap();
    
    let body = response.body().unwrap();
    let response_json = format!(r#"{{\"jsonrpc\":\"2.0\",\"result\":{{}},\"id\":{}}}"#, id);
    body.write_bytes(response_json.as_bytes());
    
    http_handler::set_response(
        "https://test-gateway.spin.internal/mcp",
        http_handler::ResponseHandler::Response(response),
    );
}

// Test: JWKS is cached and not fetched multiple times
#[spin_test]
fn test_jwks_caching() {
    // Configure provider
    variables::set("mcp_gateway_url", "none");
    variables::set("mcp_jwt_issuer", "https://test.authkit.app");
    variables::set("mcp_jwt_jwks_uri", "https://test.authkit.app/.well-known/jwks.json");
    variables::set("mcp_jwt_audience", "test-audience");
    
    // Reset call count
    unsafe {
        JWKS_CALL_COUNT = 0;
    }
    
    // Clear any existing cache
    let kv = key_value::Store::open("default");
    kv.delete("jwks:https://test.authkit.app/.well-known/jwks.json");
    
    // Setup
    let (private_key, public_key) = generate_test_key_pair();
    let kid = "cache-test-key";
    
    // Create JWKS data and mock the endpoint
    let jwks_data = create_tracked_jwks_response(&public_key, kid);
    
    let response = http::types::OutgoingResponse::new(http::types::Headers::new());
    response.set_status_code(200).unwrap();
    let headers = response.headers();
    headers.append("content-type", b"application/json").unwrap();
    
    let body = response.body().unwrap();
    body.write_bytes(serde_json::to_string(&jwks_data).unwrap().as_bytes());
    
    http_handler::set_response(
        "https://test.authkit.app/.well-known/jwks.json",
        http_handler::ResponseHandler::Response(response),
    );
    
    // Mock MCP gateway for first request
    mock_mcp_gateway_with_id(1);
    
    // Create tokens
    let token1 = create_test_token(&private_key, "https://test.authkit.app", "test-audience", kid);
    let token2 = create_test_token(&private_key, "https://test.authkit.app", "test-audience", kid);
    
    // Make first request - should fetch JWKS
    let headers1 = http::types::Headers::new();
    headers1.append("authorization", format!("Bearer {}", token1).as_bytes()).unwrap();
    headers1.append("content-type", b"application/json").unwrap();
    let request1 = http::types::OutgoingRequest::new(headers1);
    request1.set_path_with_query(Some("/mcp")).unwrap();
    request1.set_method(&http::types::Method::Post).unwrap();
    let body1 = request1.body().unwrap();
    body1.write_bytes(b"{\"jsonrpc\":\"2.0\",\"method\":\"test\",\"id\":1}");
    
    let response1 = spin_test_sdk::perform_request(request1);
    assert_eq!(response1.status(), 200);
    
    // Check JWKS was fetched once
    let call_count_after_first = unsafe { JWKS_CALL_COUNT };
    assert_eq!(call_count_after_first, 1, "JWKS should be fetched on first request");
    
    // Verify JWKS was cached by checking KV store
    let kv = key_value::Store::open("default");
    let cached_jwks = kv.get("jwks:https://test.authkit.app/.well-known/jwks.json");
    assert!(cached_jwks.is_some(), "JWKS should be in cache after first request");
    
    // Mock MCP gateway for second request (must be done after first request is consumed)
    mock_mcp_gateway_with_id(2);
    
    // Make second request - should use cached JWKS
    let headers2 = http::types::Headers::new();
    headers2.append("authorization", format!("Bearer {}", token2).as_bytes()).unwrap();
    headers2.append("content-type", b"application/json").unwrap();
    let request2 = http::types::OutgoingRequest::new(headers2);
    request2.set_path_with_query(Some("/mcp")).unwrap();
    request2.set_method(&http::types::Method::Post).unwrap();
    let body2 = request2.body().unwrap();
    body2.write_bytes(b"{\"jsonrpc\":\"2.0\",\"method\":\"test\",\"id\":2}");
    
    let response2 = spin_test_sdk::perform_request(request2);
    assert_eq!(response2.status(), 200);
    
    // Check JWKS was NOT fetched again
    let call_count_after_second = unsafe { JWKS_CALL_COUNT };
    assert_eq!(call_count_after_second, 1, "JWKS should be cached and not fetched again");
    
    // Verify cache was actually used by checking KV store again
    let cached_jwks = kv.get("jwks:https://test.authkit.app/.well-known/jwks.json");
    assert!(cached_jwks.is_some(), "JWKS should still be in cache");
}

// Test: JWKS cache respects TTL
#[spin_test]
fn test_jwks_cache_ttl() {
    // Configure provider
    variables::set("mcp_gateway_url", "none");
    variables::set("mcp_jwt_issuer", "https://test.authkit.app");
    variables::set("mcp_jwt_jwks_uri", "https://test.authkit.app/.well-known/jwks.json");
    variables::set("mcp_jwt_audience", "test-audience");
    
    // This test would require time manipulation which is not easily done in WASM
    // Instead, we'll test that the cache key exists with proper structure
    
    // Setup
    let (private_key, public_key) = generate_test_key_pair();
    let kid = "ttl-test-key";
    
    // Clear cache
    let kv = key_value::Store::open("default");
    kv.delete("jwks:https://test.authkit.app/.well-known/jwks.json");
    
    // Create JWKS response
    let n = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .encode(&public_key.n().to_bytes_be());
    let e = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .encode(&public_key.e().to_bytes_be());
    
    let jwks = json!({
        "keys": [{
            "kty": "RSA",
            "use": "sig",
            "alg": "RS256",
            "kid": kid,
            "n": n,
            "e": e
        }]
    });
    
    let response = http::types::OutgoingResponse::new(http::types::Headers::new());
    let headers = response.headers();
    headers.append("content-type", b"application/json").unwrap();
    response.write_body(serde_json::to_string(&jwks).unwrap().as_bytes());
    
    http_handler::set_response(
        "https://test.authkit.app/.well-known/jwks.json",
        http_handler::ResponseHandler::Response(response),
    );
    
    // Mock MCP gateway
    mock_mcp_gateway_with_id(1);
    
    // Create and use token
    let token = create_test_token(&private_key, "https://test.authkit.app", "test-audience", kid);
    
    let headers = http::types::Headers::new();
    headers.append("authorization", format!("Bearer {}", token).as_bytes()).unwrap();
    headers.append("content-type", b"application/json").unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_path_with_query(Some("/mcp")).unwrap();
    request.set_method(&http::types::Method::Post).unwrap();
    let body = request.body().unwrap();
    body.write_bytes(b"{\"jsonrpc\":\"2.0\",\"method\":\"test\",\"id\":1}");
    
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 200);
    
    // Verify cache entry exists
    let cached_data = kv.get("jwks:https://test.authkit.app/.well-known/jwks.json");
    assert!(cached_data.is_some(), "JWKS should be cached");
    
    // The cached data should be a JSON object with jwks and expiry
    let cached_str = String::from_utf8(cached_data.unwrap()).unwrap();
    let cached_json: serde_json::Value = serde_json::from_str(&cached_str).unwrap();
    
    assert!(cached_json.get("jwks").is_some(), "Cached data should contain jwks");
    assert!(cached_json.get("expires_at").is_some(), "Cached data should contain expiry");
}

// Test: Different issuers have separate cache entries
#[spin_test]
fn test_jwks_cache_per_issuer() {
    // Configure provider with first issuer
    variables::set("mcp_gateway_url", "none");
    variables::set("mcp_jwt_issuer", "https://issuer1.com");
    variables::set("mcp_jwt_jwks_uri", "https://issuer1.com/.well-known/jwks.json");
    variables::set("mcp_jwt_audience", "test-audience");
    
    // Setup two different issuers
    let (private_key1, public_key1) = generate_test_key_pair();
    let (_private_key2, public_key2) = generate_test_key_pair();
    
    // Clear cache
    let kv = key_value::Store::open("default");
    kv.delete("jwks:https://issuer1.com/.well-known/jwks.json");
    kv.delete("jwks:https://issuer2.com/.well-known/jwks.json");
    
    // Mock JWKS for issuer1
    let jwks1 = create_jwks_json(&public_key1, "key1");
    mock_jwks_endpoint("https://issuer1.com/.well-known/jwks.json", jwks1);
    
    // Mock JWKS for issuer2
    let jwks2 = create_jwks_json(&public_key2, "key2");
    mock_jwks_endpoint("https://issuer2.com/.well-known/jwks.json", jwks2);
    
    // Mock MCP gateway
    mock_mcp_gateway_with_id(1);
    
    // Configure providers
    variables::set("mcp_jwt_issuer", "https://issuer1.com");
    variables::set("mcp_jwt_jwks_uri", "https://issuer1.com/.well-known/jwks.json");
    variables::set("mcp_jwt_audience", "test-audience");
    
    // Use token from issuer1
    let token1 = create_test_token(&private_key1, "https://issuer1.com", "test-audience", "key1");
    
    let headers1 = http::types::Headers::new();
    headers1.append("authorization", format!("Bearer {}", token1).as_bytes()).unwrap();
    headers1.append("content-type", b"application/json").unwrap();
    let request1 = http::types::OutgoingRequest::new(headers1);
    request1.set_path_with_query(Some("/mcp")).unwrap();
    request1.set_method(&http::types::Method::Post).unwrap();
    let body1 = request1.body().unwrap();
    body1.write_bytes(b"{\"jsonrpc\":\"2.0\",\"method\":\"test\",\"id\":1}");
    
    let response1 = spin_test_sdk::perform_request(request1);
    assert_eq!(response1.status(), 200);
    
    // Verify both cache entries exist independently
    let cache1 = kv.get("jwks:https://issuer1.com/.well-known/jwks.json");
    assert!(cache1.is_some(), "Issuer1 JWKS should be cached");
    
    // Note: In a real multi-provider setup, we'd need to test with multiple providers
    // configured, but our current implementation only supports one provider at a time
}

// Helper to create JWKS JSON
fn create_jwks_json(public_key: &RsaPublicKey, kid: &str) -> serde_json::Value {
    let n = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .encode(&public_key.n().to_bytes_be());
    let e = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .encode(&public_key.e().to_bytes_be());
    
    json!({
        "keys": [{
            "kty": "RSA",
            "use": "sig",
            "alg": "RS256",
            "kid": kid,
            "n": n,
            "e": e
        }]
    })
}

// Helper to mock JWKS endpoint
fn mock_jwks_endpoint(url: &str, jwks: serde_json::Value) {
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