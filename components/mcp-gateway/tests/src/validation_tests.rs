use spin_test_sdk::{
    bindings::fermyon::spin_test_virt::variables,
    spin_test,
};
use crate::{ResponseData, test_helpers::*};

#[spin_test]
fn test_argument_validation_enabled() {
    variables::set("tool_components", "strict_tool");
    variables::set("validate_arguments", "true");
    
    // Mock tool with strict schema
    mock_tool_component("strict-tool", vec![
        ToolMetadata {
            name: "strict_add".to_string(),
            title: Some("Strict Addition".to_string()),
            description: Some("Adds two numbers with strict validation".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "a": {
                        "type": "number",
                        "description": "First number"
                    },
                    "b": {
                        "type": "number",
                        "description": "Second number"
                    }
                },
                "required": ["a", "b"],
                "additionalProperties": false
            }),
            output_schema: None,
            annotations: None,
            meta: None,
        }
    ]);
    
    // Mock successful tool execution
    mock_tool_execution("strict-tool", "strict_add", ToolResponse {
        content: vec![ToolContent::Text {
            text: "Result: 8".to_string(),
            annotations: None,
        }],
        structured_content: None,
        is_error: None,
    });
    
    // Test with valid arguments
    let valid_request = create_json_rpc_request(
        "tools/call",
        Some(serde_json::json!({
            "name": "strict_add",
            "arguments": {
                "a": 5,
                "b": 3
            }
        })),
        Some(serde_json::json!(1))
    );
    
    let request = create_mcp_request(valid_request);
    let response = spin_test_sdk::perform_request(request);
    let response_data = ResponseData::from_response(response);
    
    // Should pass validation
    assert_eq!(response_data.status, 200);
    let response_json = response_data.body_json().expect("Expected JSON response");
    assert_json_rpc_success(&response_json, Some(serde_json::json!(1)));
    
    // Test with invalid arguments (missing required field)
    let invalid_request = create_json_rpc_request(
        "tools/call",
        Some(serde_json::json!({
            "name": "strict_add",
            "arguments": {
                "a": 5
                // Missing "b"
            }
        })),
        Some(serde_json::json!(2))
    );
    
    // Re-mock the component before the second request (spin-test limitation)
    mock_tool_component("strict-tool", vec![
        ToolMetadata {
            name: "strict_add".to_string(),
            title: Some("Strict Addition".to_string()),
            description: Some("Adds two numbers with strict validation".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "a": {
                        "type": "number",
                        "description": "First number"
                    },
                    "b": {
                        "type": "number",
                        "description": "Second number"
                    }
                },
                "required": ["a", "b"],
                "additionalProperties": false
            }),
            output_schema: None,
            annotations: None,
            meta: None,
        }
    ]);
    
    let request = create_mcp_request(invalid_request);
    let response = spin_test_sdk::perform_request(request);
    let response_data = ResponseData::from_response(response);
    
    assert_eq!(response_data.status, 200);
    let response_json = response_data.body_json().expect("Expected JSON response");
    assert_json_rpc_error(&response_json, -32602, Some(serde_json::json!(2)));
    assert!(response_json["error"]["message"].as_str().unwrap().contains("Invalid params"));
}

#[spin_test]
fn test_argument_validation_disabled() {
    variables::set("tool_components", "lenient_tool");
    variables::set("validate_arguments", "false");
    
    // Mock tool with schema
    mock_tool_component("lenient-tool", vec![
        ToolMetadata {
            name: "lenient_process".to_string(),
            title: Some("Lenient Process".to_string()),
            description: Some("Process with lenient validation".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "required_field": {
                        "type": "string"
                    }
                },
                "required": ["required_field"]
            }),
            output_schema: None,
            annotations: None,
            meta: None,
        }
    ]);
    
    // Mock tool execution that accepts any input
    mock_tool_execution("lenient-tool", "lenient_process", ToolResponse {
        content: vec![ToolContent::Text {
            text: "Processed without validation".to_string(),
            annotations: None,
        }],
        structured_content: None,
        is_error: None,
    });
    
    // Send request with invalid arguments (missing required field)
    let request_json = create_json_rpc_request(
        "tools/call",
        Some(serde_json::json!({
            "name": "lenient_process",
            "arguments": {
                "some_other_field": "value"
                // Missing required_field
            }
        })),
        Some(serde_json::json!(1))
    );
    
    // Re-mock the component before the request (spin-test limitation)
    mock_tool_component("lenient-tool", vec![
        ToolMetadata {
            name: "lenient_process".to_string(),
            title: Some("Lenient Process".to_string()),
            description: Some("Process with lenient validation".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "required_field": {
                        "type": "string"
                    }
                },
                "required": ["required_field"]
            }),
            output_schema: None,
            annotations: None,
            meta: None,
        }
    ]);
    
    let request = create_mcp_request(request_json);
    let response = spin_test_sdk::perform_request(request);
    let response_data = ResponseData::from_response(response);
    
    // Should succeed because validation is disabled
    assert_eq!(response_data.status, 200);
    let response_json = response_data.body_json().expect("Expected JSON response");
    assert_json_rpc_success(&response_json, Some(serde_json::json!(1)));
}

#[spin_test]
fn test_missing_arguments_defaults_to_empty_object() {
    variables::set("tool_components", "optional_tool");
    variables::set("validate_arguments", "true");
    
    // Mock tool that accepts optional arguments
    mock_tool_component("optional-tool", vec![
        ToolMetadata {
            name: "optional_params".to_string(),
            title: Some("Optional Parameters".to_string()),
            description: Some("Tool with all optional parameters".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "optional_field": {"type": "string"}
                }
                // No required fields
            }),
            output_schema: None,
            annotations: None,
            meta: None,
        }
    ]);
    
    mock_tool_execution("optional-tool", "optional_params", ToolResponse {
        content: vec![ToolContent::Text {
            text: "Executed with default empty object".to_string(),
            annotations: None,
        }],
        structured_content: None,
        is_error: None,
    });
    
    // Call without arguments field
    let request_json = create_json_rpc_request(
        "tools/call",
        Some(serde_json::json!({
            "name": "optional_params"
            // No arguments field
        })),
        Some(serde_json::json!(1))
    );
    
    let request = create_mcp_request(request_json);
    let response = spin_test_sdk::perform_request(request);
    let response_data = ResponseData::from_response(response);
    
    // Should succeed with empty object as default
    assert_eq!(response_data.status, 200);
    let response_json = response_data.body_json().expect("Expected JSON response");
    assert_json_rpc_success(&response_json, Some(serde_json::json!(1)));
}