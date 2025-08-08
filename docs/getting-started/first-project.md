# Your First FTL Project

Welcome to FTL! In this tutorial, you'll create your first FTL project, implement a simple tool, and see it running in action.

## What You'll Build

By the end of this tutorial, you'll have:
- A working FTL project with a "hello world" tool
- Understanding of the basic FTL development workflow
- A running MCP server you can test with a client

## Prerequisites

- FTL CLI installed ([Installation Guide](../../README.md#installing-and-updating))
- Basic familiarity with command line
- A text editor
- Some familiarity with Rust is helpful. See our other [examples](../../examples/demo/) for other language examples

## Step 1: Initialize Your Project

First, let's create a new FTL project:

```bash
ftl init my-first-project
```

This creates a new directory with the basic FTL project structure:

```
my-first-project/
├── ftl.toml          # Project configuration
└── README.md         # Getting started guide
```

Let's look at what was created:

```bash
cat ftl.toml

[project]
name = "my-first-project"
version = "0.1.0"
description = "FTL MCP server for hosting MCP tools"
authors = []
access_control = "public"

[tools]

[mcp]
gateway = "ghcr.io/fastertools/mcp-gateway:0.0.XX"
authorizer = "ghcr.io/fastertools/mcp-authorizer:0.0.XX"
validate_arguments = false
```

The `ftl.toml` file defines your project metadata and which tools it contains (initially empty).

`access_control = "public"` - By default your tools will be accessible publicly.

`validate_arguments = false` - Our MCP gateway can do tool input validation for you if you want. You can also handle this yourself if desired.

## Step 2: Add Your First Tool

Now let's add a simple tool to our project. We'll create a "hello world" tool in Rust:

```bash
ftl add hello-world --language rust
```

This command:
- Creates a new tool in the `hello-world/` directory
- Updates `ftl.toml` to include the new tool
- Generates boilerplate code for a Rust-based MCP tool

You should see:

```
.
├── ftl.toml
├── hello-world    # The tool we just created
│   ├── Cargo.toml # Provides necessary dependencies, basic linting rules, etc
│   ├── Makefile   # Standardized build commands
│   ├── README.md  # Some rust specific guidance on using our SDK 
│   └── src
│       └── lib.rs # Scaffolded out Rust code for your new tool
└── README.md
```

## Step 3: Implement Your Tool

Open `components/hello-world/src/lib.rs` in your text editor. You'll see generated boilerplate code.

Let's modify it to create a simple greeting tool:

```rust
use ftl_sdk::{tools, text, ToolResponse};
use serde::Deserialize;
use schemars::JsonSchema;

#[derive(Deserialize, JsonSchema)]
struct SayHelloInput {
    message: String,
}

tools! {
    fn say_hello(input: SayHelloInput) -> ToolResponse {
        text!("Hello {}!", input.message)
    }
}   
```

This creates an MCP tool that:
- Takes a `message` parameter
- Returns a Hello with the message passed to the tool call

## Step 4: Build Your Project

Now let's build the project to compile our tool to WebAssembly:

```bash
ftl build
→ Building 1 component in parallel

  [hello-world] ✓ Built in 9.3s
✓ All components built successfully!
```

You should see output indicating successful compilation.

## Step 5: Run Your Project Locally

Start the local development server:

```bash
ftl up
```

This command:
- Starts the FTL development server
- Hosts your MCP server locally

You should see output similar to:

```
→ Starting server...

🌐 Server will start at http://127.0.0.1:3000
⏹ Press Ctrl+C to stop

Loading Wasm components is taking a few seconds...

Logging component stdio to "/private/tmp/my-first-project/.ftl/logs/"

Serving http://127.0.0.1:3000
Available Routes:
  mcp: http://127.0.0.1:3000 (wildcard)
```

## Step 6: Test Your Tool

Your tool is now ready for testing! You can add it to your client of choice.

### Using Claude Desktop

`claude mcp add -t http hello-world http://127.0.0.1:3000`

Restart Claude Desktop and you should see your tool available to test!

## Step 7: Experiment!

Now that you've got the basics you can start developing!

Try:
- Adding a second tool 
- Adding additional parameters to your say_hello tool
- Create another component with tools in a different language!

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
- **Language agnostic:** The same patterns work across all supported languages

## Next Steps

Ready for more? Try the [Polyglot Composition](./polyglot-composition.md) tutorial to see FTL's killer feature: running tools written in different languages together seamlessly.

Or explore:
- [Core Concepts](../core-concepts/) - Understand how FTL works under the hood
- [How-to Guides](../guides/) - Solve specific problems
- [Examples](../../examples/) - See more complex patterns