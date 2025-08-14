# FTL Platform Walkthrough: From Idea to spin.toml

## Overview

FTL (Faster Tool Layer) is a CDK-style framework for building MCP (Model Context Protocol) tool platforms on Spin. It provides a high-level abstraction for composing WebAssembly-based tools into a unified platform with built-in gateway and authentication support.

## Your Tools

You have two scientific computing tools available:
- `ghcr.io/bowlofarugula/geo@0.0.1` - Geological computations and GIS operations
- `ghcr.io/bowlofarugula/fluid@0.0.1` - Fluid dynamics simulations

## Step 1: Choose Your Approach

You have three ways to define your platform:

### Option A: Go CDK (Programmatic)
Best for: Dynamic configurations, CI/CD integration, complex logic

### Option B: YAML (Declarative)
Best for: Static configurations, GitOps workflows, simplicity

### Option C: Direct CUE (Advanced)
Best for: Maximum control, complex constraints, multi-environment configs

## Step 2: Define Your Platform

### Minimal Example (Go CDK)

```go
package main

import (
    "fmt"
    "log"
    "github.com/fastertools/ftl-cli/go/spindl/pkg/ftl"
)

func main() {
    // Create your platform
    app := ftl.NewApp("my-platform").
        SetDescription("My scientific computing platform").
        SetVersion("1.0.0")

    // Add your tools
    app.AddTool("geo").
        FromRegistry("ghcr.io", "bowlofarugula/geo", "0.0.1").
        Build()

    app.AddTool("fluid").
        FromRegistry("ghcr.io", "bowlofarugula/fluid", "0.0.1").
        Build()

    // Generate spin.toml
    synth := ftl.NewSynthesizer()
    manifest, err := synth.SynthesizeApp(app)
    if err != nil {
        log.Fatal(err)
    }
    fmt.Println(manifest)
}
```

### Minimal Example (YAML)

```yaml
name: my-platform
version: 1.0.0
description: My scientific computing platform

tools:
  - id: geo
    source:
      registry: ghcr.io
      package: bowlofarugula/geo
      version: 0.0.1

  - id: fluid
    source:
      registry: ghcr.io
      package: bowlofarugula/fluid
      version: 0.0.1
```

## Step 3: Add Configuration

### Environment Variables
Configure your tools with environment variables:

```go
app.AddTool("geo").
    FromRegistry("ghcr.io", "bowlofarugula/geo", "0.0.1").
    WithEnv("LOG_LEVEL", "debug").
    WithEnv("MAX_MEMORY", "4096").
    Build()
```

### Build Configuration
If you have local tools that need building:

```go
app.AddTool("my-tool").
    FromLocal("./my-tool.wasm").
    WithBuild("cargo build --release").
    WithWatch("src/**/*.rs", "Cargo.toml").
    Build()
```

## Step 4: Add Authentication (Optional)

### WorkOS (Enterprise SSO)
```go
app.EnableWorkOSAuth("org_12345")
```

### Custom JWT
```go
app.EnableCustomAuth("https://auth.example.com", "my-audience")
```

## Step 5: Generate spin.toml

Run your Go program:
```bash
go run main.go > spin.toml
```

Or use the FTL CLI with YAML:
```bash
ftl synth platform.yaml > spin.toml
```

## What Gets Generated?

The synthesizer automatically:

1. **Adds MCP Gateway** - Routes requests to your tools
2. **Configures Networking** - Sets up internal service mesh
3. **Adds Authentication** - Includes MCP Authorizer if needed
4. **Sets Up Routing** - Configures HTTP triggers
5. **Manages Dependencies** - Ensures correct component ordering

### Generated Architecture

```
Internet
    ↓
[Public Route: /...]
    ↓
[MCP Authorizer] (if auth enabled)
    ↓
[MCP Gateway] (always present)
    ↓
[Your Tools] (geo, fluid, etc.)
```

### Example Output

```toml
spin_manifest_version = 2

[application]
name = "my-platform"
version = "1.0.0"

[component.geo]
source = { registry = "ghcr.io", package = "bowlofarugula/geo", version = "0.0.1" }

[component.fluid]
source = { registry = "ghcr.io", package = "bowlofarugula/fluid", version = "0.0.1" }

[component.ftl-mcp-gateway]
source = { registry = "ghcr.io", package = "fastertools:mcp-gateway", version = "0.0.13-alpha.0" }
allowed_outbound_hosts = ["http://*.spin.internal"]

[component.ftl-mcp-gateway.variables]
component_names = "fluid,geo"

[[trigger.http]]
route = "/..."
component = "ftl-mcp-gateway"

[[trigger.http]]
route = { private = true }
component = "geo"

[[trigger.http]]
route = { private = true }
component = "fluid"
```

## Step 6: Deploy

With your generated `spin.toml`:

```bash
# Local development
spin up

# Deploy to Fermyon Cloud
spin deploy

# Deploy to your own infrastructure
spin build && docker build -t my-platform .
```

## Advanced Features

### Multi-Stage Synthesis

Under the hood, FTL uses a two-stage CUE transformation:

1. **Stage 1**: Your config → SpinDL (intermediate model)
2. **Stage 2**: SpinDL → spin.toml

This allows for:
- Type safety at each layer
- Composable transformations
- Multiple input formats (Go, YAML, JSON, CUE)
- Consistent output regardless of input

### Direct CUE Usage

For maximum control, write CUE directly:

```cue
package platform

import "github.com/fastertools/ftl"

app: ftl.#FTLApplication & {
    name: "my-platform"
    tools: [
        {
            id: "geo"
            source: {
                registry: "ghcr.io"
                package: "bowlofarugula/geo"
                version: "0.0.1"
            }
        },
    ]
}
```

### Composability

Mix and match tools from different sources:

```go
// From registries
app.AddTool("geo").FromRegistry("ghcr.io", "bowlofarugula/geo", "0.0.1")

// From local files
app.AddTool("custom").FromLocal("./my-tool.wasm")

// From URLs (coming soon)
app.AddTool("remote").FromURL("https://example.com/tool.wasm")
```

## Summary

The FTL framework makes it easy to:
1. **Compose** multiple WASM tools into a platform
2. **Configure** with environment variables and build settings
3. **Secure** with enterprise SSO or custom authentication
4. **Deploy** to any Spin-compatible infrastructure

The key insight is that FTL handles all the boilerplate - networking, routing, authentication, and configuration - so you can focus on selecting and configuring your tools.