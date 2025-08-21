use crate::{test_helpers::*, ResponseData};
use spin_test_sdk::{bindings::fermyon::spin_test_virt::variables, spin_test};

// Performance tests are limited without actual components
// These tests verify the gateway can handle various configurations

#[spin_test]
fn test_empty_components_list_performance() {
    variables::set("component_names", "");
    variables::set("validate_arguments", "true");

    let request_json = create_json_rpc_request("tools/list", None, Some(serde_json::json!(1)));
    let request = create_mcp_request(request_json);
    let response = spin_test_sdk::perform_request(request);
    let response_data = ResponseData::from_response(response);

    assert_eq!(response_data.status, 200);
    let response_json = response_data.body_json().expect("Expected JSON response");
    let tools = response_json["result"]["tools"].as_array().unwrap();

    // Should handle empty list efficiently
    assert_eq!(tools.len(), 0);
}

#[spin_test]
fn test_many_components_configuration() {
    // Test configuration parsing with many components
    let many_components = (0..20)
        .map(|i| format!("component{i}"))
        .collect::<Vec<_>>()
        .join(",");

    variables::set("component_names", &many_components);
    variables::set("validate_arguments", "false");

    // Mock all the components with empty tool lists
    for i in 0..20 {
        mock_tool_component(&format!("component{i}"), vec![]);
    }

    // Should handle configuration without issues
    let request_json = create_json_rpc_request("tools/list", None, Some(serde_json::json!(1)));
    let request = create_mcp_request(request_json);
    let response = spin_test_sdk::perform_request(request);
    let response_data = ResponseData::from_response(response);

    // Gateway should respond even if components don't exist
    assert_eq!(response_data.status, 200);
}
