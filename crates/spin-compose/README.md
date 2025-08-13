# spin-compose

> Infrastructure as Code for WebAssembly - compose and synthesize Spin applications

`spin-compose` (aliased as `spinc`) is a powerful composition framework for Spin applications, inspired by AWS CDK. It allows you to define complex, multi-component Spin applications using high-level constructs that synthesize into detailed `spin.toml` manifests.

## Features

- **High-level constructs** - Abstract away Spin manifest complexity
- **Multiple input formats** - YAML, JSON, TOML, or native CUE
- **Type-safe validation** - Catch errors before deployment
- **Composable patterns** - Reuse and share architectural patterns
- **Environment management** - Different configs for dev/staging/prod

## Installation

```bash
cargo install spin-compose
```

## Quick Start

Create a simple MCP application:

```yaml
# spinc.yaml
name: my-mcp-app
template: mcp
auth:
  enabled: true
  issuer: https://auth.example.com
  audience: [api.example.com]
components:
  calculator:
    source: ghcr.io/example/calculator:1.0.0
```

Synthesize to `spin.toml`:

```bash
spinc synth
```

Preview changes:

```bash
spinc diff
```

## Construct Levels

### L1: Core Constructs
Direct mappings to Spin manifest primitives (components, triggers, variables).

### L2: Patterns
Common architectural patterns (authenticated endpoints, service mesh, API gateways).

### L3: Solutions
Complete application templates (MCP applications, WordPress sites, microservices).

## Commands

- `spinc init` - Initialize a new spin-compose project
- `spinc synth` - Synthesize spin.toml from your configuration
- `spinc diff` - Show what would change in spin.toml
- `spinc validate` - Validate your configuration
- `spinc construct list` - List available constructs
- `spinc construct add <name>` - Add a construct to your project

## Advanced Usage

For complex scenarios, use native CUE:

```cue
// spinc.cue
import "spinc.io/solutions/mcp"

app: mcp.#McpApplication & {
    name: "my-app"
    
    if env.ENABLE_AUTH {
        auth: enabled: true
    }
    
    components: {
        for tool in tools {
            "\(tool.name)": source: tool.registry
        }
    }
}
```

## Architecture

spin-compose uses CUE as its configuration and validation engine, providing:
- Strong typing and validation
- Composition and imports
- Conditionals and loops
- Functions and computed fields

## Contributing

We welcome contributions! The MCP construct serves as our reference implementation for new patterns.

## License

MIT OR Apache-2.0