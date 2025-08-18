# FTL Go CDK API Reference

The FTL Go CDK provides a fluent, type-safe API for building FTL applications programmatically. It uses CUE internally for all transformations.

## Quick Start

```go
package main

import (
    "fmt"
    "log"
    "github.com/fastertools/ftl-cli/go/ftl/pkg/synthesis"
)

func main() {
    cdk := synthesis.NewCDK()
    app := cdk.NewApp("my-app").
        SetVersion("1.0.0")
    
    app.AddComponent("my-component").
        FromLocal("./component.wasm").
        Build()
    
    builtCDK := app.Build()
    manifest, err := builtCDK.Synthesize()
    if err != nil {
        log.Fatal(err)
    }
    fmt.Print(manifest)
}
```

## Core Types

### CDK

The main entry point for creating applications.

```go
// Create a new CDK instance
cdk := synthesis.NewCDK()
```

### AppBuilder

Fluent interface for building applications.

#### Methods

##### `NewApp(name string) *AppBuilder`
Creates a new application with the specified name.

```go
app := cdk.NewApp("my-app")
```

##### `SetVersion(version string) *AppBuilder`
Sets the application version.

```go
app.SetVersion("1.0.0")
```

##### `SetDescription(description string) *AppBuilder`
Sets the application description.

```go
app.SetDescription("My MCP platform")
```

##### `SetAccess(access string) *AppBuilder`
Sets the access level ("public" or "private").

```go
app.SetAccess("private")
```

##### `EnableWorkOSAuth(orgID string) *AppBuilder`
Enables WorkOS authentication with the specified organization ID.

```go
app.EnableWorkOSAuth("org_12345")
```

##### `EnableCustomAuth(issuer, audience string) *AppBuilder`
Enables custom JWT authentication.

```go
app.EnableCustomAuth("https://auth.example.com", "my-audience")
```

##### `AddComponent(id string) *ComponentBuilder`
Adds a new component to the application.

```go
app.AddComponent("my-component")
```

##### `Build() *CDK`
Finalizes the application and returns the CDK for synthesis.

```go
builtCDK := app.Build()
```

### ComponentBuilder

Fluent interface for configuring components.

#### Methods

##### `FromLocal(path string) *ComponentBuilder`
Sets the component source to a local Wasm file.

```go
.FromLocal("./target/wasm32-wasip1/release/component.wasm")
```

##### `FromRegistry(registry, package, version string) *ComponentBuilder`
Sets the component source to a registry package.

```go
.FromRegistry("ghcr.io", "org:component", "1.0.0")
```

##### `WithBuild(command string) *ComponentBuilder`
Sets the build command for local components.

```go
.WithBuild("cargo build --target wasm32-wasip1 --release")
```

##### `WithWatch(patterns ...string) *ComponentBuilder`
Adds file patterns to watch for changes during development.

```go
.WithWatch("src/**/*.rs", "Cargo.toml")
```

##### `WithEnv(key, value string) *ComponentBuilder`
Adds an environment variable to the component.

```go
.WithEnv("LOG_LEVEL", "debug")
.WithEnv("API_KEY", "secret")
```

##### `Build() *AppBuilder`
Completes the component and returns to the app builder.

```go
.Build()
```

## Component Sources

### Local Components

Components built from local source files:

```go
app.AddComponent("rust-component").
    FromLocal("./target/wasm32-wasip1/release/component.wasm").
    WithBuild("cargo build --target wasm32-wasip1 --release").
    WithWatch("src/**/*.rs", "Cargo.toml").
    Build()
```

**Note**: Local components always get a build section in the manifest, even if no build command is specified.

### Registry Components

Components from OCI registries:

```go
app.AddComponent("registry-component").
    FromRegistry("ghcr.io", "bowlofarugula:geo", "0.0.1").
    Build()
```

**Note**: Registry components never get a build section in the manifest.

## Authentication

### Public Access (Default)

By default, applications are publicly accessible:

```go
app := cdk.NewApp("public-app")
// No auth configuration needed
```

### WorkOS Authentication

Enable WorkOS authentication for private access:

```go
app.EnableWorkOSAuth("org_12345")
```

This automatically:
- Sets access to "private"
- Adds MCP authorizer component
- Configures JWT validation

### Custom Authentication

Use custom JWT providers:

```go
app.EnableCustomAuth("https://auth.example.com", "my-audience")
```

## Synthesis

### Generate spin.toml

```go
builtCDK := app.Build()
manifest, err := builtCDK.Synthesize()
if err != nil {
    log.Fatal(err)
}
fmt.Print(manifest)
```

### Generate CUE

For debugging or advanced use cases, generate CUE output:

```go
builtCDK := app.Build()
cueOutput, err := builtCDK.ToCUE()
if err != nil {
    log.Fatal(err)
}
fmt.Print(cueOutput)
```

## Complete Examples

### Simple Application

```go
cdk := synthesis.NewCDK()
app := cdk.NewApp("simple-app").
    SetVersion("1.0.0")

app.AddComponent("my-component").
    FromLocal("./component.wasm").
    Build()

builtCDK := app.Build()
manifest, _ := builtCDK.Synthesize()
fmt.Print(manifest)
```

### Complex Application with Auth

```go
cdk := synthesis.NewCDK()
app := cdk.NewApp("complex-app").
    SetVersion("2.0.0").
    SetDescription("Production MCP platform")

// Local component with build config
app.AddComponent("api-service").
    FromLocal("./target/wasm32-wasip1/release/api.wasm").
    WithBuild("cargo build --release").
    WithWatch("src/**/*.rs").
    WithEnv("DATABASE_URL", "postgres://localhost/db").
    Build()

// Registry component
app.AddComponent("auth-service").
    FromRegistry("ghcr.io", "org:auth", "1.2.3").
    WithEnv("JWT_SECRET", "secret").
    Build()

// Enable authentication
app.EnableWorkOSAuth("org_production")

builtCDK := app.Build()
manifest, _ := builtCDK.Synthesize()
fmt.Print(manifest)
```

## Design Principles

1. **Fluent Interface**: All methods return builders for chaining
2. **Type Safety**: Strong typing prevents configuration errors
3. **CUE Under the Hood**: All transformations use CUE patterns
4. **Smart Defaults**: Registry components don't get build sections
5. **Zero Empty Sections**: Variables only appear when set

## Error Handling

All errors are returned from `Synthesize()` and `ToCUE()`:

```go
manifest, err := builtCDK.Synthesize()
if err != nil {
    // Handle synthesis errors
    // - Invalid component names
    // - CUE transformation failures
    // - Encoding errors
    log.Fatalf("Synthesis failed: %v", err)
}
```

## Integration with FTL CLI

The CDK can be used directly or through the FTL CLI:

```bash
# Run Go file directly
go run myapp.go > spin.toml

# Or use FTL CLI
ftl synth myapp.go -o spin.toml
```

## Testing

The CDK is extensively tested with 90%+ coverage:

```go
// Example test
func TestCDK_SimpleApp(t *testing.T) {
    cdk := NewCDK()
    app := cdk.NewApp("test-app")
    app.AddComponent("test").
        FromLocal("./test.wasm").
        Build()
    
    builtCDK := app.Build()
    manifest, err := builtCDK.Synthesize()
    
    assert.NoError(t, err)
    assert.Contains(t, manifest, "test-app")
    assert.Contains(t, manifest, "[component.test]")
}
```