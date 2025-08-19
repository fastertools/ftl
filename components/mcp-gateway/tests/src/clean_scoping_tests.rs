use crate::{test_helpers::*, ResponseData};
use spin_test_sdk::{bindings::fermyon::spin_test_virt::variables, spin_test};

#[spin_test]
fn test_unscoped_tools_have_prefixes() {
    variables::set("component_names", "calc,string");
    variables::set("validate_arguments", "false");

    // Mock components
    mock_tool_component(
        "calc",
        vec![ToolMetadata {
            name: "add".to_string(),
            title: Some("Add".to_string()),
            description: Some("Add numbers".to_string()),
            input_schema: serde_json::json!({"type": "object"}),
            output_schema: None,
            annotations: None,
            meta: None,
        }],
    );

    mock_tool_component(
        "string",
        vec![ToolMetadata {
            name: "concat".to_string(),
            title: Some("Concatenate".to_string()),
            description: Some("Concatenate strings".to_string()),
            input_schema: serde_json::json!({"type": "object"}),
            output_schema: None,
            annotations: None,
            meta: None,
        }],
    );

    // Test /mcp path returns all tools WITH prefixes
    let request_json = create_json_rpc_request("tools/list", None, Some(serde_json::json!(1)));
    let request = create_mcp_request(request_json);
    request.set_path_with_query(Some("/mcp")).unwrap();

    let response = spin_test_sdk::perform_request(request);
    let response_data = ResponseData::from_response(response);

    assert_eq!(response_data.status, 200);
    let response_json = response_data.body_json().expect("Expected JSON response");
    let tools = response_json["result"]["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 2);

    // Should have prefixed names
    let tool_names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
    assert!(tool_names.contains(&"calc__add"));
    assert!(tool_names.contains(&"string__concat"));
}

#[spin_test]
fn test_scoped_tools_no_prefixes() {
    variables::set("component_names", "calc,string");
    variables::set("validate_arguments", "false");

    // Mock components
    mock_tool_component(
        "calc",
        vec![
            ToolMetadata {
                name: "add".to_string(),
                title: Some("Add".to_string()),
                description: Some("Add numbers".to_string()),
                input_schema: serde_json::json!({"type": "object"}),
                output_schema: None,
                annotations: None,
                meta: None,
            },
            ToolMetadata {
                name: "subtract".to_string(),
                title: Some("Subtract".to_string()),
                description: Some("Subtract numbers".to_string()),
                input_schema: serde_json::json!({"type": "object"}),
                output_schema: None,
                annotations: None,
                meta: None,
            },
        ],
    );

    // Test /mcp/x/calc path returns tools WITHOUT prefixes
    let request_json = create_json_rpc_request("tools/list", None, Some(serde_json::json!(1)));
    let request = create_mcp_request(request_json);
    request.set_path_with_query(Some("/mcp/x/calc")).unwrap();

    let response = spin_test_sdk::perform_request(request);
    let response_data = ResponseData::from_response(response);

    assert_eq!(response_data.status, 200);
    let response_json = response_data.body_json().expect("Expected JSON response");
    let tools = response_json["result"]["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 2);

    // Should have unprefixed names when scoped
    let tool_names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
    assert!(tool_names.contains(&"add"));
    assert!(tool_names.contains(&"subtract"));
    assert!(!tool_names.iter().any(|n| n.contains("__")));
}

#[spin_test]
fn test_scoped_tool_call_unprefixed() {
    variables::set("component_names", "calc");
    variables::set("validate_arguments", "false");

    // Mock component
    mock_tool_component(
        "calc",
        vec![ToolMetadata {
            name: "add".to_string(),
            title: Some("Add".to_string()),
            description: Some("Add numbers".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "a": {"type": "number"},
                    "b": {"type": "number"}
                }
            }),
            output_schema: None,
            annotations: None,
            meta: None,
        }],
    );

    mock_tool_execution(
        "calc",
        "add",
        ToolResponse {
            content: vec![ToolContent::Text {
                text: "Result: 5".to_string(),
                annotations: None,
            }],
            structured_content: None,
            is_error: None,
        },
    );

    // Call tool with unprefixed name in scoped context
    let call_request = create_json_rpc_request(
        "tools/call",
        Some(serde_json::json!({
            "name": "add",  // No prefix!
            "arguments": {"a": 2, "b": 3}
        })),
        Some(serde_json::json!(1)),
    );

    let request = create_mcp_request(call_request);
    request.set_path_with_query(Some("/mcp/x/calc")).unwrap();

    let response = spin_test_sdk::perform_request(request);
    let response_data = ResponseData::from_response(response);

    assert_eq!(response_data.status, 200);
    let response_json = response_data.body_json().expect("Expected JSON response");
    assert!(response_json["result"].is_object());
    assert!(response_json["result"]["content"][0]["text"]
        .as_str()
        .unwrap()
        .contains("Result: 5"));
}

