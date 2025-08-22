# FTL - Polyglot WebAssembly MCP Platform
# Main orchestration Makefile

.PHONY: all help build test clean install

# Default to showing help
help:
	@echo "FTL - Fast, polyglot toolkit for building MCP tools on WebAssembly"
	@echo ""
	@echo "Available targets:"
	@echo ""
	@echo "  Core Commands:"
	@echo "    build         - Build FTL CLI (Go)"
	@echo "    test          - Run all tests (Go CLI + Rust SDKs)"
	@echo "    install       - Install FTL CLI to system"
	@echo "    clean         - Clean all build artifacts"
	@echo ""
	@echo "  Component Commands:"
	@echo "    build-components  - Build WebAssembly components"
	@echo "    test-components   - Test WebAssembly components"
	@echo ""
	@echo "  Development Commands:"
	@echo "    generate-api  - Generate API client from OpenAPI spec"
	@echo "    fmt           - Format all code (Go + Rust)"
	@echo "    lint          - Lint all code (Go + Rust)"
	@echo "    coverage      - Generate test coverage reports"
	@echo ""
	@echo "  Quick Commands:"
	@echo "    dev           - Quick development build and test"
	@echo "    all           - Build everything (CLI + components)"

# Generate API client from OpenAPI spec
generate-api:
	@echo "🔄 Generating API client from OpenAPI spec..."
	@oapi-codegen -package api -generate types,client -o internal/api/client.gen.go internal/api/openapi.json
	@echo "✅ API client generated: internal/api/client.gen.go"

# Build FTL CLI (Go)
build:
	@echo "🔨 Building FTL CLI..."
	@echo "📝 Generating embedded files..."
	@go generate ./...
	@go build -ldflags "-X github.com/fastertools/ftl/internal/cli.version=$$(git describe --tags --always) \
		-X github.com/fastertools/ftl/internal/cli.commit=$$(git rev-parse --short HEAD) \
		-X github.com/fastertools/ftl/internal/cli.buildDate=$$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
		-o bin/ftl ./cmd/ftl
	@echo "✅ FTL CLI built: bin/ftl"

# Test Go CLI
test-cli:
	@echo "🧪 Testing FTL CLI..."
	@go test -v ./...

# Test Rust SDKs
test-sdk:
	@echo "🧪 Testing Rust SDKs..."
	@cd sdk/rust && cargo test --target $(shell rustc -vV | sed -n 's/host: //p')
	@cd sdk/rust-macros && cargo test --target $(shell rustc -vV | sed -n 's/host: //p')

# Build WebAssembly components
build-components:
	@echo "🔨 Building WebAssembly components..."
	@# With .cargo/config.toml, wasm32-wasip1 is the default target
	@cargo build --workspace --release
	@echo "✅ Components built in target/wasm32-wasip1/release/"

# Test WebAssembly components
test-components:
	@echo "🧪 Testing WebAssembly components..."
	@if command -v spin >/dev/null 2>&1; then \
		cd components/mcp-authorizer && spin test; \
		cd ../mcp-gateway && spin test; \
	else \
		echo "⚠️  spin not installed"; \
		echo "   Install from: https://developer.fermyon.com/spin/install"; \
	fi

# Run all tests
test: test-cli test-sdk test-components

# Format all code
fmt:
	@echo "🎨 Formatting code..."
	@echo "  Formatting Go code..."
	@go fmt ./...
	@echo "  Formatting Rust code..."
	@cargo fmt --all

# Lint all code
lint:
	@echo "🔍 Linting code..."
	@echo "  Linting Go code..."
	@go vet ./...
	@if command -v golangci-lint >/dev/null 2>&1; then \
		golangci-lint run ./...; \
	fi
	@echo "  Linting Rust code..."
	@cargo clippy --all-targets --all-features -- -D warnings

# Generate coverage reports
coverage:
	@echo "📊 Generating coverage reports..."
	@echo "  Go coverage..."
	@go test -coverprofile=coverage.out ./...
	@go tool cover -html=coverage.out -o coverage-go.html
	@go tool cover -func=coverage.out | tail -1
	@echo "  Coverage report: coverage-go.html"

# Install FTL CLI
install: build
	@echo "📦 Installing FTL CLI..."
	@mkdir -p ~/.local/bin
	@cp bin/ftl ~/.local/bin/
	@echo "✅ Installed to ~/.local/bin/ftl"
	@echo ""
	@echo "Make sure ~/.local/bin is in your PATH:"
	@echo '  export PATH=$$HOME/.local/bin:$$PATH'

# Install to system location (requires sudo)
install-system: build
	@echo "📦 Installing FTL CLI to system..."
	@sudo cp bin/ftl /usr/local/bin/
	@echo "✅ Installed to /usr/local/bin/ftl"

# Clean all build artifacts
clean:
	@echo "🧹 Cleaning build artifacts..."
	@rm -rf bin/ target/ coverage*.out coverage*.html
	@go clean -cache -testcache
	@cargo clean
	@echo "✅ Clean complete"

# Quick development cycle
dev: fmt build test-cli
	@echo "✅ Development build complete"

# Build everything
all: build build-components
	@echo "✅ All components built successfully"

.DEFAULT_GOAL := help