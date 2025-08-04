// Helper functions for tests

pub fn find_header<'a>(headers: &'a spin_test_sdk::bindings::wasi::http::types::Headers, name: &str) -> Option<Vec<u8>> {
    let entries = headers.entries();
    entries.iter()
        .find(|(n, _)| n.eq_ignore_ascii_case(name))
        .map(|(_, v)| v.clone())
}

pub fn find_header_str<'a>(headers: &'a spin_test_sdk::bindings::wasi::http::types::Headers, name: &str) -> Option<String> {
    find_header(headers, name)
        .map(|v| String::from_utf8_lossy(&v).to_string())
}