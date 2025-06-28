# Developing Tools

At the core of FTL is the `Tool` trait. This trait defines the interface that all FTL tools must implement. It is designed to be simple and flexible, allowing you to create a wide variety of tools.

## The `Tool` Trait

The `Tool` trait is defined in the `ftl-sdk` crate:

```rust
pub trait Tool: Send + Sync + Clone {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn input_schema(&self) -> serde_json::Value;
    fn call(&self, args: &serde_json::Value) -> Result<ToolResult, ToolError>;
}
```

- `name`: The name of your tool. This must be unique within your FTL account.
- `description`: A short description of your tool.
- `input_schema`: A JSON Schema that defines the expected input for your tool. This is used to validate the arguments that are passed to your tool.
- `call`: The main entry point for your tool. This method is called when your tool is executed. It takes a `serde_json::Value` as input and returns a `Result<ToolResult, ToolError>`.

## `ToolResult`

The `ToolResult` enum is used to return the result of a tool's execution. It can be one of the following:

- `ToolResult::text(String)`: A plain text response.
- `ToolResult::json(serde_json::Value)`: A JSON response.

## `ToolError`

The `ToolError` enum is used to return an error from a tool's execution. It can be one of the following:

- `ToolError::InvalidArguments(String)`: The arguments passed to the tool were invalid.
- `ToolError::ExecutionError(String)`: An error occurred during the execution of the tool.

## The `ftl_mcp_server!` Macro

The `ftl_mcp_server!` macro is used to create the main entry point for your tool. It takes your tool's struct as an argument and generates the necessary code to create a WebAssembly component that can be executed by the FTL runtime.

```rust
use ftl_sdk::prelude::*;

#[derive(Clone)]
struct MyTool;

// ... implement the Tool trait for MyTool ...

ftl_sdk::ftl_mcp_server!(MyTool);
```

This macro will generate a `handle_request` function that will be exported from your WebAssembly component. This function will be called by the FTL runtime when your tool is executed. It will deserialize the incoming JSON-RPC request, call your tool's `call` method, and serialize the result as a JSON-RPC response.
