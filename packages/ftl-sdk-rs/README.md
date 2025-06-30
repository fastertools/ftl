# FTL SDK

This crate provides the core building blocks for creating FTL tools. It is the open-source foundation of the FTL platform, designed for performance, safety, and efficiency.

## Purpose

`ftl-sdk-rs` is the library that developers will use to implement the logic of their tools. It provides the necessary abstractions to integrate with the FTL runtime and the Model Context Protocol (MCP) without needing to know the low-level details.

## Key Components

- **`Tool` Trait:** The central abstraction for all FTL tools. It defines the interface that the FTL runtime uses to execute a tool.
- **`ftl_mcp_server!` Macro:** A macro that generates the necessary boilerplate to expose a `Tool` implementation as a WebAssembly component that can be served over MCP.
- **`ftl_mcp_gateway!` Macro:** A macro for creating gateway components that aggregate multiple tools into a single MCP endpoint.
- **Gateway Module:** Provides `McpGateway`, `GatewayConfig`, and related types for building MCP gateways.
- **`ToolResult` and `ToolError`:** Standardized types for returning success and error states from a tool.
- **Prelude:** The `ftl_sdk_rs::prelude` module re-exports the most commonly used items for convenience.

## Usage

### Creating a Tool

To create a new tool, you will typically add `ftl-sdk-rs` as a dependency in your `Cargo.toml` and implement the `Tool` trait.

```rust
use ftl_sdk_rs::prelude::*;

#[derive(Clone)]
struct MyTool;

impl Tool for MyTool {
    // ... implementation ...
}

ftl_sdk_rs::ftl_mcp_server!(MyTool);
```

### Creating a Gateway

To create a gateway that aggregates multiple tools:

```rust
use ftl_sdk_rs::{ftl_mcp_gateway, gateway::{GatewayConfig, ToolEndpoint}, mcp::ServerInfo};

fn create_gateway_config() -> GatewayConfig {
    GatewayConfig {
        tools: vec![
            ToolEndpoint {
                name: "tool1".to_string(),
                route: "/tool1".to_string(),
                description: None,
            },
            // ... more tools
        ],
        server_info: ServerInfo {
            name: "my-gateway".to_string(),
            version: "1.0.0".to_string(),
        },
        base_url: "".to_string(),
    }
}

ftl_mcp_gateway!(create_gateway_config());
```

For more detailed information on developing tools, please see the main project [documentation](../docs/developing-tools.md).
