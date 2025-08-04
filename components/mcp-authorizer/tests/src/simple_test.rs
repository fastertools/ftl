use spin_test_sdk::{
    bindings::wasi::http,
    spin_test,
};

#[spin_test]
fn test_options_request() {
    // OPTIONS requests should always work regardless of auth
    let request = http::types::OutgoingRequest::new(http::types::Headers::new());
    request.set_method(&http::types::Method::Options).unwrap();
    request.set_path_with_query(Some("/anything")).unwrap();
    
    let response = spin_test_sdk::perform_request(request);
    
    // OPTIONS should return 204
    assert_eq!(response.status(), 204);
}

