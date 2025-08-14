# FTL CDK Examples

The FTL CDK provides a fluent Go API for defining MCP (Model Context Protocol) tool platforms that run on WebAssembly via Spin.

## Key Features

- **Fluent API**: Chain method calls for clean, readable code
- **Type Safety**: Compile-time checking of your configuration
- **Multiple Sources**: Support for local WASM files and registry packages
- **Build Integration**: Define build commands and watch patterns for development
- **Authentication**: Built-in support for WorkOS SSO and custom JWT auth
- **Environment Variables**: Configure tools with environment variables
- **Automatic Routing**: Handles public/private routing automatically

## Basic Example

```go
package main

import (
    "fmt"
    "github.com/fastertools/ftl-cli/go/spindl/pkg/ftl"
)

func main() {
    // Create an app
    app := ftl.NewApp("my-platform").
        SetDescription("My MCP Platform").
        SetVersion("1.0.0")

    // Add tools
    app.AddTool("calculator").
        FromLocal("./calc.wasm").
        WithEnv("PRECISION", "high").
        Build()

    // Generate manifest
    synth := ftl.NewSynthesizerV2()
    manifest, _ := synth.Synthesize(app)
    fmt.Println(manifest)
}
```

## Tool Sources

### Local WebAssembly Files
```go
app.AddTool("my-tool").
    FromLocal("./path/to/tool.wasm").
    Build()
```

### Registry Packages
```go
app.AddTool("my-tool").
    FromRegistry("ghcr.io", "org/package", "1.0.0").
    Build()
```

## Build Configuration

Add build commands and watch patterns for development:

```go
app.AddTool("rust-tool").
    FromLocal("./tool.wasm").
    WithBuild("cargo build --target wasm32-wasi --release").
    WithWatch("src/**/*.rs", "Cargo.toml").
    Build()
```

## Environment Variables

Configure tools with environment variables:

```go
app.AddTool("api-client").
    FromLocal("./api.wasm").
    WithEnv("API_ENDPOINT", "https://api.example.com").
    WithEnv("TIMEOUT", "30").
    WithEnv("RETRY_COUNT", "3").
    Build()
```

## Authentication

### WorkOS SSO (Enterprise)
```go
// Enable WorkOS authentication
app.EnableWorkOSAuth("org_YOUR_ORG_ID")
```

### Custom JWT Authentication
```go
// Use your own JWT issuer
app.EnableCustomAuth("https://auth.example.com", "audience-name")
```

### Public Access (Default)
```go
// No authentication - tools are publicly accessible via the MCP gateway
// This is the default, no configuration needed
```

## Architecture

When synthesized, the CDK generates a `spin.toml` manifest with:

1. **MCP Gateway**: Routes requests to appropriate tools
2. **MCP Authorizer** (if auth enabled): Handles authentication
3. **Your Tools**: The WebAssembly components you defined
4. **Routing**:
   - Public mode: Gateway on `/...` (catch-all), tools are private
   - Auth mode: Authorizer on `/...`, gateway and tools are private

## Running Examples

```bash
# Simple public platform
go run examples/simple-app.go > spin.toml

# Enterprise platform with auth
go run examples/auth-app.go > spin.toml

# Full-featured example
go run examples/cdk-app.go > spin.toml
```

## Validation

The CDK includes comprehensive validation:

```go
synth := ftl.NewSynthesizerV2()

// Validate before synthesizing
if err := synth.Validate(app); err != nil {
    log.Fatalf("Validation failed: %v", err)
}

// Synthesize (also validates internally)
manifest, err := synth.Synthesize(app)
if err != nil {
    log.Fatalf("Synthesis failed: %v", err)
}
```

## Output

The CDK generates a complete `spin.toml` manifest that can be used with:

```bash
# Build and run locally
spin build
spin up

# Deploy to Fermyon Cloud
spin deploy
```

## Integration with FTL CLI

The FTL CLI can execute CDK files directly:

```bash
# Synthesize from CDK file
ftl synth app.go > spin.toml

# Or output to file
ftl synth app.go -o spin.toml

# Validate only
ftl synth app.go --validate
```