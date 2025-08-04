.PHONY: ci fmt-check lint test coverage coverage-html fmt fix-clippy fix dev pre-push build-release default

# Default to showing help
default:
	@echo "Available targets:"
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
	cd components/mcp-authorizer && spin build && spin test

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
	cargo wasm

# Build release
build-release:
	cargo build --release
	cargo wasm --release