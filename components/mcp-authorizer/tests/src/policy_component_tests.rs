// Component-scoped authorization tests

use spin_test_sdk::{spin_test, bindings::wasi::http};
use crate::test_setup::setup_default_test_config;
use crate::policy_test_helpers::*;

#[spin_test]
fn test_component_path_extraction() {
    setup_default_test_config();
    
    // Policy that checks component name
    let policy = r#"
package mcp.authorization
import rego.v1

default allow := false

allow if {
    input.request.component == "data-processor"
}
"#;
    spin_test_sdk::bindings::fermyon::spin_test_virt::variables::set("mcp_policy", policy);
    
    let token = create_policy_test_token("user", vec![], vec![]);
    
    // Test allowed component path
    let request = http::types::OutgoingRequest::new(http::types::Headers::new()); // Fix imports
        .method(&http::types::Method::Get)
        .uri("/mcp/x/data-processor")
        .header("authorization", format!("Bearer {}", token))
        .build();
    
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 200, "Should allow data-processor component");
    
    // Test denied component path
    let request = http::types::OutgoingRequest::new(http::types::Headers::new()); // Fix imports
        .method(&http::types::Method::Get)
        .uri("/mcp/x/other-component")
        .header("authorization", format!("Bearer {}", token))
        .build();
    
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 401, "Should deny other-component");
}

#[spin_test]
fn test_component_with_subpath() {
    setup_default_test_config();
    setup_component_policy(vec!["api-gateway"]);
    
    let token = create_policy_test_token("user", vec![], vec![]);
    
    // Test component with subpath - should still extract component correctly
    let request = http::types::OutgoingRequest::new(http::types::Headers::new()); // Fix imports
        .method(&http::types::Method::Post)
        .uri("/mcp/x/api-gateway/readonly")
        .header("authorization", format!("Bearer {}", token))
        .build();
    
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 200, "Should extract component from path with subpath");
}

#[spin_test]
fn test_no_component_in_path() {
    setup_default_test_config();
    
    // Policy that only allows requests WITH a component
    let policy = r#"
package mcp.authorization
import rego.v1

default allow := false

allow if {
    input.request.component != null
}
"#;
    spin_test_sdk::bindings::fermyon::spin_test_virt::variables::set("mcp_policy", policy);
    
    let token = create_policy_test_token("user", vec![], vec![]);
    
    // Test path without component (/mcp)
    let request = http::types::OutgoingRequest::new(http::types::Headers::new()); // Fix imports
        .method(&http::types::Method::Get)
        .uri("/mcp")
        .header("authorization", format!("Bearer {}", token))
        .build();
    
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 401, "Should deny when no component in path");
    
    // Test root path
    let request = http::types::OutgoingRequest::new(http::types::Headers::new()); // Fix imports
        .method(&http::types::Method::Get)
        .uri("/")
        .header("authorization", format!("Bearer {}", token))
        .build();
    
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 401, "Should deny root path (no component)");
}

#[spin_test]
fn test_user_component_mapping() {
    setup_default_test_config();
    
    // Policy with user-to-component mapping
    let policy = r#"
package mcp.authorization
import rego.v1

default allow := false

# User-component mapping
user_components := {
    "alice": ["frontend", "backend"],
    "bob": ["backend", "database"],
    "charlie": ["frontend"]
}

allow if {
    components := user_components[input.token.sub]
    input.request.component in components
}
"#;
    spin_test_sdk::bindings::fermyon::spin_test_virt::variables::set("mcp_policy", policy);
    
    // Alice can access frontend
    let alice_token = create_policy_test_token("alice", vec![], vec![]);
    let request = http::types::OutgoingRequest::new(http::types::Headers::new()); // Fix imports
        .method(&http::types::Method::Get)
        .uri("/mcp/x/frontend")
        .header("authorization", format!("Bearer {}", alice_token))
        .build();
    
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 200, "Alice should access frontend");
    
    // Bob can access database
    let bob_token = create_policy_test_token("bob", vec![], vec![]);
    let request = http::types::OutgoingRequest::new(http::types::Headers::new()); // Fix imports
        .method(&http::types::Method::Get)
        .uri("/mcp/x/database")
        .header("authorization", format!("Bearer {}", bob_token))
        .build();
    
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 200, "Bob should access database");
    
    // Charlie cannot access backend
    let charlie_token = create_policy_test_token("charlie", vec![], vec![]);
    let request = http::types::OutgoingRequest::new(http::types::Headers::new()); // Fix imports
        .method(&http::types::Method::Get)
        .uri("/mcp/x/backend")
        .header("authorization", format!("Bearer {}", charlie_token))
        .build();
    
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 401, "Charlie should not access backend");
}

#[spin_test]
fn test_component_with_role_requirements() {
    setup_default_test_config();
    
    // Policy requiring specific roles for components
    let policy = r#"
package mcp.authorization
import rego.v1

default allow := false

# Component role requirements
component_roles := {
    "admin-panel": ["admin", "superuser"],
    "user-dashboard": ["user", "admin"],
    "public-api": []  # No role required
}

allow if {
    required_roles := component_roles[input.request.component]
    count(required_roles) == 0  # Public component
}

allow if {
    required_roles := component_roles[input.request.component]
    user_roles := input.token.claims.roles
    some role in required_roles
    role in user_roles
}
"#;
    spin_test_sdk::bindings::fermyon::spin_test_virt::variables::set("mcp_policy", policy);
    
    // Admin accessing admin-panel
    let admin_token = create_policy_test_token("admin", vec!["admin"], vec![]);
    let request = http::types::OutgoingRequest::new(http::types::Headers::new()); // Fix imports
        .method(&http::types::Method::Get)
        .uri("/mcp/x/admin-panel")
        .header("authorization", format!("Bearer {}", admin_token))
        .build();
    
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 200, "Admin should access admin-panel");
    
    // Regular user cannot access admin-panel
    let user_token = create_policy_test_token("user", vec!["user"], vec![]);
    let request = http::types::OutgoingRequest::new(http::types::Headers::new()); // Fix imports
        .method(&http::types::Method::Get)
        .uri("/mcp/x/admin-panel")
        .header("authorization", format!("Bearer {}", user_token))
        .build();
    
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 401, "User should not access admin-panel");
    
    // Anyone can access public-api
    let anon_token = create_policy_test_token("anonymous", vec![], vec![]);
    let request = http::types::OutgoingRequest::new(http::types::Headers::new()); // Fix imports
        .method(&http::types::Method::Get)
        .uri("/mcp/x/public-api")
        .header("authorization", format!("Bearer {}", anon_token))
        .build();
    
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 200, "Anyone should access public-api");
}