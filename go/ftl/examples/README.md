# FTL Examples - Multiple Configuration Formats

These examples demonstrate FTL's multi-format configuration capability. Each example creates **identical** MCP applications using different configuration formats.

## The Application

All examples create the same MCP application with:
- Two MCP tools: `geo` and `fluid` (from ghcr.io registry)
- Automatic MCP gateway integration
- Private routing for tool components
- Public `/...` route for the gateway

## Configuration Formats

### 1. YAML Format (`yaml-format/`)
Simple, declarative configuration in YAML.

```bash
cd yaml-format
ftl synth ftl.yaml -o spin.toml  # Generate spin.toml
# or
ftl build                         # Build with automatic synthesis
spin up                           # Run the application
```

### 2. Go Format (`go-format/`)
Programmatic configuration using the FTL SDK in Go.

```bash
cd go-format
ftl synth main.go -o spin.toml   # Generate spin.toml
# or
go run main.go > spin.toml        # Run directly
spin up                           # Run the application
```

### 3. JSON Format (`json-format/`)
Standard JSON configuration for programmatic generation or tool integration.

```bash
cd json-format
ftl synth ftl.json -o spin.toml  # Generate spin.toml
# or
ftl build --config ftl.json       # Build with automatic synthesis
spin up                           # Run the application
```

## Key Point: Identical Output

All three formats produce **identical** `spin.toml` files. This demonstrates FTL's powerful synthesis engine that:

1. **Abstracts complexity** - Users write simple configs
2. **Adds intelligence** - Automatically includes MCP gateway, routing, and wiring
3. **Maintains consistency** - Same output regardless of input format

## Testing the Applications

Once running with `spin up`, test the MCP endpoint:

```bash
# List available tools
curl -X POST http://127.0.0.1:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"tools/list","params":{},"id":1}'

# Call a tool
echo '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"geo__example_tool","arguments":{"message":"Hello"}},"id":2}' | \
  curl -X POST http://127.0.0.1:3000/mcp \
    -H "Content-Type: application/json" \
    -d @-
```

## Understanding the Synthesis

FTL uses a sophisticated multi-stage transformation pipeline:

```
User Config → CUE Representation → SpinDL (Intermediate) → Spin Manifest
```

The magic happens in the CUE transformations that:
- Automatically inject required components (MCP gateway)
- Configure routing (public gateway, private tools)
- Set up component communication
- Apply security defaults

This is **platform engineering as code** - encoding best practices, security, and architectural decisions into the synthesis process.