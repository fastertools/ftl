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