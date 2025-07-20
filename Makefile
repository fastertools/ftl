.PHONY: ci fmt-check lint test coverage fix-fmt fix-clippy fix dev pre-push build-release default

# Default to showing help
default:
	@echo "Available targets:"
	@echo "  ci           - Run all CI checks"
	@echo "  fmt-check    - Check formatting"
	@echo "  lint         - Run clippy"
	@echo "  test         - Run tests"
	@echo "  coverage     - Run tests with coverage"
	@echo "  fix-fmt      - Fix formatting"
	@echo "  fix-clippy   - Fix clippy warnings"
	@echo "  fix          - Fix everything"
	@echo "  dev          - Quick dev check"
	@echo "  pre-push     - Pre-push checks"
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
	cargo clippy --all-targets --all-features -- -D warnings

# Run tests
test:
	cargo nextest run

# Run tests with coverage
coverage:
	cargo llvm-cov nextest

# Fix formatting
fix-fmt:
	cargo fmt --all

# Fix clippy warnings
fix-clippy:
	cargo clippy --all-targets --all-features --fix --allow-dirty --allow-staged

# Fix everything
fix:
	@$(MAKE) fix-fmt
	@$(MAKE) fix-clippy

# Quick dev check
dev:
	@$(MAKE) fix-fmt
	@$(MAKE) lint

# Pre-push checks
pre-push:
	@$(MAKE) fix
	@$(MAKE) test

# Build release
build-release:
	cargo build --release