#!/bin/bash
set -e
mkdir -p dist

# Check if TinyGo is installed
if ! command -v tinygo &> /dev/null; then
    echo "TinyGo is required but not installed. Please install from https://tinygo.org"
    exit 1
fi

# Build with TinyGo
tinygo build -o dist/echo-go.wasm -target=wasi main.go