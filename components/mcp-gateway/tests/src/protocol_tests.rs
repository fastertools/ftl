use crate::{test_helpers::*, ResponseData};
use spin_test_sdk::spin_test;

#[spin_test]
fn test_initialize_protocol_v1() {
    setup_default_test_env();

    let request_json = create_json_rpc_request(
        "initialize",
        Some(serde_json::json!({
            "protocolVersion": "2025-06-18",
            "capabilities": {},
            "clientInfo": {
                "name": "test-client",
                "version": "1.0.0"
            }
        })),
        Some(serde_json::json!(1)),
    );

    let request = create_mcp_request(request_json);
    let response = spin_test_sdk::perform_request(request);
    let response_data = ResponseData::from_response(response);

    assert_eq!(response_data.status, 200);

    let response_json = response_data.body_json().expect("Expected JSON response");
    assert_json_rpc_success(&response_json, Some(serde_json::json!(1)));

    // Verify response structure
    let result = &response_json["result"];
    assert_eq!(result["protocolVersion"], "2025-06-18");
    assert!(result["capabilities"].is_object());
    assert!(result["serverInfo"].is_object());
    assert_eq!(result["serverInfo"]["name"], "ftl-mcp-gateway");
    assert!(result["serverInfo"]["version"].is_string());
}

#[spin_test]
fn test_initialize_unsupported_protocol_version() {
    setup_default_test_env();

    let request_json = create_json_rpc_request(
        "initialize",
        Some(serde_json::json!({
            "protocolVersion": "1.0.0", // Invalid version
            "capabilities": {},
            "clientInfo": {
                "name": "test-client",
                "version": "1.0.0"
            }
        })),
        Some(serde_json::json!(1)),
    );

    let request = create_mcp_request(request_json);
    let response = spin_test_sdk::perform_request(request);
    let response_data = ResponseData::from_response(response);

    assert_eq!(response_data.status, 200);

    let response_json = response_data.body_json().expect("Expected JSON response");
    assert_json_rpc_error(&response_json, -32602, Some(serde_json::json!(1)));
    assert!(response_json["error"]["message"]
        .as_str()
        .unwrap()
        .contains("Invalid initialize parameters"));
}

#[spin_test]
fn test_initialize_missing_params() {
    setup_default_test_env();

    let request_json = create_json_rpc_request(
        "initialize",
        None, // Missing params
        Some(serde_json::json!(1)),
    );

    let request = create_mcp_request(request_json);
    let response = spin_test_sdk::perform_request(request);
    let response_data = ResponseData::from_response(response);

    assert_eq!(response_data.status, 200);

    let response_json = response_data.body_json().expect("Expected JSON response");
    assert_json_rpc_error(&response_json, -32602, Some(serde_json::json!(1)));
}

#[spin_test]
fn test_initialized_notification() {
    setup_default_test_env();

    // First initialize
    let init_request = create_json_rpc_request(
        "initialize",
        Some(serde_json::json!({
            "protocolVersion": "2025-06-18",
            "capabilities": {},
            "clientInfo": {
                "name": "test-client",
                "version": "1.0.0"
            }
        })),
        Some(serde_json::json!(1)),
    );

    let request = create_mcp_request(init_request);
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 200);

    // Send initialized notification (no id, no response expected)
    let initialized_request = create_json_rpc_request(
        "initialized",
        None,
        None, // No ID for notifications
    );

    let request = create_mcp_request(initialized_request);
    let response = spin_test_sdk::perform_request(request);
    let response_data = ResponseData::from_response(response);

    // Should return empty response for notification
    assert_eq!(response_data.status, 200);
    assert!(response_data.body.is_empty());
}

#[spin_test]
fn test_server_capabilities() {
    setup_default_test_env();

    let request_json = create_json_rpc_request(
        "initialize",
        Some(serde_json::json!({
            "protocolVersion": "2025-06-18",
            "capabilities": {},
            "clientInfo": {
                "name": "test-client",
                "version": "1.0.0"
            }
        })),
        Some(serde_json::json!(1)),
    );

    let request = create_mcp_request(request_json);
    let response = spin_test_sdk::perform_request(request);
    let response_data = ResponseData::from_response(response);

    let response_json = response_data.body_json().expect("Expected JSON response");
    let capabilities = &response_json["result"]["capabilities"];

    // Verify all required capabilities are present
    assert!(capabilities["tools"].is_object());
    assert_eq!(capabilities["tools"]["listChanged"], true);

    assert!(capabilities["resources"].is_object());
    assert_eq!(capabilities["resources"]["subscribe"], false);
    assert_eq!(capabilities["resources"]["listChanged"], false);

    assert!(capabilities["prompts"].is_object());
    assert_eq!(capabilities["prompts"]["listChanged"], false);

    assert!(capabilities["experimental_capabilities"].is_object());
    assert!(capabilities["experimental_capabilities"]["logging"].is_object());
}

#[spin_test]
fn test_instructions_in_initialize_response() {
    setup_default_test_env();

    let request_json = create_json_rpc_request(
        "initialize",
        Some(serde_json::json!({
            "protocolVersion": "2025-06-18",
            "capabilities": {},
            "clientInfo": {
                "name": "test-client",
                "version": "1.0.0"
            }
        })),
        Some(serde_json::json!(1)),
    );

    let request = create_mcp_request(request_json);
    let response = spin_test_sdk::perform_request(request);
    let response_data = ResponseData::from_response(response);

    let response_json = response_data.body_json().expect("Expected JSON response");
    let result = &response_json["result"];

    // Verify instructions are present
    assert!(result["instructions"].is_string());
    let instructions = result["instructions"].as_str().unwrap();
    assert!(instructions.contains("WebAssembly components"));
}
