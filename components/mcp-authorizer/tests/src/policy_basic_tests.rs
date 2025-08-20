// Basic policy authorization tests

use crate::policy_test_helpers::*;
use crate::test_setup::setup_default_test_config;
use spin_test_sdk::{bindings::wasi::http, spin_test};

#[spin_test]
fn test_policy_allow_all() {
    setup_default_test_config();
    let (private_key, _public_key) = setup_test_jwt_validation();
    setup_allow_all_policy();

    // Create a token with minimal claims
    let token = create_policy_test_token_with_key(&private_key, "user123", vec![], vec![]);

    let headers = http::types::Headers::new();
    headers
        .append("authorization", format!("Bearer {}", token).as_bytes())
        .unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_method(&http::types::Method::Get).unwrap();
    request.set_path_with_query(Some("/mcp")).unwrap();

    let response = spin_test_sdk::perform_request(request);

    // Should allow access (200 OK when no gateway configured)
    assert_eq!(
        response.status(),
        200,
        "Allow-all policy should permit access"
    );
}

#[spin_test]
fn test_policy_deny_all() {
    setup_default_test_config();
    let (private_key, _public_key) = setup_test_jwt_validation();
    setup_deny_all_policy();

    // Create a token with admin role - shouldn't matter with deny-all
    let token = create_policy_test_token_with_key(&private_key, "admin", vec!["admin"], vec![]);

    let headers = http::types::Headers::new();
    headers
        .append("authorization", format!("Bearer {}", token).as_bytes())
        .unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_method(&http::types::Method::Get).unwrap();
    request.set_path_with_query(Some("/mcp")).unwrap();

    let response = spin_test_sdk::perform_request(request);

    // Should deny access
    assert_eq!(
        response.status(),
        401,
        "Deny-all policy should block access"
    );

    // Check error message
    let body = response.body().unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("Access denied by authorization policy"),
        "Should indicate policy denial"
    );
}

#[spin_test]
fn test_policy_subject_check() {
    setup_default_test_config();
    let (private_key, _public_key) = setup_test_jwt_validation();
    setup_subject_check_policy(vec!["alice", "bob"]);

    // Test allowed subject
    let alice_token = create_policy_test_token_with_key(&private_key, "alice", vec![], vec![]);
    let headers = http::types::Headers::new();
    headers
        .append(
            "authorization",
            format!("Bearer {}", alice_token).as_bytes(),
        )
        .unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_method(&http::types::Method::Get).unwrap();
    request.set_path_with_query(Some("/mcp")).unwrap();

    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 200, "Alice should be allowed");

    // Test denied subject
    let charlie_token = create_policy_test_token_with_key(&private_key, "charlie", vec![], vec![]);
    let headers = http::types::Headers::new();
    headers
        .append(
            "authorization",
            format!("Bearer {}", charlie_token).as_bytes(),
        )
        .unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_method(&http::types::Method::Get).unwrap();
    request.set_path_with_query(Some("/mcp")).unwrap();

    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 401, "Charlie should be denied");
}

#[spin_test]
fn test_policy_without_configuration() {
    setup_default_test_config();
    // Set up JWT validation but no policy - authorization should be skipped
    let (private_key, _public_key) = setup_test_jwt_validation();
    clear_policy_config();

    let token = create_policy_test_token_with_key(&private_key, "anyone", vec![], vec![]);

    let headers = http::types::Headers::new();
    headers
        .append("authorization", format!("Bearer {}", token).as_bytes())
        .unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_method(&http::types::Method::Get).unwrap();
    request.set_path_with_query(Some("/mcp")).unwrap();

    let response = spin_test_sdk::perform_request(request);

    // Should allow access when no policy is configured (auth-only mode)
    assert_eq!(
        response.status(),
        200,
        "Should allow access when policy authorization is not configured"
    );
}

#[spin_test]
fn test_policy_evaluation_error() {
    setup_default_test_config();
    let (private_key, _public_key) = setup_test_jwt_validation();

    // Set an invalid policy with actual syntax errors
    let bad_policy = r#"
package mcp.authorization
import rego.v1

default allow := false

allow if {
    input.token.sub == "test" and nonsense syntax here
}
"#;
    spin_test_sdk::bindings::fermyon::spin_test_virt::variables::set("mcp_policy", bad_policy);

    let token = create_policy_test_token_with_key(&private_key, "test", vec![], vec![]);

    let headers = http::types::Headers::new();
    headers
        .append("authorization", format!("Bearer {}", token).as_bytes())
        .unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_method(&http::types::Method::Get).unwrap();
    request.set_path_with_query(Some("/mcp")).unwrap();

    let response = spin_test_sdk::perform_request(request);

    // Should fail with 500 due to policy error
    assert_eq!(
        response.status(),
        500,
        "Invalid policy should cause server error"
    );

    let body = response.body().unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("server_error"),
        "Should indicate server error"
    );
}

#[spin_test]
fn test_policy_with_undefined_result() {
    setup_default_test_config();

    let (private_key, _public_key) = setup_test_jwt_validation(); // Ensure JWT validation is configured first

    // Policy that doesn't define allow rule properly
    let policy = r#"
package mcp.authorization
import rego.v1

# No default, no allow rule - will be undefined
"#;
    spin_test_sdk::bindings::fermyon::spin_test_virt::variables::set("mcp_policy", policy);

    let token = create_policy_test_token_with_key(&private_key, "user", vec![], vec![]);

    let headers = http::types::Headers::new();
    headers
        .append("authorization", format!("Bearer {}", token).as_bytes())
        .unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_method(&http::types::Method::Get).unwrap();
    request.set_path_with_query(Some("/mcp")).unwrap();

    let response = spin_test_sdk::perform_request(request);

    // An empty policy that doesn't define the authorization package properly will cause a 500 error
    // because Regorous can't evaluate "data.mcp.authorization.allow" when the package doesn't exist
    assert_eq!(
        response.status(),
        500,
        "Empty/invalid policy should cause server error"
    );
}
