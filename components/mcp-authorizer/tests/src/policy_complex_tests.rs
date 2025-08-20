// Complex policy tests with roles, scopes, and claims

use spin_test_sdk::{spin_test, bindings::wasi::http};
use crate::test_setup::setup_default_test_config;
use crate::policy_test_helpers::*;
use crate::test_token_utils::{TokenBuilder, KeyPairType};

#[spin_test]
fn test_role_based_authorization() {
    setup_default_test_config();
    
    // Complex role-based policy
    let policy = r#"
package mcp.authorization
import rego.v1

default allow := false

# Role hierarchy
role_hierarchy := {
    "superuser": ["admin", "moderator", "user"],
    "admin": ["moderator", "user"],
    "moderator": ["user"],
    "user": []
}

# Check if user has effective role (including inherited)
has_role(user_roles, required_role) if {
    some role in user_roles
    role == required_role
}

has_role(user_roles, required_role) if {
    some role in user_roles
    inherited := role_hierarchy[role]
    required_role in inherited
}

# Component requirements
allow if {
    input.request.component == "admin-dashboard"
    user_roles := input.token.claims.roles
    has_role(user_roles, "admin")
}

allow if {
    input.request.component == "moderation-tools"
    user_roles := input.token.claims.roles
    has_role(user_roles, "moderator")
}

allow if {
    input.request.component == "user-profile"
    user_roles := input.token.claims.roles
    has_role(user_roles, "user")
}
"#;
    spin_test_sdk::bindings::fermyon::spin_test_virt::variables::set("mcp_policy", policy);
    
    // Superuser has all permissions
    let superuser_token = create_policy_test_token("super", vec!["superuser"], vec![]);
    
    let request = http::types::OutgoingRequest::new(http::types::Headers::new()); // Fix imports
        .method(&http::types::Method::Get)
        .uri("/mcp/x/admin-dashboard")
        .header("authorization", format!("Bearer {}", superuser_token))
        .build();
    
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 200, "Superuser should access admin dashboard");
    
    let request = http::types::OutgoingRequest::new(http::types::Headers::new()); // Fix imports
        .method(&http::types::Method::Get)
        .uri("/mcp/x/user-profile")
        .header("authorization", format!("Bearer {}", superuser_token))
        .build();
    
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 200, "Superuser inherits user role");
    
    // Regular user cannot access admin
    let user_token = create_policy_test_token("user", vec!["user"], vec![]);
    
    let request = http::types::OutgoingRequest::new(http::types::Headers::new()); // Fix imports
        .method(&http::types::Method::Get)
        .uri("/mcp/x/admin-dashboard")
        .header("authorization", format!("Bearer {}", user_token))
        .build();
    
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 401, "User cannot access admin dashboard");
}

#[spin_test]
fn test_scope_based_authorization() {
    setup_default_test_config();
    
    // OAuth scope-based policy
    let policy = r#"
package mcp.authorization
import rego.v1

default allow := false

# Map components to required scopes
component_scopes := {
    "user-service": {
        "GET": ["users:read"],
        "POST": ["users:write"],
        "DELETE": ["users:delete", "admin"]
    },
    "data-export": {
        "GET": ["export:read", "data:export"],
        "POST": ["export:create", "admin"]
    }
}

allow if {
    component := input.request.component
    method := input.request.method
    required_scopes := component_scopes[component][method]
    user_scopes := input.token.scopes
    
    some required_scope in required_scopes
    required_scope in user_scopes
}
"#;
    spin_test_sdk::bindings::fermyon::spin_test_virt::variables::set("mcp_policy", policy);
    
    // User with read scope
    let mut builder = TokenBuilder::new(KeyPairType::default());
    builder.with_subject("reader");
    builder.with_audience("test-audience");
    builder.with_issuer("https://test.authkit.app");
    builder.with_scope("users:read");
    let read_token = builder.build().unwrap();
    
    // Can GET user-service
    let request = http::types::OutgoingRequest::new(http::types::Headers::new()); // Fix imports
        .method(&http::types::Method::Get)
        .uri("/mcp/x/user-service")
        .header("authorization", format!("Bearer {}", read_token))
        .build();
    
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 200, "Should allow GET with read scope");
    
    // Cannot POST to user-service
    let request = http::types::OutgoingRequest::new(http::types::Headers::new()); // Fix imports
        .method(&http::types::Method::Post)
        .uri("/mcp/x/user-service")
        .header("authorization", format!("Bearer {}", read_token))
        .build();
    
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 401, "Should deny POST without write scope");
    
    // Admin scope grants everything
    let mut builder = TokenBuilder::new(KeyPairType::default());
    builder.with_subject("admin");
    builder.with_audience("test-audience");
    builder.with_issuer("https://test.authkit.app");
    builder.with_scope("admin");
    let admin_token = builder.build().unwrap();
    
    let request = http::types::OutgoingRequest::new(http::types::Headers::new()); // Fix imports
        .method(&http::types::Method::Delete)
        .uri("/mcp/x/user-service")
        .header("authorization", format!("Bearer {}", admin_token))
        .build();
    
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 200, "Admin scope allows DELETE");
}

