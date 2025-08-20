// Helper functions for policy authorization tests

use spin_test_sdk::bindings::fermyon::spin_test_virt::variables;
use crate::test_token_utils::TestTokenBuilder;

/// Sets up a basic allow-all policy for testing
pub fn setup_allow_all_policy() {
    let policy = r#"
package mcp.authorization
import rego.v1

default allow := true
"#;
    variables::set("mcp_policy", policy);
}

/// Sets up a basic deny-all policy for testing
pub fn setup_deny_all_policy() {
    let policy = r#"
package mcp.authorization
import rego.v1

default allow := false
"#;
    variables::set("mcp_policy", policy);
}

/// Sets up a policy that checks for specific subject
pub fn setup_subject_check_policy(allowed_subjects: Vec<&str>) {
    let subjects_list = allowed_subjects.iter()
        .map(|s| format!("\"{}\"", s))
        .collect::<Vec<_>>()
        .join(", ");
    
    let policy = format!(r#"
package mcp.authorization
import rego.v1

default allow := false

allow if {{
    input.token.sub in [{}]
}}
"#, subjects_list);
    
    variables::set("mcp_policy", &policy);
}

/// Sets up a policy that checks for specific roles in claims
pub fn setup_role_based_policy(required_role: &str) {
    let policy = format!(r#"
package mcp.authorization
import rego.v1

default allow := false

allow if {{
    "{}" in input.token.claims.roles
}}
"#, required_role);
    
    variables::set("mcp_policy", &policy);
}

/// Sets up a policy that checks component access
pub fn setup_component_policy(allowed_components: Vec<&str>) {
    let components_list = allowed_components.iter()
        .map(|c| format!("\"{}\"", c))
        .collect::<Vec<_>>()
        .join(", ");
    
    let policy = format!(r#"
package mcp.authorization
import rego.v1

default allow := false

allow if {{
    input.request.component in [{}]
}}
"#, components_list);
    
    variables::set("mcp_policy", &policy);
}

/// Sets up a policy for MCP tool authorization
pub fn setup_tool_authorization_policy(allowed_tools: Vec<&str>, dangerous_tools: Vec<&str>) {
    let allowed_list = allowed_tools.iter()
        .map(|t| format!("\"{}\"", t))
        .collect::<Vec<_>>()
        .join(", ");
    
    let dangerous_list = dangerous_tools.iter()
        .map(|t| format!("\"{}\"", t))
        .collect::<Vec<_>>()
        .join(", ");
    
    let policy = format!(r#"
package mcp.authorization
import rego.v1

default allow := false

# Allow tool discovery
allow if {{
    input.mcp.method == "tools/list"
}}

# Allow safe tools
allow if {{
    input.mcp.method == "tools/call"
    input.mcp.tool in [{}]
}}

# Deny dangerous tools without admin role
deny if {{
    input.mcp.method == "tools/call"
    input.mcp.tool in [{}]
    not "admin" in input.token.claims.roles
}}

# Final allow if not denied
allow if {{
    input.mcp
    not deny
}}
"#, allowed_list, dangerous_list);
    
    variables::set("mcp_policy", &policy);
}

/// Sets up a policy with external data
pub fn setup_policy_with_data(policy: &str, data: &str) {
    variables::set("mcp_policy", policy);
    variables::set("mcp_policy_data", data);
}

/// Creates a test token with specific claims for policy testing
pub fn create_policy_test_token(
    subject: &str,
    roles: Vec<&str>,
    additional_claims: Vec<(&str, serde_json::Value)>,
) -> String {
    let mut builder = TestTokenBuilder::new();
    builder.with_subject(subject);
    builder.with_audience("test-audience");
    builder.with_issuer("https://test.authkit.app");
    
    // Add roles as a claim
    if !roles.is_empty() {
        builder.with_claim("roles", serde_json::json!(roles));
    }
    
    // Add any additional claims
    for (key, value) in additional_claims {
        builder.with_claim(key, value);
    }
    
    builder.build().expect("Failed to create test token")
}

/// Clear all policy configuration
pub fn clear_policy_config() {
    variables::set("mcp_policy", "");
    variables::set("mcp_policy_data", "");
}