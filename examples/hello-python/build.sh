#!/bin/bash
set -e

echo "Building Python WASM component..."

# Create virtual environment if it doesn't exist
if [ ! -d "venv" ]; then
    echo "Creating virtual environment..."
    # Use Python 3.11 if available, otherwise fall back to python3
    if command -v python3.11 &> /dev/null; then
        python3.11 -m venv venv
    elif [ -f "/opt/homebrew/bin/python3.11" ]; then
        /opt/homebrew/bin/python3.11 -m venv venv
    else
        python3 -m venv venv
    fi
fi

# Activate virtual environment
source venv/bin/activate

# Install dependencies if needed
if ! command -v componentize-py &> /dev/null; then
    echo "Installing dependencies..."
    pip install componentize-py spin-sdk
fi

# Build the WASM component
componentize-py -w spin-http componentize app -o app.wasm

echo "Build complete! You can now run 'spin up' to test locally."