#[spin_test]
fn test_unscoped_tool_call_requires_prefix() {
    variables::set("component_names", "calc");
    variables::set("validate_arguments", "false");

    // Mock component
    mock_tool_component(
        "calc",
        vec![ToolMetadata {
            name: "add".to_string(),
            title: Some("Add".to_string()),
            description: Some("Add numbers".to_string()),
            input_schema: serde_json::json!({"type": "object"}),
            output_schema: None,
            annotations: None,
            meta: None,
        }],
    );

    mock_tool_execution(
        "calc",
        "add",
        ToolResponse {
            content: vec![ToolContent::Text {
                text: "Result: 5".to_string(),
                annotations: None,
            }],
            structured_content: None,
            is_error: None,
        },
    );

    // Try to call with unprefixed name at /mcp - should fail
    let call_request = create_json_rpc_request(
        "tools/call",
        Some(serde_json::json!({
            "name": "add",  // Missing prefix
            "arguments": {"a": 2, "b": 3}
        })),
        Some(serde_json::json!(1)),
    );

    let request = create_mcp_request(call_request);
    request.set_path_with_query(Some("/mcp")).unwrap();

    let response = spin_test_sdk::perform_request(request);
    let response_data = ResponseData::from_response(response);

    assert_eq!(response_data.status, 200);
    let response_json = response_data.body_json().expect("Expected JSON response");
    assert!(response_json["error"].is_object());
    assert!(response_json["error"]["message"]
        .as_str()
        .unwrap()
        .contains("Invalid tool name format"));
}

#[spin_test]
fn test_unscoped_tool_call_with_prefix() {
    variables::set("component_names", "calc");
    variables::set("validate_arguments", "false");

    // Mock component
    mock_tool_component(
        "calc",
        vec![ToolMetadata {
            name: "add".to_string(),
            title: Some("Add".to_string()),
            description: Some("Add numbers".to_string()),
            input_schema: serde_json::json!({"type": "object"}),
            output_schema: None,
            annotations: None,
            meta: None,
        }],
    );

    mock_tool_execution(
        "calc",
        "add",
        ToolResponse {
            content: vec![ToolContent::Text {
                text: "Result: 5".to_string(),
                annotations: None,
            }],
            structured_content: None,
            is_error: None,
        },
    );

    // Call with prefixed name at /mcp - should work
    let call_request = create_json_rpc_request(
        "tools/call",
        Some(serde_json::json!({
            "name": "calc__add",  // Correct prefix
            "arguments": {"a": 2, "b": 3}
        })),
        Some(serde_json::json!(1)),
    );

    let request = create_mcp_request(call_request);
    request.set_path_with_query(Some("/mcp")).unwrap();

    let response = spin_test_sdk::perform_request(request);
    let response_data = ResponseData::from_response(response);

    assert_eq!(response_data.status, 200);
    let response_json = response_data.body_json().expect("Expected JSON response");
    assert!(response_json["result"].is_object());
}

#[spin_test]
fn test_individual_tool_scope_rejected() {
    variables::set("component_names", "calc");
    variables::set("validate_arguments", "false");

    // Mock component (not needed for 404 test but included for completeness)
    mock_tool_component(
        "calc",
        vec![ToolMetadata {
            name: "add".to_string(),
            title: Some("Add".to_string()),
            description: Some("Add numbers".to_string()),
            input_schema: serde_json::json!({"type": "object"}),
            output_schema: None,
            annotations: None,
            meta: None,
        }],
    );

    // Test that /mcp/x/calc/add path returns 404 (invalid - too many segments)
    let request_json = create_json_rpc_request("tools/list", None, Some(serde_json::json!(1)));
    let request = create_mcp_request(request_json);
    request
        .set_path_with_query(Some("/mcp/x/calc/add"))
        .unwrap();

    let response = spin_test_sdk::perform_request(request);
    let response_data = ResponseData::from_response(response);

    // Should return 404 for invalid path
    assert_eq!(response_data.status, 404);
    let response_json = response_data.body_json().expect("Expected JSON response");
    assert!(response_json["error"]
        .as_str()
        .unwrap()
        .contains("Invalid path"));
}

