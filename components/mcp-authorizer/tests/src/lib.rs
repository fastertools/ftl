use spin_test_sdk::{
    bindings::{fermyon::spin_test_virt::variables, wasi::http},
    spin_test,
};

mod authkit_integration_tests;
mod critical_audit_test;
mod critical_verification_test;
mod gateway_forwarding_tests;
mod jwks_caching_tests;
mod jwt_test_utils_tests;
mod jwt_tests;
mod jwt_verification_tests;
mod kid_validation_tests;
mod multiple_audiences_tests;
mod oauth_discovery_tests;
mod optional_issuer_tests;
mod policy_basic_tests;
mod policy_complex_tests;
mod policy_component_tests;
mod policy_data_tests;
mod policy_mcp_tool_tests;
mod policy_test_helpers;
mod provider_config_tests;
mod scope_validation_tests;
mod simple_test;
mod test_audience_required;
mod test_config_loading;
mod test_helpers;
mod test_setup;
mod test_token_utils;

// Response data helper to extract all needed information
pub struct ResponseData {
    pub status: u16,
    pub headers: Vec<(String, Vec<u8>)>,
    pub body: Vec<u8>,
}

impl ResponseData {
    pub fn from_response(response: http::types::IncomingResponse) -> Self {
        let status = response.status();

        // Extract headers before consuming response
        let headers = response
            .headers()
            .entries()
            .into_iter()
            .map(|(name, value)| (name.to_string(), value.to_vec()))
            .collect();

        // Now consume response to get body
        let body = response.body().unwrap_or_else(|_| Vec::new());

        Self {
            status,
            headers,
            body,
        }
    }

    pub fn find_header(&self, name: &str) -> Option<&Vec<u8>> {
        self.headers
            .iter()
            .find(|(h_name, _)| h_name.eq_ignore_ascii_case(name))
            .map(|(_, value)| value)
    }

    pub fn body_json(&self) -> Option<serde_json::Value> {
        if self.body.is_empty() {
            None
        } else {
            serde_json::from_slice(&self.body).ok()
        }
    }
}

// Existing tests from the original file

#[spin_test]
fn unauthenticated_request() {
    // Setup default configuration
    crate::test_setup::setup_default_test_config();

    // Make request without auth header
    let request = http::types::OutgoingRequest::new(http::types::Headers::new());
    request.set_path_with_query(Some("/mcp")).unwrap();
    let response = spin_test_sdk::perform_request(request);

    // Should return 401 Unauthorized
    assert_eq!(response.status(), 401);

    // Check for WWW-Authenticate header
    let headers = response.headers();
    let www_auth_exists = test_helpers::find_header(&headers, "www-authenticate").is_some();
    assert!(www_auth_exists);
}

#[spin_test]
fn options_cors_request() {
    // Make OPTIONS request (CORS preflight)
    let request = http::types::OutgoingRequest::new(http::types::Headers::new());
    request.set_method(&http::types::Method::Options).unwrap();
    request.set_path_with_query(Some("/mcp")).unwrap();
    let response = spin_test_sdk::perform_request(request);

    // Should return 204 No Content
    assert_eq!(response.status(), 204);

    // Check for CORS headers
    let headers = response.headers();
    let has_cors = test_helpers::find_header(&headers, "access-control-allow-origin").is_some();
    assert!(has_cors);
}

#[spin_test]
fn metadata_endpoint() {
    // Setup default configuration
    crate::test_setup::setup_default_test_config();

    // With the test configuration, we have a provider configured
    // Test /.well-known/oauth-protected-resource endpoint
    let headers = http::types::Headers::new();
    headers.append("host", b"example.com").unwrap();

    let request = http::types::OutgoingRequest::new(headers);
    request
        .set_path_with_query(Some("/.well-known/oauth-protected-resource"))
        .unwrap();
    let response = spin_test_sdk::perform_request(request);

    // Should return 200 when provider is configured
    assert_eq!(response.status(), 200);

    // Check for proper content type
    let headers = response.headers();
    let has_json_content = test_helpers::find_header_str(&headers, "content-type")
        .map(|ct| ct.contains("application/json"))
        .unwrap_or(false);
    assert!(has_json_content);
}

#[spin_test]
fn authorization_server_metadata() {
    // Setup default configuration
    crate::test_setup::setup_default_test_config();

    // With the test configuration, we have a provider configured
    // Test /.well-known/oauth-authorization-server endpoint
    let request = http::types::OutgoingRequest::new(http::types::Headers::new());
    request
        .set_path_with_query(Some("/.well-known/oauth-authorization-server"))
        .unwrap();
    let response = spin_test_sdk::perform_request(request);

    // Should return 200 when provider is configured
    assert_eq!(response.status(), 200);

    // Check response contains OAuth metadata
    let headers = response.headers();
    let has_json_content = test_helpers::find_header_str(&headers, "content-type")
        .map(|ct| ct.contains("application/json"))
        .unwrap_or(false);
    assert!(has_json_content);
}

#[spin_test]
fn provider_config_works() {
    // Setup default configuration
    crate::test_setup::setup_default_test_config();

    // Test that the provider configuration works correctly
    // Make request to metadata endpoint
    let request = http::types::OutgoingRequest::new(http::types::Headers::new());
    request
        .set_path_with_query(Some("/.well-known/oauth-authorization-server"))
        .unwrap();
    let response = spin_test_sdk::perform_request(request);

    // Should return 200 with configured provider
    assert_eq!(response.status(), 200);

    // Verify CORS headers are present
    let headers = response.headers();
    let has_cors = test_helpers::find_header(&headers, "access-control-allow-origin").is_some();
    assert!(has_cors);
}

