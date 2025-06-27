# FTL CLI

A standalone CLI framework for building and deploying WebAssembly-based MCP (Model Context Protocol) tools. FTL makes Spin an implementation detail, allowing developers to focus on tool logic rather than infrastructure.

> **Note**: FTL is now fully standalone! It includes its own MCP server implementation (ftl-core) and no longer depends on any external MCP libraries.

## Features

- 🚀 **Simple tool creation** - Just `lib.rs` and `Cargo.toml`, no `spin.toml` needed
- 🔧 **Dynamic toolkit composition** - Combine multiple tools into a single deployment
- 📦 **WebAssembly optimization** - Built-in support for size and performance optimization
- 🛠️ **Developer-friendly** - Hot reload, local serving, and intuitive CLI
- 🏗️ **Runtime abstraction** - Spin is an implementation detail, not a requirement

## Installation

```bash
cargo install ftl
```

## Quick Start

### Create a New Tool

```bash
ftl new my_tool --description "My awesome MCP tool"
cd my_tool
```

This creates:
- `ftl.toml` - Simple tool configuration
- `Cargo.toml` - Rust dependencies
- `src/lib.rs` - Tool implementation

### Develop Your Tool

```bash
# Serve locally with hot reload
ftl serve my_tool

# Test your tool
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"my_tool","arguments":{"input":"test"}},"id":1}'
```

### Build and Deploy

```bash
# Build optimized version
ftl build my_tool

# Deploy to FTL Edge
ftl deploy my_tool
```

## Tool Configuration

Tools are configured with a simple `ftl.toml` file:

```toml
[tool]
name = "my_tool"
version = "1.0.0"
description = "My awesome tool"

[build]
profile = "release"    # or "dev", "tiny"
features = ["simd"]    # optional features

[optimization]
flags = [
    "-O4",                      # Maximum optimization
    "--enable-simd",           # SIMD support
    "--enable-bulk-memory",    # Bulk memory operations
]

[runtime]
# List of external hosts this tool is allowed to make HTTP requests to.
# Use exact hostnames or patterns with wildcards (e.g., "*.googleapis.com").
# Leave empty to deny all external requests.
allowed_hosts = []
```

## Toolkit Composition

Combine multiple tools into a single deployable unit:

```bash
# Build a toolkit from existing tools
ftl toolkit build --name "dev-tools" json_query regex_match hash

# Serve the toolkit locally
ftl toolkit serve dev-tools

# Deploy the toolkit
ftl toolkit deploy dev-tools
```

## Tool Implementation

Tools implement a simple trait:

```rust
use ftl_core::prelude::*;

#[derive(Clone)]
struct MyTool;

impl Tool for MyTool {
    fn name(&self) -> &'static str { "my_tool" }
    fn description(&self) -> &'static str { "My tool description" }
    
    fn input_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "input": {"type": "string"}
            },
            "required": ["input"]
        })
    }
    
    fn call(&self, args: &serde_json::Value) -> Result<ToolResult, ToolError> {
        let input = args["input"].as_str()
            .ok_or(ToolError::InvalidArguments("input required".into()))?;
            
        Ok(ToolResult::text(format!("Processed: {}", input)))
    }
}

ftl_core::ftl_mcp_server!(MyTool);
```

## CLI Commands

### Tool Management
- `ftl new <name>` - Create a new tool
- `ftl build [name]` - Build tool(s)
- `ftl serve <name>` - Serve tool locally
- `ftl test [name]` - Run tests (unit tests, not in WASM runtime)
- `ftl deploy <name>` - Deploy to FTL Edge
- `ftl watch <name>` - Watch and rebuild on changes
- `ftl validate <name>` - Validate tool configuration
- `ftl size <name>` - Show binary size information

### Toolkit Management
- `ftl toolkit build --name <name> <tools...>` - Build a toolkit
- `ftl toolkit serve <name>` - Serve toolkit locally
- `ftl toolkit deploy <name>` - Deploy toolkit

## Testing

FTL runs standard Rust unit tests using `cargo test`. This approach:
- Tests your tool's logic without WASM runtime complexity
- Runs quickly during development
- Works with existing Rust testing tools and CI/CD

Example test (included in tool template):
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_metadata() {
        let tool = MyTool;
        assert_eq!(tool.name(), "my_tool");
    }
}
```

**Note**: These tests run in native Rust, not in the WebAssembly environment. For full WASM runtime testing, consider using [spin-test](https://developer.fermyon.com/spin/testing-apps) (experimental).

## Why FTL?

Traditional MCP tool development requires understanding Spin's configuration, managing `spin.toml` files, and dealing with deployment complexity. FTL abstracts away these details:

**Before (with Spin)**:
- Write `spin.toml` configuration
- Understand Spin's component model
- Manage deployment manifests
- Handle multi-tool composition manually

**After (with FTL)**:
- Write your tool logic
- Run `ftl serve` to test
- Run `ftl deploy` to ship
- Compose toolkits with one command

## Architecture

FTL consists of three main components:

1. **ftl-cli** - Command-line interface
2. **ftl-runtime** - Runtime abstraction (currently Spin, extensible to others)
3. **ftl-core** - Core MCP server implementation

## Contributing

Contributions are welcome! Please read our [Contributing Guide](CONTRIBUTING.md) for details.

## License

Apache-2.0