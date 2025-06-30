<div align="center">

# `ftl`

Fast tools for AI agents

A [Rust](https://www.rust-lang.org) + [WebAssembly](https://webassembly.org) project.

[![CI](https://github.com/fastertools/ftl-cli/actions/workflows/ci.yml/badge.svg)](https://github.com/fastertools/core/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.87+-orange.svg)](https://www.rust-lang.org)
[![WebAssembly](https://img.shields.io/badge/WebAssembly-compatible-purple.svg)](https://webassembly.org/)

[Docs](./docs/introduction.md) | [Contributing](./CONTRIBUTING.md) | [Security](./SECURITY.md) | [Releases](https://github.com/fastertools/ftl-cli/releases)

</div>

FTL is a platform for edge-hosted [Model Context Protocol](https://modelcontextprotocol.io/introduction) tools for AI agents.

This repository contains the `ftl` command-line interface, which is the primary entry point for developers using the FTL platform.

## Getting Started

### Installation

```bash
cargo install ftl-cli
```

### Create a New Tool

<details>
<summary><b>🦀 Rust</b></summary>

```bash
ftl new my-tool --rust
```

This creates a new directory with:
- `ftl.toml` - Tool manifest
- `Cargo.toml` - Rust dependencies
- `src/lib.rs` - Tool implementation

```rust
use ftl_sdk_rs::prelude::*;

#[derive(Clone)]
struct MyTool;

impl Tool for MyTool {
    fn name(&self) -> &'static str { "my-tool" }
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

ftl_sdk_rs::ftl_mcp_server!(MyTool);
```

</details>

<details>
<summary><b>🟨 JavaScript</b></summary>

```bash
ftl new my-tool --javascript
```

This creates a new directory with:
- `ftl.toml` - Tool manifest
- `package.json` - Node dependencies
- `src/index.js` - Tool implementation

```javascript
import { Tool } from '@fastertools/ftl-sdk-js';

export default class MyTool extends Tool {
    get name() { return 'my-tool'; }
    get description() { return 'My tool description'; }
    
    get inputSchema() {
        return {
            type: 'object',
            properties: {
                input: { type: 'string' }
            },
            required: ['input']
        };
    }
    
    async execute(args) {
        const { input } = args;
        
        if (!input) {
            throw new ToolError.invalidArguments('input required');
        }
        
        return ToolResult.text(`Processed: ${input}`);
    }
}
```

</details>

### Serve Locally

```bash
ftl serve
```

This will start a local development server with hot reloading. You can test your tool by sending it a JSON-RPC request:

```bash
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"my-tool","arguments":{"input":"test"}},"id":1}'
```

Each FTL tool is a complete MCP server that exposes a single tool. When you deploy an individual tool, you're deploying a standalone MCP server. Toolkits (described below) bundle multiple tools together with a gateway that provides a unified MCP server interface.

### Deploy to FTL Edge

```bash
ftl deploy
```

This will deploy your tool to the FTL Edge, where it can be called by your AI agents.

## Toolkits

FTL supports bundling multiple tools together as a toolkit, providing a powerful way to create comprehensive agent capabilities. Toolkits leverage the WebAssembly component model to enable secure, high-performance composition of tools.

### Architecture

Each FTL tool is a self-contained WebAssembly component that implements its own MCP server exposing a single tool. Toolkits take this further by:

- **Component Composition**: Multiple WebAssembly components (tools) are bundled together using the component model
- **Automatic Gateway**: FTL generates a gateway component that acts as a logical MCP server over all tools
- **Language Agnostic**: Each tool can be written in a different language (Rust, JavaScript, etc.), allowing you to mix languages within a single toolkit. Through the gateway component, this means that you can expose a polyglot MCP server, with tools implemented in the language most appropriate for its task.
- **Local Development**: Toolkits work seamlessly both locally and when deployed to the edge

### How It Works

```
┌─────────────────────────────────────────────────┐
│                 Toolkit                         │
├─────────────────────────────────────────────────┤
│  Gateway Component (MCP Server)                 │
│    ├─ /mcp (unified endpoint)                  │
│    └─ Routes to:                               │
│         ├─ Rust Tool 1 (MCP Server)            │
│         ├─ JavaScript Tool 2 (MCP Server)      │
│         └─ Rust Tool 3 (MCP Server)            │
└─────────────────────────────────────────────────┘
```

The gateway component:
- Exposes a single MCP endpoint that aggregates all tools
- Dynamically discovers tools at runtime
- Routes `tools/call` requests to the appropriate tool component
- Maintains protocol compatibility across all tools
- The request is passed from the gateway component to the tool in memory without leaving the host process. This is fast.

### Create a Toolkit

```bash
# Build individual tools (can be different languages)
ftl build rust-analyzer    # Rust tool
ftl build js-formatter     # JavaScript tool  
ftl build data-processor   # Another Rust tool

# Bundle them as a toolkit
ftl toolkit build --name dev-toolkit rust-analyzer js-formatter data-processor
```

### Serve a Toolkit Locally

```bash
ftl toolkit serve dev-toolkit
```

This starts a local server with:
- `/mcp` - Unified endpoint that aggregates all tools
- `/rust-analyzer/mcp` - Direct access to individual tool
- `/js-formatter/mcp` - Direct access to individual tool
- `/data-processor/mcp` - Direct access to individual tool

### Deploy a Toolkit

```bash
ftl toolkit deploy dev-toolkit
```

### Benefits

- **Single Integration Point**: AI agents connect to one MCP endpoint to access all tools
- **Mixed Language Support**: Combine Rust tools for performance-critical operations with JavaScript tools for rapid development
- **Component Isolation**: Each tool runs in its own sandboxed WebAssembly module
- **Local-First Development**: Test complete toolkits locally before deployment
- **Dynamic Composition**: Add or remove tools without changing agent configurations

## Documentation

For more detailed documentation, please see the [docs](./docs/introduction.md) directory in this repository.

## Contributing

Contributions are welcome! Please see our [Contributing Guide](CONTRIBUTING.md) for more information.

## License

This project is licensed under the Apache-2.0 License.
