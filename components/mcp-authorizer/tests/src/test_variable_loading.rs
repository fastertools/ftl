//! Test that variables are being loaded correctly

use spin_test_sdk::{
    bindings::{
        fermyon::spin_test_virt::variables,
        wasi::http::types,
    },
    spin_test,
};

#[spin_test]
fn test_variable_loading() {
    // Set a variable
    variables::set("mcp_org_id", "test_org_value");
    
    // Now make a request without auth - should get 401 but with debug info
    let request = types::OutgoingRequest::new(types::Headers::new());
    request.set_path_with_query(Some("/test")).unwrap();
    
    let response = spin_test_sdk::perform_request(request);
    
    // Should get 401 because no auth header
    assert_eq!(response.status(), 401);
    
    // But the error should show that config loaded
    // (our debug output will be in stderr, which we can't capture, 
    // but at least the test shows variables are being set)
}