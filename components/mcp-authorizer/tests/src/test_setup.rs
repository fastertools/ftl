use spin_test_sdk::bindings::fermyon::spin_test_virt::variables;

/// Sets up the default test configuration
/// This ensures tests have a consistent baseline configuration
pub fn setup_default_test_config() {
    // Core settings - set gateway URL to "none" to disable forwarding in tests
    variables::set("mcp_gateway_url", "none");
    variables::set("mcp_trace_header", "x-trace-id");
    
    // JWT provider settings
    variables::set("mcp_jwt_issuer", "https://test.authkit.app");
    variables::set("mcp_jwt_audience", "test-audience");
    // JWKS URI will be auto-derived for AuthKit domains
}