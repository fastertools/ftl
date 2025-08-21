.PHONY: build test clean lint check release

# Default target
build:
	cargo build --target wasm32-wasip1 --release --target-dir ./target

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
	@echo "Release build complete: target/wasm32-wasip1/release/mcp_authorizer.wasm"

publish: build
	@VERSION=$$(cargo read-manifest | jq -r .version) && \
	spin deps publish --registry ghcr.io --package fastertools:mcp-authorizer@$$VERSION target/wasm32-wasip1/release/mcp_authorizer.wasm

# Help
help:
	@echo "Available targets:"
	@echo "  build        - Build the auth gateway for WASM"
	@echo "  test         - Run tests"
	@echo "  clean        - Clean build artifacts"
	@echo "  lint         - Run clippy linter"
	@echo "  format       - Format code"
	@echo "  format-check - Check code formatting"
	@echo "  check        - Run all checks (format, lint, test)"
	@echo "  release      - Build optimized release"
	@echo "  publish      - Publish to ghcr.io"
	@echo "  help         - Show this help message"
