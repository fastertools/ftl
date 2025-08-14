# FTL Examples

This directory contains examples of using the FTL (Faster Tool Layer) framework to build MCP tool platforms.

## Quick Start

Choose your preferred approach:

### 1. Go CDK (Programmatic)

```bash
go run simple-public.go > spin.toml
spin up
```

### 2. YAML (Declarative)

```bash
ftl synth my-platform.yaml > spin.toml
spin up
```

### 3. Direct CUE (Advanced)

```bash
ftl synth direct.cue > spin.toml
spin up
```

## Examples by Category

### Basic Examples

| File | Description | Key Features |
|------|-------------|--------------|
| `simple-public.go` | Minimal public platform | Basic tool setup |
| `simple-app.go` | Simple app with local WASM | Local component |
| `my-platform.yaml` | YAML configuration | Declarative approach |

### Authentication Examples

| File | Description | Key Features |
|------|-------------|--------------|
| `auth-app.go` | WorkOS authentication | Enterprise SSO |
| `scientific-platform.yaml` | Full platform with auth | Private access, WorkOS |

### Advanced Examples

| File | Description | Key Features |
|------|-------------|--------------|
| `cdk-app.go` | Complex multi-tool platform | Build configs, watch patterns |
| `scientific-platform.go` | Programmatic with env vars | Complete configuration |
| `direct.cue` | Direct CUE definition | Maximum control, type safety |
| `working-example.go` | Production-ready example | All features combined |

## Using the Examples

### Step 1: Choose Your Approach

**Go CDK** - Best for:
- Dynamic configurations
- CI/CD integration
- Complex logic and conditionals

**YAML** - Best for:
- Static configurations
- GitOps workflows
- Simplicity and readability

**CUE** - Best for:
- Maximum type safety
- Complex constraints
- Multi-environment configs

### Step 2: Generate spin.toml

Using Go:
```bash
go run examples/simple-public.go > spin.toml
```

Using FTL CLI with YAML:
```bash
ftl synth examples/my-platform.yaml > spin.toml
```

Using FTL CLI with CUE:
```bash
ftl synth examples/direct.cue > spin.toml
```

### Step 3: Deploy

Local development:
```bash
spin up
# Your platform is now running at http://localhost:3000
```

Deploy to Fermyon Cloud:
```bash
spin deploy
```

## Key Concepts

### Tools
Components that provide MCP functionality. Examples:
- `ghcr.io/bowlofarugula/geo@0.0.1` - Geological computations
- `ghcr.io/bowlofarugula/fluid@0.0.1` - Fluid dynamics

### MCP Gateway
Automatically added to route requests to your tools. Handles:
- Request routing
- Protocol translation
- Service discovery

### MCP Authorizer
Automatically added when authentication is enabled. Provides:
- JWT validation
- WorkOS SSO integration
- Request authorization

### Routes
- **Public route** (`/...`) - Entry point for all requests
- **Private routes** - Internal communication between components

## Architecture

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

## The Synthesis Pipeline

FTL uses a two-stage CUE transformation pipeline:

1. **Stage 1**: Your config (Go/YAML/CUE) → SpinDL (intermediate model)
2. **Stage 2**: SpinDL → spin.toml

This provides:
- Type safety at each layer
- Composable transformations
- Consistent output regardless of input format

## Environment Variables

Configure your tools with environment variables:

```go
app.AddTool("geo").
    FromRegistry("ghcr.io", "bowlofarugula/geo", "0.0.1").
    WithEnv("LOG_LEVEL", "debug").
    WithEnv("MAX_MEMORY", "4096").
    Build()
```

```yaml
tools:
  - id: geo
    environment:
      LOG_LEVEL: debug
      MAX_MEMORY: "4096"
```

## Authentication

### WorkOS (Enterprise SSO)

```go
app.EnableWorkOSAuth("org_12345")
```

```yaml
access: private
auth:
  provider: workos
  org_id: org_12345
```

### Custom JWT

```go
app.EnableCustomAuth("https://auth.example.com", "my-audience")
```

```yaml
access: private
auth:
  provider: custom
  jwt_issuer: https://auth.example.com
  jwt_audience: my-audience
```

## Build Configuration

For local tools that need building:

```go
app.AddTool("my-tool").
    FromLocal("./my-tool.wasm").
    WithBuild("cargo build --release").
    WithWatch("src/**/*.rs", "Cargo.toml").
    Build()
```

```yaml
tools:
  - id: my-tool
    source: ./my-tool.wasm
    build:
      command: cargo build --release
      watch:
        - src/**/*.rs
        - Cargo.toml
```

## More Information

- [WALKTHROUGH.md](WALKTHROUGH.md) - Detailed tutorial from idea to deployment
- [FTL Documentation](https://github.com/fastertools/ftl-cli) - Main project docs
- [Spin Documentation](https://developer.fermyon.com/spin) - WebAssembly platform docs