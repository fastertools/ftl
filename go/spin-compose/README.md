# spin-compose

Infrastructure as Code for WebAssembly - A clean, production-ready Go implementation.

## Overview

spin-compose is a modern Infrastructure as Code tool for WebAssembly applications built with Spin. It allows you to define, synthesize, and manage Spin applications using high-level constructs and CUE-based configuration synthesis.

## Features

- **Clean Architecture**: Built with Go using industry best practices
- **Native CUE Integration**: Uses the CUE Go API for schema validation and synthesis
- **Embedded Schemas**: No external dependencies - everything is bundled in the binary
- **Beautiful CLI**: Colorful, intuitive command-line interface using Cobra
- **MCP Construct**: Flagship Model Context Protocol application pattern
- **Single Binary**: Zero-dependency distribution
- **Production Ready**: Comprehensive tests, elegant error handling

## Installation

### From Source

```bash
git clone https://github.com/fastertools/ftl-cli.git
cd ftl-cli/go/spin-compose
make build
```

### Using Go Install

```bash
go install github.com/fastertools/ftl-cli/go/spin-compose@latest
```

## Quick Start

### 1. Initialize a new project

```bash
spin-compose init my-mcp-app --template mcp
cd my-mcp-app
```

### 2. Edit your configuration

```yaml
# spinc.yaml
name: my-mcp-app
version: 1.0.0
description: My MCP application

auth:
  enabled: true
  issuer: https://auth.example.com
  audience:
    - api.example.com

mcp:
  gateway: ghcr.io/fastertools/mcp-gateway:latest
  authorizer: ghcr.io/fastertools/mcp-authorizer:latest
  validate_arguments: false

components:
  my-tool:
    source: ./build/tool.wasm
    route: /tool
```

### 3. Synthesize and run

```bash
spin-compose synth       # Generate spin.toml
spin up                  # Start your application
```

## Commands

- **`init`** - Initialize a new project with templates
- **`synth`** - Synthesize spin.toml from configuration
- **`validate`** - Validate configuration against schema
- **`diff`** - Show what would change in the manifest
- **`construct`** - Manage high-level constructs

## Architecture

### Project Structure

```
go/spin-compose/
├── cmd/                    # CLI commands
│   ├── root.go            # Root command and styling
│   ├── init.go            # Project initialization
│   ├── synth.go           # Configuration synthesis
│   ├── validate.go        # Configuration validation
│   ├── diff.go            # Manifest diffing
│   └── construct.go       # Construct management
├── internal/
│   ├── schema/            # Embedded CUE schemas
│   │   ├── embedded.go    # Schema embedding
│   │   └── cue/           # CUE schema definitions
│   │       ├── core/      # Core Spin manifest schemas
│   │       └── solutions/ # High-level construct schemas
│   └── synth/             # Synthesis engine
│       └── engine.go      # CUE-based synthesis
├── pkg/
│   └── construct/         # Construct registry
│       └── registry.go    # Available constructs
├── main.go                # Entry point
├── go.mod                 # Go module
├── Makefile              # Build system
└── README.md             # Documentation
```

### Design Principles

1. **Single Responsibility**: Each package has a clear, focused purpose
2. **Embedded Resources**: All schemas are embedded for zero-dependency distribution
3. **Native CUE**: Uses CUE's Go API directly, not external CLI tools
4. **Elegant Error Handling**: Comprehensive error messages with context
5. **Beautiful Output**: Colorful, user-friendly CLI feedback

## MCP Construct

The Model Context Protocol (MCP) construct is our flagship example, demonstrating how to build sophisticated application patterns with minimal configuration.

### Features

- JWT-based authentication with configurable providers
- MCP gateway for tool orchestration
- Automatic component discovery and routing
- Built-in security and validation
- Scalable multi-tool architecture

### Example Configuration

```yaml
name: ai-assistant
description: AI assistant with tool integration

auth:
  enabled: true
  issuer: https://auth.provider.com
  audience: [api.example.com]
  required_scopes: "read:tools write:tools"

mcp:
  validate_arguments: true
  gateway: ghcr.io/fastertools/mcp-gateway:v1.2.0
  authorizer: ghcr.io/fastertools/mcp-authorizer:v1.2.0

components:
  weather-tool:
    source: ./tools/weather.wasm
    route: /weather
    environment:
      API_KEY: "{{ weather_api_key }}"
  
  calculator:
    source: ghcr.io/example/calculator:latest
    route: /calc

variables:
  weather_api_key:
    required: true
  log_level: info
```

## Development

### Prerequisites

- Go 1.23 or later
- Make

### Building

```bash
make build          # Build binary
make test           # Run tests
make lint           # Run linter
make qa             # Run all quality checks
make build-all      # Cross-compile for all platforms
```

### Testing

```bash
make test                # Run all tests
make test-coverage      # Run tests with coverage report
```

### Code Quality

```bash
make fmt            # Format code
make vet            # Run go vet
make lint           # Run golangci-lint
make qa             # Run all quality checks
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run `make qa` to ensure quality
5. Submit a pull request

## License

This project is licensed under the MIT License. See the LICENSE file for details.

## Philosophy

spin-compose embodies the philosophy of artisanal software:

- **Every line deliberate**: Clean, purposeful code without cruft
- **Every feature essential**: No bloat, only what adds real value
- **Production-first**: Built for real-world use from day one
- **Beautiful by default**: Elegant CLI experience and clean architecture

This is not a prototype or proof-of-concept. This is production software, crafted with care.