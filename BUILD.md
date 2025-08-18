# Building FTL

This monorepo embraces a polyglot philosophy, using the best tool for each job:
- **Go** for the CLI (excellent for command-line tools, great ecosystem)
- **Rust** for WebAssembly components (performance, safety, WASM support)
- **Multiple languages** for SDKs (meet developers where they are)

## Prerequisites

### Required
- **Go** 1.21+ for the FTL CLI
- **Rust** 1.75+ with cargo for WebAssembly components
- **Make** for build automation

### Optional but Recommended
- **cargo-component** for building WebAssembly components
  ```bash
  cargo install cargo-component
  ```
- **Spin** CLI for testing WebAssembly applications
  ```bash
  curl -fsSL https://developer.fermyon.com/downloads/install.sh | bash
  ```

## Quick Start

Build everything:
```bash
make all
```

This will:
1. Build the FTL CLI (Go)
2. Build WebAssembly components (Rust)

## Development Workflow

### Quick development cycle:
```bash
make dev
```
This runs formatting, builds the CLI, and runs tests.

### Run tests:
```bash
make test
```

### Format code:
```bash
make fmt
```

### Check code quality:
```bash
make lint
```

## Individual Component Builds

### FTL CLI (Go)
```bash
# Using make (recommended)
make build

# Or directly with Go
go build -o bin/ftl ./cmd/ftl
```

The CLI binary will be at `bin/ftl`

### WebAssembly Components
```bash
# Using make
make build-components

# Or directly with cargo-component
cargo component build --workspace --release --target wasm32-wasip1
```

Components will be in `target/wasm32-wasip1/release/`

### SDKs

Each SDK can be built/tested independently:

**Rust SDK:**
```bash
cd sdk/rust
cargo build
cargo test
```

**Python SDK:**
```bash
cd sdk/python
pip install -e .
pytest
```

**TypeScript SDK:**
```bash
cd sdk/typescript
npm install
npm run build
npm test
```

**Go SDK:**
```bash
cd sdk/go
go build ./...
go test ./...
```

## Installation

Install the FTL CLI to your system:

```bash
# Install to ~/.local/bin (user installation)
make install

# Install to /usr/local/bin (system-wide, requires sudo)
make install-system
```

## Testing

Run all tests:
```bash
make test
```

Run specific test suites:
```bash
# Test Go CLI only
make test-cli

# Test Rust SDKs only
make test-sdk

# Test WebAssembly components
make test-components
```

Generate coverage reports:
```bash
make coverage
# Open coverage-go.html in your browser
```

## Troubleshooting

### Missing cargo-component
If you see warnings about cargo-component not being installed:
```bash
cargo install cargo-component
```

### Missing Spin CLI
If you see warnings about spin not being installed:
```bash
curl -fsSL https://developer.fermyon.com/downloads/install.sh | bash
```

### Go module issues
If you encounter Go module problems:
```bash
go mod tidy
go mod download
```

### Rust toolchain issues
Ensure you have the correct Rust toolchain:
```bash
rustup update
rustup target add wasm32-wasip1
```

## Release Builds

For optimized release builds:

```bash
# Build everything in release mode
make all

# The CLI will have version information embedded:
./bin/ftl --version
```

## Cross-Platform Building

The project supports cross-platform builds:

### macOS (Apple Silicon or Intel)
All components build natively on macOS.

### Linux (x86_64 or ARM64)
All components build natively on Linux.

### Windows
- Go CLI builds natively
- Rust components require WSL2 or Windows-native Rust toolchain
- Use Git Bash or WSL2 for make commands

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development guidelines and contribution process.