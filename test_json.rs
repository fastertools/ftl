fn main() {
    let v = serde_json::json!("test");
    println!("Type: {:?}", v);
    println!("Is string: {}", v.is_string());
    println!("as_str: {:?}", v.as_str());
    
    // Also test the exact thing we're doing
    let mut map = std::collections::HashMap::new();
    map.insert("org_id".to_string(), serde_json::json!("org_wrong"));
    
    let org_id = map.remove("org_id")
        .and_then(|v| v.as_str().map(String::from));
    
    println!("org_id extracted: {:?}", org_id);
}