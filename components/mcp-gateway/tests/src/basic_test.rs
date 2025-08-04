use spin_test_sdk::{
    spin_test,
};
use crate::{ResponseData, test_helpers::*};

#[spin_test]
fn test_ping() {
    // Just test the basic ping functionality
    let request_json = create_json_rpc_request("ping", None, Some(serde_json::json!(1)));
    let request = create_mcp_request(request_json);
    let response = spin_test_sdk::perform_request(request);
    let response_data = ResponseData::from_response(response);
    
    assert_eq!(response_data.status, 200);
    let response_json = response_data.body_json().expect("Expected JSON response");
    assert_json_rpc_success(&response_json, Some(serde_json::json!(1)));
}