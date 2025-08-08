use spin_test_sdk::{
    bindings::fermyon::spin_test_virt::variables,
    spin_test,
};
use crate::{ResponseData, test_helpers::*};

#[spin_test]
fn test_full_mcp_session_flow() {
    variables::set("component_names", "calculator");
    variables::set("validate_arguments", "true");
    
    // Mock calculator component
    mock_tool_component("calculator", vec![
        ToolMetadata {
            name: "add".to_string(),
            title: Some("Addition".to_string()),
            description: Some("Add two numbers".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "a": {"type": "number"},
                    "b": {"type": "number"}
                },
                "required": ["a", "b"]
            }),
            output_schema: None,
            annotations: None,
            meta: None,
        },
        ToolMetadata {
            name: "subtract".to_string(),
            title: Some("Subtraction".to_string()),
            description: Some("Subtract b from a".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "a": {"type": "number"},
                    "b": {"type": "number"}
                },
                "required": ["a", "b"]
            }),
            output_schema: None,
            annotations: None,
            meta: None,
        }
    ]);
    
    // Mock tool executions
    mock_tool_execution("calculator", "add", ToolResponse {
        content: vec![ToolContent::Text {
            text: "Result: 30.8".to_string(),
            annotations: None,
        }],
        structured_content: Some(serde_json::json!({
            "result": 30.8,
            "operation": "add"
        })),
        is_error: None,
    });
    
    mock_tool_execution("calculator", "subtract", ToolResponse {
        content: vec![ToolContent::Text {
            text: "Result: 58".to_string(),
            annotations: None,
        }],
        structured_content: Some(serde_json::json!({
            "result": 58,
            "operation": "subtract"
        })),
        is_error: None,
    });
    
    // Step 1: Initialize
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
        Some(serde_json::json!(1))
    );
    
    let request = create_mcp_request(init_request);
    let response = spin_test_sdk::perform_request(request);
    let response_data = ResponseData::from_response(response);
    
    assert_eq!(response_data.status, 200);
    let response_json = response_data.body_json().expect("Expected JSON response");
    assert_json_rpc_success(&response_json, Some(serde_json::json!(1)));
    assert_eq!(response_json["result"]["protocolVersion"], "2025-06-18");
    
    // Step 2: Send initialized notification
    let initialized_request = create_json_rpc_request("initialized", None, None);
    let request = create_mcp_request(initialized_request);
    let response = spin_test_sdk::perform_request(request);
    assert_eq!(response.status(), 200);
    
    // Step 3: List tools
    let list_request = create_json_rpc_request("tools/list", None, Some(serde_json::json!(2)));
    let request = create_mcp_request(list_request);
    let response = spin_test_sdk::perform_request(request);
    let response_data = ResponseData::from_response(response);
    
    assert_eq!(response_data.status, 200);
    let response_json = response_data.body_json().expect("Expected JSON response");
    let tools = response_json["result"]["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 2);
    
    // Step 4: Call add tool
    // Re-mock the component before the tool call (spin-test limitation)
    mock_tool_component("calculator", vec![
        ToolMetadata {
            name: "add".to_string(),
            title: Some("Addition".to_string()),
            description: Some("Add two numbers".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "a": {"type": "number"},
                    "b": {"type": "number"}
                },
                "required": ["a", "b"]
            }),
            output_schema: None,
            annotations: None,
            meta: None,
        },
        ToolMetadata {
            name: "subtract".to_string(),
            title: Some("Subtraction".to_string()),
            description: Some("Subtract b from a".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "a": {"type": "number"},
                    "b": {"type": "number"}
                },
                "required": ["a", "b"]
            }),
            output_schema: None,
            annotations: None,
            meta: None,
        }
    ]);
    
    let add_request = create_json_rpc_request(
        "tools/call",
        Some(serde_json::json!({
            "name": "calculator__add",
            "arguments": {
                "a": 10.5,
                "b": 20.3
            }
        })),
        Some(serde_json::json!(3))
    );
    
    let request = create_mcp_request(add_request);
    let response = spin_test_sdk::perform_request(request);
    let response_data = ResponseData::from_response(response);
    
    assert_eq!(response_data.status, 200);
    let response_json = response_data.body_json().expect("Expected JSON response");
    assert_json_rpc_success(&response_json, Some(serde_json::json!(3)));
    
    let content = &response_json["result"]["content"][0];
    assert_eq!(content["type"], "text");
    assert!(content["text"].as_str().unwrap().contains("30.8"));
    
    // Verify structured content
    assert_eq!(response_json["result"]["structuredContent"]["result"], 30.8);
    assert_eq!(response_json["result"]["structuredContent"]["operation"], "add");
    
    // Step 5: Call subtract tool
    // Re-mock the component again (spin-test limitation)
    mock_tool_component("calculator", vec![
        ToolMetadata {
            name: "add".to_string(),
            title: Some("Addition".to_string()),
            description: Some("Add two numbers".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "a": {"type": "number"},
                    "b": {"type": "number"}
                },
                "required": ["a", "b"]
            }),
            output_schema: None,
            annotations: None,
            meta: None,
        },
        ToolMetadata {
            name: "subtract".to_string(),
            title: Some("Subtraction".to_string()),
            description: Some("Subtract b from a".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "a": {"type": "number"},
                    "b": {"type": "number"}
                },
                "required": ["a", "b"]
            }),
            output_schema: None,
            annotations: None,
            meta: None,
        }
    ]);
    
    let subtract_request = create_json_rpc_request(
        "tools/call",
        Some(serde_json::json!({
            "name": "calculator__subtract",
            "arguments": {
                "a": 100,
                "b": 42
            }
        })),
        Some(serde_json::json!(4))
    );
    
    let request = create_mcp_request(subtract_request);
    let response = spin_test_sdk::perform_request(request);
    let response_data = ResponseData::from_response(response);
    
    assert_eq!(response_data.status, 200);
    let response_json = response_data.body_json().expect("Expected JSON response");
    assert_json_rpc_success(&response_json, Some(serde_json::json!(4)));
    assert_eq!(response_json["result"]["structuredContent"]["result"], 58);
}