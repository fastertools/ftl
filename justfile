# Run all CI checks
ci:
    @echo "🔍 Running CI checks..."
    @just fmt-check
    @just lint
    @just test-all
    @echo "✅ All CI checks passed!"

# Check formatting
fmt-check:
    cargo fmt --all -- --check

# Run clippy  
lint:
    cargo clippy --all-targets --all-features -- -D warnings

# Run all tests
test-all:
    cargo test --all-features

# Run unit tests only
test-unit:
    cargo test --lib --all-features

# Run integration tests only  
test-integration:
    cargo test --test '*' --all-features

# Run tests with coverage
coverage:
    @echo "Running test coverage..."
    @echo "Excluding:"
    @echo "  - main.rs (CLI entry point)"
    @echo "  - deps.rs (testing infrastructure)" 
    @echo "  - ui.rs (interactive UI components)"
    @echo "  - api_client.rs (auto-generated API client wrapper)"
    @echo "  - *_test.rs, *_tests.rs, test_*.rs (test files)"
    @echo ""
    cargo llvm-cov test --all-features --ignore-filename-regex="(main\.rs|deps\.rs|ui\.rs|api_client\.rs|.*_test\.rs|.*_tests\.rs|test_.*\.rs)"
    @echo ""
    @echo "Generating HTML report..."
    cargo llvm-cov report --html --ignore-filename-regex="(main\.rs|deps\.rs|ui\.rs|api_client\.rs|.*_test\.rs|.*_tests\.rs|test_.*\.rs)"
    @echo ""
    @echo "Coverage report saved to: target/llvm-cov/html/index.html"

# Run tests with coverage and open report
coverage-open:
    @just coverage
    @open target/llvm-cov/html/index.html

# Show coverage summary
coverage-summary:
    @cargo llvm-cov report --ignore-filename-regex="(main\.rs|deps\.rs|ui\.rs|api_client\.rs|.*_test\.rs|.*_tests\.rs|test_.*\.rs)" | tail -5

# Run doc tests
test-doc:
    cargo test --doc --all-features

# Run a specific test
test name:
    cargo test {{name}} --all-features -- --nocapture

# Run tests in watch mode
test-watch:
    cargo watch -x "test --all-features"

# Fix formatting
fix-fmt:
    cargo fmt --all

# Fix clippy warnings
fix-clippy:
    cargo clippy --all-targets --all-features --fix --allow-dirty --allow-staged

# Fix everything
fix:
    @just fix-fmt
    @just fix-clippy

# Quick dev check
dev:
    @just fix-fmt
    @just lint

# Pre-push checks
pre-push:
    @just fix
    @just test-all

# Build release
build-release:
    cargo build --release

# Install Spin
spin-install:
    cargo run -- spin install

# Spin info
spin-info:
    cargo run -- spin info

# Default to showing available commands
default:
    @just --list