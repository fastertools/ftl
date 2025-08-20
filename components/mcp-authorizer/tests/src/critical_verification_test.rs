// Ultimate verification test - prove authorization can be turned ON and OFF

use crate::policy_test_helpers::*;
use crate::test_setup::setup_default_test_config;
use rsa::pkcs1::EncodeRsaPrivateKey;
use spin_test_sdk::{bindings::wasi::http, spin_test};

#[spin_test]
fn test_authorization_can_be_disabled() {
    setup_default_test_config();
    let (private_key, _public_key) = setup_test_jwt_validation();

    // Step 1: Set a DENY ALL policy
    let deny_all_policy = r#"
package mcp.authorization
import rego.v1
default allow := false
"#;
    spin_test_sdk::bindings::fermyon::spin_test_virt::variables::set("mcp_policy", deny_all_policy);

    let token = create_policy_test_token_with_key(&private_key, "user", vec![], vec![]);

    // Should be DENIED with policy
    let headers = http::types::Headers::new();
    headers
        .append("authorization", format!("Bearer {}", token).as_bytes())
        .unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_method(&http::types::Method::Get).unwrap();
    request.set_path_with_query(Some("/mcp")).unwrap();

    let response = spin_test_sdk::perform_request(request);
    assert_eq!(
        response.status(),
        401,
        "With deny-all policy, should be denied"
    );

    // Step 2: REMOVE the policy (empty string = no policy)
    spin_test_sdk::bindings::fermyon::spin_test_virt::variables::set("mcp_policy", "");

    // Same token should now be ALLOWED (only JWT validation, no authorization)
    let headers = http::types::Headers::new();
    headers
        .append("authorization", format!("Bearer {}", token).as_bytes())
        .unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_method(&http::types::Method::Get).unwrap();
    request.set_path_with_query(Some("/mcp")).unwrap();

    let response = spin_test_sdk::perform_request(request);
    assert_eq!(
        response.status(),
        200,
        "Without policy, valid JWT should be allowed"
    );
}

#[spin_test]
fn test_policy_evaluation_changes_with_input() {
    setup_default_test_config();
    let (private_key, _public_key) = setup_test_jwt_validation();

    // Policy that allows based on HTTP method
    let method_policy = r#"
package mcp.authorization
import rego.v1

default allow := false

# Only allow GET requests
allow if {
    input.request.method == "GET"
}
"#;
    spin_test_sdk::bindings::fermyon::spin_test_virt::variables::set("mcp_policy", method_policy);

    let token = create_policy_test_token_with_key(&private_key, "user", vec![], vec![]);

    // Test 1: GET should be allowed
    let headers = http::types::Headers::new();
    headers
        .append("authorization", format!("Bearer {}", token).as_bytes())
        .unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_method(&http::types::Method::Get).unwrap();
    request.set_path_with_query(Some("/mcp")).unwrap();

    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 200, "GET should be allowed");

    // Test 2: POST should be denied
    let headers = http::types::Headers::new();
    headers
        .append("authorization", format!("Bearer {}", token).as_bytes())
        .unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_method(&http::types::Method::Post).unwrap();
    request.set_path_with_query(Some("/mcp")).unwrap();

    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 401, "POST should be denied");

    // Test 3: DELETE should be denied
    let headers = http::types::Headers::new();
    headers
        .append("authorization", format!("Bearer {}", token).as_bytes())
        .unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_method(&http::types::Method::Delete).unwrap();
    request.set_path_with_query(Some("/mcp")).unwrap();

    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 401, "DELETE should be denied");
}

#[spin_test]
fn test_expired_token_fails_before_authorization() {
    setup_default_test_config();
    let (private_key, _public_key) = setup_test_jwt_validation();

    // Set an ALLOW ALL policy
    let allow_all_policy = r#"
package mcp.authorization
import rego.v1
default allow := true
"#;
    spin_test_sdk::bindings::fermyon::spin_test_virt::variables::set(
        "mcp_policy",
        allow_all_policy,
    );

    // Create an EXPIRED token
    use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
    use serde::{Deserialize, Serialize};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[derive(Debug, Serialize, Deserialize)]
    struct Claims {
        sub: String,
        iss: String,
        aud: String,
        exp: i64,
        iat: i64,
    }

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let claims = Claims {
        sub: "user".to_string(),
        iss: "https://test.example.com".to_string(),
        aud: "test-audience".to_string(),
        exp: now - 3600, // Expired 1 hour ago!
        iat: now - 7200,
    };

    let private_key_pem = private_key
        .to_pkcs1_pem(rsa::pkcs1::LineEnding::LF)
        .expect("failed to encode private key")
        .to_string();

    let header = Header::new(Algorithm::RS256);
    let encoding_key = EncodingKey::from_rsa_pem(private_key_pem.as_bytes())
        .expect("failed to create encoding key");

    let expired_token = encode(&header, &claims, &encoding_key).expect("failed to encode token");

    // Even with ALLOW ALL policy, expired token should fail
    let headers = http::types::Headers::new();
    headers
        .append(
            "authorization",
            format!("Bearer {}", expired_token).as_bytes(),
        )
        .unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_method(&http::types::Method::Get).unwrap();
    request.set_path_with_query(Some("/mcp")).unwrap();

    let response = spin_test_sdk::perform_request(request);
    assert_eq!(
        response.status(),
        401,
        "Expired token must fail even with allow-all policy"
    );

    // Verify the error is about expiration, not authorization
    let body = response.body().unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("expired") || body_str.contains("invalid_token"),
        "Should indicate token expiration"
    );
}
