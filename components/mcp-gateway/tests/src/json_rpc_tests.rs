use spin_test_sdk::{
    bindings::wasi::http,
    spin_test,
};
use crate::{ResponseData, test_helpers::*};

#[spin_test]
fn test_json_rpc_version_validation() {
    setup_default_test_env();
    
    // Test without jsonrpc field
    let request_json = serde_json::json!({
        "method": "ping",
        "id": 1
    });
    
    let request = create_mcp_request(request_json);
    let response = spin_test_sdk::perform_request(request);
    let response_data = ResponseData::from_response(response);
    
    assert_eq!(response_data.status, 200);
    let response_json = response_data.body_json().expect("Expected JSON response");
    assert_json_rpc_error(&response_json, -32700, None);
    
    // Test with wrong version
    let request_json = serde_json::json!({
        "jsonrpc": "1.0",
        "method": "ping",
        "id": 1
    });
    
    let request = create_mcp_request(request_json);
    let response = spin_test_sdk::perform_request(request);
    let response_data = ResponseData::from_response(response);
    
    assert_eq!(response_data.status, 200);
    let response_json = response_data.body_json().expect("Expected JSON response");
    assert_json_rpc_error(&response_json, -32600, Some(serde_json::json!(1))); // Invalid request
}

#[spin_test]
fn test_json_rpc_id_types() {
    setup_default_test_env();
    
    // Test with number ID
    let request_json = create_json_rpc_request("ping", None, Some(serde_json::json!(42)));
    let request = create_mcp_request(request_json);
    let response = spin_test_sdk::perform_request(request);
    let response_data = ResponseData::from_response(response);
    
    let response_json = response_data.body_json().expect("Expected JSON response");
    assert_eq!(response_json["id"], 42);
    
    // Test with string ID
    let request_json = create_json_rpc_request("ping", None, Some(serde_json::json!("test-id")));
    let request = create_mcp_request(request_json);
    let response = spin_test_sdk::perform_request(request);
    let response_data = ResponseData::from_response(response);
    
    let response_json = response_data.body_json().expect("Expected JSON response");
    assert_eq!(response_json["id"], "test-id");
    
    // Test with null ID
    let request_json = create_json_rpc_request("ping", None, Some(serde_json::json!(null)));
    let request = create_mcp_request(request_json);
    let response = spin_test_sdk::perform_request(request);
    let response_data = ResponseData::from_response(response);
    
    let response_json = response_data.body_json().expect("Expected JSON response");
    assert_eq!(response_json["id"], serde_json::json!(null));
}

#[spin_test]
fn test_notification_no_response() {
    setup_default_test_env();
    
    // Notification has no ID
    let notification_json = create_json_rpc_request("initialized", None, None);
    let request = create_mcp_request(notification_json);
    let response = spin_test_sdk::perform_request(request);
    let response_data = ResponseData::from_response(response);
    
    // Should return 200 with empty body for notifications
    assert_eq!(response_data.status, 200);
    assert!(response_data.body.is_empty());
}

#[spin_test]
fn test_prompts_list_method() {
    setup_default_test_env();
    
    let request_json = create_json_rpc_request("prompts/list", None, Some(serde_json::json!(1)));
    let request = create_mcp_request(request_json);
    let response = spin_test_sdk::perform_request(request);
    let response_data = ResponseData::from_response(response);
    
    assert_eq!(response_data.status, 200);
    let response_json = response_data.body_json().expect("Expected JSON response");
    assert_json_rpc_success(&response_json, Some(serde_json::json!(1)));
    
    // Should return empty prompts array
    assert!(response_json["result"]["prompts"].is_array());
    assert_eq!(response_json["result"]["prompts"].as_array().unwrap().len(), 0);
}

#[spin_test]
fn test_resources_list_method() {
    setup_default_test_env();
    
    let request_json = create_json_rpc_request("resources/list", None, Some(serde_json::json!(1)));
    let request = create_mcp_request(request_json);
    let response = spin_test_sdk::perform_request(request);
    let response_data = ResponseData::from_response(response);
    
    assert_eq!(response_data.status, 200);
    let response_json = response_data.body_json().expect("Expected JSON response");
    assert_json_rpc_success(&response_json, Some(serde_json::json!(1)));
    
    // Should return empty resources array
    assert!(response_json["result"]["resources"].is_array());
    assert_eq!(response_json["result"]["resources"].as_array().unwrap().len(), 0);
}

#[spin_test]
fn test_response_content_type() {
    setup_default_test_env();
    
    let request_json = create_json_rpc_request("ping", None, Some(serde_json::json!(1)));
    let request = create_mcp_request(request_json);
    let response = spin_test_sdk::perform_request(request);
    let response_data = ResponseData::from_response(response);
    
    // Verify content-type header
    let content_type = response_data.find_header("content-type")
        .map(|v| String::from_utf8_lossy(v).to_string());
    
    assert_eq!(content_type, Some("application/json".to_string()));
}

#[spin_test]
fn test_json_rpc_empty_request_body() {
    setup_default_test_env();
    
    let headers = http::types::Headers::new();
    headers.append("content-type", b"application/json").unwrap();
    
    let request = http::types::OutgoingRequest::new(headers);
    request.set_method(&http::types::Method::Post).unwrap();
    request.set_path_with_query(Some("/mcp")).unwrap();
    
    // Empty body
    let body = request.body().unwrap();
    body.write_bytes(b"");
    
    let response = spin_test_sdk::perform_request(request);
    let response_data = ResponseData::from_response(response);
    
    assert_eq!(response_data.status, 200);
    let response_json = response_data.body_json().expect("Expected JSON response");
    assert_json_rpc_error(&response_json, -32700, None); // Parse error
}

#[spin_test]
fn test_missing_method_field() {
    setup_default_test_env();
    
    let request_json = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1
        // Missing method
    });
    
    let request = create_mcp_request(request_json);
    let response = spin_test_sdk::perform_request(request);
    let response_data = ResponseData::from_response(response);
    
    assert_eq!(response_data.status, 200);
    let response_json = response_data.body_json().expect("Expected JSON response");
    assert_json_rpc_error(&response_json, -32700, None);
}