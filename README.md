<div align="center">

# `ftl`

Fast tools for AI agents

[![CI](https://github.com/fastertools/ftl-cli/actions/workflows/ci.yml/badge.svg)](https://github.com/fastertools/ftl-cli/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.87+-orange.svg)](https://www.rust-lang.org)
[![WebAssembly](https://img.shields.io/badge/WebAssembly-compatible-purple.svg)](https://webassembly.org/)

[Docs](./docs/introduction.md) | [Contributing](./CONTRIBUTING.md) | [Security](./SECURITY.md) | [Releases](https://github.com/fastertools/ftl-cli/releases)

</div>

FTL is a platform on the network edge that makes it easy to deploy and manage secure [Model Context Protocol](https://modelcontextprotocol.io) servers with SOTA performance. It builds on the [WebAssembly Component Model](https://component-model.bytecodealliance.org/design/why-component-model.html) via [Spin](https://github.com/spinframework/spin) to provide a *just works* DX for authoring MCP tools in any source lanaguage and running them natively on the [most distributed](https://www.akamai.com/why-akamai/global-infrastructure) edge network.

## Why FTL?

When an AI agent connects to MCP tools over the network, every tool call adds latency. For agents deployed in realtime and other performance sensitive applications, that latency adds up to impact the performance of the whole system. FTL solves this problem by providing:

- **Sub-millisecond cold starts**: Backed by [Fermyon Wasm Functions](https://www.fermyon.com/wasm-functions) running on Akamai's globally distributed edge network. Agents deployed anywhere can instanly access their networked tools with almost no latency.
- **Mix source languages within one MCP server**: Write your MCP tools in Rust, TypeScript, Python, Go, C, and [more](https://component-model.bytecodealliance.org/language-support.html). If you can implement a basic HTTP route as a Wasm component, you can run it as an MCP tool with FTL.
- **Tiny artifacts, fast deployments**: WebAssembly binaries are self-contained and often < 1MB vs. 100MB+ containers.
- **Secure by Default**: WebAssembly provides sandboxed tool executions on a provably airtight [security model](https://webassembly.org/docs/security/).
- **Deploy Anywhere**: While FTL provides a managed platform optimized for MCP workloads and management, you can run your FTL-produced wasm components on Fermyon directly, or on Kubernetes, Wasmtime, or any WASI-compatible runtime, including your own computer or Docker Desktop.

## Quick Start

```bash
# Install FTL
cargo install ftl-cli

# Set up templates
ftl setup templates

# Create a new project
ftl init my-tools
cd my-tools

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

## Creating tools

<details>
<summary><strong>🦀 Rust example</strong></summary>

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
</details>

<details>
<summary><strong>🟦 TypeScript example</strong></summary>

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
</details>

### Workflow

### 1. Develop
```bash
# From project root (with spin.toml)
ftl build           # Build all tools
ftl test            # Run tests
ftl watch           # Auto-rebuild on changes
ftl up              # Run the MCP server
```

### 2. Plug in to your local MCP Client Configuration
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

### 3. Deploy
```bash
ftl deploy
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

### Required
- **Rust 1.87+** - [Install Rust](https://rustup.rs/)
- **Node.js 20+** (for TypeScript tools) - [Install Node.js](https://nodejs.org/)

### Platform-Specific

<details>
<summary>macOS</summary>

```bash
# Using Homebrew
brew install rust node

# Or install Rust directly
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
</details>

<details>
<summary>Linux</summary>

```bash
# Ubuntu/Debian
sudo apt update
sudo apt install build-essential pkg-config libssl-dev
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Node.js via NodeSource
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt-get install -y nodejs
```
</details>

<details>
<summary>Windows</summary>

- Install [Visual Studio C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
- Install [Rust for Windows](https://rust-lang.org/tools/install)
- Install [Node.js for Windows](https://nodejs.org/en/download/)
- Or use [WSL2](https://docs.microsoft.com/en-us/windows/wsl/install) for Linux environment
</details>

### Auto-Installed by FTL
- ✅ Spin runtime (prompted on first use)
- ✅ cargo-component (for Rust tools)
- ✅ wasm32-wasip1 target

## Documentation

### Getting Started
- 📖 [Introduction](./docs/introduction.md) - Overview and concepts
- 🚀 [Quick Start](./docs/quickstart.md) - 5-minute tutorial
- 🛠️ [Getting Started Guide](./docs/getting-started.md) - Detailed setup

### Development
- 🔧 [Tool Development](./docs/developing-tools.md) - Building MCP tools
- 📚 [SDK Reference](./docs/sdk-reference.md) - API documentation
- 🏗️ [Architecture](./docs/architecture.md) - System design
- 📡 [API Reference](./docs/api.md) - MCP protocol details

### Operations
- 🚢 [Deployment Guide](./docs/deployment.md) - Production deployment
- 📊 [Monitoring](./docs/monitoring.md) - Observability setup
- 🔒 [Security](./docs/security.md) - Security best practices
- ⚡ [Performance](./docs/performance.md) - Optimization guide

### Reference
- 📋 [CLI Reference](./docs/cli-reference.md) - All commands
- 🐛 [Troubleshooting](./docs/troubleshooting.md) - Common issues
- 📦 [Publishing](./docs/publishing.md) - Share your tools

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

## Performance

FTL delivers exceptional performance through WebAssembly optimization:

| Metric | FTL | Traditional Container |
|--------|-----|----------------------|
| Cold Start | <50ms | 500-2000ms |
| Memory Usage | 5-10MB | 50-200MB |
| Bundle Size | <1MB | 50MB+ |
| Build Time | 2-5s | 30-60s |

## Community & Support

- 💬 [GitHub Discussions](https://github.com/fastertools/ftl-cli/discussions) - Ask questions
- 🐛 [Issue Tracker](https://github.com/fastertools/ftl-cli/issues) - Report bugs
- 📺 [YouTube Channel](https://youtube.com/@fastertools) - Video tutorials
- 🐦 [Twitter/X](https://twitter.com/fastertools) - Updates and tips

## Contributing

We love contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Quick Contribution Guide

```bash
# Fork and clone
git clone https://github.com/YOUR-USERNAME/ftl-cli
cd ftl-cli

# Install dependencies
just install-deps

# Run tests
just test-all

# Make your changes and test
just dev
```

## License

Apache-2.0 - see [LICENSE](LICENSE) for details.

## Acknowledgments

FTL is built on top of these excellent projects:
- [Fermyon Spin](https://github.com/fermyon/spin) - WebAssembly runtime
- [Model Context Protocol](https://modelcontextprotocol.io) - AI tool protocol
- [WebAssembly](https://webassembly.org) - Portable binary format

---

<p align="center">
  Made with ❤️ by the <a href="https://github.com/fastertools">Faster Tools</a> team
</p>