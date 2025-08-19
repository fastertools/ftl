use serde_json::Value;
use spin_test_sdk::bindings::wasi::http;

// Helper functions for tests

// JSON-RPC request builder
pub fn create_json_rpc_request(method: &str, params: Option<Value>, id: Option<Value>) -> Value {
    let mut request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": method
    });

    if let Some(params_value) = params {
        request["params"] = params_value;
    }

    if let Some(id_value) = id {
        request["id"] = id_value;
    }

    request
}

// Create HTTP request with JSON-RPC body
pub fn create_mcp_request(json_rpc: Value) -> http::types::OutgoingRequest {
    let headers = http::types::Headers::new();
    headers.append("content-type", b"application/json").unwrap();

    let request = http::types::OutgoingRequest::new(headers);
    request.set_method(&http::types::Method::Post).unwrap();
    request.set_path_with_query(Some("/mcp")).unwrap();

    let request_body_bytes = serde_json::to_vec(&json_rpc).unwrap();
    let body = request.body().unwrap();
    body.write_bytes(&request_body_bytes);

    request
}

// Re-export types from ftl-sdk for convenience
pub use ftl_sdk::{ToolContent, ToolMetadata, ToolResponse};

// Setup default test environment
pub fn setup_default_test_env() {
    spin_test_sdk::bindings::fermyon::spin_test_virt::variables::set(
        "component_names",
        "echo,calculator",
    );
    spin_test_sdk::bindings::fermyon::spin_test_virt::variables::set("validate_arguments", "true");

    // Mock the default components to prevent errors
    mock_tool_component("echo", vec![]);
    mock_tool_component("calculator", vec![]);
}

// Verify JSON-RPC error response
pub fn assert_json_rpc_error(response_json: &Value, expected_code: i32, id: Option<Value>) {
    assert_eq!(response_json["jsonrpc"], "2.0");
    assert!(response_json["error"].is_object());
    assert_eq!(response_json["error"]["code"], expected_code);
    assert!(response_json["error"]["message"].is_string());

    if let Some(expected_id) = id {
        assert_eq!(response_json["id"], expected_id);
    } else {
        assert!(response_json.get("id").is_none() || response_json["id"].is_null());
    }
}

// Verify successful JSON-RPC response
pub fn assert_json_rpc_success(response_json: &Value, id: Option<Value>) {
    assert_eq!(response_json["jsonrpc"], "2.0");
    assert!(response_json["result"].is_object() || response_json["result"].is_array());
    assert!(response_json.get("error").is_none());

    if let Some(expected_id) = id {
        assert_eq!(response_json["id"], expected_id);
    } else {
        assert!(response_json.get("id").is_none() || response_json["id"].is_null());
    }
}

// Mock a tool component that returns metadata
pub fn mock_tool_component(component_name: &str, tools: Vec<ToolMetadata>) {
    use spin_test_sdk::bindings::fermyon::spin_wasi_virt::http_handler;

    // Create a function to generate fresh responses
    let create_response = || {
        let headers = http::types::Headers::new();
        headers.append("content-type", b"application/json").unwrap();

        let response = http::types::OutgoingResponse::new(headers);
        response.set_status_code(200).unwrap();

        let body = response.body().unwrap();
        body.write_bytes(&serde_json::to_vec(&tools).unwrap());

        response
    };

    // The gateway makes GET requests to http://{component}.spin.internal/
    // But the URL might be normalized by the test framework, so mock both versions
    let url_with_slash = format!("http://{component_name}.spin.internal/");
    let url_without_slash = format!("http://{component_name}.spin.internal");

    http_handler::set_response(
        &url_with_slash,
        http_handler::ResponseHandler::Response(create_response()),
    );

    http_handler::set_response(
        &url_without_slash,
        http_handler::ResponseHandler::Response(create_response()),
    );
}

// Mock a tool execution response
pub fn mock_tool_execution(component_name: &str, tool_name: &str, response_data: ToolResponse) {
    use spin_test_sdk::bindings::fermyon::spin_wasi_virt::http_handler;

    let headers = http::types::Headers::new();
    headers.append("content-type", b"application/json").unwrap();

    let response = http::types::OutgoingResponse::new(headers);
    response.set_status_code(200).unwrap();

    let body = response.body().unwrap();
    body.write_bytes(&serde_json::to_vec(&response_data).unwrap());

    let url = format!("http://{component_name}.spin.internal/{tool_name}");
    http_handler::set_response(&url, http_handler::ResponseHandler::Response(response));
}