#[spin_test]
fn test_cross_component_call_blocked() {
    variables::set("component_names", "calc,string");
    variables::set("validate_arguments", "false");

    // Mock components
    mock_tool_component("calc", vec![]);
    mock_tool_component(
        "string",
        vec![ToolMetadata {
            name: "concat".to_string(),
            title: Some("Concatenate".to_string()),
            description: Some("Concatenate strings".to_string()),
            input_schema: serde_json::json!({"type": "object"}),
            output_schema: None,
            annotations: None,
            meta: None,
        }],
    );

    // Try to call string tool while scoped to calc
    let call_request = create_json_rpc_request(
        "tools/call",
        Some(serde_json::json!({
            "name": "concat",  // This is a string tool
            "arguments": {}
        })),
        Some(serde_json::json!(1)),
    );

    let request = create_mcp_request(call_request);
    request.set_path_with_query(Some("/mcp/x/calc")).unwrap(); // But we're scoped to calc

    let response = spin_test_sdk::perform_request(request);
    let response_data = ResponseData::from_response(response);

    assert_eq!(response_data.status, 200);
    let response_json = response_data.body_json().expect("Expected JSON response");
    assert!(response_json["error"].is_object());
    // The error might be "unknown tool" since concat doesn't exist in calc component
}

#[spin_test]
fn test_get_method_rejected() {
    use spin_test_sdk::bindings::wasi::http::types;

    variables::set("component_names", "calc");
    variables::set("validate_arguments", "false");

    let headers = types::Headers::new();
    headers.append("content-type", b"application/json").unwrap();

    let request = types::OutgoingRequest::new(headers);
    request.set_method(&types::Method::Get).unwrap();
    request.set_path_with_query(Some("/mcp")).unwrap();

    let response = spin_test_sdk::perform_request(request);
    let response_data = ResponseData::from_response(response);

    // Should return 405 Method Not Allowed
    assert_eq!(response_data.status, 405);
}

#[spin_test]
fn test_invalid_path_outside_mcp() {
    variables::set("component_names", "calc");
    variables::set("validate_arguments", "false");

    let request_json = create_json_rpc_request("tools/list", None, Some(serde_json::json!(1)));
    let request = create_mcp_request(request_json);
    request.set_path_with_query(Some("/api/tools")).unwrap();

    let response = spin_test_sdk::perform_request(request);
    let response_data = ResponseData::from_response(response);

    // Should return 404 for paths not under /mcp
    assert_eq!(response_data.status, 404);
    let response_json = response_data.body_json().expect("Expected JSON response");
    assert!(response_json["error"]
        .as_str()
        .unwrap()
        .contains("must start with /mcp"));
}

#[spin_test]
fn test_readonly_header_blocks_execution() {
    use spin_test_sdk::bindings::wasi::http::types;

    variables::set("component_names", "calc");
    variables::set("validate_arguments", "false");

    // Mock component
    mock_tool_component(
        "calc",
        vec![ToolMetadata {
            name: "add".to_string(),
            title: Some("Add".to_string()),
            description: Some("Add numbers".to_string()),
            input_schema: serde_json::json!({"type": "object"}),
            output_schema: None,
            annotations: None,
            meta: None,
        }],
    );

    // Try to call tool with X-MCP-Readonly header set to true
    let call_request = create_json_rpc_request(
        "tools/call",
        Some(serde_json::json!({
            "name": "calc__add",
            "arguments": {"a": 2, "b": 3}
        })),
        Some(serde_json::json!(1)),
    );

    let headers = types::Headers::new();
    headers.append("content-type", b"application/json").unwrap();
    headers.append("x-mcp-readonly", b"true").unwrap();

    let request = types::OutgoingRequest::new(headers);
    request.set_method(&types::Method::Post).unwrap();
    request.set_path_with_query(Some("/mcp")).unwrap();

    let body = request.body().unwrap();
    body.write_bytes(serde_json::to_vec(&call_request).unwrap().as_slice());

    let response = spin_test_sdk::perform_request(request);
    let response_data = ResponseData::from_response(response);

    // Should return error due to readonly mode
    assert_eq!(response_data.status, 200); // JSON-RPC error, not HTTP error
    let response_json = response_data.body_json().expect("Expected JSON response");
    assert!(response_json["error"].is_object());
    assert!(response_json["error"]["message"]
        .as_str()
        .unwrap()
        .contains("readonly mode"));
}