#[spin_test]
fn trace_id_header() {
    // Setup default configuration
    crate::test_setup::setup_default_test_config();

    // Test that trace ID is propagated through requests
    let headers = http::types::Headers::new();
    headers.append("x-trace-id", b"test-trace-123").unwrap();

    let request = http::types::OutgoingRequest::new(headers);
    request.set_path_with_query(Some("/mcp")).unwrap();
    let response = spin_test_sdk::perform_request(request);

    // Should return 401
    assert_eq!(response.status(), 401);

    // Check for trace ID in response
    let response_headers = response.headers();
    let has_trace = test_helpers::find_header(&response_headers, "x-trace-id").is_some();
    assert!(has_trace);
}

#[spin_test]
fn auth_enabled_requires_token() {
    // Setup default configuration
    crate::test_setup::setup_default_test_config();

    // With auth enabled in test config, requests without auth should fail
    // Make request without auth header
    let request = http::types::OutgoingRequest::new(http::types::Headers::new());
    request.set_path_with_query(Some("/mcp")).unwrap();
    let response = spin_test_sdk::perform_request(request);

    // Should return 401 because auth is required
    assert_eq!(response.status(), 401);

    // Check for WWW-Authenticate header
    let headers = response.headers();
    let www_auth_exists = test_helpers::find_header(&headers, "www-authenticate").is_some();
    assert!(www_auth_exists);
}

#[spin_test]
fn metadata_endpoint_with_provider() {
    // Setup default configuration
    crate::test_setup::setup_default_test_config();

    // Test /.well-known/oauth-protected-resource endpoint
    let headers = http::types::Headers::new();
    headers.append("host", b"example.com").unwrap();

    let request = http::types::OutgoingRequest::new(headers);
    request
        .set_path_with_query(Some("/.well-known/oauth-protected-resource"))
        .unwrap();
    let response = spin_test_sdk::perform_request(request);

    // Should return 200 when provider is configured
    assert_eq!(response.status(), 200);

    // Check for content type
    let headers = response.headers();
    let has_content_type = test_helpers::find_header(&headers, "content-type").is_some();
    assert!(has_content_type);
}

#[spin_test]
fn https_enforcement_rejects_http() {
    // Test that HTTP URLs are rejected for security
    // Override the test config to use HTTP
    variables::set("mcp_jwt_issuer", "http://example.authkit.app");

    // Try to make a request - the component should fail to initialize
    let request = http::types::OutgoingRequest::new(http::types::Headers::new());
    request.set_path_with_query(Some("/mcp")).unwrap();
    let response = spin_test_sdk::perform_request(request);

    // Should get an internal error because the component failed to initialize
    assert_eq!(response.status(), 500);
}

#[spin_test]
fn https_enforcement_accepts_bare_domain() {
    // Test that bare domains work (https:// is added automatically)
    variables::set("mcp_jwt_issuer", "example.authkit.app");
    variables::set("mcp_jwt_audience", "test-api");
    // Don't set jwks_uri - let auto-derivation work for .authkit.app domain

    // Make a metadata request to verify it initialized correctly
    let request = http::types::OutgoingRequest::new(http::types::Headers::new());
    request
        .set_path_with_query(Some("/.well-known/oauth-authorization-server"))
        .unwrap();
    let response = spin_test_sdk::perform_request(request);

    // Should return 200 because component initialized successfully
    assert_eq!(response.status(), 200);
}

#[spin_test]
fn https_enforcement_accepts_https_prefix() {
    // Test that explicit https:// URLs work
    variables::set("mcp_jwt_issuer", "https://example.authkit.app");
    variables::set("mcp_jwt_audience", "test-api");
    // Don't set jwks_uri - let auto-derivation work for .authkit.app domain

    // Make a metadata request to verify it initialized correctly
    let request = http::types::OutgoingRequest::new(http::types::Headers::new());
    request
        .set_path_with_query(Some("/.well-known/oauth-authorization-server"))
        .unwrap();
    let response = spin_test_sdk::perform_request(request);

    // Should return 200 because component initialized successfully
    assert_eq!(response.status(), 200);
}

#[spin_test]
fn https_enforcement_oauth_urls() {
    // Test that OAuth URLs also enforce HTTPS
    variables::set("mcp_jwt_issuer", "https://example.com");
    variables::set("mcp_jwt_jwks_uri", "http://example.com/jwks"); // HTTP should fail
    variables::set("mcp_oauth_authorize_endpoint", "example.com/auth");
    variables::set("mcp_oauth_token_endpoint", "example.com/token");
    variables::set("mcp_oauth_userinfo_endpoint", "");
    variables::set("mcp_jwt_audience", "");

    // Try to make a request - the component should fail to initialize
    let request = http::types::OutgoingRequest::new(http::types::Headers::new());
    request.set_path_with_query(Some("/mcp")).unwrap();
    let response = spin_test_sdk::perform_request(request);

    // Should get an internal error because the component failed to initialize
    assert_eq!(response.status(), 500);
}
