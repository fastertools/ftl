use spin_test_sdk::{
    bindings::{
        fermyon::spin_wasi_virt::http_handler,
        wasi::http,
    },
    spin_test,
};
use crate::{ResponseData, jwt_verification_tests::{Claims, AudienceValue, configure_test_provider, create_test_token}};

/// Mock gateway that returns a successful response
fn mock_gateway_success_with_headers() {
    let headers = http::types::Headers::new();
    headers.append("content-type", b"application/json").unwrap();
    headers.append("x-gateway-response", b"success").unwrap();
    
    let response = http::types::OutgoingResponse::new(headers);
    response.set_status_code(200).unwrap();
    
    let body = response.body().unwrap();
    body.write_bytes(b"{\"jsonrpc\":\"2.0\",\"result\":{\"tools\":[\"test-tool\"]},\"id\":1}");
    
    http_handler::set_response(
        "https://test-gateway.spin.internal/mcp-internal",
        http_handler::ResponseHandler::Response(response),
    );
}

// Test: Verify gateway response is passed through correctly
#[spin_test]
fn test_gateway_response_passthrough() {
    // Configure provider
    configure_test_provider();
    
    // Setup keys and mock JWKS
    let (private_key, public_key) = crate::jwt_verification_tests::generate_test_key_pair();
    let kid = "test-key";
    let jwks = crate::jwt_verification_tests::create_jwks_response(&public_key, kid);
    crate::jwt_verification_tests::mock_jwks_endpoint("https://test.authkit.app/.well-known/jwks.json", jwks);
    
    // Mock the gateway with specific response
    mock_gateway_success_with_headers();
    
    // Create a valid token with specific claims
    let now = chrono::Utc::now();
    let claims = Claims {
        sub: "user123".to_string(),
        iss: "https://test.authkit.app".to_string(),
        aud: Some(AudienceValue::Single("test-audience".to_string())),
        exp: (now + chrono::Duration::hours(1)).timestamp(),
        iat: now.timestamp(),
        scope: Some("read write admin".to_string()),
        scp: None,
        client_id: Some("app456".to_string()),
        additional: serde_json::Map::new(),
    };
    
    let token = create_test_token(&private_key, claims, Some(kid));
    
    // Make authenticated request
    let headers = http::types::Headers::new();
    headers.append("authorization", format!("Bearer {}", token).as_bytes()).unwrap();
    headers.append("content-type", b"application/json").unwrap();
    headers.append("x-custom-header", b"custom-value").unwrap();
    
    let request = http::types::OutgoingRequest::new(headers);
    request.set_path_with_query(Some("/mcp")).unwrap();
    request.set_method(&http::types::Method::Post).unwrap();
    
    let body = request.body().unwrap();
    body.write_bytes(b"{\"jsonrpc\":\"2.0\",\"method\":\"tools/list\",\"id\":1}");
    
    let response = spin_test_sdk::perform_request(request);
    let response_data = ResponseData::from_response(response);
    
    // Request should succeed
    assert_eq!(response_data.status, 200);
    
    // Verify gateway response headers are passed through (except CORS which we override)
    let gateway_header = response_data.find_header("x-gateway-response")
        .map(|v| String::from_utf8_lossy(v));
    assert_eq!(gateway_header.as_deref(), Some("success"), 
        "Gateway headers should be passed through");
    
    // Verify CORS headers are added
    assert!(response_data.find_header("access-control-allow-origin").is_some(),
        "CORS headers should be added");
    
    // Verify response body is passed through from gateway
    let response_json = response_data.body_json()
        .expect("Response should have JSON body");
    assert_eq!(response_json["jsonrpc"], "2.0");
    assert_eq!(response_json["result"]["tools"][0], "test-tool");
    assert_eq!(response_json["id"], 1);
}

