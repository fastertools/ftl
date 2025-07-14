# Quick Start Guide

This guide will walk you through creating your first MCP component with FTL in under 5 minutes.

## Prerequisites

Before you begin, ensure you have:
- Rust toolchain installed ([rustup.rs](https://rustup.rs))
- Node.js 20+ (for TypeScript/JavaScript components)

## Installation

Install the FTL CLI using cargo:

```bash
cargo install ftl-cli
```

Or if you have cargo-binstall for faster installation:

```bash
cargo binstall ftl-cli
```

## Create Your First Project

### 1. Initialize a Project

```bash
ftl init my-assistant
cd my-assistant
```

This creates a new FTL project with:
- `spin.toml` - The Spin manifest file
- `.gitignore` - Git ignore configuration

### 2. Add a Component

Let's add a TypeScript component that provides a simple echo tool:

```bash
ftl add echo-tool --language typescript --description "A simple echo tool"
```

FTL will:
- Create the component directory structure
- Install the TypeScript SDK
- Set up the build configuration
- Add the component to your Spin manifest

### 3. Explore the Component

Navigate to your component:

```bash
cd echo-tool
```

The component structure:

<pre>
echo-tool/
├── ftl.toml           # Component metadata
├── Makefile           # Build commands
└── handler/           # Component source
    ├── package.json   # Node dependencies
    ├── src/
    │   ├── index.ts   # Main handler
    │   └── features.ts # Tools, resources, prompts
    └── test/          # Component tests
</pre>

### 4. Customize Your Tool

Open `handler/src/features.ts` and modify the echo tool:

```typescript
import { createTool } from 'ftl-mcp';

export const tools = [
  createTool({
    name: 'echo',
    description: 'Echo a message with enthusiasm!',
    inputSchema: {
      type: 'object',
      properties: {
        message: { 
          type: 'string', 
          description: 'Message to echo back' 
        },
        excitement: {
          type: 'number',
          description: 'Excitement level (1-10)',
          minimum: 1,
          maximum: 10
        }
      },
      required: ['message']
    },
    execute: async (args) => {
      const message = args.message || 'Hello, world!';
      const excitement = args.excitement || 5;
      const exclamation = '!'.repeat(excitement);
      return `Echo: ${message}${exclamation}`;
    }
  }),
];
```

### 5. Test Your Component

Run the component tests:

```bash
make test
```

### 6. Run Your Project

Go back to the project root and start the development server:

```bash
cd ..
ftl watch
```

Your MCP server is now running! The echo tool is available at:
- `http://localhost:3000/echo-tool/mcp`

### 7. Test with an MCP Client

You can test your MCP server using any MCP client. Here's a quick test using curl:

```bash
# List available tools
curl -X POST http://localhost:3000/echo-tool/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "tools/list",
    "id": 1
  }'

# Call the echo tool
curl -X POST http://localhost:3000/echo-tool/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "tools/call",
    "params": {
      "name": "echo",
      "arguments": {
        "message": "Hello FTL",
        "excitement": 8
      }
    },
    "id": 2
  }'
```

## Next Steps

### Add More Components

Try adding components in different languages:

```bash
# Add a Rust component
ftl add rust-tool --language rust --description "A Rust-powered tool"

# Add a JavaScript component  
ftl add js-tool --language javascript --description "A JavaScript tool"
```

### Build for Production

```bash
# Build all components in release mode
ftl build --release

# Run in production mode
ftl up --port 8080
```

### Publish Your Component

```bash
# Publish to GitHub Container Registry
ftl publish --tag v1.0.0

# Or publish to Docker Hub
ftl publish --registry docker.io --tag latest
```

## Common Commands

| Command | Description |
|---------|-------------|
| `ftl init` | Create a new project |
| `ftl add` | Add a component to the project |
| `ftl build` | Build components |
| `ftl test` | Run component tests |
| `ftl watch` | Start dev server with hot reload |
| `ftl up` | Run the project |
| `ftl publish` | Publish to registry |

## Learn More

- [Component Development](./components.md) - Deep dive into building components
- [CLI Reference](./cli-reference.md) - Complete command documentation
- [Publishing Guide](./publishing.md) - Share your components
- [Deployment Guide](./deployment.md) - Deploy to production