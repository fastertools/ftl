<div align="center">

# `ftl`

Build and deploy Model Context Protocol (MCP) servers on WebAssembly

[![CI](https://github.com/fastertools/ftl-cli/actions/workflows/ci.yml/badge.svg)](https://github.com/fastertools/ftl-cli/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.87+-orange.svg)](https://www.rust-lang.org)
[![WebAssembly](https://img.shields.io/badge/WebAssembly-compatible-purple.svg)](https://webassembly.org/)

[Docs](./docs/introduction.md) | [Contributing](./CONTRIBUTING.md) | [Security](./SECURITY.md) | [Releases](https://github.com/fastertools/ftl-cli/releases)

</div>

FTL is a command-line tool that wraps [Fermyon Spin](https://www.fermyon.com/spin) to simplify building and deploying [Model Context Protocol](https://modelcontextprotocol.io) (MCP) servers as WebAssembly components. It uses the [wasmcp](https://github.com/fastertools/wasmcp) templates and SDKs to provide a streamlined workflow for creating, testing, and deploying MCP servers on the Fermyon Akamai platform.

## Quick Start

```bash
# Install FTL
cargo install ftl-cli

# Create a new project
ftl init my-assistant
cd my-assistant

# Add a component
ftl add weather-tool --language typescript

# Start development server with auto-rebuild
ftl watch

# Run tests
ftl test

# Build and deploy
ftl build --release
ftl publish
```

## Key Features

- **Component-First Architecture**: Build MCP servers as reusable WebAssembly components
- **Multi-Language Support**: Write components in Rust, TypeScript, or JavaScript  
- **Registry Publishing**: Share components via OCI registries (GitHub, Docker Hub)
- **Project Composition**: Combine multiple MCP components into a single deployable unit
- **Automatic Dependency Management**: Tools like cargo-component installed on-demand
- **Hot Reload Development**: Auto-rebuild on file changes with `ftl watch`
- **Edge Deployment**: Deploy anywhere Spin runs

## Creating MCP Projects

### TypeScript Example

```bash
# Create project and add TypeScript component
ftl init my-project
cd my-project
ftl add my-tool --language typescript
```

```typescript
// my-tool/src/index.ts
import { createHandler } from 'wasmcp';
import { tools, resources, prompts } from './features.js';

export const handler = createHandler({
    tools,     // Your MCP tools
    resources, // Your MCP resources  
    prompts    // Your MCP prompts
});
```

### Rust Example

```bash
# Create project and add Rust component
ftl init my-project
cd my-project
ftl add my-tool --language rust
```

```rust
// my-tool/src/lib.rs
use wasmcp::*;

create_handler!(
    tools: get_tools,
    resources: get_resources,
    prompts: get_prompts
);

fn get_tools() -> Vec<Tool> {
    vec![
        tool!("my_tool", "Tool description", schema, execute_tool)
    ]
}
```

## Component Development Workflow

### 1. Development
```bash
# From component directory
ftl build           # Build the component
ftl test            # Run component tests
ftl watch           # Auto-rebuild on changes

# From project root (with spin.toml)
ftl build           # Build all components
ftl up --port 3000  # Run the composed application
```

### 2. Publishing
```bash
# Publish to GitHub Container Registry
ftl publish --tag v1.0.0

# Publish to Docker Hub  
ftl publish --registry docker.io --tag latest
```

### 3. Composition
```bash
# Create a project composed of multiple components
ftl init my-assistant
cd my-assistant

# Add components with custom routes
ftl add weather-tool --language typescript --route /weather
ftl add github-tool --language rust --route /github
ftl add calculator --language javascript --route /calc

# Each component gets its own MCP endpoint
# /weather/mcp - Weather tool MCP endpoint
# /github/mcp  - GitHub tool MCP endpoint  
# /calc/mcp    - Calculator MCP endpoint

# Run the composed project
ftl watch  # Development with auto-rebuild
ftl up     # Production mode
```

### 4. Deployment
```bash
# Deploy to FTL
ftl deploy

# Or use Spin directly
spin deploy
```

## Architecture

FTL leverages the WebAssembly component model and Spin platform:

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│   MCP Client    │────▶│  Spin Runtime   │────▶│  MCP Component  │
│   (AI Agent)    │     │  (HTTP Router)  │     │ (WASM Module)   │
└─────────────────┘     └─────────────────┘     └─────────────────┘
                               │
                               ├── /weather/mcp ──▶ Weather Component (TypeScript)
                               ├── /github/mcp  ──▶ GitHub Component (Rust)
                               └── /calc/mcp    ──▶ Calculator Component (JavaScript)
```

### Project Structure

```
my-assistant/
├── spin.toml           # Spin manifest (project root)
├── weather-tool/       # TypeScript component
│   ├── ftl.toml       # Component metadata
│   ├── Makefile       # Build automation
│   └── handler/       # Component source
│       ├── package.json
│       └── src/
├── github-tool/       # Rust component  
│   ├── ftl.toml
│   ├── Makefile
│   └── handler/
│       ├── Cargo.toml
│       └── src/
└── calculator/        # JavaScript component
    ├── ftl.toml
    ├── Makefile
    └── handler/
```

Each component:
- Is a standalone WebAssembly module
- Implements the MCP protocol
- Can be developed and tested independently
- Can be composed with other components
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
  - cargo-component (for Rust components)

## Documentation

- [Getting Started Guide](./docs/introduction.md)
- [CLI Reference](./docs/cli-reference.md)
- [Component Development](./docs/components.md)
- [Publishing Components](./docs/publishing.md)
- [Project Composition](./docs/composition.md)
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