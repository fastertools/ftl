// MCP tool-level authorization tests

use spin_test_sdk::{spin_test, bindings::wasi::http};
use crate::test_setup::setup_default_test_config;
use crate::policy_test_helpers::*;

#[spin_test]
fn test_mcp_tools_list_allowed() {
    setup_default_test_config();
    
    // Policy that allows tools/list for everyone
    let policy = r#"
package mcp.authorization
import rego.v1

default allow := false

allow if {
    input.mcp.method == "tools/list"
}
"#;
    let (private_key, _public_key) = setup_test_jwt_validation();  // Ensure JWT validation is configured
    spin_test_sdk::bindings::fermyon::spin_test_virt::variables::set("mcp_policy", policy);
    
    let token = create_policy_test_token_with_key(&private_key, "user", vec![], vec![]);
    
    // Create JSON-RPC tools/list request
    let body = r#"{"jsonrpc":"2.0","id":1,"method":"tools/list"}"#;
    
    let headers = http::types::Headers::new();
    headers.append("authorization", format!("Bearer {}", token).as_bytes()).unwrap();
    headers.append("content-type", b"application/json").unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_method(&http::types::Method::Post).unwrap();
    request.set_path_with_query(Some("/mcp/x/data-processor")).unwrap();
    let body_stream = request.body().unwrap();
    body_stream.write_bytes(body.as_bytes());
    
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 200, "Should allow tools/list");
}

#[spin_test]
fn test_mcp_tool_call_authorization() {
    setup_default_test_config();
    let (private_key, _public_key) = setup_tool_authorization_policy(
        vec!["read_data", "list_items"],  // allowed tools
        vec!["delete_database", "reset_system"]  // dangerous tools
    );
    
    // User without admin role
    let user_token = create_policy_test_token_with_key(&private_key, "user", vec!["user"], vec![]);
    
    // Test allowed tool
    let body = r#"{
        "jsonrpc":"2.0",
        "id":1,
        "method":"tools/call",
        "params":{
            "name":"read_data",
            "arguments":{}
        }
    }"#;
    
    let headers = http::types::Headers::new();
    headers.append("authorization", format!("Bearer {}", user_token).as_bytes()).unwrap();
    headers.append("content-type", b"application/json").unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_method(&http::types::Method::Post).unwrap();
    request.set_path_with_query(Some("/mcp/x/data-processor")).unwrap();
    let body_stream = request.body().unwrap();
    body_stream.write_bytes(body.as_bytes());
    
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 200, "Should allow safe tool");
    
    // Test dangerous tool without admin
    let body = r#"{
        "jsonrpc":"2.0",
        "id":1,
        "method":"tools/call",
        "params":{
            "name":"delete_database",
            "arguments":{"confirm":true}
        }
    }"#;
    
    let headers = http::types::Headers::new();
    headers.append("authorization", format!("Bearer {}", user_token).as_bytes()).unwrap();
    headers.append("content-type", b"application/json").unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_method(&http::types::Method::Post).unwrap();
    request.set_path_with_query(Some("/mcp/x/data-processor")).unwrap();
    let body_stream = request.body().unwrap();
    body_stream.write_bytes(body.as_bytes());
    
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 401, "Should deny dangerous tool without admin");
    
    // Admin can use dangerous tools
    let admin_token = create_policy_test_token_with_key(&private_key, "admin", vec!["admin"], vec![]);
    
    let headers = http::types::Headers::new();
    headers.append("authorization", format!("Bearer {}", admin_token).as_bytes()).unwrap();
    headers.append("content-type", b"application/json").unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_method(&http::types::Method::Post).unwrap();
    request.set_path_with_query(Some("/mcp/x/data-processor")).unwrap();
    let body_stream = request.body().unwrap();
    body_stream.write_bytes(body.as_bytes());
    
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 200, "Admin should access dangerous tool");
}

#[spin_test]
fn test_mcp_context_not_added_for_non_json() {
    setup_default_test_config();
    
    // Policy that checks for MCP context
    let policy = r#"
package mcp.authorization
import rego.v1

default allow := false

# Only allow if MCP context exists
allow if {
    input.mcp
}
"#;
    let (private_key, _public_key) = setup_test_jwt_validation();  // Ensure JWT validation is configured
    spin_test_sdk::bindings::fermyon::spin_test_virt::variables::set("mcp_policy", policy);
    
    let token = create_policy_test_token_with_key(&private_key, "user", vec![], vec![]);
    
    // POST request with non-JSON content
    let headers = http::types::Headers::new();
    headers.append("authorization", format!("Bearer {}", token).as_bytes()).unwrap();
    headers.append("content-type", b"text/plain").unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_method(&http::types::Method::Post).unwrap();
    request.set_path_with_query(Some("/mcp/x/data-processor")).unwrap();
    let body_stream = request.body().unwrap();
    body_stream.write_bytes(b"plain text body");
    
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 401, "Should deny when MCP context not available");
    
    // GET request (no body)
    let headers = http::types::Headers::new();
    headers.append("authorization", format!("Bearer {}", token).as_bytes()).unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_method(&http::types::Method::Get).unwrap();
    request.set_path_with_query(Some("/mcp/x/data-processor")).unwrap();
    
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 401, "Should deny GET request (no MCP context)");
}