#[spin_test]
fn test_toolsets_header_filters_components() {
    use spin_test_sdk::bindings::wasi::http::types;

    variables::set("component_names", "calc,string,math");
    variables::set("validate_arguments", "false");

    // Mock calc component
    mock_tool_component(
        "calc",
        vec![ToolMetadata {
            name: "add".to_string(),
            title: Some("Add".to_string()),
            description: Some("Add numbers".to_string()),
            input_schema: serde_json::json!({"type": "object"}),
            output_schema: None,
            annotations: None,
            meta: None,
        }],
    );

    // Test with X-MCP-Toolsets header filtering to just calc
    let request_json = create_json_rpc_request("tools/list", None, Some(serde_json::json!(1)));

    let headers = types::Headers::new();
    headers.append("content-type", b"application/json").unwrap();
    headers.append("x-mcp-toolsets", b"calc").unwrap();

    let request = types::OutgoingRequest::new(headers);
    request.set_method(&types::Method::Post).unwrap();
    request.set_path_with_query(Some("/mcp")).unwrap();

    let body = request.body().unwrap();
    body.write_bytes(serde_json::to_vec(&request_json).unwrap().as_slice());

    let response = spin_test_sdk::perform_request(request);
    let response_data = ResponseData::from_response(response);

    assert_eq!(response_data.status, 200);
    let response_json = response_data.body_json().expect("Expected JSON response");
    let tools = response_json["result"]["tools"].as_array().unwrap();

    // Should only return calc tools due to X-MCP-Toolsets header
    assert!(
        tools.len() >= 1,
        "Should have at least calc component tools"
    );

    // Verify the tools we get are from calc (prefixed since we're at /mcp)
    for tool in tools {
        let name = tool["name"].as_str().unwrap();
        assert!(
            name.starts_with("calc__"),
            "Tool {} should be from calc component",
            name
        );
    }
}

#[spin_test]
fn test_toolsets_header_multiple_components() {
    use spin_test_sdk::bindings::wasi::http::types;

    variables::set("component_names", "calc,string,math");
    variables::set("validate_arguments", "false");

    mock_tool_component(
        "calc",
        vec![ToolMetadata {
            name: "add".to_string(),
            title: Some("Add".to_string()),
            description: Some("Add numbers".to_string()),
            input_schema: serde_json::json!({"type": "object"}),
            output_schema: None,
            annotations: None,
            meta: None,
        }],
    );

    // First request - get calc tools
    let request_json = create_json_rpc_request("tools/list", None, Some(serde_json::json!(1)));

    let headers = types::Headers::new();
    headers.append("content-type", b"application/json").unwrap();
    headers.append("x-mcp-toolsets", b"calc").unwrap();

    let request = types::OutgoingRequest::new(headers);
    request.set_method(&types::Method::Post).unwrap();
    request.set_path_with_query(Some("/mcp")).unwrap();

    let body = request.body().unwrap();
    body.write_bytes(serde_json::to_vec(&request_json).unwrap().as_slice());

    let response = spin_test_sdk::perform_request(request);
    let response_data = ResponseData::from_response(response);

    assert_eq!(response_data.status, 200);
    let response_json = response_data.body_json().expect("Expected JSON response");
    let tools = response_json["result"]["tools"].as_array().unwrap();

    // Verify we only get calc tools
    for tool in tools {
        let name = tool["name"].as_str().unwrap();
        assert!(
            name.starts_with("calc__"),
            "Should only have calc tools when filtered"
        );
    }
}

#[spin_test]
fn test_toolsets_header_with_readonly() {
    use spin_test_sdk::bindings::wasi::http::types;

    variables::set("component_names", "calc");
    variables::set("validate_arguments", "false");

    mock_tool_component(
        "calc",
        vec![ToolMetadata {
            name: "add".to_string(),
            title: Some("Add".to_string()),
            description: Some("Add numbers".to_string()),
            input_schema: serde_json::json!({"type": "object"}),
            output_schema: None,
            annotations: None,
            meta: None,
        }],
    );

    // Try to call tool with both X-MCP-Toolsets and X-MCP-Readonly headers
    let call_request = create_json_rpc_request(
        "tools/call",
        Some(serde_json::json!({
            "name": "calc__add",
            "arguments": {"a": 2, "b": 3}
        })),
        Some(serde_json::json!(1)),
    );

    let headers = types::Headers::new();
    headers.append("content-type", b"application/json").unwrap();
    headers.append("x-mcp-toolsets", b"calc").unwrap();
    headers.append("x-mcp-readonly", b"true").unwrap();

    let request = types::OutgoingRequest::new(headers);
    request.set_method(&types::Method::Post).unwrap();
    request.set_path_with_query(Some("/mcp")).unwrap();

    let body = request.body().unwrap();
    body.write_bytes(serde_json::to_vec(&call_request).unwrap().as_slice());

    let response = spin_test_sdk::perform_request(request);
    let response_data = ResponseData::from_response(response);

    // Should fail due to readonly mode even though calc is in allowed toolsets
    assert_eq!(response_data.status, 200);
    let response_json = response_data.body_json().expect("Expected JSON response");
    assert!(response_json["error"].is_object());
    assert!(response_json["error"]["message"]
        .as_str()
        .unwrap()
        .contains("readonly mode"));
}
