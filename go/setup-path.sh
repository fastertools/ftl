#!/bin/bash

# Script to add Go bin directory to PATH

GOBIN=$(go env GOPATH)/bin
SHELL_RC=""

# Detect shell configuration file
if [ -n "$ZSH_VERSION" ]; then
    SHELL_RC="$HOME/.zshrc"
    echo "Detected Zsh shell"
elif [ -n "$BASH_VERSION" ]; then
    SHELL_RC="$HOME/.bashrc"
    echo "Detected Bash shell"
else
    echo "Could not detect shell. Please manually add the following to your shell configuration:"
    echo "export PATH=\$PATH:$GOBIN"
    exit 1
fi

# Check if GOBIN is already in PATH
if echo "$PATH" | grep -q "$GOBIN"; then
    echo "✅ $GOBIN is already in your PATH"
else
    echo "Adding $GOBIN to PATH in $SHELL_RC..."
    
    # Add to shell configuration
    echo "" >> "$SHELL_RC"
    echo "# FTL CLI - Go binaries" >> "$SHELL_RC"
    echo "export PATH=\$PATH:$GOBIN" >> "$SHELL_RC"
    
    echo "✅ Added to $SHELL_RC"
    echo ""
    echo "To use FTL immediately, run:"
    echo "  export PATH=\$PATH:$GOBIN"
    echo ""
    echo "Or reload your shell configuration:"
    echo "  source $SHELL_RC"
fi

# Check if ftl is installed
if [ -f "$GOBIN/ftl" ]; then
    echo ""
    echo "✅ FTL is installed at: $GOBIN/ftl"
    echo ""
    echo "After updating PATH, you can use:"
    echo "  ftl --version"
    echo "  ftl init my-project"
else
    echo ""
    echo "⚠️  FTL is not installed yet. Run:"
    echo "  cd /home/ian/Dev/ftl-cli/go"
    echo "  make install"
fi