.PHONY: build test clean lint check release

# Default target
build:
	spin build

# Run tests
test:
	spin test

# Clean build artifacts
clean:
	cargo clean

# Run linter
lint:
	cargo clippy -- -D warnings

# Format code
format:
	cargo fmt

# Check formatting
format-check:
	cargo fmt -- --check

# Run all checks (format, lint, test)
check: format-check lint test

# Build optimized release
release: clean
	cargo build --target wasm32-wasip1 --release
	@echo "Release build complete: target/wasm32-wasip1/release/ftl_mcp_gateway.wasm"

publish: build
	@VERSION=$$(cargo read-manifest | jq -r .version) && \
	wkg oci push ghcr.io/fastertools/mcp-gateway:$$VERSION target/wasm32-wasip1/release/ftl_mcp_gateway.wasm

# Help
help:
	@echo "Available targets:"
	@echo "  build        - Build the MCP gateway for WASM"
	@echo "  test         - Run tests"
	@echo "  clean        - Clean build artifacts"
	@echo "  lint         - Run clippy linter"
	@echo "  format       - Format code"
	@echo "  format-check - Check code formatting"
	@echo "  check        - Run all checks (format, lint, test)"
	@echo "  release      - Build optimized release"
	@echo "  publish      - Publish to ghcr.io"
	@echo "  help         - Show this help message"