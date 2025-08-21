use crate::{test_helpers::*, ResponseData};
use spin_test_sdk::{bindings::fermyon::spin_test_virt::variables, spin_test};

#[spin_test]
fn test_list_tools_empty() {
    variables::set("component_names", "empty_component");
    variables::set("validate_arguments", "true");

    // Mock component that returns empty tools array
    mock_tool_component("empty-component", vec![]);

    let request_json = create_json_rpc_request("tools/list", None, Some(serde_json::json!(1)));
    let request = create_mcp_request(request_json);
    let response = spin_test_sdk::perform_request(request);
    let response_data = ResponseData::from_response(response);

    assert_eq!(response_data.status, 200);
    let response_json = response_data.body_json().expect("Expected JSON response");
    assert_json_rpc_success(&response_json, Some(serde_json::json!(1)));

    let tools = response_json["result"]["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 0);
}

#[spin_test]
fn test_list_tools_multiple_components() {
    variables::set("component_names", "math,string,data");
    variables::set("validate_arguments", "true");

    mock_tool_component(
        "math",
        vec![
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
                name: "multiply".to_string(),
                title: Some("Multiplication".to_string()),
                description: Some("Multiply two numbers".to_string()),
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
        ],
    );

    mock_tool_component(
        "string",
        vec![ToolMetadata {
            name: "concat".to_string(),
            title: Some("String Concatenation".to_string()),
            description: Some("Concatenate strings".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "strings": {
                        "type": "array",
                        "items": {"type": "string"}
                    }
                },
                "required": ["strings"]
            }),
            output_schema: None,
            annotations: None,
            meta: None,
        }],
    );

    mock_tool_component(
        "data",
        vec![ToolMetadata {
            name: "parse_json".to_string(),
            title: Some("JSON Parser".to_string()),
            description: Some("Parse JSON data".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "json": {"type": "string"}
                },
                "required": ["json"]
            }),
            output_schema: None,
            annotations: None,
            meta: None,
        }],
    );

    let request_json = create_json_rpc_request("tools/list", None, Some(serde_json::json!(1)));
    let request = create_mcp_request(request_json);
    let response = spin_test_sdk::perform_request(request);
    let response_data = ResponseData::from_response(response);

    assert_eq!(response_data.status, 200);
    let response_json = response_data.body_json().expect("Expected JSON response");
    assert_json_rpc_success(&response_json, Some(serde_json::json!(1)));

    let tools = response_json["result"]["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 4); // 2 + 1 + 1 tools

    // Verify all tools are present
    let tool_names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();

    assert!(tool_names.contains(&"math__add"));
    assert!(tool_names.contains(&"math__multiply"));
    assert!(tool_names.contains(&"string__concat"));
    assert!(tool_names.contains(&"data__parse_json"));
}

#[spin_test]
fn test_tool_metadata_completeness() {
    variables::set("component_names", "detailed");
    variables::set("validate_arguments", "true");

    mock_tool_component(
        "detailed",
        vec![ToolMetadata {
            name: "complete_tool".to_string(),
            title: Some("Complete Tool Example".to_string()),
            description: Some("A tool with all metadata fields populated".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "title": "Input Parameters",
                "description": "Parameters for the complete tool",
                "properties": {
                    "field1": {
                        "type": "string",
                        "description": "First field",
                        "minLength": 1,
                        "maxLength": 100
                    },
                    "field2": {
                        "type": "integer",
                        "description": "Second field",
                        "minimum": 0,
                        "maximum": 1000
                    },
                    "field3": {
                        "type": "boolean",
                        "description": "Third field",
                        "default": false
                    }
                },
                "required": ["field1", "field2"],
                "additionalProperties": false
            }),
            output_schema: None,
            annotations: None,
            meta: None,
        }],
    );

    let request_json = create_json_rpc_request("tools/list", None, Some(serde_json::json!(1)));
    let request = create_mcp_request(request_json);
    let response = spin_test_sdk::perform_request(request);
    let response_data = ResponseData::from_response(response);

    assert_eq!(response_data.status, 200);
    let response_json = response_data.body_json().expect("Expected JSON response");
    let tools = response_json["result"]["tools"].as_array().unwrap();

    assert_eq!(tools.len(), 1);
    let tool = &tools[0];

    // Verify all metadata fields
    assert_eq!(tool["name"], "detailed__complete_tool");
    assert_eq!(tool["title"], "Complete Tool Example");
    assert_eq!(
        tool["description"],
        "A tool with all metadata fields populated"
    );

    // Verify schema structure
    let schema = &tool["inputSchema"];
    assert_eq!(schema["type"], "object");
    assert_eq!(schema["title"], "Input Parameters");
    assert!(schema["properties"].is_object());
    assert!(schema["required"].is_array());
    assert_eq!(schema["additionalProperties"], false);
}
