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

</details>

### Develop Your Tool

<details open>
<summary><b>🦀 Rust Implementation</b></summary>

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
<summary><b>🟨 JavaScript Implementation</b></summary>

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

### Deploy to FTL Edge

```bash
ftl deploy
```

This will deploy your tool to the FTL Edge, where it can be called by your AI agents.

## Toolkits

FTL supports bundling multiple tools together as a toolkit, with an automatic gateway that provides a unified MCP endpoint.

### Create a Toolkit

```bash
# Build multiple tools first
ftl new foo
ftl new bar
ftl new baz

# Bundle them as a toolkit
ftl toolkit build --name my-toolkit foo bar baz
```

### Serve a Toolkit Locally

```bash
ftl toolkit serve my-toolkit
```

This starts a local server with:
- `/mcp` - Unified endpoint that aggregates all tools
- `/tool1/mcp` - Direct access to individual tools
- `/tool2/mcp`
- `/tool3/mcp`

### Deploy a Toolkit

```bash
ftl toolkit deploy my-toolkit
```

The gateway automatically handles:
- Tool discovery across all bundled tools
- Request routing to the appropriate tool
- Protocol compatibility between tools

## Documentation

For more detailed documentation, please see the [docs](./docs/introduction.md) directory in this repository.

## Contributing

Contributions are welcome! Please see our [Contributing Guide](CONTRIBUTING.md) for more information.

## License

This project is licensed under the Apache-2.0 License.
