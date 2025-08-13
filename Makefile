.PHONY: all ci fmt-check lint test coverage coverage-html fmt fix-clippy fix dev pre-push build-release default install clean

# Default to showing help
default:
	@echo "Available targets:"
	@echo "  all           - Build everything (ftl, spin-compose, components)"
	@echo "  ci            - Run all CI checks"
	@echo "  fmt-check     - Check formatting"
	@echo "  lint          - Run clippy"
	@echo "  test          - Run tests"
	@echo "  coverage      - Run tests with coverage"
	@echo "  coverage-html - Generate HTML coverage report"
	@echo "  fmt       	   - Fix formatting"
	@echo "  fix-clippy    - Fix clippy warnings"
	@echo "  fix           - Fix everything"
	@echo "  dev           - Quick dev check"
	@echo "  pre-push      - Pre-push checks"
	@echo "  build-release - Build release"
	@echo "  install       - Install binaries to /usr/local/bin"
	@echo "  clean         - Clean all build artifacts"

# Run all CI checks
ci:
	@echo "🔍 Running CI checks..."
	@$(MAKE) fmt-check
	@$(MAKE) lint
	@$(MAKE) test
	@echo "✅ All CI checks passed!"

# Check formatting
fmt-check:
	cargo fmt --all -- --check

# Run clippy
lint:
	cargo clippy --all-targets --all-features --workspace -- -D warnings

# Run tests
test:
	cargo nextest run

test-all: test
	cd components/mcp-authorizer && spin build && spin test
	cd components/mcp-gateway && spin build && spin test

# Run tests with coverage
# Note: Spin components (ftl-mcp-*) are excluded as they require WASM coverage tooling
coverage:
	cargo llvm-cov nextest --workspace --exclude ftl-cli --exclude ftl-sdk-macros --ignore-filename-regex '(test_helpers|api_client|deps)\.rs|sdk/rust-macros|components/mcp-'

# Generate HTML coverage report
# Note: Spin components (ftl-mcp-*) are excluded as they require WASM coverage tooling
coverage-open:
	cargo llvm-cov nextest --workspace --exclude ftl-cli --exclude ftl-sdk-macros --ignore-filename-regex '(test_helpers|api_client|deps)\.rs|sdk/rust-macros|components/mcp-' --open

# Fix formatting
fmt:
	cargo fmt --all

# Fix clippy warnings
fix-clippy:
	cargo clippy --all-targets --all-features --workspace --fix --allow-dirty --allow-staged

# Fix everything
fix:
	@$(MAKE) fmt
	@$(MAKE) fix-clippy

# Quick dev check
dev:
	@$(MAKE) fmt
	@$(MAKE) lint

# Pre-push checks
pre-push:
	@$(MAKE) fix
	@$(MAKE) test

build:
	cargo build

build-all: build
	cargo build-wasm

# Build release
build-release:
	cargo build --release

build-all-release: build-release
	cargo build-wasm --release

# Build everything
all: build-ftl build-spin-compose build-components
	@echo "✅ All components built successfully!"

# Build FTL CLI (Rust)
build-ftl:
	@echo "🔨 Building FTL CLI..."
	cargo build --release --bin ftl
	@echo "✅ FTL CLI built: target/release/ftl"

# Build spin-compose (Go)
build-spin-compose:
	@echo "🔨 Building spin-compose..."
	@if command -v go >/dev/null 2>&1; then \
		cd go/spin-compose && go build -o ../../target/release/spin-compose .; \
		echo "✅ spin-compose built: target/release/spin-compose"; \
	else \
		echo "⚠️  Go not installed, skipping spin-compose build"; \
		echo "   Install Go 1.21+ to build spin-compose"; \
	fi

# Build WebAssembly components
build-components:
	@echo "🔨 Building WebAssembly components..."
	@if command -v cargo-component >/dev/null 2>&1; then \
		cargo component build --workspace --release --target wasm32-wasip1; \
		echo "✅ WebAssembly components built"; \
	else \
		echo "⚠️  cargo-component not installed, skipping component build"; \
		echo "   Install with: cargo install cargo-component"; \
	fi

# Install binaries
install: all
	@echo "📦 Installing binaries..."
	@mkdir -p /usr/local/bin
	@if [ -f target/release/ftl ]; then \
		sudo cp target/release/ftl /usr/local/bin/; \
		echo "✅ Installed ftl to /usr/local/bin/ftl"; \
	fi
	@if [ -f target/release/spin-compose ]; then \
		sudo cp target/release/spin-compose /usr/local/bin/; \
		echo "✅ Installed spin-compose to /usr/local/bin/spin-compose"; \
	fi

# Clean all build artifacts
clean:
	@echo "🧹 Cleaning build artifacts..."
	cargo clean
	@if [ -d go/spin-compose ]; then \
		cd go/spin-compose && go clean -cache -testcache -modcache; \
	fi
	@echo "✅ Clean complete"