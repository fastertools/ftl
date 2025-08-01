#!/bin/bash
set -e
mkdir -p dist

# Check if TinyGo is installed
if ! command -v tinygo &> /dev/null; then
    echo "TinyGo is required but not installed. Please install from https://tinygo.org"
    exit 1
fi

# Build with TinyGo using Spin-specific flags
tinygo build -target=wasip1 -gc=leaking -buildmode=c-shared -no-debug -o dist/weather-go.wasm main.go