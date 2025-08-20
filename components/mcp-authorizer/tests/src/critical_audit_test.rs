// Critical audit test - verify authorization is REAL, not a facade

use crate::policy_test_helpers::*;
use crate::test_setup::setup_default_test_config;
use spin_test_sdk::{bindings::wasi::http, spin_test};

#[spin_test]
fn test_authorization_is_real_not_facade() {
    setup_default_test_config();

    // Set up a STRICT policy that ONLY allows a specific subject
    let (private_key, _public_key) = setup_test_jwt_validation();

    let strict_policy = r#"
package mcp.authorization
import rego.v1

default allow := false

# ONLY allow if subject is exactly "allowed-user"
allow if {
    input.token.sub == "allowed-user"
}
"#;
    spin_test_sdk::bindings::fermyon::spin_test_virt::variables::set("mcp_policy", strict_policy);

    // Test 1: Create a token with WRONG subject - should be DENIED
    let wrong_token = create_policy_test_token_with_key(&private_key, "wrong-user", vec![], vec![]);

    let headers = http::types::Headers::new();
    headers
        .append(
            "authorization",
            format!("Bearer {}", wrong_token).as_bytes(),
        )
        .unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_method(&http::types::Method::Get).unwrap();
    request.set_path_with_query(Some("/mcp")).unwrap();

    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 401, "Wrong subject MUST be denied");

    // Test 2: Create a token with CORRECT subject - should be ALLOWED
    let correct_token =
        create_policy_test_token_with_key(&private_key, "allowed-user", vec![], vec![]);

    let headers = http::types::Headers::new();
    headers
        .append(
            "authorization",
            format!("Bearer {}", correct_token).as_bytes(),
        )
        .unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_method(&http::types::Method::Get).unwrap();
    request.set_path_with_query(Some("/mcp")).unwrap();

    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 200, "Correct subject MUST be allowed");
}

#[spin_test]
fn test_jwt_signature_is_validated() {
    setup_default_test_config();

    // Set up JWT validation with one keypair
    let (private_key_1, public_key_1) = generate_test_keypair();
    setup_test_jwt_validation_with_keypair(&public_key_1);

    // But create token with DIFFERENT private key
    let (private_key_2, _public_key_2) = generate_test_keypair();
    let token_wrong_key = create_policy_test_token_with_key(&private_key_2, "user", vec![], vec![]);

    // This token should FAIL validation due to signature mismatch
    let headers = http::types::Headers::new();
    headers
        .append(
            "authorization",
            format!("Bearer {}", token_wrong_key).as_bytes(),
        )
        .unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_method(&http::types::Method::Get).unwrap();
    request.set_path_with_query(Some("/mcp")).unwrap();

    let response = spin_test_sdk::perform_request(request);
    assert_eq!(
        response.status(),
        401,
        "Token with wrong signature MUST be rejected"
    );

    // Now create token with CORRECT private key
    let token_correct_key =
        create_policy_test_token_with_key(&private_key_1, "user", vec![], vec![]);

    let headers = http::types::Headers::new();
    headers
        .append(
            "authorization",
            format!("Bearer {}", token_correct_key).as_bytes(),
        )
        .unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_method(&http::types::Method::Get).unwrap();
    request.set_path_with_query(Some("/mcp")).unwrap();

    let response = spin_test_sdk::perform_request(request);
    assert_eq!(
        response.status(),
        200,
        "Token with correct signature MUST be accepted"
    );
}

#[spin_test]
fn test_policy_actually_evaluates_input() {
    setup_default_test_config();
    let (private_key, _public_key) = setup_test_jwt_validation();

    // Policy that checks both token claims AND request path
    let complex_policy = r#"
package mcp.authorization
import rego.v1

default allow := false

# Only allow if:
# 1. User has admin role
# 2. Request is to /mcp/x/admin-only
allow if {
    "admin" in input.token.claims.roles
    input.request.path == "/mcp/x/admin-only"
}
"#;
    spin_test_sdk::bindings::fermyon::spin_test_virt::variables::set("mcp_policy", complex_policy);

    // Test 1: Admin role but WRONG path - should be DENIED
    let admin_token =
        create_policy_test_token_with_key(&private_key, "admin", vec!["admin"], vec![]);

    let headers = http::types::Headers::new();
    headers
        .append(
            "authorization",
            format!("Bearer {}", admin_token).as_bytes(),
        )
        .unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_method(&http::types::Method::Get).unwrap();
    request
        .set_path_with_query(Some("/mcp/x/wrong-path"))
        .unwrap();

    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 401, "Admin on wrong path MUST be denied");

    // Test 2: Admin role AND correct path - should be ALLOWED
    let headers = http::types::Headers::new();
    headers
        .append(
            "authorization",
            format!("Bearer {}", admin_token).as_bytes(),
        )
        .unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_method(&http::types::Method::Get).unwrap();
    request
        .set_path_with_query(Some("/mcp/x/admin-only"))
        .unwrap();

    let response = spin_test_sdk::perform_request(request);
    assert_eq!(
        response.status(),
        200,
        "Admin on correct path MUST be allowed"
    );

    // Test 3: Non-admin on correct path - should be DENIED
    let user_token = create_policy_test_token_with_key(&private_key, "user", vec!["user"], vec![]);

    let headers = http::types::Headers::new();
    headers
        .append("authorization", format!("Bearer {}", user_token).as_bytes())
        .unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_method(&http::types::Method::Get).unwrap();
    request
        .set_path_with_query(Some("/mcp/x/admin-only"))
        .unwrap();

    let response = spin_test_sdk::perform_request(request);
    assert_eq!(
        response.status(),
        401,
        "Non-admin on admin path MUST be denied"
    );
}

#[spin_test]
fn test_no_token_is_rejected() {
    setup_default_test_config();
    setup_test_jwt_validation(); // Set up JWT validation

    // Request with NO authorization header
    let headers = http::types::Headers::new();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_method(&http::types::Method::Get).unwrap();
    request.set_path_with_query(Some("/mcp")).unwrap();

    let response = spin_test_sdk::perform_request(request);
    assert_eq!(
        response.status(),
        401,
        "Request without token MUST be rejected"
    );
}

#[spin_test]
fn test_malformed_token_is_rejected() {
    setup_default_test_config();
    setup_test_jwt_validation();

    // Send completely invalid JWT
    let headers = http::types::Headers::new();
    headers
        .append("authorization", b"Bearer not-a-real-jwt-token")
        .unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_method(&http::types::Method::Get).unwrap();
    request.set_path_with_query(Some("/mcp")).unwrap();

    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 401, "Malformed token MUST be rejected");
}
