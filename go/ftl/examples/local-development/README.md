# Local Development Example

This example shows how to configure FTL for local development with build configurations and file watching.

## Features Demonstrated

1. **Local source paths** with build commands
2. **File watching** for automatic rebuilds
3. **Mixed sources** (local + registry)
4. **Development variables** (debug modes, local endpoints)

## Build Configuration

Local components can specify build commands:

```yaml
components:
  - id: my-rust-tool
    source: "./path/to/built.wasm"  # Built artifact location
    build:
      command: "cargo build --target wasm32-wasi --release"
      workdir: "./rust-tool"        # Where to run the command
      watch:                         # Files to watch for changes
        - "src/**/*.rs"
        - "Cargo.toml"
```

## Development Workflow

### 1. Initial Setup

```bash
# Generate the manifest
ftl synth ftl.yaml -o spin.toml

# Or using Go
go run main.go > spin.toml
```

### 2. Build Components

```bash
# FTL can build your components
ftl build

# This runs the build commands for each local component
```

### 3. Development Mode

```bash
# Run with file watching
ftl up

# Or with Spin directly
spin up --watch
```

When files change in watched directories, components automatically rebuild!

## Project Structure

```
local-development/
├── ftl.yaml                 # FTL configuration
├── main.go                  # Alternative Go config
├── spin.toml               # Generated manifest
├── rust-tool/              # Local Rust component
│   ├── Cargo.toml
│   ├── src/
│   │   └── lib.rs
│   └── target/
│       └── wasm32-wasi/
│           └── release/
│               └── rust_tool.wasm
└── ts-tool/                # Local TypeScript component
    ├── package.json
    ├── src/
    │   └── index.ts
    └── dist/
        └── component.wasm
```

## Supported Build Systems

FTL works with any build system that produces WebAssembly:

### Rust
```yaml
build:
  command: "cargo build --target wasm32-wasi --release"
```

### TypeScript/JavaScript
```yaml
build:
  command: "npm run build"  # Using componentize-js or similar
```

### Go
```yaml
build:
  command: "tinygo build -target=wasi -o component.wasm main.go"
```

### Python
```yaml
build:
  command: "componentize-py componentize app -o component.wasm"
```

## Environment Variables

Development-specific variables:

```yaml
variables:
  DEBUG_MODE: "true"              # Enable debug logging
  LOG_LEVEL: "debug"              # Verbose logging
  API_ENDPOINT: "http://localhost:8080"  # Local services
  DATABASE_URL: "sqlite:///tmp/dev.db"   # Local database
```

## Tips for Local Development

1. **Use public access** during development to avoid auth complexity
2. **Set debug variables** for better error messages
3. **Configure watch patterns** to match your source structure
4. **Mix local and registry** components to test integration
5. **Use workdir** for monorepo structures

## Testing Local Components

```bash
# List all available tools (local + registry)
curl -X POST http://127.0.0.1:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"tools/list","params":{},"id":1}'

# Call your local tool
curl -X POST http://127.0.0.1:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc":"2.0",
    "method":"tools/call",
    "params":{
      "name":"my-rust-tool__process",
      "arguments":{"input":"test data"}
    },
    "id":2
  }'
```

## Moving to Production

When ready for production:
1. Change `access: public` to `access: private`
2. Add authentication configuration
3. Update variables for production endpoints
4. Consider using registry for distribution