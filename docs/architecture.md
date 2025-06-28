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

### `ftl-core`

The `ftl-core` crate is the core library for FTL tools. It provides:

- The `Tool` trait, which defines the interface for all FTL tools.
- The `ftl_mcp_server!` macro, which generates the necessary boilerplate for creating a WebAssembly component that can be executed by the FTL runtime.
- A standard library of pre-built tools for common tasks.

### FTL Edge

The FTL Edge is a commercial, managed platform for deploying and serving tools. It is a global network of edge servers that can execute tools with "sub-millisecond compute overhead." The FTL Edge is responsible for:

- Hosting and serving FTL tools.
- Scaling the execution of tools to meet demand.
- Providing a secure and reliable environment for executing tools.

## Workflow

1.  **Develop:** The developer uses the `ftl` CLI to create a new tool and implement the `ftl_core::Tool` trait.
2.  **Build:** The `ftl build` command compiles the tool to a WebAssembly component.
3.  **Test:** The `ftl serve` command starts a local development server that can be used to test the tool.
4.  **Deploy:** The `ftl deploy` command deploys the tool to the FTL Edge.
5.  **Execute:** An AI agent sends a JSON-RPC request to the FTL Edge to execute the tool. The FTL Edge routes the request to the nearest edge server, which executes the tool and returns the result to the agent.
