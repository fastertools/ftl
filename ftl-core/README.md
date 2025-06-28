# FTL Core

This crate provides the core building blocks for creating FTL tools. It is the open-source foundation of the FTL platform, designed for performance, safety, and efficiency.

## Purpose

`ftl-core` is the library that developers will use to implement the logic of their tools. It provides the necessary abstractions to integrate with the FTL runtime and the Model Context Protocol (MCP) without needing to know the low-level details.

## Key Components

- **`Tool` Trait:** The central abstraction for all FTL tools. It defines the interface that the FTL runtime uses to execute a tool.
- **`ftl_mcp_server!` Macro:** A macro that generates the necessary boilerplate to expose a `Tool` implementation as a WebAssembly component that can be served over MCP.
- **`ToolResult` and `ToolError`:** Standardized types for returning success and error states from a tool.
- **Prelude:** The `ftl_core::prelude` module re-exports the most commonly used items for convenience.

## Usage

To create a new tool, you will typically add `ftl-core` as a dependency in your `Cargo.toml` and implement the `Tool` trait.

```rust
use ftl_core::prelude::*;

#[derive(Clone)]
struct MyTool;

impl Tool for MyTool {
    // ... implementation ...
}

ftl_core::ftl_mcp_server!(MyTool);
```

For more detailed information on developing tools, please see the main project [documentation](../docs/developing-tools.md).
