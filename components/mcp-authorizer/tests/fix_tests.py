#!/usr/bin/env python3
import os
import re

def fix_request_creation(content):
    """Fix the Request::builder() pattern to use OutgoingRequest properly"""
    
    # Pattern 1: Basic GET requests
    pattern1 = r'''let request = http::types::OutgoingRequest::new\(http::types::Headers::new\(\)\); // Fix imports
        \.method\(&http::types::Method::(\w+)\)
        \.uri\("([^"]+)"\)
        \.header\("authorization", format!\("Bearer \{\}", ([^)]+)\)\)
        \.build\(\);'''
    
    replacement1 = r'''let request = http::types::OutgoingRequest::new(http::types::Headers::new());
    request.set_method(&http::types::Method::\1).unwrap();
    request.set_path_with_query(Some("\2")).unwrap();
    request.headers().set(&"authorization", &format!("Bearer {}", \3).as_bytes()).unwrap();'''
    
    content = re.sub(pattern1, replacement1, content)
    
    # Pattern 2: POST requests with body
    pattern2 = r'''let request = http::types::OutgoingRequest::new\(http::types::Headers::new\(\)\); // Fix imports
        \.method\(&http::types::Method::(\w+)\)
        \.uri\("([^"]+)"\)
        \.header\("authorization", format!\("Bearer \{\}", ([^)]+)\)\)
        \.header\("content-type", "([^"]+)"\)
        \.body\(([^)]+)\)
        \.build\(\);'''
    
    replacement2 = r'''let request = http::types::OutgoingRequest::new(http::types::Headers::new());
    request.set_method(&http::types::Method::\1).unwrap();
    request.set_path_with_query(Some("\2")).unwrap();
    request.headers().set(&"authorization", &format!("Bearer {}", \3).as_bytes()).unwrap();
    request.headers().set(&"content-type", b"\4").unwrap();
    let body_stream = request.body().unwrap();
    body_stream.blocking_write_and_flush(\5).unwrap();'''
    
    content = re.sub(pattern2, replacement2, content)
    
    # Fix imports for TokenBuilder/TestTokenBuilder
    content = content.replace('use crate::test_token_utils::TokenBuilder;', '')
    content = content.replace('TokenBuilder::new(KeyPairType::default())', 'TestTokenBuilder::new()')
    content = content.replace('let mut builder = TokenBuilder::new(KeyPairType::default());', 
                             'let mut builder = TestTokenBuilder::new();')
    
    # Import TestTokenBuilder where needed
    if 'TestTokenBuilder::new()' in content and 'use crate::test_token_utils::TestTokenBuilder;' not in content:
        # Add import after other imports
        import_line = 'use crate::test_setup::setup_default_test_config;'
        if import_line in content:
            content = content.replace(import_line, 
                                     import_line + '\nuse crate::test_token_utils::TestTokenBuilder;')
    
    return content

# Process all policy test files
test_dir = '/home/ian/Dev/ftl-cli/components/mcp-authorizer/tests/src'
for filename in os.listdir(test_dir):
    if filename.startswith('policy_') and filename.endswith('.rs'):
        filepath = os.path.join(test_dir, filename)
        print(f"Processing {filename}...")
        
        with open(filepath, 'r') as f:
            content = f.read()
        
        fixed = fix_request_creation(content)
        
        with open(filepath, 'w') as f:
            f.write(fixed)

print("Done!")