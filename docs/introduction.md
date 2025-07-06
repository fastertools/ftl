# Introduction

Welcome to FTL! This guide will help you understand what FTL is, how it works, and how to get started building MCP servers as WebAssembly components.

## What is FTL?

FTL (Faster Tools Library) is a developer platform for building and deploying Model Context Protocol (MCP) servers as WebAssembly components. It provides a complete workflow for creating, testing, composing, and deploying MCP components that can be used by AI agents and assistants.

FTL solves several key challenges in MCP development:
- **Multi-language Support**: Write MCP servers in Rust, TypeScript, or JavaScript
- **Component Composition**: Combine multiple MCP servers into a single deployable unit
- **Edge Deployment**: Deploy your MCP servers anywhere using Spin's WebAssembly runtime
- **Developer Experience**: Hot reload, automatic dependency management, and intuitive CLI

## Core Concepts

### MCP Components

An **MCP component** is a WebAssembly module that implements the Model Context Protocol. Each component can expose:
- **Tools**: Functions that AI agents can call to perform actions
- **Resources**: Data sources that AI agents can read
- **Prompts**: Reusable prompt templates for AI interactions

### Projects

A **project** is a collection of MCP components that are deployed together. Projects use Spin's manifest format to define routing and configuration for each component.

### Component Development

FTL provides SDKs for building MCP components in multiple languages:

**TypeScript/JavaScript:**
```typescript
import { createHandler } from '@fastertools/ftl-sdk';

export const handler = createHandler({
    tools: [...],      // Your MCP tools
    resources: [...],  // Your MCP resources
    prompts: [...]     // Your MCP prompts
});
```

**Rust:**
```rust
use ftl_sdk::*;

create_handler!(
    tools: get_tools,
    resources: get_resources,
    prompts: get_prompts
);
```

### WebAssembly Runtime

FTL uses [Spin](https://www.fermyon.com/spin) as its WebAssembly runtime, providing:
- Secure sandboxing for each component
- HTTP routing between components
- Fast cold starts and execution
- Deploy anywhere Spin runs

## Architecture Overview

```
┌─────────────────┐
│   AI Agent      │
│ (Claude, GPT-4) │
└────────┬────────┘
         │ MCP Protocol
         ▼
┌─────────────────┐
│  Spin Runtime   │
│  (HTTP Router)  │
└────────┬────────┘
         │
    ┌────┴────┬─────────┬─────────┐
    ▼         ▼         ▼         ▼
┌────────┐┌────────┐┌────────┐┌────────┐
│Weather ││GitHub  ││Database││Custom  │
│Tool    ││Tool    ││Tool    ││Tool    │
│(TS)    ││(Rust)  ││(JS)    ││(Any)   │
└────────┘└────────┘└────────┘└────────┘
```

Each component:
- Runs in its own WebAssembly sandbox
- Has its own HTTP route (e.g., `/weather/mcp`)
- Can be developed and tested independently
- Can be published and shared via OCI registries

## Why FTL?

### For MCP Developers

- **Language Choice**: Use your preferred language (Rust, TypeScript, JavaScript)
- **Fast Iteration**: Hot reload with `ftl watch` for rapid development
- **Easy Testing**: Built-in test runners for each language
- **Simple Deployment**: One command to build and deploy

### For AI Applications

- **Composability**: Mix and match components from different sources
- **Performance**: WebAssembly provides near-native execution speed
- **Security**: Components run in isolated sandboxes
- **Portability**: Deploy anywhere - edge, cloud, or on-premise

### For Teams

- **Component Marketplace**: Share components via OCI registries
- **Version Control**: Standard Git workflows for collaboration
- **Independent Development**: Teams can work on components separately
- **Unified Deployment**: Compose components into cohesive applications

## Getting Started

Ready to build your first MCP component? Continue to the [Quick Start Guide](./quickstart.md) or dive into the [CLI Reference](./cli-reference.md).

## Learn More

- [Model Context Protocol](https://modelcontextprotocol.io) - The protocol specification
- [Spin Documentation](https://developer.fermyon.com/spin) - The WebAssembly runtime
- [WebAssembly Component Model](https://component-model.bytecodealliance.org/) - The component standard