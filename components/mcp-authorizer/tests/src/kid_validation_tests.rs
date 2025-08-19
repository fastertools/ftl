use spin_test_sdk::{
    bindings::{fermyon::spin_wasi_virt::http_handler, wasi::http},
    spin_test,
};

use base64::Engine;
use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use rsa::traits::PublicKeyParts;
use rsa::{pkcs1::EncodeRsaPrivateKey, RsaPrivateKey, RsaPublicKey};
use serde_json::json;

// Import common test types
use crate::jwt_verification_tests::{configure_test_provider, AudienceValue, Claims};

/// Helper to generate RSA key pair
fn generate_test_key_pair() -> (RsaPrivateKey, RsaPublicKey) {
    let mut rng = rand::thread_rng();
    let bits = 2048;
    let private_key = RsaPrivateKey::new(&mut rng, bits).expect("failed to generate private key");
    let public_key = RsaPublicKey::from(&private_key);
    (private_key, public_key)
}

/// Create a JWT token with optional KID
fn create_token_with_kid(
    private_key: &RsaPrivateKey,
    issuer: &str,
    audience: &str,
    kid: Option<&str>,
) -> String {
    let now = Utc::now();
    let claims = Claims {
        sub: "test-user".to_string(),
        iss: issuer.to_string(),
        aud: Some(AudienceValue::Single(audience.to_string())),
        exp: (now + Duration::hours(1)).timestamp(),
        iat: now.timestamp(),
        scope: Some("read write".to_string()),
        scp: None,
        client_id: None,
        additional: serde_json::Map::new(),
    };

    let mut header = Header::new(Algorithm::RS256);
    if let Some(k) = kid {
        header.kid = Some(k.to_string());
    }

    let pem_string = private_key
        .to_pkcs1_pem(rsa::pkcs1::LineEnding::LF)
        .unwrap();
    let encoding_key = EncodingKey::from_rsa_pem(pem_string.as_bytes()).unwrap();

    jsonwebtoken::encode(&header, &claims, &encoding_key).unwrap()
}

/// Create JWKS with KID
fn create_jwks_with_kid(public_key: &RsaPublicKey, kid: &str) -> serde_json::Value {
    let n = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(public_key.n().to_bytes_be());
    let e = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(public_key.e().to_bytes_be());

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

/// Mock JWKS endpoint
fn mock_jwks_endpoint(jwks: serde_json::Value) {
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
        "https://test-gateway.spin.internal/mcp",
        http_handler::ResponseHandler::Response(response),
    );
}

// Test: Token with KID matching JWKS
#[spin_test]
fn test_jwks_token_validation_with_kid() {
    configure_test_provider();

    let (private_key, public_key) = generate_test_key_pair();
    let kid = "test-key-1";

    // Create JWKS with KID
    let jwks = create_jwks_with_kid(&public_key, kid);
    mock_jwks_endpoint(jwks);
    mock_gateway();

    // Create token with matching KID
    let token = create_token_with_kid(
        &private_key,
        "https://test.authkit.app",
        "test-audience",
        Some(kid),
    );

    // Make request
    let headers = http::types::Headers::new();
    headers
        .append("authorization", format!("Bearer {}", token).as_bytes())
        .unwrap();
    headers.append("content-type", b"application/json").unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_path_with_query(Some("/mcp")).unwrap();
    request.set_method(&http::types::Method::Post).unwrap();

    let body = request.body().unwrap();
    body.write_bytes(b"{\"jsonrpc\":\"2.0\",\"method\":\"test\",\"id\":1}");

    let response = spin_test_sdk::perform_request(request);

    // Should succeed
    assert_eq!(response.status(), 200);
}

// Test: Token without KID when JWKS has KID
#[spin_test]
fn test_jwks_token_validation_with_kid_and_no_kid_in_token() {
    configure_test_provider();

    let (private_key, public_key) = generate_test_key_pair();
    let kid = "test-key-1";

    // Create JWKS with KID
    let jwks = create_jwks_with_kid(&public_key, kid);
    mock_jwks_endpoint(jwks);
    mock_gateway();

    // Create token WITHOUT KID
    let token = create_token_with_kid(
        &private_key,
        "https://test.authkit.app",
        "test-audience",
        None,
    );

    // Make request
    let headers = http::types::Headers::new();
    headers
        .append("authorization", format!("Bearer {}", token).as_bytes())
        .unwrap();
    headers.append("content-type", b"application/json").unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_path_with_query(Some("/mcp")).unwrap();
    request.set_method(&http::types::Method::Post).unwrap();

    let body = request.body().unwrap();
    body.write_bytes(b"{\"jsonrpc\":\"2.0\",\"method\":\"test\",\"id\":1}");

    let response = spin_test_sdk::perform_request(request);

    // Should succeed - when token has no KID, we try all keys
    assert_eq!(response.status(), 200);
}

