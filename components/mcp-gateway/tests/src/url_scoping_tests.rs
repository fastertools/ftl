use spin_test_sdk::{
    bindings::fermyon::spin_test_virt::variables,
    spin_test,
};
use crate::{ResponseData, test_helpers::*};

#[spin_test]
fn test_no_scope_returns_all_tools() {
    variables::set("component_names", "calc,string");
    variables::set("validate_arguments", "false");
    
    // Mock components
    mock_tool_component("calc", vec![
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
        }
    ]);
    
    mock_tool_component("string", vec![
        ToolMetadata {
            name: "concat".to_string(),
            title: Some("Concatenate".to_string()),
            description: Some("Concatenate strings".to_string()),
            input_schema: serde_json::json!({"type": "object"}),
            output_schema: None,
            annotations: None,
            meta: None,
        }
    ]);
    
    // Test with root path
    let request_json = create_json_rpc_request("tools/list", None, Some(serde_json::json!(1)));
    let request = create_mcp_request(request_json);
    let response = spin_test_sdk::perform_request(request);
    let response_data = ResponseData::from_response(response);
    
    assert_eq!(response_data.status, 200);
    let response_json = response_data.body_json().expect("Expected JSON response");
    assert_json_rpc_success(&response_json, Some(serde_json::json!(1)));
    
    let tools = response_json["result"]["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 3);
    
    let tool_names: Vec<&str> = tools.iter()
        .map(|t| t["name"].as_str().unwrap())
        .collect();
    
    assert!(tool_names.contains(&"calc__add"));
    assert!(tool_names.contains(&"calc__subtract"));
    assert!(tool_names.contains(&"string__concat"));
}

#[spin_test]
fn test_component_scope_filters_tools() {
    variables::set("component_names", "calc,string");
    variables::set("validate_arguments", "false");
    
    // Mock components
    mock_tool_component("calc", vec![
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
        }
    ]);
    
    mock_tool_component("string", vec![
        ToolMetadata {
            name: "concat".to_string(),
            title: Some("Concatenate".to_string()),
            description: Some("Concatenate strings".to_string()),
            input_schema: serde_json::json!({"type": "object"}),
            output_schema: None,
            annotations: None,
            meta: None,
        }
    ]);
    
    // Test with /calc path
    let request_json = create_json_rpc_request("tools/list", None, Some(serde_json::json!(1)));
    let request = create_mcp_request(request_json);
    request.set_path_with_query(Some("/calc")).unwrap();
    
    let response = spin_test_sdk::perform_request(request);
    let response_data = ResponseData::from_response(response);
    
    assert_eq!(response_data.status, 200);
    let response_json = response_data.body_json().expect("Expected JSON response");
    assert_json_rpc_success(&response_json, Some(serde_json::json!(1)));
    
    let tools = response_json["result"]["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 2);
    
    let tool_names: Vec<&str> = tools.iter()
        .map(|t| t["name"].as_str().unwrap())
        .collect();
    
    assert!(tool_names.contains(&"calc__add"));
    assert!(tool_names.contains(&"calc__subtract"));
    assert!(!tool_names.contains(&"string__concat"));
}

#[spin_test]
fn test_specific_tool_scope() {
    variables::set("component_names", "calc");
    variables::set("validate_arguments", "false");
    
    // Mock component
    mock_tool_component("calc", vec![
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
        }
    ]);
    
    // Test with /calc/add path
    let request_json = create_json_rpc_request("tools/list", None, Some(serde_json::json!(1)));
    let request = create_mcp_request(request_json);
    request.set_path_with_query(Some("/calc/add")).unwrap();
    
    let response = spin_test_sdk::perform_request(request);
    let response_data = ResponseData::from_response(response);
    
    assert_eq!(response_data.status, 200);
    let response_json = response_data.body_json().expect("Expected JSON response");
    assert_json_rpc_success(&response_json, Some(serde_json::json!(1)));
    
    let tools = response_json["result"]["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0]["name"], "calc__add");
}

