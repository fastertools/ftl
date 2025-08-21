//! Test config loading to see what values are actually loaded

use spin_test_sdk::{
    bindings::{fermyon::spin_test_virt::variables, wasi::http::types},
    spin_test,
};

use crate::test_token_utils::{TestKeyPair, TestTokenBuilder};

#[spin_test]
fn test_config_loading() {
    let key_pair = TestKeyPair::generate();

    // Set variables explicitly for a standards-compliant configuration
    println!("Setting variables:");
    println!("  mcp_gateway_url = none");
    println!("  mcp_jwt_issuer = https://test.authkit.app");
    println!("  mcp_jwt_public_key = <key>");
    println!("  mcp_jwt_audience = test-api");

    variables::set("mcp_gateway_url", "none");
    variables::set("mcp_jwt_issuer", "https://test.authkit.app");
    variables::set("mcp_jwt_public_key", &key_pair.public_key_pem());
    variables::set("mcp_jwt_audience", "test-api");

    // Create a valid token with correct issuer but wrong audience
    let token = key_pair.create_token(
        TestTokenBuilder::new()
            .issuer("https://test.authkit.app")
            .subject("client_123")
            .client_id("client_123")
            .audience("wrong-audience"), // Wrong audience
    );

    let headers = types::Headers::new();
    headers
        .append("authorization", format!("Bearer {}", token).as_bytes())
        .unwrap();
    let request = types::OutgoingRequest::new(headers);
    request.set_path_with_query(Some("/test")).unwrap();

    let response = spin_test_sdk::perform_request(request);
    let status = response.status();

    println!("Response status: {}", status);

    if status == 401 {
        // Good - token was rejected as expected due to wrong audience
        if let Ok(body) = response.body() {
            let body_str = String::from_utf8_lossy(&body);
            println!("Error message: {}", body_str);
            assert!(
                body_str.contains("audience") || body_str.contains("invalid_token"),
                "Error should mention audience or invalid token"
            );
        }
    } else if status == 200 {
        println!("ERROR: Token was accepted when it should have been rejected!");
        println!("This means audience validation is NOT working");
        panic!("Token with wrong audience was accepted");
    } else {
        println!("Unexpected status: {}", status);
        if let Ok(body) = response.body() {
            println!("Body: {}", String::from_utf8_lossy(&body));
        }
        panic!("Unexpected status code");
    }
}
