#!/bin/bash
set -euo pipefail

echo "Building mcp-gateway tests..."
cargo component build --target wasm32-wasip1 --release

echo "Running spin-test..."
spin-test

echo "Tests completed successfully!"