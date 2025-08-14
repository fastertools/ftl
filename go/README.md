# FTL CLI - Faster Tool Layer

A production-ready CLI for building and deploying MCP (Model Context Protocol) tool platforms on WebAssembly.

## 🚀 Features

✅ **Complete CLI Implementation**
- Full command suite: `init`, `build`, `deploy`, `up`, `test`, `synth`
- Component management: `add`, `list`, `remove`
- Registry operations: `push`, `pull`, `list`
- Authentication support: `login`, `logout`, `status`

✅ **Pure CUE-Based Synthesis**
- Direct CUE transformations (no intermediate layers)
- All business logic in CUE patterns
- Native support for YAML, JSON, CUE, and Go inputs
- Idiomatic Go CDK that uses CUE internally

✅ **Multiple Input Formats**
- **YAML/JSON**: Simple declarative configuration
- **CUE**: Type-safe configuration with validation
- **Go CDK**: Programmatic API with fluent interface
- All formats produce identical output

✅ **Production Ready**
- 90.5% test coverage
- Zero lint warnings
- Comprehensive error handling
- Smart component handling (registry vs local sources)

## 📦 Installation

### Prerequisites

- Go 1.21 or later
- Spin CLI (for running applications)
  - Install from: https://developer.fermyon.com/spin/install

### Install from Source

```bash
# Clone the repository
git clone https://github.com/fastertools/ftl-cli
cd ftl-cli/go

# Install FTL CLI
make install

# Verify installation
ftl --version
```

### Build Binaries

```bash
# Build to ./bin/
make build

# Run directly
./bin/ftl --help
```

## 🎯 Quick Start

### 1. Initialize a New Project

```bash
# Choose your preferred format
ftl init my-platform --format yaml  # or json, cue, go
cd my-platform
```

### 2. Add Components

From registry:
```bash
ftl component add geo --from ghcr.io/bowlofarugula:geo:0.0.1
```

From local source:
```bash
ftl component add my-component --from ./my-component.wasm
```

### 3. Build and Deploy

```bash
ftl build    # Synthesizes and builds
ftl up       # Local development
ftl deploy   # Deploy to production
```

## 🔧 Synthesis Examples

### Go CDK

```go
package main

import (
    "fmt"
    "log"
    "github.com/fastertools/ftl-cli/go/ftl/pkg/synthesis"
)

func main() {
    cdk := synthesis.NewCDK()
    app := cdk.NewApp("my-platform").
        SetVersion("1.0.0").
        SetDescription("My MCP platform")
    
    app.AddComponent("geo").
        FromRegistry("ghcr.io", "bowlofarugula:geo", "0.0.1").
        WithEnv("LOG_LEVEL", "info").
        Build()
    
    // Enable authentication
    app.EnableWorkOSAuth("org_12345")
    
    builtCDK := app.Build()
    manifest, err := builtCDK.Synthesize()
    if err != nil {
        log.Fatal(err)
    }
    fmt.Print(manifest)
}
```

### YAML Configuration

```yaml
application:
  name: my-platform
  version: "1.0.0"
  description: My MCP platform

components:
  - id: geo
    source:
      registry: ghcr.io
      package: "bowlofarugula:geo"
      version: "0.0.1"
    variables:
      LOG_LEVEL: info

# Optional authentication
# access: private
# auth:
#   provider: workos
#   org_id: org_12345
```

### CUE Configuration

```cue
application: {
    name: "my-platform"
    version: "1.0.0"
    description: "My MCP platform"
}

components: [{
    id: "geo"
    source: {
        registry: "ghcr.io"
        package: "bowlofarugula:geo"
        version: "0.0.1"
    }
    variables: {
        LOG_LEVEL: "info"
    }
}]
```

Generate spin.toml:
```bash
# From any format
ftl synth config.yaml > spin.toml
ftl synth app.go > spin.toml
ftl synth platform.cue > spin.toml
```

## 🏗️ Architecture

FTL uses pure CUE for all transformations:

```
User Input (YAML/JSON/CUE/Go)
    ↓ [Parse to CUE Value]
FTL Application Model
    ↓ [CUE Pattern Matching]
spin.toml (WebAssembly Manifest)
```

### Automatic Components

FTL automatically adds required infrastructure:

```
Internet
    ↓
[Public Route: /...]
    ↓
[MCP Authorizer] (if auth enabled)
    ↓
[MCP Gateway] (always present)
    ↓
[Your Tools] (geo, fluid, etc.)
```

## 📖 Commands Reference

### Project Management
```bash
ftl init <name>           # Initialize new project
ftl build                 # Build the application
ftl up                    # Run locally
ftl deploy                # Deploy to production
ftl test [path]           # Run tests
```

### Component Management
```bash
ftl component add <name> --from <source>  # Add component
ftl component list                        # List components
ftl component remove <name>               # Remove component
```

### Registry Operations
```bash
ftl registry push <ref>           # Push to registry
ftl registry pull <ref>           # Pull from registry
ftl registry list --registry <r>  # List contents
```

### Authentication
```bash
ftl auth login     # Login to Fermyon Cloud
ftl auth logout    # Logout
ftl auth status    # Check auth status
```

### Synthesis
```bash
ftl synth <file>           # Generate spin.toml
ftl synth <file> -o out.toml  # Write to file
```

## 🧪 Testing

```bash
# Run all tests
make test

# With coverage
make test-coverage

# Specific package
go test ./ftl/cmd -v
```

Current coverage: **90.5%**

## 📁 Project Structure

```
go/
├── ftl/                      # FTL CLI implementation
│   ├── cmd/                  # Command implementations
│   ├── pkg/synthesis/        # Pure CUE synthesis engine
│   │   ├── cdk.go           # Go CDK API
│   │   ├── synthesizer.go   # CUE transformations
│   │   ├── patterns.cue     # Core CUE patterns
│   │   └── helpers.go       # Format detection
│   ├── examples/             # All format examples
│   │   ├── yaml-format/     # YAML example
│   │   ├── json-format/     # JSON example
│   │   ├── cue-format/      # CUE example
│   │   └── go-format/       # Go CDK example
│   └── main.go              # Entry point
│
└── shared/                   # Shared utilities
    ├── spin/                 # Spin CLI wrapper
    ├── auth/                 # Authentication
    └── config/               # Configuration schemas
```

## 📚 Examples

See [ftl/examples/](ftl/examples/) for complete examples in all formats:
- YAML declarative configuration
- JSON declarative configuration  
- CUE type-safe configuration
- Go programmatic configuration

All examples produce identical `spin.toml` output.

## 🛠️ Development

### Prerequisites

- Go 1.21+
- Make
- Spin CLI

### Building

```bash
make all          # Build everything
make test         # Run tests
make lint         # Run linters
make clean        # Clean build artifacts
```

### Code Quality

- Test coverage: 90.5%
- Zero lint warnings
- Pure CUE transformations
- Smart component handling

## 🤝 Contributing

This is a production-ready system with GNU-level completeness. When contributing:

1. Maintain test coverage above 85%
2. No lint warnings
3. No TODOs or stubs in production code
4. Use CUE for all transformations
5. Follow established patterns

## 📄 License

[MIT License](LICENSE)

## 🆘 Support

For issues and feature requests, please use the GitHub issue tracker.

## 🎉 Acknowledgments

Built on top of:
- [Fermyon Spin](https://www.fermyon.com/spin) - WebAssembly platform
- [CUE](https://cuelang.org/) - Configuration language
- [MCP](https://modelcontextprotocol.io/) - Model Context Protocol