// Test: Token with KID mismatch
#[spin_test]
fn test_jwks_token_validation_with_kid_mismatch() {
    configure_test_provider();

    let (private_key, public_key) = generate_test_key_pair();

    // Create JWKS with one KID
    let jwks = create_jwks_with_kid(&public_key, "test-key-1");
    mock_jwks_endpoint(jwks);

    // Create token with different KID
    let token = create_token_with_kid(
        &private_key,
        "https://test.authkit.app",
        "test-audience",
        Some("test-key-2"),
    );

    // Make request
    let headers = http::types::Headers::new();
    headers
        .append("authorization", format!("Bearer {}", token).as_bytes())
        .unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_path_with_query(Some("/mcp")).unwrap();

    let response = spin_test_sdk::perform_request(request);

    // Should fail - KID mismatch
    assert_eq!(response.status(), 401);
}

// Test: Multiple keys in JWKS with no KID in token
#[spin_test]
fn test_jwks_token_validation_with_multiple_keys_and_no_kid_in_token() {
    configure_test_provider();

    let (private_key1, public_key1) = generate_test_key_pair();
    let (_private_key2, public_key2) = generate_test_key_pair();

    // Create JWKS with multiple keys
    let n1 =
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(public_key1.n().to_bytes_be());
    let e1 =
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(public_key1.e().to_bytes_be());

    let n2 =
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(public_key2.n().to_bytes_be());
    let e2 =
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(public_key2.e().to_bytes_be());

    let jwks = json!({
        "keys": [
            {
                "kty": "RSA",
                "use": "sig",
                "alg": "RS256",
                "kid": "test-key-1",
                "n": n1,
                "e": e1
            },
            {
                "kty": "RSA",
                "use": "sig",
                "alg": "RS256",
                "kid": "test-key-2",
                "n": n2,
                "e": e2
            }
        ]
    });

    mock_jwks_endpoint(jwks);
    mock_gateway();

    // Create token without KID
    let token = create_token_with_kid(
        &private_key1,
        "https://test.authkit.app",
        "test-audience",
        None,
    );

    // Make request
    let headers = http::types::Headers::new();
    headers
        .append("authorization", format!("Bearer {}", token).as_bytes())
        .unwrap();
    headers.append("content-type", b"application/json").unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_path_with_query(Some("/mcp")).unwrap();
    request.set_method(&http::types::Method::Post).unwrap();

    let body = request.body().unwrap();
    body.write_bytes(b"{\"jsonrpc\":\"2.0\",\"method\":\"test\",\"id\":1}");

    let response = spin_test_sdk::perform_request(request);

    // Should fail - multiple keys but no KID in token
    assert_eq!(response.status(), 401);
}

// Test: Token without KID when JWKS has KID
#[spin_test]
fn test_jwks_token_validation_with_no_kid_and_kid_in_jwks() {
    configure_test_provider();

    let (private_key, public_key) = generate_test_key_pair();

    // Create JWKS WITH KID
    let n = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(public_key.n().to_bytes_be());
    let e = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(public_key.e().to_bytes_be());

    let jwks = json!({
        "keys": [{
            "kty": "RSA",
            "use": "sig",
            "alg": "RS256",
            "kid": "test-key-1",  // JWKS has KID
            "n": n,
            "e": e
        }]
    });

    mock_jwks_endpoint(jwks);
    mock_gateway();

    // Create token WITHOUT KID
    let token = create_token_with_kid(
        &private_key,
        "https://test.authkit.app",
        "test-audience",
        None,
    );

    // Make request
    let headers = http::types::Headers::new();
    headers
        .append("authorization", format!("Bearer {}", token).as_bytes())
        .unwrap();
    headers.append("content-type", b"application/json").unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_path_with_query(Some("/mcp")).unwrap();
    request.set_method(&http::types::Method::Post).unwrap();

    let body = request.body().unwrap();
    body.write_bytes(b"{\"jsonrpc\":\"2.0\",\"method\":\"test\",\"id\":1}");

    let response = spin_test_sdk::perform_request(request);

    // Should succeed - when token has no KID, it can match a JWKS key with KID
    assert_eq!(response.status(), 200);
}
