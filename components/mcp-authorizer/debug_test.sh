#!/bin/bash

echo "Running a single test with verbose output..."
cd tests && cargo build --target wasm32-wasip1 --release && cd ..

# Run with environment variable to see more output
RUST_LOG=debug spin test run 2>&1 | grep -A 20 "test-missing-authorization-header"