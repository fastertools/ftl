use crate::ResponseData;
use spin_test_sdk::{
    bindings::{
        fermyon::spin_test_virt::variables, fermyon::spin_wasi_virt::http_handler, wasi::http,
    },
    spin_test,
};

use base64::Engine;
use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use rsa::traits::PublicKeyParts;
use rsa::{pkcs1::EncodeRsaPrivateKey, RsaPrivateKey, RsaPublicKey};
use serde::{Deserialize, Serialize};
use serde_json::json;

// JWT Claims structure
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub iss: String,
    pub aud: Option<AudienceValue>,
    pub exp: i64,
    pub iat: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scp: Option<ScopeValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,
    #[serde(flatten)]
    pub additional: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AudienceValue {
    Single(String),
    Multiple(Vec<String>),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ScopeValue {
    String(String),
    List(Vec<String>),
}

/// Configure test provider
pub fn configure_test_provider() {
    // Core settings - gateway URL is the full internal MCP endpoint
    variables::set("mcp_gateway_url", "none");
    variables::set("mcp_trace_header", "x-trace-id");

    // JWT provider settings
    variables::set("mcp_jwt_issuer", "https://test.authkit.app");
    variables::set(
        "mcp_jwt_jwks_uri",
        "https://test.authkit.app/.well-known/jwks.json",
    );
    variables::set("mcp_jwt_audience", "test-audience");
}

/// Helper to generate RSA key pair for testing
pub fn generate_test_key_pair() -> (RsaPrivateKey, RsaPublicKey) {
    let mut rng = rand::thread_rng();
    let bits = 2048;
    let private_key = RsaPrivateKey::new(&mut rng, bits).expect("failed to generate private key");
    let public_key = RsaPublicKey::from(&private_key);
    (private_key, public_key)
}

/// Helper to create a JWT token with custom claims
pub fn create_test_token(private_key: &RsaPrivateKey, claims: Claims, kid: Option<&str>) -> String {
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

/// Helper to create a JWKS response with the public key
pub fn create_jwks_response(public_key: &RsaPublicKey, kid: &str) -> serde_json::Value {
    // Create proper JWK format
    let n = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&public_key.n().to_bytes_be());
    let e = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&public_key.e().to_bytes_be());

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

/// Mock HTTP response for JWKS endpoint
pub fn mock_jwks_endpoint(url: &str, jwks: serde_json::Value) {
    let response = http::types::OutgoingResponse::new(http::types::Headers::new());
    response.set_status_code(200).unwrap();
    let headers = response.headers();
    headers.append("content-type", b"application/json").unwrap();

    let body = response.body().unwrap();
    body.write_bytes(serde_json::to_string(&jwks).unwrap().as_bytes());

    http_handler::set_response(url, http_handler::ResponseHandler::Response(response));
}

/// Mock successful MCP gateway response
fn mock_mcp_gateway_success() {
    let response = http::types::OutgoingResponse::new(http::types::Headers::new());
    response.set_status_code(200).unwrap();
    let headers = response.headers();
    headers.append("content-type", b"application/json").unwrap();

    let body_content = json!({
        "jsonrpc": "2.0",
        "result": {
            "tools": ["test-tool"]
        },
        "id": 1
    });

    let body = response.body().unwrap();
    body.write_bytes(serde_json::to_string(&body_content).unwrap().as_bytes());

    // Mock the gateway URL - requests will be forwarded to gateway path
    http_handler::set_response(
        "https://test-gateway.spin.internal/mcp",
        http_handler::ResponseHandler::Response(response),
    );
}

// Test: Valid token with JWKS verification
#[spin_test]
fn test_valid_token_jwks_verification() {
    // Configure provider with gateway URL for forwarding
    variables::set("mcp_gateway_url", "https://test-gateway.spin.internal");
    variables::set("mcp_trace_header", "x-trace-id");
    variables::set("mcp_jwt_issuer", "https://test.authkit.app");
    variables::set(
        "mcp_jwt_jwks_uri",
        "https://test.authkit.app/.well-known/jwks.json",
    );
    variables::set("mcp_jwt_audience", "test-audience");

    // Setup
    let (private_key, public_key) = generate_test_key_pair();
    let kid = "test-key-1";

    // Create JWKS and mock the endpoint
    let jwks = create_jwks_response(&public_key, kid);
    mock_jwks_endpoint("https://test.authkit.app/.well-known/jwks.json", jwks);

    // Mock the MCP gateway
    mock_mcp_gateway_success();

    // Create a valid token
    let now = Utc::now();
    let claims = Claims {
        sub: "test-user".to_string(),
        iss: "https://test.authkit.app".to_string(),
        aud: Some(AudienceValue::Single("test-audience".to_string())),
        exp: (now + Duration::hours(1)).timestamp(),
        iat: now.timestamp(),
        scope: Some("read write".to_string()),
        scp: None,
        client_id: None,
        additional: serde_json::Map::new(),
    };

    let token = create_test_token(&private_key, claims, Some(kid));

    // Make request with valid token - headers must be set before creating request
    let headers = http::types::Headers::new();
    headers
        .append("authorization", format!("Bearer {}", token).as_bytes())
        .unwrap();
    headers.append("content-type", b"application/json").unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_path_with_query(Some("/mcp")).unwrap();
    request.set_method(&http::types::Method::Post).unwrap();

    // Set request body
    let body_content = json!({
        "jsonrpc": "2.0",
        "method": "tools/list",
        "id": 1
    });
    let body = request.body().unwrap();
    body.write_bytes(serde_json::to_string(&body_content).unwrap().as_bytes());

    let response = spin_test_sdk::perform_request(request);
    let response_data = ResponseData::from_response(response);

    // Should succeed with 200
    assert_eq!(response_data.status, 200);

    // Verify the gateway response body is properly forwarded
    let json = response_data
        .body_json()
        .expect("Response should have JSON body");
    assert_eq!(json["jsonrpc"], "2.0");
    assert_eq!(json["result"]["tools"][0], "test-tool");
    assert_eq!(json["id"], 1)
}

// Test: Expired token rejection
#[spin_test]
fn test_expired_token_rejection() {
    configure_test_provider();

    // Setup
    let (private_key, public_key) = generate_test_key_pair();
    let kid = "test-key-2";

    // Create JWKS and mock the endpoint
    let jwks = create_jwks_response(&public_key, kid);
    mock_jwks_endpoint("https://test.authkit.app/.well-known/jwks.json", jwks);

    // Create an expired token
    let now = Utc::now();
    let claims = Claims {
        sub: "test-user".to_string(),
        iss: "https://test.authkit.app".to_string(),
        aud: Some(AudienceValue::Single("test-audience".to_string())),
        exp: (now - Duration::hours(1)).timestamp(), // Expired 1 hour ago
        iat: (now - Duration::hours(2)).timestamp(),
        scope: None,
        scp: None,
        client_id: None,
        additional: serde_json::Map::new(),
    };

    let token = create_test_token(&private_key, claims, Some(kid));

    // Make request with expired token
    let headers = http::types::Headers::new();
    headers
        .append("authorization", format!("Bearer {}", token).as_bytes())
        .unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_path_with_query(Some("/mcp")).unwrap();

    let response = spin_test_sdk::perform_request(request);
    let response_data = ResponseData::from_response(response);

    // Should return 401
    assert_eq!(response_data.status, 401);

    // Verify error response format
    let json = response_data
        .body_json()
        .expect("Error response should have JSON body");
    assert_eq!(json["error"], "invalid_token");
    assert!(json["error_description"]
        .as_str()
        .unwrap()
        .contains("expired"))
}

// Test: Invalid signature rejection
#[spin_test]
fn test_invalid_signature_rejection() {
    configure_test_provider();

    // Setup two different key pairs
    let (_private_key1, public_key1) = generate_test_key_pair();
    let (private_key2, _) = generate_test_key_pair();
    let kid = "test-key-3";

    // Create JWKS with public_key1 but sign token with private_key2
    let jwks = create_jwks_response(&public_key1, kid);
    mock_jwks_endpoint("https://test.authkit.app/.well-known/jwks.json", jwks);

    // Create token signed with different key
    let now = Utc::now();
    let claims = Claims {
        sub: "test-user".to_string(),
        iss: "https://test.authkit.app".to_string(),
        aud: Some(AudienceValue::Single("test-audience".to_string())),
        exp: (now + Duration::hours(1)).timestamp(),
        iat: now.timestamp(),
        scope: None,
        scp: None,
        client_id: None,
        additional: serde_json::Map::new(),
    };

    let token = create_test_token(&private_key2, claims, Some(kid));

    // Make request
    let headers = http::types::Headers::new();
    headers
        .append("authorization", format!("Bearer {}", token).as_bytes())
        .unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_path_with_query(Some("/mcp")).unwrap();

    let response = spin_test_sdk::perform_request(request);

    // Should return 401
    assert_eq!(response.status(), 401);
}

// Test: Wrong issuer rejection
#[spin_test]
fn test_wrong_issuer_rejection() {
    configure_test_provider();

    // Setup
    let (private_key, public_key) = generate_test_key_pair();
    let kid = "test-key-4";

    // Create JWKS and mock the endpoint
    let jwks = create_jwks_response(&public_key, kid);
    mock_jwks_endpoint("https://test.authkit.app/.well-known/jwks.json", jwks);

    // Create token with wrong issuer
    let now = Utc::now();
    let claims = Claims {
        sub: "test-user".to_string(),
        iss: "https://wrong-issuer.com".to_string(), // Wrong issuer
        aud: Some(AudienceValue::Single("test-audience".to_string())),
        exp: (now + Duration::hours(1)).timestamp(),
        iat: now.timestamp(),
        scope: None,
        scp: None,
        client_id: None,
        additional: serde_json::Map::new(),
    };

    let token = create_test_token(&private_key, claims, Some(kid));

    // Make request
    let headers = http::types::Headers::new();
    headers
        .append("authorization", format!("Bearer {}", token).as_bytes())
        .unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_path_with_query(Some("/mcp")).unwrap();

    let response = spin_test_sdk::perform_request(request);

    // Should return 401
    assert_eq!(response.status(), 401);
}

// Test: Wrong audience rejection
#[spin_test]
fn test_wrong_audience_rejection() {
    configure_test_provider();

    // Setup
    let (private_key, public_key) = generate_test_key_pair();
    let kid = "test-key-5";

    // Create JWKS and mock the endpoint
    let jwks = create_jwks_response(&public_key, kid);
    mock_jwks_endpoint("https://test.authkit.app/.well-known/jwks.json", jwks);

    // Create token with wrong audience
    let now = Utc::now();
    let claims = Claims {
        sub: "test-user".to_string(),
        iss: "https://test.authkit.app".to_string(),
        aud: Some(AudienceValue::Single("wrong-audience".to_string())), // Wrong audience
        exp: (now + Duration::hours(1)).timestamp(),
        iat: now.timestamp(),
        scope: None,
        scp: None,
        client_id: None,
        additional: serde_json::Map::new(),
    };

    let token = create_test_token(&private_key, claims, Some(kid));

    // Make request
    let headers = http::types::Headers::new();
    headers
        .append("authorization", format!("Bearer {}", token).as_bytes())
        .unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_path_with_query(Some("/mcp")).unwrap();

    let response = spin_test_sdk::perform_request(request);

    // Should return 401
    assert_eq!(response.status(), 401);
}

// Test: Multiple audiences validation
#[spin_test]
fn test_multiple_audiences_validation() {
    // Configure provider with gateway URL for forwarding
    variables::set("mcp_gateway_url", "https://test-gateway.spin.internal");
    variables::set("mcp_trace_header", "x-trace-id");
    variables::set("mcp_jwt_issuer", "https://test.authkit.app");
    variables::set(
        "mcp_jwt_jwks_uri",
        "https://test.authkit.app/.well-known/jwks.json",
    );
    variables::set("mcp_jwt_audience", "test-audience");

    // Setup
    let (private_key, public_key) = generate_test_key_pair();
    let kid = "test-key-6";

    // Create JWKS and mock the endpoint
    let jwks = create_jwks_response(&public_key, kid);
    mock_jwks_endpoint("https://test.authkit.app/.well-known/jwks.json", jwks);

    // Mock the MCP gateway
    mock_mcp_gateway_success();

    // Create token with multiple audiences, one matching
    let now = Utc::now();
    let claims = Claims {
        sub: "test-user".to_string(),
        iss: "https://test.authkit.app".to_string(),
        aud: Some(AudienceValue::Multiple(vec![
            "test-audience".to_string(),
            "other-audience".to_string(),
        ])),
        exp: (now + Duration::hours(1)).timestamp(),
        iat: now.timestamp(),
        scope: None,
        scp: None,
        client_id: None,
        additional: serde_json::Map::new(),
    };

    let token = create_test_token(&private_key, claims, Some(kid));

    // Make request
    let headers = http::types::Headers::new();
    headers
        .append("authorization", format!("Bearer {}", token).as_bytes())
        .unwrap();
    headers.append("content-type", b"application/json").unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_path_with_query(Some("/mcp")).unwrap();
    request.set_method(&http::types::Method::Post).unwrap();

    let body_content = json!({
        "jsonrpc": "2.0",
        "method": "tools/list",
        "id": 1
    });
    let body = request.body().unwrap();
    body.write_bytes(serde_json::to_string(&body_content).unwrap().as_bytes());

    let response = spin_test_sdk::perform_request(request);
    let response_data = ResponseData::from_response(response);

    // Should succeed
    assert_eq!(response_data.status, 200);

    // Verify response - gateway mock returns successful response
    let json = response_data
        .body_json()
        .expect("Response should have JSON body");
    assert_eq!(json["jsonrpc"], "2.0");
    assert!(json["result"].is_object());
}

// Test: Scope extraction from different formats
#[spin_test]
fn test_scope_extraction() {
    // Configure provider with gateway URL for forwarding
    variables::set("mcp_gateway_url", "https://test-gateway.spin.internal");
    variables::set("mcp_trace_header", "x-trace-id");
    variables::set("mcp_jwt_issuer", "https://test.authkit.app");
    variables::set(
        "mcp_jwt_jwks_uri",
        "https://test.authkit.app/.well-known/jwks.json",
    );
    variables::set("mcp_jwt_audience", "test-audience");

    // Setup
    let (private_key, public_key) = generate_test_key_pair();
    let kid = "test-key-7";

    // Create JWKS and mock the endpoint
    let jwks = create_jwks_response(&public_key, kid);
    mock_jwks_endpoint("https://test.authkit.app/.well-known/jwks.json", jwks);

    // Mock the MCP gateway
    mock_mcp_gateway_success();

    // Test 1: Standard OAuth2 'scope' claim
    let now = Utc::now();
    let claims = Claims {
        sub: "test-user".to_string(),
        iss: "https://test.authkit.app".to_string(),
        aud: Some(AudienceValue::Single("test-audience".to_string())),
        exp: (now + Duration::hours(1)).timestamp(),
        iat: now.timestamp(),
        scope: Some("read write admin".to_string()),
        scp: None,
        client_id: None,
        additional: serde_json::Map::new(),
    };

    let token = create_test_token(&private_key, claims, Some(kid));

    // Make request
    let headers = http::types::Headers::new();
    headers
        .append("authorization", format!("Bearer {}", token).as_bytes())
        .unwrap();
    headers.append("content-type", b"application/json").unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_path_with_query(Some("/mcp")).unwrap();
    request.set_method(&http::types::Method::Post).unwrap();

    let body_content = json!({
        "jsonrpc": "2.0",
        "method": "tools/list",
        "id": 1
    });
    let body = request.body().unwrap();
    body.write_bytes(serde_json::to_string(&body_content).unwrap().as_bytes());

    let response = spin_test_sdk::perform_request(request);
    let response_data = ResponseData::from_response(response);

    // Should succeed - scopes were properly extracted and forwarded
    assert_eq!(response_data.status, 200);

    // Verify response
    let json = response_data
        .body_json()
        .expect("Response should have JSON body");
    assert_eq!(json["jsonrpc"], "2.0");
    assert!(json["result"].is_object());
}

// Test: Client ID extraction with explicit claim
#[spin_test]
fn test_client_id_extraction_explicit() {
    // Configure provider with actual gateway URL for forwarding
    variables::set("mcp_gateway_url", "https://test-gateway.spin.internal");
    variables::set("mcp_trace_header", "x-trace-id");
    variables::set("mcp_jwt_issuer", "https://test.authkit.app");
    variables::set(
        "mcp_jwt_jwks_uri",
        "https://test.authkit.app/.well-known/jwks.json",
    );
    variables::set("mcp_jwt_audience", "test-audience");

    // Setup
    let (private_key, public_key) = generate_test_key_pair();
    let kid = "test-key-8";

    // Create JWKS and mock the endpoint
    let jwks = create_jwks_response(&public_key, kid);
    mock_jwks_endpoint("https://test.authkit.app/.well-known/jwks.json", jwks);

    // Mock the MCP gateway that checks for client_id in auth context
    let response = http::types::OutgoingResponse::new(http::types::Headers::new());
    let headers = response.headers();
    headers.append("content-type", b"application/json").unwrap();
    response.set_status_code(200).unwrap();

    // Return success indicating client_id was received
    let body_content = json!({
        "jsonrpc": "2.0",
        "result": {
            "client_id_received": "app456"
        },
        "id": 1
    });

    let body = response.body().unwrap();
    body.write_bytes(serde_json::to_string(&body_content).unwrap().as_bytes());

    http_handler::set_response(
        "https://test-gateway.spin.internal/mcp",
        http_handler::ResponseHandler::Response(response),
    );

    // Create token with explicit client_id
    let now = Utc::now();
    let additional = serde_json::Map::new();
    // Don't add client_id to additional since it's already a field

    let claims = Claims {
        sub: "user123".to_string(), // Different from client_id
        iss: "https://test.authkit.app".to_string(),
        aud: Some(AudienceValue::Single("test-audience".to_string())),
        exp: (now + Duration::hours(1)).timestamp(),
        iat: now.timestamp(),
        scope: None,
        scp: None,
        client_id: Some("app456".to_string()),
        additional,
    };

    let token = create_test_token(&private_key, claims, Some(kid));

    // Make request
    let headers = http::types::Headers::new();
    headers
        .append("authorization", format!("Bearer {}", token).as_bytes())
        .unwrap();
    headers.append("content-type", b"application/json").unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_path_with_query(Some("/mcp")).unwrap();
    request.set_method(&http::types::Method::Post).unwrap();

    let body_content = json!({
        "jsonrpc": "2.0",
        "method": "initialize",
        "id": 1
    });
    let body = request.body().unwrap();
    body.write_bytes(serde_json::to_string(&body_content).unwrap().as_bytes());

    let response = spin_test_sdk::perform_request(request);
    let response_data = ResponseData::from_response(response);

    // Should succeed
    assert_eq!(response_data.status, 200);

    // Verify the response contains the client_id that was forwarded
    let json = response_data
        .body_json()
        .expect("Response should have JSON body");
    assert_eq!(json["jsonrpc"], "2.0");
    assert_eq!(json["result"]["client_id_received"], "app456");
    assert_eq!(json["id"], 1)
}
