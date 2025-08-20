// Complex policy tests with roles, scopes, and claims

use spin_test_sdk::{spin_test, bindings::wasi::http};
use crate::test_setup::setup_default_test_config;
use crate::policy_test_helpers::*;

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
    let (private_key, _public_key) = setup_test_jwt_validation();  // Ensure JWT validation is configured
    spin_test_sdk::bindings::fermyon::spin_test_virt::variables::set("mcp_policy", policy);
    
    // Superuser has all permissions
    let superuser_token = create_policy_test_token_with_key(&private_key, "super", vec!["superuser"], vec![]);
    
    let headers = http::types::Headers::new();
    headers.append("authorization", format!("Bearer {}", superuser_token).as_bytes()).unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_method(&http::types::Method::Get).unwrap();
    request.set_path_with_query(Some("/mcp/x/admin-dashboard")).unwrap();
    
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 200, "Superuser should access admin dashboard");
    
    let headers = http::types::Headers::new();
    headers.append("authorization", format!("Bearer {}", superuser_token).as_bytes()).unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_method(&http::types::Method::Get).unwrap();
    request.set_path_with_query(Some("/mcp/x/user-profile")).unwrap();
    
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 200, "Superuser inherits user role");
    
    // Regular user cannot access admin
    let user_token = create_policy_test_token_with_key(&private_key, "user", vec!["user"], vec![]);
    
    let headers = http::types::Headers::new();
    headers.append("authorization", format!("Bearer {}", user_token).as_bytes()).unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_method(&http::types::Method::Get).unwrap();
    request.set_path_with_query(Some("/mcp/x/admin-dashboard")).unwrap();
    
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 401, "User cannot access admin dashboard");
}

#[spin_test]
fn test_scope_based_authorization() {
    setup_default_test_config();
    
    let (private_key, _public_key) = setup_test_jwt_validation();  // Ensure JWT validation is configured
    
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
    let read_token = create_policy_test_token_with_key(&private_key, "reader", vec![], vec![("scopes", serde_json::json!(["users:read"]))]);
    
    // Can GET user-service
    let headers = http::types::Headers::new();
    headers.append("authorization", format!("Bearer {}", read_token).as_bytes()).unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_method(&http::types::Method::Get).unwrap();
    request.set_path_with_query(Some("/mcp/x/user-service")).unwrap();
    
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 200, "Should allow GET with read scope");
    
    // Cannot POST to user-service
    let headers = http::types::Headers::new();
    headers.append("authorization", format!("Bearer {}", read_token).as_bytes()).unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_method(&http::types::Method::Post).unwrap();
    request.set_path_with_query(Some("/mcp/x/user-service")).unwrap();
    
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 401, "Should deny POST without write scope");
    
    // Admin scope grants everything
    let admin_token = create_policy_test_token_with_key(&private_key, "admin", vec![], vec![("scopes", serde_json::json!(["admin"]))]);
    
    let headers = http::types::Headers::new();
    headers.append("authorization", format!("Bearer {}", admin_token).as_bytes()).unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_method(&http::types::Method::Delete).unwrap();
    request.set_path_with_query(Some("/mcp/x/user-service")).unwrap();
    
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
    let (private_key, _public_key) = setup_test_jwt_validation();  // Ensure JWT validation is configured
    spin_test_sdk::bindings::fermyon::spin_test_virt::variables::set("mcp_policy", policy);
    
    // User from engineering at acme-corp
    let eng_token = create_policy_test_token_with_key(&private_key,
        "engineer",
        vec![],
        vec![
            ("organization", serde_json::json!("acme-corp")),
            ("department", serde_json::json!("engineering")),
        ]
    );
    
    let headers = http::types::Headers::new();
    headers.append("authorization", format!("Bearer {}", eng_token).as_bytes()).unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_method(&http::types::Method::Get).unwrap();
    request.set_path_with_query(Some("/mcp/x/internal-tools")).unwrap();
    
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 200, "Engineering at acme-corp should have access");
    
    // User with high clearance
    let cleared_token = create_policy_test_token_with_key(&private_key,
        "agent",
        vec![],
        vec![("clearance_level", serde_json::json!(4))]
    );
    
    let headers = http::types::Headers::new();
    headers.append("authorization", format!("Bearer {}", cleared_token).as_bytes()).unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_method(&http::types::Method::Get).unwrap();
    request.set_path_with_query(Some("/mcp/x/classified-data")).unwrap();
    
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 200, "High clearance should access classified data");
    
    // User with beta access
    let beta_token = create_policy_test_token_with_key(&private_key,
        "beta_tester",
        vec![],
        vec![("feature_flags", serde_json::json!({"beta_access": true, "alpha_access": false}))]
    );
    
    let headers = http::types::Headers::new();
    headers.append("authorization", format!("Bearer {}", beta_token).as_bytes()).unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_method(&http::types::Method::Get).unwrap();
    request.set_path_with_query(Some("/mcp/x/beta-features")).unwrap();
    
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 200, "Beta access flag should grant access");
}

