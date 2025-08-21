use spin_test_sdk::{bindings::fermyon::spin_test_virt::variables, spin_test};

#[spin_test]
fn test_audience_required_with_authkit_issuer() {
    // Set AuthKit issuer (should auto-derive JWKS)
    variables::set("mcp_jwt_issuer", "https://tenant.authkit.app");
    // No audience - should fail
    variables::set("mcp_jwt_audience", "");

    // Any request should fail during config loading
    let request = spin_test_sdk::bindings::wasi::http::types::OutgoingRequest::new(
        spin_test_sdk::bindings::wasi::http::types::Headers::new(),
    );
    request.set_path_with_query(Some("/test")).unwrap();

    let response = spin_test_sdk::perform_request(request);

    // Should get 500 because audience is required
    println!("Response status: {}", response.status());
    assert_eq!(response.status(), 500, "Should fail without audience");
}

#[spin_test]
fn test_audience_required_with_generic_issuer() {
    // Set generic issuer (no auto-derive JWKS)
    variables::set("mcp_jwt_issuer", "https://example.com");
    variables::set("mcp_jwt_jwks_uri", "https://example.com/jwks");
    // No audience - should fail
    variables::set("mcp_jwt_audience", "");

    // Any request should fail during config loading
    let request = spin_test_sdk::bindings::wasi::http::types::OutgoingRequest::new(
        spin_test_sdk::bindings::wasi::http::types::Headers::new(),
    );
    request.set_path_with_query(Some("/test")).unwrap();

    let response = spin_test_sdk::perform_request(request);

    // Should get 500 because audience is required
    println!("Response status: {}", response.status());
    assert_eq!(response.status(), 500, "Should fail without audience");
}
