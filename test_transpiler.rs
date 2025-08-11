use ftl_commands::config::{ftl_config::FtlConfig, transpiler::transpile_ftl_to_spin};

fn main() {
    let ftl_toml = r#"
[project]
name = "test-app"
version = "0.1.0"
description = "Test app"

[oauth]
issuer = "https://test.authkit.app"
audience = "test-api"

[mcp]
gateway = "ghcr.io/fastertools/mcp-gateway:latest"
authorizer = "ghcr.io/fastertools/mcp-authorizer:latest"
"#;

    let config = FtlConfig::parse(ftl_toml).unwrap();
    let spin_toml = transpile_ftl_to_spin(&config).unwrap();
    
    // Check if mcp_static_tokens is present
    if spin_toml.contains("mcp_static_tokens") {
        println!("✓ mcp_static_tokens is present in the output");
        // Print the line containing it
        for line in spin_toml.lines() {
            if line.contains("mcp_static_tokens") {
                println!("  Found: {}", line);
            }
        }
    } else {
        println!("✗ mcp_static_tokens is MISSING from the output");
        println!("\nGenerated spin.toml variables section:");
        let mut in_variables = false;
        for line in spin_toml.lines() {
            if line == "[variables]" {
                in_variables = true;
            } else if in_variables && line.starts_with('[') {
                break;
            }
            if in_variables {
                println!("{}", line);
            }
        }
    }
}