# Installing FTL for Development

## Quick Install

From the `go/` directory:

```bash
make install
```

This will:
1. Build the FTL binary
2. Install it to `$GOPATH/bin/ftl`
3. Make it available in your PATH (if `$GOPATH/bin` is in your PATH)

## Verify Installation

```bash
ftl --version
ftl --help
```

## Make sure FTL is in your PATH

If `ftl` command is not found, add Go's bin directory to your PATH:

```bash
export PATH=$PATH:$(go env GOPATH)/bin
```

Add this to your `~/.bashrc` or `~/.zshrc` to make it permanent.

## Other Make Commands

```bash
make build    # Just build, don't install
make test     # Run all tests
make clean    # Clean build artifacts
make uninstall # Remove installed ftl
```

## Manual Installation

If you prefer not to use make:

```bash
cd ftl
go build -o ftl .
sudo mv ftl /usr/local/bin/  # Or wherever you prefer
```

## Testing Your Installation

Try creating a new project:

```bash
ftl init my-test-app
cd my-test-app
ftl build
```