#[spin_test]
fn test_combined_authorization_rules() {
    setup_default_test_config();
    
    let (private_key, _public_key) = setup_test_jwt_validation();  // Ensure JWT validation is configured
    
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
    let service_token = create_policy_test_token_with_key(&private_key, "service-account-1", vec![], vec![
        ("scopes", serde_json::json!(["service:internal"])),
        ("account_type", serde_json::json!("service"))
    ]);
    
    let headers = http::types::Headers::new();
    headers.append("authorization", format!("Bearer {}", service_token).as_bytes()).unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_method(&http::types::Method::Get).unwrap();
    request.set_path_with_query(Some("/mcp/x/internal-api")).unwrap();
    
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 200, "Service account should access internal API");
    
    // Payment processor with MFA
    let payment_token = create_policy_test_token_with_key(&private_key, "payment-user", vec!["payments"], vec![
        ("scopes", serde_json::json!(["payments:process"])),
        ("mfa_verified", serde_json::json!(true))
    ]);
    
    let headers = http::types::Headers::new();
    headers.append("authorization", format!("Bearer {}", payment_token).as_bytes()).unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_method(&http::types::Method::Post).unwrap();
    request.set_path_with_query(Some("/mcp/x/payment-processor")).unwrap();
    
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 200, "Payment user with MFA should process payments");
    
    // Free tier limitations
    let free_token = create_policy_test_token_with_key(&private_key,
        "free-user",
        vec![],
        vec![("tier", serde_json::json!("free"))]
    );
    
    let headers = http::types::Headers::new();
    headers.append("authorization", format!("Bearer {}", free_token).as_bytes()).unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_method(&http::types::Method::Get).unwrap();
    request.set_path_with_query(Some("/mcp/x/premium-features")).unwrap();
    
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 401, "Free tier cannot access premium features");
}

#[spin_test]
fn test_deny_rules_precedence() {
    setup_default_test_config();
    
    let (private_key, _public_key) = setup_test_jwt_validation();  // Ensure JWT validation is configured
    
    // Policy with explicit deny rules that override allows
    let policy = r#"
package mcp.authorization
import rego.v1

default allow := false

# Deny if account is suspended
deny if {
    input.token.claims.account_suspended == true
}

# Deny blacklisted users
deny if {
    input.token.sub in ["hacker1", "spammer2", "banned3"]
}

# Allow users with valid subscription (but only if not denied)
allow if {
    input.token.claims.subscription_status == "active"
    not deny
}
"#;
    spin_test_sdk::bindings::fermyon::spin_test_virt::variables::set("mcp_policy", policy);
    
    // Active subscription but suspended account
    let suspended_token = create_policy_test_token_with_key(&private_key,
        "suspended-user",
        vec![],
        vec![
            ("subscription_status", serde_json::json!("active")),
            ("account_suspended", serde_json::json!(true))
        ]
    );
    
    let headers = http::types::Headers::new();
    headers.append("authorization", format!("Bearer {}", suspended_token).as_bytes()).unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_method(&http::types::Method::Get).unwrap();
    request.set_path_with_query(Some("/mcp/x/service")).unwrap();
    
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 401, "Suspended account should be denied despite active subscription");
    
    // Blacklisted user
    let blacklisted_token = create_policy_test_token_with_key(&private_key,
        "hacker1",
        vec![],
        vec![("subscription_status", serde_json::json!("active"))]
    );
    
    let headers = http::types::Headers::new();
    headers.append("authorization", format!("Bearer {}", blacklisted_token).as_bytes()).unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_method(&http::types::Method::Get).unwrap();
    request.set_path_with_query(Some("/mcp/x/service")).unwrap();
    
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 401, "Blacklisted user should be denied");
}
