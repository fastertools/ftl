use crate::{test_helpers::*, ResponseData};
use spin_test_sdk::{bindings::wasi::http, spin_test};

#[spin_test]
fn test_invalid_json_rpc_request() {
    setup_default_test_env();

    // Send malformed JSON
    let headers = http::types::Headers::new();
    headers.append("content-type", b"application/json").unwrap();

    let request = http::types::OutgoingRequest::new(headers);
    request.set_method(&http::types::Method::Post).unwrap();
    request.set_path_with_query(Some("/mcp")).unwrap();

    let body = request.body().unwrap();
    body.write_bytes(b"{ invalid json }");

    let response = spin_test_sdk::perform_request(request);
    let response_data = ResponseData::from_response(response);

    assert_eq!(response_data.status, 200);
    let response_json = response_data.body_json().expect("Expected JSON response");
    assert_json_rpc_error(&response_json, -32700, None); // Parse error
}

#[spin_test]
fn test_method_not_found() {
    setup_default_test_env();

    let request_json = create_json_rpc_request("unknown/method", None, Some(serde_json::json!(1)));

    let request = create_mcp_request(request_json);
    let response = spin_test_sdk::perform_request(request);
    let response_data = ResponseData::from_response(response);

    assert_eq!(response_data.status, 200);
    let response_json = response_data.body_json().expect("Expected JSON response");
    assert_json_rpc_error(&response_json, -32601, Some(serde_json::json!(1)));
    assert!(response_json["error"]["message"]
        .as_str()
        .unwrap()
        .contains("not found"));
}

#[spin_test]
fn test_empty_request_body() {
    setup_default_test_env();

    let headers = http::types::Headers::new();
    headers.append("content-type", b"application/json").unwrap();

    let request = http::types::OutgoingRequest::new(headers);
    request.set_method(&http::types::Method::Post).unwrap();
    request.set_path_with_query(Some("/mcp")).unwrap();

    // Empty body
    let body = request.body().unwrap();
    body.write_bytes(b"");

    let response = spin_test_sdk::perform_request(request);
    let response_data = ResponseData::from_response(response);

    assert_eq!(response_data.status, 200);
    let response_json = response_data.body_json().expect("Expected JSON response");
    assert_json_rpc_error(&response_json, -32700, None); // Parse error
}

#[spin_test]
fn test_json_rpc_batch_not_supported() {
    setup_default_test_env();

    // Send batch request (array of requests)
    let batch_request = serde_json::json!([
        {
            "jsonrpc": "2.0",
            "method": "ping",
            "id": 1
        },
        {
            "jsonrpc": "2.0",
            "method": "tools/list",
            "id": 2
        }
    ]);

    let request = create_mcp_request(batch_request);
    let response = spin_test_sdk::perform_request(request);
    let response_data = ResponseData::from_response(response);

    assert_eq!(response_data.status, 200);
    let response_json = response_data.body_json().expect("Expected JSON response");

    // Should return parse error for batch requests
    assert_json_rpc_error(&response_json, -32700, None);
}
