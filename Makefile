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
	cargo clippy --all-targets --all-features -- -D warnings

# Run tests
test:
	cargo nextest run

# Run tests with coverage
coverage:
	cargo llvm-cov nextest --ignore-filename-regex="(main\.rs|deps\.rs|ui\.rs|api_client\.rs|.*_test\.rs|.*_tests\.rs|test_.*\.rs)"

# Generate HTML coverage report
coverage-html:
	cargo llvm-cov nextest --html --ignore-filename-regex="(main\.rs|deps\.rs|ui\.rs|api_client\.rs|.*_test\.rs|.*_tests\.rs|test_.*\.rs)"
	@echo "Coverage report generated at target/llvm-cov/html/index.html"

# Fix formatting
fmt:
	cargo fmt --all

# Fix clippy warnings
fix-clippy:
	cargo clippy --all-targets --all-features --fix --allow-dirty --allow-staged

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

# Build release
build-release:
	cargo build --release