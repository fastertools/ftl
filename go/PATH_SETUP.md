# ✅ PATH Setup Complete!

The Go version of FTL is now installed and your PATH has been updated.

## What was done:

1. **FTL installed to**: `/home/ian/go/bin/ftl`
2. **PATH updated in**: `/home/ian/.bashrc`
3. **Added line**: `export PATH=$PATH:/home/ian/go/bin`

## To use FTL right now:

```bash
# For the current terminal session, run:
export PATH=$PATH:/home/ian/go/bin

# Then you can use:
ftl --version
ftl init my-project
```

## To make it permanent:

The PATH has already been added to your `~/.bashrc`. For new terminal sessions, it will work automatically.

For the current session, either:
- Run: `source ~/.bashrc`
- Or: `export PATH=$PATH:/home/ian/go/bin`

## Verify it works:

```bash
# Check FTL is found
which ftl
# Should output: /home/ian/go/bin/ftl

# Check version
ftl --version
# Should output: ftl version dev (commit: 7bed71d, built: ...)

# Create a test project
ftl init my-app --template mcp
cd my-app
ls -la
```

## Quick Commands:

```bash
# Initialize projects
ftl init my-mcp-server --template mcp    # MCP server with auth
ftl init my-app --template basic          # Basic Spin app
ftl init my-app --template empty          # Minimal setup

# Build and run (requires Spin CLI)
ftl build
ftl up
ftl up --watch  # Auto-reload on changes

# Deploy
ftl deploy --environment production

# Component management
ftl component add my-tool
ftl component list

# Registry operations
ftl registry push ghcr.io/myorg/app:latest
```

## Troubleshooting:

If `ftl` command is not found:
1. Make sure you ran: `export PATH=$PATH:/home/ian/go/bin`
2. Or reload your shell: `source ~/.bashrc`
3. Or open a new terminal (will automatically have the updated PATH)

## Note:
The version shows as "dev" with "unknown" commit because we're running from the development directory. This is normal for a development build.