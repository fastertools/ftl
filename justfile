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

# Default to showing available commands
default:
    @just --list