# Your First FTL Project

Welcome to FTL! In this tutorial, you'll create your first FTL project, implement a simple tool, and see it running in action.

## What You'll Build

By the end of this tutorial, you'll have:
- A working FTL project with a "hello world" tool
- Understanding of the basic FTL development workflow
- A running MCP server you can test with a client

## Prerequisites

- FTL CLI installed ([Installation Guide](../../README.md#installation))
- Basic familiarity with command line
- A text editor

## Step 1: Initialize Your Project

First, let's create a new FTL project:

```bash
ftl init my-first-project
cd my-first-project
```

This creates a new directory with the basic FTL project structure:

```
my-first-project/
├── ftl.toml          # Project configuration
├── spin.toml         # Spin framework configuration
└── README.md         # Getting started guide
```

Let's look at what was created:

```bash
cat ftl.toml
```

The `ftl.toml` file defines your project metadata and which tools it contains (initially empty).

## Step 2: Add Your First Tool

Now let's add a simple tool to our project. We'll create a "hello world" tool in Rust:

```bash
ftl add hello-world --language rust
```

This command:
- Creates a new tool component in the `components/hello-world/` directory
- Updates `ftl.toml` to include the new tool
- Generates boilerplate code for a Rust-based MCP tool

Let's examine what was created:

```bash
tree components/hello-world/
```

You should see:

```
components/hello-world/
├── Cargo.toml        # Rust package configuration
├── src/
│   └── lib.rs        # Your tool implementation
└── README.md         # Tool-specific documentation
```

## Step 3: Implement Your Tool

Open `components/hello-world/src/lib.rs` in your text editor. You'll see generated boilerplate code.

Let's modify it to create a simple greeting tool:

```rust
use ftl_sdk::prelude::*;

#[tool]
pub fn say_hello(name: Option<String>) -> ToolResponse {
    let name = name.unwrap_or_else(|| "World".to_string());
    let message = format!("Hello, {}! Welcome to FTL.", name);
    
    ToolResponse::ok(message)
}
```

This creates an MCP tool that:
- Takes an optional `name` parameter
- Returns a greeting message
- Uses "World" as the default if no name is provided

## Step 4: Build Your Project

Now let's build the project to compile our tool to WebAssembly:

```bash
ftl build
```

This command:
- Compiles each tool to a WebAssembly component
- Generates MCP tool schemas automatically
- Updates the Spin configuration with component information

You should see output indicating successful compilation.

## Step 5: Run Your Project Locally

Start the local development server:

```bash
ftl up
```

This command:
- Starts the FTL development server
- Hosts your MCP server locally
- Provides logging and debugging information

You should see output similar to:

```
✅ FTL server started successfully
🌐 MCP server available at: http://localhost:3000
📋 Available tools: hello-world/say_hello
```

## Step 6: Test Your Tool

Now let's test your tool! You have several options:

### Option A: Using curl

```bash
curl -X POST http://localhost:3000/tools/call \
  -H "Content-Type: application/json" \
  -d '{
    "name": "hello-world/say_hello",
    "arguments": {
      "name": "Developer"
    }
  }'
```

### Option B: Using Claude Desktop

1. Add this server to your Claude Desktop configuration:
   ```json
   {
     "mcpServers": {
       "my-first-project": {
         "command": "curl",
         "args": ["-X", "GET", "http://localhost:3000/tools/list"]
       }
     }
   }
   ```

2. Restart Claude Desktop and you should see your tool available

### Option C: Using the MCP Inspector

Visit `http://localhost:3000` in your browser to use the built-in MCP inspector.

## Step 7: Experiment

Try modifying your tool:

1. **Add more parameters:**
   ```rust
   #[tool]
   pub fn say_hello(name: Option<String>, greeting: Option<String>) -> ToolResponse {
       let name = name.unwrap_or_else(|| "World".to_string());
       let greeting = greeting.unwrap_or_else(|| "Hello".to_string());
       let message = format!("{}, {}! Welcome to FTL.", greeting, name);
       
       ToolResponse::ok(message)
   }
   ```

2. **Rebuild and test:**
   ```bash
   ftl build
   # The server will automatically reload
   ```

## What You've Learned

Congratulations! You've just:

✅ **Created your first FTL project** using `ftl init`  
✅ **Added a tool component** with `ftl add`  
✅ **Implemented a simple MCP tool** in Rust  
✅ **Built your project** to WebAssembly with `ftl build`  
✅ **Run a local MCP server** with `ftl up`  
✅ **Tested your tool** with a real client  

## Key Concepts

- **Tools are WebAssembly components:** Your Rust code compiles to WASM for security and performance
- **Automatic schema generation:** FTL generates MCP schemas from your tool signatures
- **Hot reload:** Changes rebuild automatically during development
- **Language agnostic:** The same patterns work across all supported languages

## Next Steps

Ready for more? Try the [Polyglot Composition](./polyglot-composition.md) tutorial to see FTL's killer feature: running tools written in different languages together seamlessly.

Or explore:
- [Core Concepts](../core-concepts/) - Understand how FTL works under the hood
- [How-to Guides](../guides/) - Solve specific problems
- [Examples](../../examples/) - See more complex patterns