#[spin_test]
fn test_mcp_suffix_paths() {
    variables::set("component_names", "calc");
    variables::set("validate_arguments", "false");
    
    // Mock component
    mock_tool_component("calc", vec![
        ToolMetadata {
            name: "add".to_string(),
            title: Some("Add".to_string()),
            description: Some("Add numbers".to_string()),
            input_schema: serde_json::json!({"type": "object"}),
            output_schema: None,
            annotations: None,
            meta: None,
        }
    ]);
    
    // Test various path formats with /mcp suffix
    let paths = vec![
        "/mcp",
        "/calc/mcp",
        "/calc/add/mcp",
    ];
    
    let expected_counts = vec![1, 1, 1];
    
    for (path, expected_count) in paths.iter().zip(expected_counts.iter()) {
        // Re-mock the component before each request (spin-test limitation)
        mock_tool_component("calc", vec![
            ToolMetadata {
                name: "add".to_string(),
                title: Some("Add".to_string()),
                description: Some("Add numbers".to_string()),
                input_schema: serde_json::json!({"type": "object"}),
                output_schema: None,
                annotations: None,
                meta: None,
            }
        ]);
        
        let request_json = create_json_rpc_request("tools/list", None, Some(serde_json::json!(1)));
        let request = create_mcp_request(request_json);
        request.set_path_with_query(Some(path)).unwrap();
        
        let response = spin_test_sdk::perform_request(request);
        let response_data = ResponseData::from_response(response);
        
        assert_eq!(response_data.status, 200);
        let response_json = response_data.body_json().expect("Expected JSON response");
        let tools = response_json["result"]["tools"].as_array().unwrap();
        assert_eq!(tools.len(), *expected_count, "Failed for path: {}", path);
    }
}

#[spin_test]
fn test_scoped_tool_call_allowed() {
    variables::set("component_names", "calc");
    variables::set("validate_arguments", "false");
    
    // Mock component and tool execution
    mock_tool_component("calc", vec![
        ToolMetadata {
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
        }
    ]);
    
    mock_tool_execution("calc", "add", ToolResponse {
        content: vec![ToolContent::Text {
            text: "Result: 5".to_string(),
            annotations: None,
        }],
        structured_content: None,
        is_error: None,
    });
    
    // Call tool within scope
    let call_request = create_json_rpc_request(
        "tools/call",
        Some(serde_json::json!({
            "name": "calc__add",
            "arguments": {"a": 2, "b": 3}
        })),
        Some(serde_json::json!(1))
    );
    
    let request = create_mcp_request(call_request);
    request.set_path_with_query(Some("/calc")).unwrap();
    
    let response = spin_test_sdk::perform_request(request);
    let response_data = ResponseData::from_response(response);
    
    assert_eq!(response_data.status, 200);
    let response_json = response_data.body_json().expect("Expected JSON response");
    assert_json_rpc_success(&response_json, Some(serde_json::json!(1)));
}

#[spin_test]
fn test_scoped_tool_call_blocked() {
    variables::set("component_names", "calc,string");
    variables::set("validate_arguments", "false");
    
    // Mock components
    mock_tool_component("calc", vec![]);
    mock_tool_component("string", vec![
        ToolMetadata {
            name: "concat".to_string(),
            title: Some("Concatenate".to_string()),
            description: Some("Concatenate strings".to_string()),
            input_schema: serde_json::json!({"type": "object"}),
            output_schema: None,
            annotations: None,
            meta: None,
        }
    ]);
    
    // Try to call string tool while scoped to calc
    let call_request = create_json_rpc_request(
        "tools/call",
        Some(serde_json::json!({
            "name": "string__concat",
            "arguments": {}
        })),
        Some(serde_json::json!(1))
    );
    
    let request = create_mcp_request(call_request);
    request.set_path_with_query(Some("/calc")).unwrap();
    
    let response = spin_test_sdk::perform_request(request);
    let response_data = ResponseData::from_response(response);
    
    assert_eq!(response_data.status, 200);
    let response_json = response_data.body_json().expect("Expected JSON response");
    assert_json_rpc_error(&response_json, -32602, Some(serde_json::json!(1)));
    assert!(response_json["error"]["message"].as_str().unwrap().contains("not accessible in the current scope"));
}