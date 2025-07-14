<div align="center">

# `ftl`

Build and deploy Model Context Protocol (MCP) tools on WebAssembly

[![CI](https://github.com/fastertools/ftl-cli/actions/workflows/ci.yml/badge.svg)](https://github.com/fastertools/ftl-cli/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.87+-orange.svg)](https://www.rust-lang.org)
[![WebAssembly](https://img.shields.io/badge/WebAssembly-compatible-purple.svg)](https://webassembly.org/)

[Docs](./docs/introduction.md) | [Contributing](./CONTRIBUTING.md) | [Security](./SECURITY.md) | [Releases](https://github.com/fastertools/ftl-cli/releases)

</div>

FTL is a command-line tool that wraps [Fermyon Spin](https://www.fermyon.com/spin) to simplify building and deploying [Model Context Protocol](https://modelcontextprotocol.io) (MCP) tools as WebAssembly components. It uses the [ftl-mcp](https://github.com/fastertools/ftl-mcp) framework to provide a streamlined workflow for creating, testing, and deploying MCP tools on the Fermyon platform.

## Quick Start

```bash
# Install FTL
cargo install ftl-cli

# Set up templates
ftl setup templates

# Create a new project
ftl init my-assistant
cd my-assistant

# Add a tool
ftl add weather-tool --language typescript

# Start development server with auto-rebuild
ftl watch

# Run tests
ftl test

# Build and deploy
ftl build --release
ftl deploy
```

## Key Features

- **Tool-First Architecture**: Build individual MCP tools as reusable WebAssembly components
- **Multi-Language Support**: Write tools in Rust or TypeScript  
- **Automatic Tool Registration**: Tools are automatically registered with the MCP gateway
- **Local Service Chaining**: Tools communicate efficiently via HTTP without network overhead
- **Hot Reload Development**: Auto-rebuild on file changes with `ftl watch`
- **Edge Deployment**: Deploy anywhere Spin runs
- **Simple Tool Management**: Just run `ftl add` and start coding

## Creating MCP Projects

### TypeScript Example

```bash
# Create project and add TypeScript tool
ftl init my-project
cd my-project
ftl add my-tool --language typescript
```

```typescript
// my-tool/src/index.ts
import { createTool, ToolResponse } from 'ftl-sdk'
import { z } from 'zod'

// Define the schema using Zod
const InputSchema = z.object({
  message: z.string().describe('The message to process')
})

type ToolInput = z.infer<typeof InputSchema>

const tool = createTool<ToolInput>({
  metadata: {
    name: 'my_tool',
    title: 'My Tool',
    description: 'A simple MCP tool',
    inputSchema: z.toJSONSchema(InputSchema)
  },
  handler: async (input) => {
    return ToolResponse.text(`Processed: ${input.message}`)
  }
})

//@ts-ignore
addEventListener('fetch', (event: FetchEvent) => {
  event.respondWith(tool(event.request))
})
```

### Rust Example

```bash
# Create project and add Rust tool
ftl init my-project
cd my-project
ftl add my-tool --language rust
```

```rust
// my-tool/src/lib.rs
use ftl_sdk::{tool, ToolResponse};
use serde::Deserialize;
use schemars::JsonSchema;

#[derive(Deserialize, JsonSchema)]
struct MyToolInput {
    /// The message to process
    message: String,
}

/// A simple MCP tool
#[tool]
fn my_tool(input: MyToolInput) -> ToolResponse {
    ToolResponse::text(format!("Processed: {}", input.message))
}
```

## Tool Development Workflow

### 1. Development
```bash
# From project root (with spin.toml)
ftl build           # Build all tools
ftl test            # Run tests
ftl watch           # Auto-rebuild on changes
ftl up              # Run the MCP server
```

### 2. Testing Your Tools
```bash
# List available tools
curl -X POST http://127.0.0.1:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"tools/list","id":1}'

# Call a tool
curl -X POST http://127.0.0.1:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc":"2.0",
    "method":"tools/call",
    "params": {
      "name": "my_tool",
      "arguments": {"message": "Hello, World!"}
    },
    "id": 2
  }'
```

### 3. MCP Client Configuration
```json
{
  "mcpServers": {
    "my-assistant": {
      "url": "http://127.0.0.1:3000/mcp",
      "transport": "http"
    }
  }
}
```

### 4. Deployment
```bash
# Deploy to FTL/Fermyon
ftl deploy

# Or use Spin directly
spin deploy
```

## Architecture

FTL leverages the ftl-mcp framework and Spin platform:

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│   MCP Client    │────▶│  MCP Gateway    │────▶│   Tool Component│
│   (AI Agent)    │     │  (Router)       │     │ (WASM Module)   │
└─────────────────┘     └─────────────────┘     └─────────────────┘
                               │
                               ├── /weather-tool ──▶ Weather Tool (TypeScript)
                               ├── /calculator   ──▶ Calculator Tool (Rust)
                               └── /database     ──▶ Database Tool (TypeScript)
```

### Project Structure

```
my-assistant/
├── spin.toml           # Spin manifest with MCP gateway
├── weather-tool/       # TypeScript tool
│   ├── package.json
│   └── src/
│       └── index.ts
├── calculator/         # Rust tool  
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs
└── database/          # Another tool
    ├── package.json
    └── src/
        └── index.ts
```

Each tool:
- Is a standalone WebAssembly component
- Implements a specific MCP tool
- Can be developed and tested independently
- Communicates via local HTTP (no network overhead)
- Runs in a secure sandbox

## Prerequisites

- **Rust toolchain** (for FTL CLI)
- **Language-specific requirements**:
  - Rust: cargo with wasm32-wasip1 target (cargo-component auto-installed)
  - TypeScript/JavaScript: Node.js 20+
- **Optional**:
  - wkg for publishing ([install](https://github.com/bytecodealliance/wasm-pkg-tools))
  - cargo-binstall for faster tool installation
- **Auto-installed**:
  - Spin runtime (prompted on first use)
  - cargo-component (for Rust tools)

## Documentation

- [Getting Started Guide](./docs/introduction.md)
- [CLI Reference](./docs/cli-reference.md)
- [Tool Development](./docs/components.md)
- [Publishing Tools](./docs/publishing.md)
- [SDK Reference](./docs/sdk-reference.md)

## Development

### Running CI Checks Locally

This project uses [just](https://github.com/casey/just) for task automation:

```bash
# Install just
cargo install just

# Run all CI checks
just ci

# Development workflow
just dev        # Format and lint
just test-all   # Run all tests
just pre-push   # Full check before pushing

# See all commands
just --list
```

## Contributing

Contributions are welcome! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

## License

Apache-2.0