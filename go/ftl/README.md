# FTL - Faster Than Light CLI

A comprehensive toolkit for building, composing, and deploying AI tools on WebAssembly. FTL provides everything you need to create secure, high-performance MCP (Model Context Protocol) servers that can run anywhere.

## Features

- **Multi-Format Configuration**: Choose between YAML, JSON, Go, or CUE for defining your applications
- **Intelligent Synthesis**: Automatically generates complex Spin manifests from simple configurations
- **MCP Integration**: Built-in support for MCP gateway and authorization
- **Platform Deployment**: Deploy to Fermyon Cloud with a single command
- **Component Management**: Easy addition and removal of WebAssembly components
- **CUE-Powered**: Advanced configuration validation and transformation using CUE

## Installation

```bash
go install github.com/fastertools/ftl-cli/go/ftl@latest
```

Or build from source:
```bash
git clone https://github.com/fastertools/ftl-cli.git
cd ftl-cli/go/ftl
go build -o ftl .
```

## Quick Start

1. **Initialize a new project:**
```bash
ftl init my-app
# Choose your preferred format: yaml, json, go, or cue
```

2. **Add components:**
```bash
ftl component add oci ghcr.io/example/tool:latest --name my-tool
```

3. **Build and run:**
```bash
ftl build
ftl up
```

## Architecture

FTL uses a pure CUE-based transformation pipeline:

```
User Config → CUE Patterns → Spin Manifest
```

- **User Config**: Simple, declarative configuration (YAML, JSON, CUE, or Go)
- **CUE Patterns**: Powerful transformation and validation rules
- **Spin Manifest**: Complete Spin v3 manifest with all components and configuration

## Commands

- `ftl init` - Initialize a new FTL project
- `ftl build` - Build the application and generate spin.toml
- `ftl up` - Run the application locally
- `ftl deploy` - Deploy to Fermyon Cloud
- `ftl component` - Manage application components
- `ftl synth` - Synthesize spin.toml from various formats
- `ftl auth` - Manage authentication
- `ftl registry` - Registry operations
- `ftl test` - Run application tests

## Examples

See the [examples](examples/) directory for working examples in different configuration formats.

## Development

### Running Tests
```bash
go test ./...
```

### Building
```bash
go build -o ftl .
```

## License

[License information here]