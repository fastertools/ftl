// Policy tests with external data

use spin_test_sdk::{spin_test, bindings::wasi::http};
use crate::test_setup::setup_default_test_config;
use crate::policy_test_helpers::*;

#[spin_test]
fn test_policy_with_external_data() {
    setup_default_test_config();
    
    // Policy that uses external data
    let policy = r#"
package mcp.authorization
import rego.v1

default allow := false

# Check if user is in allowed list from data
allow if {
    input.token.sub in data.allowed_users
}

# Check component permissions from data
allow if {
    permissions := data.component_permissions[input.request.component]
    input.token.sub in permissions.users
}
"#;
    
    // External data JSON
    let data = r#"{
        "allowed_users": ["alice", "bob", "charlie"],
        "component_permissions": {
            "api-gateway": {
                "users": ["alice", "bob"],
                "roles": ["admin"]
            },
            "database": {
                "users": ["charlie"],
                "roles": ["dba", "admin"]
            }
        }
    }"#;
    
    let (private_key, _public_key) = setup_policy_with_data(policy, data);
    
    // Alice in allowed_users
    let alice_token = create_policy_test_token_with_key(&private_key, "alice", vec![], vec![]);
    
    let headers = http::types::Headers::new();
    headers.append("authorization", format!("Bearer {}", alice_token).as_bytes()).unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_method(&http::types::Method::Get).unwrap();
    request.set_path_with_query(Some("/mcp/x/api-gateway")).unwrap();
    
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 200, "Alice should be allowed via data");
    
    // Dave not in allowed_users
    let dave_token = create_policy_test_token_with_key(&private_key, "dave", vec![], vec![]);
    
    let headers = http::types::Headers::new();
    headers.append("authorization", format!("Bearer {}", dave_token).as_bytes()).unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_method(&http::types::Method::Get).unwrap();
    request.set_path_with_query(Some("/mcp/x/api-gateway")).unwrap();
    
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 401, "Dave should be denied");
    
    // Charlie can access database component
    let charlie_token = create_policy_test_token_with_key(&private_key, "charlie", vec![], vec![]);
    
    let headers = http::types::Headers::new();
    headers.append("authorization", format!("Bearer {}", charlie_token).as_bytes()).unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_method(&http::types::Method::Get).unwrap();
    request.set_path_with_query(Some("/mcp/x/database")).unwrap();
    
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 200, "Charlie should access database component");
}

#[spin_test]
fn test_complex_data_structures() {
    setup_default_test_config();
    
    let policy = r#"
package mcp.authorization
import rego.v1

default allow := false

# Check tool permissions from nested data
allow if {
    input.mcp.method == "tools/call"
    tool_config := data.tools[input.mcp.tool]
    
    # Check minimum role level
    user_level := data.role_levels[input.token.claims.role]
    user_level >= tool_config.min_level
}

# IP-based access control from data
allow if {
    input.request.headers["x-forwarded-for"] in data.allowed_ips
}

# Time-window access from data
allow if {
    window := data.maintenance_windows[input.request.component]
    window.active == false
}
"#;
    
    let data = r#"{
        "tools": {
            "read_config": {"min_level": 1, "category": "read"},
            "update_config": {"min_level": 2, "category": "write"},
            "delete_all": {"min_level": 4, "category": "admin"}
        },
        "role_levels": {
            "viewer": 1,
            "editor": 2,
            "admin": 3,
            "superadmin": 4
        },
        "allowed_ips": ["10.0.0.1", "192.168.1.1"],
        "maintenance_windows": {
            "api-gateway": {"active": false, "message": "No maintenance"},
            "database": {"active": true, "message": "Database maintenance in progress"}
        }
    }"#;
    
    let (private_key, _public_key) = setup_policy_with_data(policy, data);
    
    // Test role levels with tools
    let admin_token = create_policy_test_token_with_key(&private_key, "admin", vec![], vec![("role", serde_json::json!("admin"))]);
    
    let body = r#"{
        "jsonrpc":"2.0",
        "id":1,
        "method":"tools/call",
        "params":{
            "name":"update_config"
        }
    }"#;
    
    let headers = http::types::Headers::new();
    headers.append("authorization", format!("Bearer {}", admin_token).as_bytes()).unwrap();
    headers.append("content-type", b"application/json").unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_method(&http::types::Method::Post).unwrap();
    request.set_path_with_query(Some("/mcp/x/config-service")).unwrap();
    let body_stream = request.body().unwrap();
    body_stream.write_bytes(body.as_bytes());
    
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 200, "Admin level 3 >= required level 2");
    
    // Viewer cannot use delete_all
    let viewer_token = create_policy_test_token_with_key(&private_key, "viewer", vec![], vec![("role", serde_json::json!("viewer"))]);
    
    let body = r#"{
        "jsonrpc":"2.0",
        "id":1,
        "method":"tools/call",
        "params":{
            "name":"delete_all"
        }
    }"#;
    
    let headers = http::types::Headers::new();
    headers.append("authorization", format!("Bearer {}", viewer_token).as_bytes()).unwrap();
    headers.append("content-type", b"application/json").unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_method(&http::types::Method::Post).unwrap();
    request.set_path_with_query(Some("/mcp/x/config-service")).unwrap();
    let body_stream = request.body().unwrap();
    body_stream.write_bytes(body.as_bytes());
    
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 401, "Viewer level 1 < required level 4");
}