#[spin_test]
fn test_custom_claims_authorization() {
    setup_default_test_config();
    
    // Policy using custom JWT claims
    let policy = r#"
package mcp.authorization
import rego.v1

default allow := false

# Check organization membership
allow if {
    input.token.claims.organization == "acme-corp"
    input.token.claims.department in ["engineering", "devops"]
}

# Check security clearance level
allow if {
    input.request.component == "classified-data"
    input.token.claims.clearance_level >= 3
}

# Check feature flags
allow if {
    input.request.component == "beta-features"
    input.token.claims.feature_flags.beta_access == true
}

# Time-based access
allow if {
    input.token.claims.access_hours.start <= 14  # 2 PM
    input.token.claims.access_hours.end >= 14
}
"#;
    spin_test_sdk::bindings::fermyon::spin_test_virt::variables::set("mcp_policy", policy);
    
    // User from engineering at acme-corp
    let eng_token = create_policy_test_token(
        "engineer",
        vec![],
        vec![
            ("organization", serde_json::json!("acme-corp")),
            ("department", serde_json::json!("engineering")),
        ]
    );
    
    let request = http::types::OutgoingRequest::new(http::types::Headers::new()); // Fix imports
        .method(&http::types::Method::Get)
        .uri("/mcp/x/internal-tools")
        .header("authorization", format!("Bearer {}", eng_token))
        .build();
    
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 200, "Engineering at acme-corp should have access");
    
    // User with high clearance
    let cleared_token = create_policy_test_token(
        "agent",
        vec![],
        vec![("clearance_level", serde_json::json!(4))]
    );
    
    let request = http::types::OutgoingRequest::new(http::types::Headers::new()); // Fix imports
        .method(&http::types::Method::Get)
        .uri("/mcp/x/classified-data")
        .header("authorization", format!("Bearer {}", cleared_token))
        .build();
    
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 200, "High clearance should access classified data");
    
    // User with beta access
    let beta_token = create_policy_test_token(
        "beta_tester",
        vec![],
        vec![("feature_flags", serde_json::json!({"beta_access": true, "alpha_access": false}))]
    );
    
    let request = http::types::OutgoingRequest::new(http::types::Headers::new()); // Fix imports
        .method(&http::types::Method::Get)
        .uri("/mcp/x/beta-features")
        .header("authorization", format!("Bearer {}", beta_token))
        .build();
    
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 200, "Beta access flag should grant access");
}

