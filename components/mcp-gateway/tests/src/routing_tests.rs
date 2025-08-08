use spin_test_sdk::{
    bindings::{
        fermyon::spin_test_virt::variables,
        fermyon::spin_wasi_virt::http_handler,
        wasi::http
    },
    spin_test,
};
use crate::{ResponseData, test_helpers::*};

#[spin_test]
fn test_tool_routing_snake_to_kebab_case() {
    // Setup with a tool component that has underscores
    variables::set("component_names", "echo_tool,math_calculator");
    variables::set("validate_arguments", "false");
    
    // The gateway will convert echo_tool to echo-tool when making requests
    // We need to mock both the snake_case and kebab-case versions
    
    // Mock the echo-tool component (note kebab-case in URL)
    mock_tool_component("echo-tool", vec![
        ToolMetadata {
            name: "echo_message".to_string(),
            title: Some("Echo Message".to_string()),
            description: Some("Echoes back the message".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "message": {
                        "type": "string",
                        "description": "Message to echo"
                    }
                },
                "required": ["message"]
            }),
            output_schema: None,
            annotations: None,
            meta: None,
        }
    ]);
    
    // Mock tool execution
    mock_tool_execution("echo-tool", "echo_message", ToolResponse {
        content: vec![ToolContent::Text {
            text: "Echo: test message".to_string(),
            annotations: None,
        }],
        structured_content: None,
        is_error: None,
    });
    
    // Mock empty response for math-calculator
    mock_tool_component("math-calculator", vec![]);
    
    // Test listing tools
    let list_request = create_json_rpc_request(
        "tools/list",
        None,
        Some(serde_json::json!(1))
    );
    
    let request = create_mcp_request(list_request);
    let response = spin_test_sdk::perform_request(request);
    let response_data = ResponseData::from_response(response);
    
    assert_eq!(response_data.status, 200);
    let response_json = response_data.body_json().expect("Expected JSON response");
    assert_json_rpc_success(&response_json, Some(serde_json::json!(1)));
    
    // Should find the echo_message tool
    let tools = response_json["result"]["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0]["name"], "echo_message");
    
    // Test calling the tool
    // Re-mock the component before the tool call (spin-test limitation)
    mock_tool_component("echo-tool", vec![
        ToolMetadata {
            name: "echo_message".to_string(),
            title: Some("Echo Message".to_string()),
            description: Some("Echoes back the message".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "message": {
                        "type": "string",
                        "description": "Message to echo"
                    }
                },
                "required": ["message"]
            }),
            output_schema: None,
            annotations: None,
            meta: None,
        }
    ]);
    
    let call_request = create_json_rpc_request(
        "tools/call",
        Some(serde_json::json!({
            "name": "echo_message",
            "arguments": {
                "message": "test message"
            }
        })),
        Some(serde_json::json!(2))
    );
    
    let request = create_mcp_request(call_request);
    let response = spin_test_sdk::perform_request(request);
    let response_data = ResponseData::from_response(response);
    
    assert_eq!(response_data.status, 200);
    let response_json = response_data.body_json().expect("Expected JSON response");
    assert_json_rpc_success(&response_json, Some(serde_json::json!(2)));
    
    // Verify the response content
    let content = &response_json["result"]["content"][0];
    assert_eq!(content["type"], "text");
    assert_eq!(content["text"], "Echo: test message");
}

#[spin_test]
fn test_tool_not_found() {
    setup_default_test_env();
    
    // Mock empty tool lists for configured components
    mock_tool_component("echo", vec![]);
    mock_tool_component("calculator", vec![]);
    
    let call_request = create_json_rpc_request(
        "tools/call",
        Some(serde_json::json!({
            "name": "non_existent_tool",
            "arguments": {}
        })),
        Some(serde_json::json!(1))
    );
    
    let request = create_mcp_request(call_request);
    let response = spin_test_sdk::perform_request(request);
    let response_data = ResponseData::from_response(response);
    
    assert_eq!(response_data.status, 200);
    let response_json = response_data.body_json().expect("Expected JSON response");
    assert_json_rpc_error(&response_json, -32602, Some(serde_json::json!(1)));
    assert!(response_json["error"]["message"].as_str().unwrap().contains("Unknown tool"));
}

#[spin_test]
fn test_component_failure_handling() {
    variables::set("component_names", "broken_tool");
    variables::set("validate_arguments", "false");
    
    // Mock a component that returns 500 error
    let url = "http://broken-tool.spin.internal/";
    let response = http::types::OutgoingResponse::new(http::types::Headers::new());
    response.set_status_code(500).unwrap();
    let body = response.body().unwrap();
    body.write_bytes(b"Internal component error");
    
    http_handler::set_response(
        url,
        http_handler::ResponseHandler::Response(response),
    );
    
    // Try to list tools - should handle the error gracefully
    let list_request = create_json_rpc_request(
        "tools/list",
        None,
        Some(serde_json::json!(1))
    );
    
    let request = create_mcp_request(list_request);
    let response = spin_test_sdk::perform_request(request);
    let response_data = ResponseData::from_response(response);
    
    assert_eq!(response_data.status, 200);
    let response_json = response_data.body_json().expect("Expected JSON response");
    assert_json_rpc_success(&response_json, Some(serde_json::json!(1)));
    
    // Should return empty tools list when component fails
    let tools = response_json["result"]["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 0);
}

#[spin_test]
fn test_parallel_tool_discovery() {
    // Test that multiple components are queried in parallel
    variables::set("component_names", "tool1,tool2,tool3");
    variables::set("validate_arguments", "false");
    
    // Mock three different tool components
    mock_tool_component("tool1", vec![
        ToolMetadata {
            name: "tool_1".to_string(),
            title: Some("Tool 1".to_string()),
            description: Some("Description for tool 1".to_string()),
            input_schema: serde_json::json!({"type": "object"}),
            output_schema: None,
            annotations: None,
            meta: None,
        }
    ]);
    
    mock_tool_component("tool2", vec![
        ToolMetadata {
            name: "tool_2".to_string(),
            title: Some("Tool 2".to_string()),
            description: Some("Description for tool 2".to_string()),
            input_schema: serde_json::json!({"type": "object"}),
            output_schema: None,
            annotations: None,
            meta: None,
        }
    ]);
    
    mock_tool_component("tool3", vec![
        ToolMetadata {
            name: "tool_3".to_string(),
            title: Some("Tool 3".to_string()),
            description: Some("Description for tool 3".to_string()),
            input_schema: serde_json::json!({"type": "object"}),
            output_schema: None,
            annotations: None,
            meta: None,
        }
    ]);
    
    let list_request = create_json_rpc_request(
        "tools/list",
        None,
        Some(serde_json::json!(1))
    );
    
    let request = create_mcp_request(list_request);
    let response = spin_test_sdk::perform_request(request);
    let response_data = ResponseData::from_response(response);
    
    assert_eq!(response_data.status, 200);
    let response_json = response_data.body_json().expect("Expected JSON response");
    let tools = response_json["result"]["tools"].as_array().unwrap();
    
    // Should have discovered tools from all components
    assert_eq!(tools.len(), 3);
    
    // Verify all tools are present
    let tool_names: Vec<&str> = tools.iter()
        .map(|t| t["name"].as_str().unwrap())
        .collect();
    
    assert!(tool_names.contains(&"tool_1"));
    assert!(tool_names.contains(&"tool_2"));
    assert!(tool_names.contains(&"tool_3"));
}