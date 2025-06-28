# Getting Started

This guide will walk you through the process of creating, building, and deploying your first FTL tool.

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)
- [Docker](https://docs.docker.com/get-docker/) (for the local development server)

## 1. Install the FTL CLI

The FTL command-line interface is the primary entry point for developers using the FTL platform. You can install it using `cargo`:

```bash
cargo install ftl
```

## 2. Create a New Tool

The `ftl new` command will create a new directory with a simple `ftl.toml` manifest, a `Cargo.toml` file, and a `src/lib.rs` file with a boilerplate tool implementation.

```bash
ftl new my-tool --description "A new tool for my agent"
cd my-tool
```

## 3. Implement Your Tool

Open `src/lib.rs` in your favorite editor and implement the `ftl_sdk::Tool` trait. This trait defines the name, description, input schema, and `call` method for your tool.

```rust
use ftl_sdk::prelude::*;

#[derive(Clone)]
struct MyTool;

impl Tool for MyTool {
    fn name(&self) -> &'static str { "my-tool" }
    fn description(&self) -> &'static str { "My tool description" }
    
    fn input_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "input": {"type": "string"}
            },
            "required": ["input"]
        })
    }
    
    fn call(&self, args: &serde_json::Value) -> Result<ToolResult, ToolError> {
        let input = args["input"].as_str()
            .ok_or(ToolError::InvalidArguments("input required".into()))?;
            
        Ok(ToolResult::text(format!("Processed: {}", input)))
    }
}

ftl_sdk::ftl_mcp_server!(MyTool);
```

## 4. Serve Locally

The `ftl serve` command will start a local development server with hot reloading. This allows you to test your tool without having to deploy it to the FTL Edge.

```bash
ftl serve
```

You can test your tool by sending it a JSON-RPC request:

```bash
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"my-tool","arguments":{"input":"test"}},"id":1}'
```

## 5. Deploy to FTL Edge

The `ftl deploy` command will deploy your tool to the FTL Edge, where it can be called by your AI agents.

```bash
ftl deploy
```

You will be prompted to log in to your FTL account. Once you are logged in, your tool will be deployed and you will be given a URL that you can use to call it.

