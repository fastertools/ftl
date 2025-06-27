# ftl-core

Core library for building WebAssembly-based MCP (Model Context Protocol) tools.

## Overview

`ftl-core` provides a lightweight, WebAssembly-optimized implementation of the MCP server protocol. It's designed to compile to small WASM binaries (typically under 500KB) suitable for edge deployment.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
ftl-core = "0.1"
```

Then implement a tool:

```rust
use ftl_core::prelude::*;
use serde_json::json;

#[derive(Clone)]
struct MyTool;

impl Tool for MyTool {
    fn name(&self) -> &'static str {
        "my_tool"
    }

    fn description(&self) -> &'static str {
        "My awesome tool"
    }

    fn input_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "input": {
                    "type": "string",
                    "description": "Input text"
                }
            },
            "required": ["input"]
        })
    }

    fn call(&self, args: &serde_json::Value) -> Result<ToolResult, ToolError> {
        let input = args["input"].as_str()
            .ok_or_else(|| ToolError::InvalidArguments("input required".to_string()))?;
        
        Ok(ToolResult::text(format!("Processed: {}", input)))
    }
}

// Create the WebAssembly component
ftl_core::ftl_mcp_server!(MyTool);
```

## Features

- **Minimal dependencies** - Optimized for small WASM binary size
- **MCP protocol support** - Full JSON-RPC 2.0 implementation
- **Memory efficient** - Uses custom allocator optimized for edge environments
- **Type safe** - Strong typing with serde serialization

## License

Apache-2.0