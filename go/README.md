# FTL CLI - Go Implementation

The Fast Tools Language (FTL) CLI is a comprehensive toolkit for building, composing, and deploying AI tools on WebAssembly.

## 🚀 Quick Start

### Prerequisites

- Go 1.21 or later
- Spin CLI (for running applications)
  - Install from: https://developer.fermyon.com/spin/install

### Installation

```bash
# Clone the repository (if not already)
git clone https://github.com/fastertools/ftl-cli
cd ftl-cli/go

# Install FTL CLI
make install

# Or install both FTL and spin-compose
make install-all

# Verify installation
ftl --version
```

### Alternative: Run without installing

```bash
# Run directly with go run
make run ARGS="--help"

# Or
cd ftl && go run . --help
```

## 📦 Build from Source

```bash
# Build binaries to ./bin/
make build

# Or build everything
make all

# Run the binary directly
./bin/ftl --help
```

## 🛠️ Usage

### Initialize a new project

```bash
# Create a new MCP (Model Context Protocol) project
ftl init my-mcp-tool

# Or specify a template
ftl init my-app --template mcp    # MCP server with auth support
ftl init my-app --template basic  # Basic Spin application
ftl init my-app --template empty  # Minimal configuration

# Non-interactive mode
ftl init my-app --no-interactive
```

### Build your application

```bash
# Build the application
ftl build

# Run locally
ftl up

# Run with auto-reload on changes
ftl up --watch

# Deploy to production
ftl deploy --environment production
```

### Manage components

```bash
# Add a new component
ftl component add my-tool --language rust

# List components
ftl component list

# Remove a component
ftl component remove my-tool
```

### Registry operations

```bash
# Push to registry
ftl registry push ghcr.io/myorg/my-app:latest

# Pull from registry
ftl registry pull ghcr.io/myorg/my-app:latest
```

## 🧪 Testing

```bash
# Run all tests
make test

# Run tests with coverage
make coverage

# Quick test run
make test-short

# Test specific package
cd shared && go test ./... -v -cover
```

## 📊 Current Test Coverage

- `shared/config`: 94.7% ✅
- `shared/spin`: 90.9% ✅  
- `ftl/cmd`: 49.0% 🟡

## 🏗️ Project Structure

```
go/
├── ftl/                 # FTL CLI implementation
│   ├── cmd/            # Command implementations
│   └── main.go         # Entry point
├── spin-compose/        # Infrastructure as Code tool
│   ├── cmd/            # Commands
│   └── internal/       # CUE synthesis engine
├── shared/             # Shared libraries
│   ├── config/         # Configuration types
│   └── spin/           # Spin executor
└── Makefile            # Build automation
```

## 🛠️ Development

```bash
# Format code
make fmt

# Run linters
make vet

# Tidy dependencies
make tidy

# Clean build artifacts
make clean
```

## 🤝 Contributing

This is a pre-release greenfield project. We're building "Rails for AI Tools" - making MCP server development trivially easy.

### Design Principles

- **85%+ test coverage** - Ironclad GNU quality
- **Clean architecture** - No technical debt
- **Modern Go** - Latest patterns and practices
- **Modular design** - Shared libraries for reuse

## 🚦 Status

The Go implementation is functional with core commands working:
- ✅ Project initialization with templates
- ✅ Build and deployment integration with Spin
- ✅ Component management (stubs)
- ✅ Registry operations
- ✅ Authentication (stubs)

### Next Steps

1. Complete remaining command implementations
2. Improve test coverage to 85%+ across all packages
3. Fix spin-compose CUE integration tests
4. Migrate authentication logic from Rust
5. Remove old Rust implementation

## 📝 License

[Your License Here]

## 🔗 Links

- [Spin Documentation](https://developer.fermyon.com/spin)
- [MCP Specification](https://modelcontextprotocol.io)
- [WebAssembly Component Model](https://component-model.bytecodealliance.org)