#[spin_test]
fn test_tool_specific_permissions() {
    setup_default_test_config();
    
    let (private_key, _public_key) = setup_test_jwt_validation();  // Ensure JWT validation is configured
    
    // Policy with tool-specific permission requirements
    let policy = r#"
package mcp.authorization
import rego.v1

default allow := false

# Tool permission mapping
tool_permissions := {
    "read_users": ["user:read", "admin"],
    "create_user": ["user:write", "admin"],
    "delete_user": ["admin"],
    "export_data": ["data:export", "admin"]
}

allow if {
    input.mcp.method == "tools/list"
}

allow if {
    input.mcp.method == "tools/call"
    required_scopes := tool_permissions[input.mcp.tool]
    user_scopes := input.token.scopes
    some scope in required_scopes
    scope in user_scopes
}
"#;
    spin_test_sdk::bindings::fermyon::spin_test_virt::variables::set("mcp_policy", policy);
    
    // User with read scope
    let reader_token = create_policy_test_token_with_key(&private_key, "reader", vec![], vec![("scopes", serde_json::json!(["user:read"]))]);
    
    // Can read users
    let body = r#"{
        "jsonrpc":"2.0",
        "id":1,
        "method":"tools/call",
        "params":{"name":"read_users"}
    }"#;
    
    let headers = http::types::Headers::new();
    headers.append("authorization", format!("Bearer {}", reader_token).as_bytes()).unwrap();
    headers.append("content-type", b"application/json").unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_method(&http::types::Method::Post).unwrap();
    request.set_path_with_query(Some("/mcp/x/user-service")).unwrap();
    let body_stream = request.body().unwrap();
    body_stream.write_bytes(body.as_bytes());
    
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 200, "Should allow read with read scope");
    
    // Cannot create users
    let body = r#"{
        "jsonrpc":"2.0",
        "id":1,
        "method":"tools/call",
        "params":{"name":"create_user"}
    }"#;
    
    let headers = http::types::Headers::new();
    headers.append("authorization", format!("Bearer {}", reader_token).as_bytes()).unwrap();
    headers.append("content-type", b"application/json").unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_method(&http::types::Method::Post).unwrap();
    request.set_path_with_query(Some("/mcp/x/user-service")).unwrap();
    let body_stream = request.body().unwrap();
    body_stream.write_bytes(body.as_bytes());
    
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 401, "Should deny create without write scope");
}

#[spin_test]
fn test_mcp_invalid_json_handling() {
    setup_default_test_config();
    
    // Policy that allows if no MCP context (graceful degradation)
    let policy = r#"
package mcp.authorization
import rego.v1

default allow := false

# Allow if MCP context is missing (non-MCP request)
allow if {
    not input.mcp
}

# For MCP requests, only allow safe methods
allow if {
    input.mcp.method in ["tools/list", "prompts/list", "resources/list"]
}
"#;
    let (private_key, _public_key) = setup_test_jwt_validation();  // Ensure JWT validation is configured
    spin_test_sdk::bindings::fermyon::spin_test_virt::variables::set("mcp_policy", policy);
    
    let token = create_policy_test_token_with_key(&private_key, "user", vec![], vec![]);
    
    // Invalid JSON body
    let body = r#"{"invalid json": }"#;
    
    let headers = http::types::Headers::new();
    headers.append("authorization", format!("Bearer {}", token).as_bytes()).unwrap();
    headers.append("content-type", b"application/json").unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_method(&http::types::Method::Post).unwrap();
    request.set_path_with_query(Some("/mcp/x/data-processor")).unwrap();
    let body_stream = request.body().unwrap();
    body_stream.write_bytes(body.as_bytes());
    
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 200, 
               "Should handle invalid JSON gracefully (no MCP context)");
}

#[spin_test]
fn test_tool_arguments_inspection() {
    setup_default_test_config();
    
    // Policy that inspects tool arguments
    let policy = r#"
package mcp.authorization
import rego.v1

default allow := false

# Allow read operations
allow if {
    input.mcp.method == "tools/call"
    input.mcp.tool == "query_database"
    input.mcp.arguments.operation == "SELECT"
}

# Deny destructive operations
deny if {
    input.mcp.method == "tools/call"
    input.mcp.tool == "query_database"
    input.mcp.arguments.operation in ["DROP", "DELETE", "TRUNCATE"]
}

allow if {
    not deny
}
"#;
    let (private_key, _public_key) = setup_test_jwt_validation();  // Ensure JWT validation is configured
    spin_test_sdk::bindings::fermyon::spin_test_virt::variables::set("mcp_policy", policy);
    
    let token = create_policy_test_token_with_key(&private_key, "user", vec![], vec![]);
    
    // Test SELECT query (allowed)
    let body = r#"{
        "jsonrpc":"2.0",
        "id":1,
        "method":"tools/call",
        "params":{
            "name":"query_database",
            "arguments":{
                "operation":"SELECT",
                "table":"users"
            }
        }
    }"#;
    
    let headers = http::types::Headers::new();
    headers.append("authorization", format!("Bearer {}", token).as_bytes()).unwrap();
    headers.append("content-type", b"application/json").unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_method(&http::types::Method::Post).unwrap();
    request.set_path_with_query(Some("/mcp/x/database")).unwrap();
    let body_stream = request.body().unwrap();
    body_stream.write_bytes(body.as_bytes());
    
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 200, "Should allow SELECT operation");
    
    // Test DROP query (denied)
    let body = r#"{
        "jsonrpc":"2.0",
        "id":1,
        "method":"tools/call",
        "params":{
            "name":"query_database",
            "arguments":{
                "operation":"DROP",
                "table":"users"
            }
        }
    }"#;
    
    let headers = http::types::Headers::new();
    headers.append("authorization", format!("Bearer {}", token).as_bytes()).unwrap();
    headers.append("content-type", b"application/json").unwrap();
    let request = http::types::OutgoingRequest::new(headers);
    request.set_method(&http::types::Method::Post).unwrap();
    request.set_path_with_query(Some("/mcp/x/database")).unwrap();
    let body_stream = request.body().unwrap();
    body_stream.write_bytes(body.as_bytes());
    
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 401, "Should deny DROP operation");
}
