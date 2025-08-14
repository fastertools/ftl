# FTL CDK Usage Guide

## Quick Start

### 1. Write a CDK App

Create a Go file that uses the FTL CDK:

```go
package main

import (
    "fmt"
    "github.com/fastertools/ftl-cli/go/spindl/pkg/ftl"
)

func main() {
    app := ftl.NewApp("my-platform").
        SetDescription("My MCP Platform")
    
    app.AddTool("my-tool").
        FromLocal("./tool.wasm").
        Build()
    
    synth := ftl.NewSynthesizerV2()
    manifest, _ := synth.Synthesize(app)
    fmt.Println(manifest)
}
```

### 2. Generate spin.toml

```bash
# Direct execution
go run my-app.go > spin.toml

# Using the helper script
./synthesize.sh my-app.go

# Using FTL CLI (when fixed)
ftl synth my-app.go -o spin.toml
```

### 3. Run with Spin

```bash
# Build and run locally
spin build
spin up

# Deploy to cloud
spin deploy
```

## Examples

### Public Platform (Default)
```go
app := ftl.NewApp("public-platform")
app.AddTool("tool").FromLocal("./tool.wasm").Build()
// No auth config = public access via MCP gateway
```

### Private Platform with SSO
```go
app := ftl.NewApp("enterprise-platform")
app.AddTool("tool").FromLocal("./tool.wasm").Build()
app.EnableWorkOSAuth("org_YOUR_ORG_ID")
// All tools require authentication
```

### Tools from Registry
```go
app.AddTool("remote-tool").
    FromRegistry("ghcr.io", "org/package", "1.0.0").
    Build()
```

### Build Configuration
```go
app.AddTool("rust-tool").
    FromLocal("./tool.wasm").
    WithBuild("cargo build --target wasm32-wasi --release").
    WithWatch("src/**/*.rs", "Cargo.toml").
    Build()
```

### Environment Variables
```go
app.AddTool("configured-tool").
    FromLocal("./tool.wasm").
    WithEnv("API_KEY", "secret").
    WithEnv("LOG_LEVEL", "debug").
    Build()
```

## Generated Output

The CDK generates a complete `spin.toml` with:

- **Application metadata** (name, version, description)
- **Components**:
  - Your tools (as private components)
  - MCP Gateway (routes requests)
  - MCP Authorizer (if auth enabled)
- **Triggers**:
  - Public mode: Gateway on `/...`, tools private
  - Auth mode: Authorizer on `/...`, gateway and tools private
- **Configuration**:
  - Environment variables
  - Build commands
  - Watch patterns
  - Component names for gateway routing

## File Structure

```
my-project/
├── app.go           # Your CDK app
├── spin.toml        # Generated manifest
├── tools/           # Local WASM files
│   ├── tool1.wasm
│   └── tool2.wasm
└── src/             # Source code for tools
    └── ...
```

## Testing

The CDK includes comprehensive validation:

```go
synth := ftl.NewSynthesizerV2()

// Validate before synthesis
if err := synth.Validate(app); err != nil {
    log.Fatal(err)
}

// Synthesize (also validates)
manifest, err := synth.Synthesize(app)
```

## Troubleshooting

### "File not found" errors
- Ensure local WASM paths are relative to where you run the command
- Use `./` prefix for local files

### Registry authentication
- Some registries require authentication
- Configure with `spin registry login`

### Build failures
- Ensure build tools (cargo, npm, etc.) are installed
- Check that build commands are correct
- Watch patterns use glob syntax

## Architecture

```
Your CDK App (Go)
    ↓ (synthesize)
spin.toml manifest
    ↓ (spin build)
WebAssembly modules
    ↓ (spin up)
Running MCP platform
```

## Complete Example

See `examples/` directory for complete, working examples:
- `simple-app.go` - Basic public platform
- `auth-app.go` - Enterprise platform with SSO
- `cdk-app.go` - Full-featured example