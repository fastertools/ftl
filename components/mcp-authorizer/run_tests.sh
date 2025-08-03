#!/bin/bash

# Get the directory of this script
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

echo "Building MCP Authorizer tests..."
cd tests && cargo build --target wasm32-wasip1 --release && cd ..

echo ""
echo "Running Spin tests..."
spin test

echo ""
echo "Test run complete!"