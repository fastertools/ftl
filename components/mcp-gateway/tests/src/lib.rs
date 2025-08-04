use spin_test_sdk::{
    bindings::{
        fermyon::spin_test_virt::variables,
        wasi::http
    },
    spin_test,
};

mod test_helpers;
mod protocol_tests;
mod routing_tests;
mod validation_tests;
mod error_handling_tests;
mod tool_discovery_tests;
mod performance_tests;
mod cors_tests;
mod json_rpc_tests;
mod integration_tests;
mod basic_test;

// Response data helper to extract all needed information
pub struct ResponseData {
    pub status: u16,
    pub headers: Vec<(String, Vec<u8>)>,
    pub body: Vec<u8>,
}

impl ResponseData {
    pub fn from_response(response: http::types::IncomingResponse) -> Self {
        let status = response.status();
        
        // Extract headers before consuming response
        let headers = response.headers()
            .entries()
            .into_iter()
            .map(|(name, value)| (name.to_string(), value.to_vec()))
            .collect();
        
        // Now consume response to get body
        let body = response.body().unwrap_or_else(|_| Vec::new());
        
        Self { status, headers, body }
    }
    
    pub fn find_header(&self, name: &str) -> Option<&Vec<u8>> {
        self.headers.iter()
            .find(|(h_name, _)| h_name.eq_ignore_ascii_case(name))
            .map(|(_, value)| value)
    }
    
    pub fn body_json(&self) -> Option<serde_json::Value> {
        if self.body.is_empty() {
            None
        } else {
            serde_json::from_slice(&self.body).ok()
        }
    }
}

// Simple ping test to verify basic functionality
#[spin_test]
fn basic_ping_test() {
    // Setup test configuration
    variables::set("tool_components", "echo");
    variables::set("validate_arguments", "true");
    
    // Create JSON-RPC ping request
    let request_body = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "ping",
        "id": 1
    });
    
    let headers = http::types::Headers::new();
    headers.append("content-type", b"application/json").unwrap();
    
    let request = http::types::OutgoingRequest::new(headers);
    request.set_method(&http::types::Method::Post).unwrap();
    request.set_path_with_query(Some("/mcp")).unwrap();
    
    let request_body_bytes = serde_json::to_vec(&request_body).unwrap();
    let body = request.body().unwrap();
    body.write_bytes(&request_body_bytes);
    
    let response = spin_test_sdk::perform_request(request);
    let response_data = ResponseData::from_response(response);
    
    // Verify response
    assert_eq!(response_data.status, 200);
    
    let response_json = response_data.body_json().expect("Expected JSON response");
    assert_eq!(response_json["jsonrpc"], "2.0");
    assert_eq!(response_json["id"], 1);
    assert!(response_json["result"].is_object());
}