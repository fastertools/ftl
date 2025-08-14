#!/usr/bin/env bash
# SpinDL Installation Script

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "Installing SpinDL (Spin Development Layer)..."

# Check if Go is installed
if ! command -v go &> /dev/null; then
    echo -e "${RED}Error: Go is not installed${NC}"
    echo "Please install Go from https://golang.org/dl/"
    exit 1
fi

# Get the directory of this script
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

# Build SpinDL
echo "Building SpinDL..."
cd "$SCRIPT_DIR"
make build

# Determine installation directory
if [ -n "$GOBIN" ]; then
    INSTALL_DIR="$GOBIN"
elif [ -n "$GOPATH" ]; then
    INSTALL_DIR="$GOPATH/bin"
else
    INSTALL_DIR="$HOME/go/bin"
fi

# Create bin directory if it doesn't exist
mkdir -p "$INSTALL_DIR"

# Install the binary
echo "Installing to $INSTALL_DIR..."
cp dist/spindl "$INSTALL_DIR/spindl"
chmod +x "$INSTALL_DIR/spindl"

# Check if install directory is in PATH
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo -e "${YELLOW}Warning: $INSTALL_DIR is not in your PATH${NC}"
    echo "Add the following to your shell configuration file (.bashrc, .zshrc, etc.):"
    echo "  export PATH=\"\$PATH:$INSTALL_DIR\""
fi

echo -e "${GREEN}✓ SpinDL installed successfully!${NC}"
echo ""
echo "Run 'spindl --help' to get started"
echo "Or try 'spindl init my-app' to create a new project"