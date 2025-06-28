use std::path::Path;

use anyhow::Result;
use handlebars::Handlebars;
use serde_json::json;

pub fn create_tool(name: &str, description: &str, target_dir: &Path) -> Result<()> {
    // Create directory structure
    std::fs::create_dir_all(target_dir)?;
    std::fs::create_dir_all(target_dir.join("src"))?;

    // Convert tool name to struct name (PascalCase)
    let struct_name = name
        .split('-')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<String>();

    let handlebars = Handlebars::new();
    let data = json!({
        "tool_name": name,
        "struct_name": struct_name,
        "description": description,
        "version": "0.1.0",
    });

    // Create ftl.toml
    let ftl_toml_template = r#"[tool]
name = "{{tool_name}}"
version = "{{version}}"
description = "{{description}}"

[build]
profile = "release"
features = []

[optimization]
# See https://github.com/WebAssembly/binaryen/blob/version_123/test/lit/help/wasm-opt.test
flags = [
    "-O4",
    "-Oz",
]

[runtime]
# List of external hosts this tool is allowed to make HTTP requests to.
# Use exact hostnames or patterns with wildcards (e.g., "*.googleapis.com").
# Leave empty to deny all external requests.
allowed_hosts = []
"#;

    let ftl_toml = handlebars.render_template(ftl_toml_template, &data)?;
    std::fs::write(target_dir.join("ftl.toml"), ftl_toml)?;

    // Determine ftl-core dependency based on context
    // This logic handles three scenarios:
    // 1. Creating tools within ftl-cli repository (use relative paths)
    // 2. Creating tools in subdirectories of ftl-cli (use ../../ftl-core)
    // 3. Creating tools outside ftl-cli (use crates.io version once published)
    let ftl_core_dep = if target_dir.join("../ftl-core").exists() {
        // Tool is being created directly in ftl-cli directory
        "{ path = \"../ftl-core\" }".to_string()
    } else if target_dir.join("../../ftl-core").exists() {
        // Tool is being created in a subdirectory of ftl-cli (e.g.,
        // test_validation/tool_name)
        "{ path = \"../../ftl-core\" }".to_string()
    } else {
        // Tool is being created outside ftl-cli repository
        // Use crates.io version once published
        "{ version = \"0.1\" }".to_string()
    };

    // Create Cargo.toml
    let cargo_toml_template = format!(
        r#"[package]
name = "{{{{tool_name}}}}"
version = "{{{{version}}}}"
edition = "2021"

[dependencies]
ftl-core = {}
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"
anyhow = "1.0"
spin-sdk = "3.1.1"

[dev-dependencies]

[lib]
crate-type = ["cdylib"]

[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
panic = "abort"
strip = "symbols"

[profile.dev]
opt-level = 1
debug = true

[workspace]
"#,
        ftl_core_dep
    );

    let cargo_toml = handlebars.render_template(&cargo_toml_template, &data)?;
    std::fs::write(target_dir.join("Cargo.toml"), cargo_toml)?;

    // Create src/lib.rs
    let lib_rs_template = r#"use ftl_core::prelude::*;
use serde_json::json;

#[derive(Clone)]
struct {{struct_name}};

impl Tool for {{struct_name}} {
    fn name(&self) -> &'static str {
        "{{tool_name}}"
    }

    fn description(&self) -> &'static str {
        "{{description}}"
    }

    fn input_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "input": {
                    "type": "string",
                    "description": "Input to process"
                }
            },
            "required": ["input"]
        })
    }

    fn call(&self, args: &serde_json::Value) -> Result<ToolResult, ToolError> {
        let input = args["input"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidArguments("input is required".to_string()))?;

        // TODO: Implement your tool logic here
        let result = format!("Processed: {}", input);

        Ok(ToolResult::text(result))
    }
}

ftl_core::ftl_mcp_server!({{struct_name}});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_metadata() {
        let tool = {{struct_name}};
        assert_eq!(tool.name(), "{{tool_name}}");
        assert_eq!(tool.description(), "{{description}}");
    }

    #[test]
    fn test_tool_call() {
        let tool = {{struct_name}};
        let args = json!({
            "input": "test input"
        });

        let result = tool.call(&args).unwrap();
        // Result is a text block
        assert!(!result.content.is_empty());
        let text = &result.content[0].text;
        assert!(text.contains("test input"));
    }
}
"#;

    let lib_rs = handlebars.render_template(lib_rs_template, &data)?;
    std::fs::write(target_dir.join("src").join("lib.rs"), lib_rs)?;

    // Create README.md
    let readme_template = r#"# {{tool_name}}

{{description}}

## Usage

This tool is designed to be used with the Model Context Protocol (MCP).

### Development

```bash
# Serve locally for testing
ftl serve {{tool_name}}

# Build the tool
ftl build {{tool_name}}

# Run tests (requires spin test setup)
ftl test {{tool_name}}
```

### Input Schema

```json
{
  "type": "object",
  "properties": {
    "input": {
      "type": "string",
      "description": "Input to process"
    }
  },
  "required": ["input"]
}
```

### Example

```bash
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "tools/call",
    "params": {
      "name": "{{tool_name}}",
      "arguments": {
        "input": "Hello, world!"
      }
    },
    "id": 1
  }'
```

## Configuration

Edit `ftl.toml` to configure build and runtime settings.

## License

Apache-2.0
"#;

    let readme = handlebars.render_template(readme_template, &data)?;
    std::fs::write(target_dir.join("README.md"), readme)?;

    // Create .gitignore
    let gitignore = r#"target/
Cargo.lock
.ftl/
*.wasm
"#;
    std::fs::write(target_dir.join(".gitignore"), gitignore)?;

    Ok(())
}