#[spin_test]
fn test_combined_authorization_rules() {
    setup_default_test_config();
    
    // Policy combining multiple authorization strategies
    let policy = r#"
package mcp.authorization
import rego.v1

default allow := false

# Admins can do anything
allow if {
    "admin" in input.token.claims.roles
}

# Service accounts with specific scope
allow if {
    input.token.claims.account_type == "service"
    "service:internal" in input.token.scopes
    input.request.component in ["internal-api", "metrics", "health"]
}

# Users need both role AND scope for sensitive operations
allow if {
    input.request.component == "payment-processor"
    "payments" in input.token.claims.roles
    "payments:process" in input.token.scopes
    input.token.claims.mfa_verified == true
}

# Rate limiting by user tier
allow if {
    input.token.claims.tier == "premium"
    input.request.component != "admin-dashboard"
}

allow if {
    input.token.claims.tier == "free"
    input.request.component in ["public-api", "user-profile"]
}
"#;
    spin_test_sdk::bindings::fermyon::spin_test_virt::variables::set("mcp_policy", policy);
    
    // Service account test
    let mut builder = TokenBuilder::new(KeyPairType::default());
    builder.with_subject("service-account-1");
    builder.with_audience("test-audience");
    builder.with_issuer("https://test.authkit.app");
    builder.with_scope("service:internal");
    builder.with_claim("account_type", serde_json::json!("service"));
    let service_token = builder.build().unwrap();
    
    let request = http::types::OutgoingRequest::new(http::types::Headers::new()); // Fix imports
        .method(&http::types::Method::Get)
        .uri("/mcp/x/internal-api")
        .header("authorization", format!("Bearer {}", service_token))
        .build();
    
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 200, "Service account should access internal API");
    
    // Payment processor with MFA
    let mut builder = TokenBuilder::new(KeyPairType::default());
    builder.with_subject("payment-user");
    builder.with_audience("test-audience");
    builder.with_issuer("https://test.authkit.app");
    builder.with_scope("payments:process");
    builder.with_claim("roles", serde_json::json!(["payments"]));
    builder.with_claim("mfa_verified", serde_json::json!(true));
    let payment_token = builder.build().unwrap();
    
    let request = http::types::OutgoingRequest::new(http::types::Headers::new()); // Fix imports
        .method(&http::types::Method::Post)
        .uri("/mcp/x/payment-processor")
        .header("authorization", format!("Bearer {}", payment_token))
        .build();
    
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 200, "Payment user with MFA should process payments");
    
    // Free tier limitations
    let free_token = create_policy_test_token(
        "free-user",
        vec![],
        vec![("tier", serde_json::json!("free"))]
    );
    
    let request = http::types::OutgoingRequest::new(http::types::Headers::new()); // Fix imports
        .method(&http::types::Method::Get)
        .uri("/mcp/x/premium-features")
        .header("authorization", format!("Bearer {}", free_token))
        .build();
    
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 401, "Free tier cannot access premium features");
}

#[spin_test]
fn test_deny_rules_precedence() {
    setup_default_test_config();
    
    // Policy with explicit deny rules that override allows
    let policy = r#"
package mcp.authorization
import rego.v1

default allow := false

# Allow users with valid subscription
allow if {
    input.token.claims.subscription_status == "active"
}

# But deny if account is suspended (overrides allow)
deny if {
    input.token.claims.account_suspended == true
}

# Deny blacklisted users
deny if {
    input.token.sub in ["hacker1", "spammer2", "banned3"]
}

# Final decision
allow if {
    not deny
}
"#;
    spin_test_sdk::bindings::fermyon::spin_test_virt::variables::set("mcp_policy", policy);
    
    // Active subscription but suspended account
    let suspended_token = create_policy_test_token(
        "suspended-user",
        vec![],
        vec![
            ("subscription_status", serde_json::json!("active")),
            ("account_suspended", serde_json::json!(true))
        ]
    );
    
    let request = http::types::OutgoingRequest::new(http::types::Headers::new()); // Fix imports
        .method(&http::types::Method::Get)
        .uri("/mcp/x/service")
        .header("authorization", format!("Bearer {}", suspended_token))
        .build();
    
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 401, "Suspended account should be denied despite active subscription");
    
    // Blacklisted user
    let blacklisted_token = create_policy_test_token(
        "hacker1",
        vec![],
        vec![("subscription_status", serde_json::json!("active"))]
    );
    
    let request = http::types::OutgoingRequest::new(http::types::Headers::new()); // Fix imports
        .method(&http::types::Method::Get)
        .uri("/mcp/x/service")
        .header("authorization", format!("Bearer {}", blacklisted_token))
        .build();
    
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 401, "Blacklisted user should be denied");
}