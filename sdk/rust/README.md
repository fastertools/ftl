# ftl-sdk (Rust)

Rust SDK for building Model Context Protocol (MCP) tools on WebAssembly.

[Documentation](https://docs.rs/ftl-sdk) | [Examples](https://github.com/fastertools/ftl/tree/main/templates/rust)

## Installation

```toml
[dependencies]
ftl-sdk = { version = "0.2.10", features = ["macros"] }
schemars = "0.8"  # For automatic schema generation
serde = { version = "1.0", features = ["derive"] }
```

## Overview

This SDK provides:
- MCP protocol type definitions
- `tools!` macro for defining multiple tools with minimal boilerplate
- Response macros (`text!`, `error!`, `structured!`) for ergonomic responses
- Automatic JSON schema generation using schemars
- Convenience methods for creating responses

## Quick Start

### Using the `tools!` Macro

The simplest way to create tools:

```rust
use ftl_sdk::{tools, text, ToolResponse};
use serde::Deserialize;
use schemars::JsonSchema;

#[derive(Deserialize, JsonSchema)]
struct AddInput {
    /// First number to add
    a: i32,
    /// Second number to add
    b: i32,
}

#[derive(Deserialize, JsonSchema)]
struct SubtractInput {
    /// Number to subtract from
    a: i32,
    /// Number to subtract
    b: i32,
}

tools! {
    /// Adds two numbers together
    fn add(input: AddInput) -> ToolResponse {
        let result = input.a + input.b;
        text!("{} + {} = {}", input.a, input.b, result)
    }
    
    /// Subtracts two numbers
    fn subtract(input: SubtractInput) -> ToolResponse {
        let result = input.a - input.b;
        text!("{} - {} = {}", input.a, input.b, result)
    }
}
```

The `tools!` macro automatically
- Generates the HTTP handler for all tools
- Creates metadata from function names and doc comments
- Derives JSON schema from your input types using schemars
- Routes GET/POST requests appropriately
- Supports multiple tools in one component

### Manual Implementation

For more control, implement the protocol manually:

## Building to WebAssembly

Tools must be compiled to WebAssembly for the Spin platform:

```toml
# Cargo.toml
[dependencies]
ftl-sdk = { version = "0.2.10", features = ["macros"] }
schemars = "0.8"
serde = { version = "1.0", features = ["derive"] }
spin-sdk = "4.0"

[lib]
crate-type = ["cdylib"]
```

Build command:
```bash
cargo build --target wasm32-wasip1 --release
```

## Response Helpers

```rust
use ftl_sdk::{text, error, structured, ToolResponse, ToolContent};
use serde_json::json;

// Simple text response with macros
let response = text!("Hello, world!");

// With formatting
let response = text!("Hello, {}!", name);

// Error response
let response = error!("Something went wrong: {}", reason);

// Response with structured content
let data = serde_json::json!({ "result": 42 });
let response = structured!(data, "Calculation complete");

// Or use the builder methods directly
let response = ToolResponse::text("Hello, world!");
let response = ToolResponse::error("Something went wrong");
let response = ToolResponse::with_structured(
    "Calculation complete",
    serde_json::json!({ "result": 42 })
);

// Multiple content items
let response = ToolResponse {
    content: vec![
        ToolContent::text("Processing complete"),
        ToolContent::image(base64_data, "image/png"),
    ],
    structured_content: None,
    is_error: None,
};
```

## Advanced Features

### Async Tools

The `tools!` macro supports async functions:

```rust
use ftl_sdk::{tools, text, ToolResponse};
use serde::Deserialize;
use schemars::JsonSchema;

#[derive(Deserialize, JsonSchema)]
struct WeatherInput {
    location: String,
}

#[derive(Deserialize, JsonSchema)]
struct StatusInput {
    service: String,
}

tools! {
    /// Fetch weather data
    async fn fetch_weather(input: WeatherInput) -> ToolResponse {
        let weather = fetch_from_api(&input.location).await;
        text!("Weather in {}: {}", input.location, weather)
    }
    
    /// Another async tool
    async fn check_status(input: StatusInput) -> ToolResponse {
        let status = get_status(&input.service).await;
        text!("Status: {}", status)
    }
}
```

### Multiple Tools Per Component

Define as many tools as needed in one component:

```rust
use ftl_sdk::{tools, text, structured, ToolResponse};
use serde::Deserialize;
use schemars::JsonSchema;
use serde_json::json;

#[derive(Deserialize, JsonSchema)]
struct TextInput {
    text: String,
}

#[derive(Deserialize, JsonSchema)]
struct DataInput {
    data: serde_json::Value,
}

#[derive(Deserialize, JsonSchema)]
struct ReportInput {
    topic: String,
}

tools! {
    /// Process text
    fn process_text(input: TextInput) -> ToolResponse {
        text!("Processed: {}", input.text)
    }
    
    /// Analyze data
    fn analyze_data(input: DataInput) -> ToolResponse {
        let result = analyze(&input.data);
        structured!(result, "Analysis complete")
    }
    
    /// Generate report
    async fn generate_report(input: ReportInput) -> ToolResponse {
        let report = create_report(&input).await;
        text!("{}", report)
    }
}
```

## Development

### Building

```bash
cargo build --target wasm32-wasip1 --release
```

### Testing

```bash
cargo test
```

### Code Quality

```bash
# Format code
cargo fmt

# Run linting
cargo clippy

# Run all checks
make quality
```

## License

Apache-2.0 - see [LICENSE](../../LICENSE) for details.