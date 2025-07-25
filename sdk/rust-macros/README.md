# FTL SDK Rust Macros

Procedural macros for reducing boilerplate in FTL tool components written in Rust.

## Overview

This crate provides the `tools!` macro for defining multiple tool handler functions with minimal boilerplate. The macro:

- Supports multiple tools in a single component
- Automatically derives JSON schemas from your input types (requires `JsonSchema` derive)
- Supports both synchronous and asynchronous functions
- Generates the complete HTTP handler with routing
- Handles all the boilerplate for you

## Usage

### Basic Tool Handler

The `tools!` macro simplifies creating tool handlers:

```rust
use ftl_sdk::{tools, text};
use serde::Deserialize;
use schemars::JsonSchema;

#[derive(Deserialize, JsonSchema)]
struct EchoRequest {
    message: String,
}

#[derive(Deserialize, JsonSchema)]
struct ReverseRequest {
    text: String,
}

tools! {
    /// Echoes back the input message
    fn echo(req: EchoRequest) -> ToolResponse {
        text!("Echo: {}", req.message)
    }
    
    /// Reverses the input text
    fn reverse(req: ReverseRequest) -> ToolResponse {
        let reversed: String = req.text.chars().rev().collect();
        text!("{}", reversed)
    }
}
```

### Complete Example

Here's a more complete example showing how the macro works with multiple tools:

```rust
use ftl_sdk::{tools, text, error};
use serde::Deserialize;
use schemars::JsonSchema;

#[derive(Deserialize, JsonSchema)]
struct CalculatorRequest {
    a: f64,
    b: f64,
}

#[derive(Deserialize, JsonSchema)]
struct ConvertRequest {
    value: f64,
    from_unit: String,
    to_unit: String,
}

tools! {
    /// Add two numbers
    fn add(req: CalculatorRequest) -> ToolResponse {
        text!("{} + {} = {}", req.a, req.b, req.a + req.b)
    }
    
    /// Subtract two numbers
    fn subtract(req: CalculatorRequest) -> ToolResponse {
        text!("{} - {} = {}", req.a, req.b, req.a - req.b)
    }
    
    /// Divide two numbers
    fn divide(req: CalculatorRequest) -> ToolResponse {
        if req.b == 0.0 {
            return error!("Cannot divide by zero");
        }
        text!("{} / {} = {}", req.a, req.b, req.a / req.b)
    }
    
    /// Convert between units
    fn convert(req: ConvertRequest) -> ToolResponse {
        // Conversion logic here
        text!("Converted {} {} to {}", req.value, req.from_unit, req.to_unit)
    }
}

// The macro generates the HTTP handler with routing automatically!
```

## Generated Code

The `tools!` macro generates:
- A `handle_tool_component` async function that returns metadata for all tools on GET
- Path-based routing (e.g., `/add`, `/subtract`) for POST requests
- Automatic JSON deserialization of request bodies
- Error handling with proper HTTP status codes
- Correct Content-Type headers
- Full Spin HTTP component integration

## Important: Input Validation

Just like with the TypeScript SDK, **tools should NOT validate inputs themselves**. The FTL gateway handles all input validation against your tool's JSON Schema before invoking your handler. This means:

- Your handler can assume all inputs match the schema
- Focus on business logic, not validation
- The gateway enforces all JSON Schema constraints

## Best Practices

1. **Use serde for Input Types**: Define input structs with `#[derive(Deserialize)]`

2. **Use Response Macros**: Use `text!()`, `error!()`, and `structured!()` for cleaner code

3. **Keep Metadata in Sync**: Ensure your input schema matches your Rust struct definition

4. **Error Handling**: Return `error!("message")` for business logic errors - the macro handles panics

## Example with Spin

The `tools!` macro automatically generates the Spin HTTP component handler:

```rust
use ftl_sdk::{tools, text};
use serde::Deserialize;
use schemars::JsonSchema;

#[derive(Deserialize, JsonSchema)]
struct MyInput {
    value: String,
}

tools! {
    /// Processes the input value
    fn process(input: MyInput) -> ToolResponse {
        text!("Processed: {}", input.value)
    }
    
    /// Validates the input value
    fn validate(input: MyInput) -> ToolResponse {
        if input.value.is_empty() {
            return error!("Value cannot be empty");
        }
        text!("Valid: {}", input.value)
    }
}

// That's it! The macro generates the HTTP handler for both tools.
// No need to write any additional code.
```

## Async Support

The `tools!` macro automatically detects whether your function is async and generates the appropriate code:

```rust
use ftl_sdk::{tools, text, error};
use serde::Deserialize;
use schemars::JsonSchema;
use spin_sdk::http::{send, Method, Request};

#[derive(Deserialize, JsonSchema)]
struct WeatherInput {
    location: String,
}

#[derive(Deserialize, JsonSchema)]
struct ForecastInput {
    location: String,
    days: u32,
}

tools! {
    /// Get current weather (async)
    async fn get_weather(input: WeatherInput) -> ToolResponse {
        let req = Request::builder()
            .method(Method::Get)
            .uri(format!("https://api.example.com/weather?location={}", input.location))
            .build();
        
        match send(req).await {
            Ok(res) => text!("Weather in {}: sunny", input.location),
            Err(e) => error!("Failed to fetch weather: {}", e)
        }
    }
    
    /// Get weather forecast (async)
    async fn get_forecast(input: ForecastInput) -> ToolResponse {
        // Another async operation
        text!("{}-day forecast for {}", input.days, input.location)
    }
}
```

## Response Macros

The SDK provides convenient macros for creating responses:

```rust
tools! {
    fn demo_responses(input: DemoInput) -> ToolResponse {
        // Simple text response
        text!("Hello, {}!", input.name)
        
        // Error response
        error!("Something went wrong: {}", reason)
        
        // Structured response with data
        let data = json!({ "result": 42, "status": "complete" });
        structured!(data, "Calculation finished")
    }
}
```

## License

Apache-2.0