#[spin_test]
fn test_dynamic_data_rules() {
    setup_default_test_config();
    
    let policy = r#"
package mcp.authorization
import rego.v1

default allow := false

# Rate limiting from data
allow if {
    user_limits := data.rate_limits[input.token.claims.tier]
    current_requests := data.user_request_counts[input.token.sub]
    
    # Default to 0 if user not in counts
    count := current_requests
    count < user_limits.max_requests
}

# Feature flags from data
allow if {
    feature := data.feature_flags[input.request.component]
    feature.enabled == true
    
    # Check if user is in rollout percentage (simplified)
    feature.rollout_percentage == 100
}

# Dynamic role assignments from data
allow if {
    project := data.projects[input.request.component]
    team := data.teams[project.team_id]
    input.token.sub in team.members
}
"#;
    
    let data = r#"{
        "rate_limits": {
            "free": {"max_requests": 10, "window": "hour"},
            "premium": {"max_requests": 1000, "window": "hour"},
            "enterprise": {"max_requests": 10000, "window": "hour"}
        },
        "user_request_counts": {
            "user1": 5,
            "user2": 15,
            "user3": 0
        },
        "feature_flags": {
            "new-api": {"enabled": true, "rollout_percentage": 100},
            "beta-feature": {"enabled": true, "rollout_percentage": 50},
            "disabled-feature": {"enabled": false, "rollout_percentage": 0}
        },
        "projects": {
            "project-alpha": {"team_id": "team-1", "status": "active"},
            "project-beta": {"team_id": "team-2", "status": "active"}
        },
        "teams": {
            "team-1": {"members": ["alice", "bob"], "lead": "alice"},
            "team-2": {"members": ["charlie", "dave"], "lead": "charlie"}
        }
    }"#;
    
    let (private_key, _public_key) = setup_policy_with_data(policy, data);
    
    // Test rate limiting
    let user1_token = create_policy_test_token_with_key(&private_key, "user1", vec![], vec![("tier", serde_json::json!("free"))]);
    
    let headers = http::types::Headers::new();
    headers.append("authorization", format!("Bearer {}", user1_token).as_bytes()).unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_method(&http::types::Method::Get).unwrap();
    request.set_path_with_query(Some("/mcp/x/api")).unwrap();
    
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 200, "User1 under rate limit (5 < 10)");
    
    let user2_token = create_policy_test_token_with_key(&private_key, "user2", vec![], vec![("tier", serde_json::json!("free"))]);
    
    let headers = http::types::Headers::new();
    headers.append("authorization", format!("Bearer {}", user2_token).as_bytes()).unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_method(&http::types::Method::Get).unwrap();
    request.set_path_with_query(Some("/mcp/x/api")).unwrap();
    
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 401, "User2 over rate limit (15 > 10)");
    
    // Test feature flags
    let user_token = create_policy_test_token_with_key(&private_key, "user", vec![], vec![]);
    
    let headers = http::types::Headers::new();
    headers.append("authorization", format!("Bearer {}", user_token).as_bytes()).unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_method(&http::types::Method::Get).unwrap();
    request.set_path_with_query(Some("/mcp/x/new-api")).unwrap();
    
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 200, "New API feature is enabled");
    
    let headers = http::types::Headers::new();
    headers.append("authorization", format!("Bearer {}", user_token).as_bytes()).unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_method(&http::types::Method::Get).unwrap();
    request.set_path_with_query(Some("/mcp/x/disabled-feature")).unwrap();
    
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 401, "Disabled feature should deny access");
    
    // Test team-based access
    let alice_token = create_policy_test_token_with_key(&private_key, "alice", vec![], vec![]);
    
    let headers = http::types::Headers::new();
    headers.append("authorization", format!("Bearer {}", alice_token).as_bytes()).unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_method(&http::types::Method::Get).unwrap();
    request.set_path_with_query(Some("/mcp/x/project-alpha")).unwrap();
    
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 200, "Alice is in team-1 for project-alpha");
    
    let headers = http::types::Headers::new();
    headers.append("authorization", format!("Bearer {}", alice_token).as_bytes()).unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_method(&http::types::Method::Get).unwrap();
    request.set_path_with_query(Some("/mcp/x/project-beta")).unwrap();
    
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 401, "Alice is not in team-2 for project-beta");
}

#[spin_test]
fn test_missing_data_handling() {
    setup_default_test_config();
    
    // Policy that references non-existent data
    let policy = r#"
package mcp.authorization
import rego.v1

default allow := false

# Try to access data that doesn't exist
allow if {
    # This will be undefined if data.users doesn't exist
    input.token.sub in data.users
}

# Fallback rule that doesn't depend on data
allow if {
    input.token.claims.bypass == true
}
"#;
    
    // Empty data
    let (private_key, _public_key) = setup_policy_with_data(policy, "{}");
    
    // Normal user - should be denied (data.users doesn't exist)
    let user_token = create_policy_test_token_with_key(&private_key, "user", vec![], vec![]);
    
    let headers = http::types::Headers::new();
    headers.append("authorization", format!("Bearer {}", user_token).as_bytes()).unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_method(&http::types::Method::Get).unwrap();
    request.set_path_with_query(Some("/mcp/x/service")).unwrap();
    
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 401, "Should deny when referenced data doesn't exist");
    
    // User with bypass claim
    let bypass_token = create_policy_test_token_with_key(&private_key, "special", vec![], vec![("bypass", serde_json::json!(true))]);
    
    let headers = http::types::Headers::new();
    headers.append("authorization", format!("Bearer {}", bypass_token).as_bytes()).unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_method(&http::types::Method::Get).unwrap();
    request.set_path_with_query(Some("/mcp/x/service")).unwrap();
    
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 200, "Bypass rule should work without data");
}