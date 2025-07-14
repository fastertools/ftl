# Architecture

The FTL platform is designed to be a high-performance, scalable, and secure system for executing AI agent tools. It is built on a foundation of Rust, WebAssembly, and the Model Context Protocol (MCP).

## Technology Stack

- **Language:** Rust is used for its performance, safety, and concurrency features.
- **Infrastructure:** FTL tools are compiled to WebAssembly and executed on an edge-first infrastructure. This provides sandboxing, portability, and near-native performance.
- **Protocol:** The Model Context Protocol (MCP) is used for communication between AI agents and FTL tools.

## Components

### `ftl-cli`

The `ftl-cli` is the command-line interface for the FTL platform. It is responsible for:

- Scaffolding new tools.
- Building tools into WebAssembly components.
- Serving tools locally for development.
- Deploying tools to the FTL Edge.
- Managing toolkits.

### `ftl-mcp`

FTL uses the `ftl-mcp` SDK for building MCP components. The ftl-mcp repository provides:

- Templates for creating new MCP components in Rust, TypeScript, and JavaScript
- The core infrastructure for building WebAssembly-based MCP servers
- Gateway functionality for aggregating multiple tools
- Utilities for routing MCP requests between components
- Examples and best practices for component development

### FTL Edge

The FTL Edge is a managed platform for deploying and serving tools. It is a global network of edge servers that can execute efficient tools with sub-millisecond compute overhead. The FTL Edge is responsible for:

- Hosting and serving FTL tools.
- Scaling the execution of tools to meet demand.
- Providing a secure and reliable environment for executing tools.

### Toolkits and Gateways

FTL supports bundling multiple tools into a single deployable unit called a toolkit. Each toolkit includes:

- Multiple tool components (WebAssembly modules)
- An automatically generated gateway component
- Unified configuration and deployment

The gateway component:
- Provides a single MCP endpoint (`/mcp`) that aggregates all tools
- Handles tool discovery dynamically
- Routes tool calls to the appropriate component
- Maintains individual tool endpoints for direct access

## Workflow

### Individual Tools

1.  **Develop:** The developer uses the `ftl` CLI to create a new tool using ftl-mcp templates and implements the MCP protocol handlers.
2.  **Build:** The `ftl build` command compiles the tool to a WebAssembly component.
3.  **Test:** The `ftl serve` command starts a local development server that can be used to test the tool.
4.  **Deploy:** The `ftl deploy` command deploys the tool to the FTL Edge.
5.  **Execute:** An AI agent sends a JSON-RPC request to the FTL Edge to execute the tool. The FTL Edge routes the request to the nearest edge server, which executes the tool and returns the result to the agent.

### Toolkits

1.  **Build Tools:** Individual tools are built separately using `ftl build`.
2.  **Create Toolkit:** The `ftl toolkit build` command bundles multiple tools and generates a gateway component.
3.  **Test Toolkit:** The `ftl toolkit serve` command starts a local server with all tools and the gateway.
4.  **Deploy Toolkit:** The `ftl toolkit deploy` command deploys the entire toolkit as a single unit.
5.  **Execute:** AI agents can either:
    - Call the gateway endpoint to discover and use any tool in the toolkit
    - Call individual tool endpoints directly
