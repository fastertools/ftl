<div align="center">

# `ftl`

Build and deploy Model Context Protocol (MCP) servers on WebAssembly

[![CI](https://github.com/fastertools/ftl-cli/actions/workflows/ci.yml/badge.svg)](https://github.com/fastertools/ftl-cli/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.87+-orange.svg)](https://www.rust-lang.org)
[![WebAssembly](https://img.shields.io/badge/WebAssembly-compatible-purple.svg)](https://webassembly.org/)

[Docs](./docs/introduction.md) | [Contributing](./CONTRIBUTING.md) | [Security](./SECURITY.md) | [Releases](https://github.com/fastertools/ftl-cli/releases)

</div>

FTL is a developer platform for building and deploying [Model Context Protocol](https://modelcontextprotocol.io) (MCP) servers as WebAssembly components that run on the [Fermyon Spin](https://www.fermyon.com/spin) platform.

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

# Publish to registry
ftl publish
```

## Key Features

- **Component-First Architecture**: Build MCP servers as reusable WebAssembly components
- **Multi-Language Support**: Write components in Rust, TypeScript, or JavaScript  
- **Registry Publishing**: Share components via OCI registries (GitHub, Docker Hub)
- **Project Composition**: Combine multiple MCP components into a single Spin project
- **Edge Deployment**: Deploy to FTL or self-host with Spin

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
import { createHandler } from '@fastertools/ftl-sdk';
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
use ftl_sdk::*;

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

## Component Lifecycle

### 1. Development
```bash
ftl build           # Build all components (from project root)
ftl test            # Run tests
ftl up --port 3000  # Run locally
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

# Add components
ftl add weather-tool --language typescript
ftl add github-tool --language rust
ftl add my-custom-tool --language javascript

# Run the composed project
ftl up --build
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
│   MCP Client    │────▶│  Spin Project   │────▶│  MCP Component  │
│   (AI Agent)    │     │  (HTTP Router)  │     │ (WASM Module)   │
└─────────────────┘     └─────────────────┘     └─────────────────┘
                               │
                               ├────▶ Component 1 (Rust)
                               ├────▶ Component 2 (TypeScript)  
                               └────▶ Component 3 (JavaScript)
```

Each component:
- Is a standalone WebAssembly module
- Implements the MCP protocol
- Can be developed and tested independently
- Can be composed with other components
- Runs in a secure sandbox

## Prerequisites

- **Rust toolchain** (for FTL CLI)
- **Language toolchains**:
  - Rust: cargo with wasm32-wasip1 target
  - TypeScript/JavaScript: Node.js 20+
- **wkg** for publishing ([install](https://github.com/bytecodealliance/wasm-pkg-tools))
- **Spin** (auto-installed by FTL)

## Documentation

- [Getting Started Guide](./docs/introduction.md)
- [Component Development](./docs/components.md)
- [Publishing Components](./docs/publishing.md)
- [Project Composition](./docs/composition.md)
- [API Reference](./docs/api.md)

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