// Test: Verify gateway errors are passed through
#[spin_test]
fn test_gateway_error_passthrough() {
    configure_test_provider();
    
    // Setup valid auth
    let (private_key, public_key) = crate::jwt_verification_tests::generate_test_key_pair();
    let kid = "test-key";
    let jwks = crate::jwt_verification_tests::create_jwks_response(&public_key, kid);
    crate::jwt_verification_tests::mock_jwks_endpoint("https://test.authkit.app/.well-known/jwks.json", jwks);
    
    // Mock gateway to return an error
    let headers = http::types::Headers::new();
    headers.append("content-type", b"application/json").unwrap();
    headers.append("x-gateway-error", b"internal-failure").unwrap();
    
    let gateway_response = http::types::OutgoingResponse::new(headers);
    gateway_response.set_status_code(500).unwrap();
    
    let body = gateway_response.body().unwrap();
    body.write_bytes(b"{\"error\":\"internal_server_error\",\"message\":\"Gateway failed\"}");
    
    http_handler::set_response(
        "https://test-gateway.spin.internal/mcp-internal",
        http_handler::ResponseHandler::Response(gateway_response),
    );
    
    // Create valid token
    let token = create_test_token(&private_key, crate::jwt_verification_tests::Claims {
        sub: "user123".to_string(),
        iss: "https://test.authkit.app".to_string(),
        aud: Some(AudienceValue::Single("test-audience".to_string())),
        exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp(),
        iat: chrono::Utc::now().timestamp(),
        scope: None,
        scp: None,
        client_id: None,
        additional: serde_json::Map::new(),
    }, Some(kid));
    
    // Make authenticated request
    let headers = http::types::Headers::new();
    headers.append("authorization", format!("Bearer {}", token).as_bytes()).unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_path_with_query(Some("/mcp")).unwrap();
    
    let response = spin_test_sdk::perform_request(request);
    let response_data = ResponseData::from_response(response);
    
    // Gateway error should be passed through
    assert_eq!(response_data.status, 500);
    
    // Gateway headers should be preserved (except CORS which we override)
    let gateway_error_header = response_data.find_header("x-gateway-error")
        .map(|v| String::from_utf8_lossy(v));
    assert_eq!(gateway_error_header.as_deref(), Some("internal-failure"));
    
    // Gateway error body should be passed through
    let json = response_data.body_json()
        .expect("Gateway error should have JSON body");
    assert_eq!(json["error"], "internal_server_error");
    assert_eq!(json["message"], "Gateway failed");
}

// Test: Verify successful auth with all token variations
#[spin_test]
fn test_various_token_scenarios() {
    configure_test_provider();
    
    let (private_key, public_key) = crate::jwt_verification_tests::generate_test_key_pair();
    let kid = "test-key";
    let jwks = crate::jwt_verification_tests::create_jwks_response(&public_key, kid);
    crate::jwt_verification_tests::mock_jwks_endpoint("https://test.authkit.app/.well-known/jwks.json", jwks);
    
    // Mock successful gateway
    mock_gateway_success_with_headers();
    
    // Test 1: Token WITHOUT explicit client_id (should use sub)
    let token_no_client_id = create_test_token(&private_key, crate::jwt_verification_tests::Claims {
        sub: "user789".to_string(),
        iss: "https://test.authkit.app".to_string(),
        aud: Some(AudienceValue::Single("test-audience".to_string())),
        exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp(),
        iat: chrono::Utc::now().timestamp(),
        scope: None,
        scp: None,
        client_id: None, // No explicit client_id
        additional: serde_json::Map::new(),
    }, Some(kid));
    
    let headers = http::types::Headers::new();
    headers.append("authorization", format!("Bearer {}", token_no_client_id).as_bytes()).unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_path_with_query(Some("/mcp")).unwrap();
    
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 200, "Token without client_id should succeed using sub as fallback");
    
    // Test 2: Token with explicit client_id different from sub
    let token_with_client_id = create_test_token(&private_key, crate::jwt_verification_tests::Claims {
        sub: "user123".to_string(),
        iss: "https://test.authkit.app".to_string(),
        aud: Some(AudienceValue::Single("test-audience".to_string())),
        exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp(),
        iat: chrono::Utc::now().timestamp(),
        scope: None,
        scp: None,
        client_id: Some("app999".to_string()), // Different from sub
        additional: serde_json::Map::new(),
    }, Some(kid));
    
    let headers2 = http::types::Headers::new();
    headers2.append("authorization", format!("Bearer {}", token_with_client_id).as_bytes()).unwrap();
    let request2 = http::types::OutgoingRequest::new(headers2);
    request2.set_path_with_query(Some("/mcp")).unwrap();
    
    // Need to re-mock gateway for second request
    mock_gateway_success_with_headers();
    let response2 = spin_test_sdk::perform_request(request2);
    assert_eq!(response2.status(), 200, "Token with explicit client_id should succeed");
}

// Test: Verify auth failures return proper error format
#[spin_test]
fn test_auth_failure_error_format() {
    configure_test_provider();
    
    // Don't mock JWKS - token validation will fail
    
    let headers = http::types::Headers::new();
    headers.append("authorization", b"Bearer invalid.jwt.token").unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_path_with_query(Some("/mcp")).unwrap();
    
    let response = spin_test_sdk::perform_request(request);
    let response_data = ResponseData::from_response(response);
    
    // Should return 401
    assert_eq!(response_data.status, 401);
    
    // Verify error response format
    let json = response_data.body_json()
        .expect("Auth error must return JSON body");
    assert!(json["error"].is_string(), "Error response must have 'error' field");
    assert!(json["error_description"].is_string(), "Error response must have 'error_description' field");
    
    // Verify WWW-Authenticate header
    let www_auth = response_data.find_header("www-authenticate")
        .expect("401 response must have WWW-Authenticate header");
    let auth_str = String::from_utf8_lossy(www_auth);
    assert!(auth_str.starts_with("Bearer"), "WWW-Authenticate must use Bearer scheme");
    assert!(auth_str.contains("error="), "WWW-Authenticate must contain error parameter");
}