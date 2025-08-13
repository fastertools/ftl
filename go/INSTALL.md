# 📦 Installing FTL CLI (Go Version)

## Quick Install

```bash
# From the go/ directory
cd /home/ian/Dev/ftl-cli/go

# Install to your GOPATH
make install

# The binary will be installed to:
# /home/ian/go/bin/ftl
```

## Add to PATH

Since you have the Rust version in `/home/ian/.cargo/bin/ftl`, you have a few options:

### Option 1: Use the full path
```bash
/home/ian/go/bin/ftl --version
```

### Option 2: Create an alias
```bash
# Add to your ~/.bashrc or ~/.zshrc
alias ftl-go='/home/ian/go/bin/ftl'

# Then use:
ftl-go --version
ftl-go init my-project
```

### Option 3: Prepend Go bin to PATH (to override Rust version)
```bash
# Add to your ~/.bashrc or ~/.zshrc
export PATH=/home/ian/go/bin:$PATH

# This will make the Go version take precedence
ftl --version  # Will use Go version
```

### Option 4: Replace the Rust version
```bash
# Backup the Rust version first
mv /home/ian/.cargo/bin/ftl /home/ian/.cargo/bin/ftl-rust

# Then add Go bin to PATH
export PATH=$PATH:/home/ian/go/bin
```

## Try It Out

### Create a test project
```bash
# Using full path
/home/ian/go/bin/ftl init test-mcp-app --template mcp

# Or with alias
ftl-go init test-mcp-app --template mcp

# Navigate to the project
cd test-mcp-app

# View the generated files
ls -la
cat ftl.yaml
cat spinc.yaml
```

### Build and run (requires Spin CLI)
```bash
# Build the application
/home/ian/go/bin/ftl build

# Run locally
/home/ian/go/bin/ftl up

# Run with auto-reload
/home/ian/go/bin/ftl up --watch
```

## Available Commands

```bash
# See all commands
/home/ian/go/bin/ftl --help

# Initialize projects with different templates
/home/ian/go/bin/ftl init my-app --template mcp    # MCP server
/home/ian/go/bin/ftl init my-app --template basic  # Basic Spin app
/home/ian/go/bin/ftl init my-app --template empty  # Minimal config

# Component management
/home/ian/go/bin/ftl component list
/home/ian/go/bin/ftl component add my-tool

# Registry operations
/home/ian/go/bin/ftl registry push ghcr.io/myorg/app:latest
```

## Development Mode

If you want to run without installing:

```bash
# From the go directory
cd /home/ian/Dev/ftl-cli/go

# Run directly
make run ARGS="--help"
make run ARGS="init test-app"

# Or from ftl directory
cd ftl
go run . --help
go run . init test-app
```

## Uninstall

```bash
# Remove the installed binary
make uninstall

# Or manually
rm /home/ian/go/bin/ftl
```

## Note on Spin

Some commands (build, up, deploy) require Spin CLI to be installed:
- Install Spin: https://developer.fermyon.com/spin/install
- The Go FTL will check if Spin is installed and provide guidance if it's missing

## Building from Source

```bash
# Build to bin/ directory (doesn't install)
make build

# Binary will be at:
./bin/ftl

# Run it directly:
./bin/ftl --version
```