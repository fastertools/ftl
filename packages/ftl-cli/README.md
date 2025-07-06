# FTL CLI

Build and deploy Model Context Protocol (MCP) servers on WebAssembly.

FTL provides a streamlined developer experience for creating, testing, and deploying MCP components that run on the Fermyon Spin platform.

## Installation

```bash
cargo install ftl-cli
```

Or download pre-built binaries from the [releases page](https://github.com/fastertools/ftl-cli/releases).

## Quick Start

```bash
# Create a new MCP project
ftl init my-assistant
cd my-assistant

# Add a component
ftl add weather-tool --language typescript

# Build the components
ftl build

# Run locally
ftl up

# Publish to registry
ftl publish
```

## Commands

### Project & Component Management

#### `ftl init [name]`
Create a new MCP project for composing components.

Options:
- `--here` - Initialize in current directory

#### `ftl add [name]`
Add a new MCP component to the current project.

Options:
- `--language <lang>` - Language to use (rust, typescript, javascript)
- `--description <desc>` - Component description
- `--route <route>` - HTTP route for the component

#### `ftl build`
Build the component in the current directory.

Options:
- `--release` - Build in release mode
- `--path <path>` - Path to component directory

#### `ftl up`
Run the component locally for development.

Options:
- `--build` - Build before running
- `--port <port>` - Port to serve on (default: 3000)

#### `ftl test`
Run component tests.

Options:
- `--path <path>` - Path to component directory

#### `ftl publish`
Publish component to an OCI registry.

Options:
- `--registry <url>` - Registry URL (default: ghcr.io)
- `--tag <version>` - Version tag to publish

#### `ftl deploy`
Deploy the project to FTL.

Options:
- `-e, --environment <name>` - Target environment

### Configuration

#### `ftl setup templates`
Install or update FTL component templates.

Options:
- `--force` - Force reinstall

#### `ftl setup info`
Show FTL configuration and status.

### Registry Operations

#### `ftl registry list`
List available components (coming soon).

#### `ftl registry search <query>`
Search for components (coming soon).

#### `ftl registry info <component>`
Show component details (coming soon).

## Project Structure

FTL projects follow a standard structure:

```
my-project/
├── spin.toml        # Spin project configuration
├── weather-tool/    # Component directory
│   ├── ftl.toml     # Component manifest
│   ├── Makefile     # Build automation
│   └── src/         # Component source code
│       ├── index.ts # Main entry point
│       └── features.ts # MCP tools/resources
└── calculator/      # Another component
    ├── ftl.toml
    ├── Makefile
    └── src/
        └── lib.rs
```

## Publishing Components

Components are published as OCI artifacts to container registries:

```bash
# Publish to GitHub Container Registry (default)
ftl publish

# Publish with specific version
ftl publish --tag v1.0.0

# Publish to Docker Hub
ftl publish --registry docker.io
```

Published components can be referenced as:
- `ghcr.io/username/component-name:version`
- `docker.io/username/component-name:version`

## Prerequisites

- **Spin**: Automatically installed by FTL if not present
- **wkg**: Required for publishing ([install](https://github.com/bytecodealliance/wasm-pkg-tools))
- **Language toolchains**:
  - Rust: cargo with wasm32-wasi target
  - TypeScript/JavaScript: Node.js 20+

## License

Apache-2.0