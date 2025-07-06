# Getting Started

This guide will walk you through creating, building, and deploying your first MCP component with FTL.

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (for the FTL CLI)
- Language-specific requirements:
  - **Rust**: cargo with wasm32-wasip1 target
  - **TypeScript/JavaScript**: Node.js 20+
- [wkg](https://github.com/bytecodealliance/wasm-pkg-tools) (for publishing)

## 1. Install the FTL CLI

```bash
cargo install ftl-cli
```

## 2. Create a New Project

Start by creating a new MCP project:

```bash
ftl init my-assistant
cd my-assistant
```

This creates an empty Spin project ready for adding components.

## 3. Add Your First Component

Now add a component to your project:

```bash
ftl add weather-tool --language typescript --description "Weather information for AI agents"
```

This creates:
- `weather-tool/` - Component directory
- `weather-tool/ftl.toml` - Component configuration
- `weather-tool/Makefile` - Build automation
- `weather-tool/src/` - Component source code
- Updates `spin.toml` to include the component

## 4. Implement Your Component

Edit the component implementation in `weather-tool/src/`:

### TypeScript Example

```typescript
// weather-tool/src/features.ts
import { createTool } from '@fastertools/ftl-sdk';

export const tools = [
    createTool({
        name: 'get_weather',
        description: 'Get current weather for a location',
        inputSchema: {
            type: 'object',
            properties: {
                location: { type: 'string', description: 'City name' }
            },
            required: ['location']
        },
        async execute(args) {
            return `The weather in ${args.location} is sunny and 72°F`;
        }
    })
];

export const resources = [];
export const prompts = [];
```

### Rust Example

```rust
// weather-tool/src/features.rs
use ftl_sdk::*;

pub fn get_tools() -> Vec<Tool> {
    vec![
        create_tool(
            "get_weather",
            "Get current weather for a location",
            json!({
                "type": "object",
                "properties": {
                    "location": { "type": "string", "description": "City name" }
                },
                "required": ["location"]
            }),
            |args| {
                let location = args["location"].as_str().unwrap_or("unknown");
                Ok(format!("The weather in {} is sunny and 72°F", location))
            }
        )
    ]
}
```

## 5. Build Your Components

```bash
ftl build
```

This compiles all components in your project into optimized WebAssembly modules.

## 6. Test Locally

Run your project locally with automatic rebuilds:

```bash
ftl watch
```

Test it with a curl request:

```bash
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "tools/call",
    "params": {
      "name": "get_weather",
      "arguments": {"location": "San Francisco"}
    },
    "id": 1
  }'
```

## 7. Publish Your Components

Publish components to a container registry:

```bash
# Publish to GitHub Container Registry (default)
ftl publish --tag v1.0.0

# Or publish to Docker Hub
ftl publish --registry docker.io --tag latest
```

Your components are now available at:
- `ghcr.io/[username]/weather-tool:v1.0.0`

## 8. Add More Components

Add additional components to your project:

```bash
# Add more components
ftl add news-tool --language typescript
ftl add calculator --language rust

# Run the project with all components
ftl watch
```

## 9. Deploy to Production

Deploy your project to FTL:

```bash
ftl deploy
```

## Next Steps

- Read the [Component Development Guide](./developing-tools.md)
- Learn about [Publishing Components](./publishing.md)
- Explore [Project Composition](./composition.md)
- Check the [CLI Reference](./cli-reference.md)