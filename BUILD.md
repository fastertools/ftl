# Building FTL CLI

This monorepo contains both Rust and Go components that work together to provide the complete FTL experience.

## Prerequisites

### Required
- **Rust** 1.75+ with cargo
- **Go** 1.21+ for spin-compose
- **Make** for build automation

### Optional
- **cargo-component** for building WebAssembly components
- **spin** CLI for local testing

## Quick Build

Build everything:
```bash
make all
```

This will:
1. Build the FTL CLI (Rust)
2. Build spin-compose (Go)  
3. Build MCP components (WebAssembly)

## Individual Builds

### FTL CLI (Rust)
```bash
cargo build --release
# Binary at: target/release/ftl
```

### spin-compose (Go)
```bash
cd go/spin-compose
go build -o ../../target/release/spin-compose
# Or use make:
make build
```

### MCP Components (WebAssembly)
```bash
cargo component build --workspace --release --target wasm32-wasip1
```

## Installation

Install to system:
```bash
make install
```

This installs:
- `ftl` to `/usr/local/bin/ftl`
- `spin-compose` to `/usr/local/bin/spin-compose`

## Development

### Running Tests
```bash
# Rust tests
cargo test

# Go tests  
cd go/spin-compose && go test ./...

# Integration tests
make test-integration
```

### Code Quality
```bash
# Rust
cargo clippy
cargo fmt --check

# Go
cd go/spin-compose
go fmt ./...
go vet ./...
golangci-lint run
```

## Cross-Compilation

### spin-compose for multiple platforms
```bash
cd go/spin-compose
make build-all  # Builds for linux, darwin, windows
```

### FTL for multiple platforms
```bash
# Linux
cargo build --release --target x86_64-unknown-linux-gnu

# macOS
cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin

# Windows
cargo build --release --target x86_64-pc-windows-msvc
```

## Docker Build

Build everything in Docker:
```bash
docker build -t ftl-cli .
docker run --rm ftl-cli ftl --version
docker run --rm ftl-cli spin-compose --version
```

## Release Build

Create release artifacts:
```bash
make release VERSION=v1.0.0
```

This creates:
- `dist/ftl-v1.0.0-linux-amd64.tar.gz`
- `dist/ftl-v1.0.0-darwin-amd64.tar.gz`
- `dist/ftl-v1.0.0-darwin-arm64.tar.gz`
- `dist/ftl-v1.0.0-windows-amd64.zip`
- `dist/spin-compose-v1.0.0-linux-amd64.tar.gz`
- `dist/spin-compose-v1.0.0-darwin-amd64.tar.gz`
- `dist/spin-compose-v1.0.0-darwin-arm64.tar.gz`
- `dist/spin-compose-v1.0.0-windows-amd64.zip`

## Troubleshooting

### Go module issues
```bash
cd go/spin-compose
go mod tidy
go mod download
```

### Rust build issues
```bash
cargo clean
cargo update
cargo build
```

### Missing wasm32-wasip1 target
```bash
rustup target add wasm32-wasip1
cargo install cargo-component
```