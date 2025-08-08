# Rust SDK

The FTL Rust SDK is the primary, most feature-complete SDK for building WebAssembly tools. It provides a powerful macro system, automatic schema generation, and seamless async support.

## Quick Reference

- **Crate**: `ftl-sdk` (v0.2.10)
- **Rust Version**: 1.70+
- **Target**: `wasm32-wasip1`
- **Features**: Macros, async/await, schema generation
- **Status**: ✅ Stable, primary SDK

## Overview

```rust
use ftl_sdk::{tools, text, error, ToolResponse};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(Deserialize, JsonSchema)]
struct MyInput {
    name: String,
    count: Option<u32>,
}

tools! {
    /// A simple greeting tool
    fn greet(input: MyInput) -> ToolResponse {
        let count = input.count.unwrap_or(1);
        text!("Hello, {}! (x{})", input.name, count)
    }
    
    /// An async tool that makes HTTP requests
    async fn fetch_data(url: String) -> ToolResponse {
        match spin_sdk::http::send(
            spin_sdk::http::Request::builder()
                .method("GET")
                .uri(&url)
                .build()
        ).await {
            Ok(response) => {
                let body = std::str::from_utf8(response.body()).unwrap_or("Invalid UTF-8");
                text!("Fetched: {}", body)
            }
            Err(e) => error!("Request failed: {}", e)
        }
    }
}
```

## Documentation Sections

### [Core API](./api.md)
Complete API reference including:
- **`tools!` macro** - Multi-tool definition and code generation
- **`ToolResponse`** - Response types and constructors  
- **Response macros** - `text!()`, `error!()`, `structured!()` helpers
- **Type requirements** - Constraints for input/output types

### [Types and Schema](./types.md)  
Type system and schema generation:
- **Input types** - Requirements and patterns
- **JSON Schema** - Automatic generation from Rust types
- **Serialization** - Serde integration and custom serializers
- **Validation** - How input validation works

### [Async Programming](./async.md)
Asynchronous tool development:
- **Async tools** - Writing async functions with proper error handling
- **HTTP requests** - Using `spin_sdk::http::send()`
- **Runtime constraints** - WASM async limitations
- **Best practices** - Patterns for async tool development

### [Error Handling](./errors.md)
Comprehensive error handling:
- **Error responses** - Creating structured error responses
- **HTTP errors** - Handling external service failures  
- **Validation** - Input validation and schema compliance
- **Debugging** - Logging and troubleshooting techniques

### [Examples](./examples.md)
Complete working examples:
- **Basic tools** - Simple input/output patterns
- **HTTP integration** - External API calls and error handling
- **Multi-tool components** - Multiple tools in one WebAssembly module
- **Complex types** - Advanced data structures and schemas

### [Migration Guide](./migration.md)
Upgrading between SDK versions:
- **Version compatibility** - Which CLI versions work with which SDK versions
- **Breaking changes** - API changes between major versions
- **Upgrade paths** - Step-by-step migration instructions
- **Deprecated features** - Features being phased out

## Getting Started

### 1. Add Dependencies

```toml
# Cargo.toml
[package]
name = "my-tool"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
ftl-sdk = { version = "0.2.10", features = ["macros"] }
serde = { version = "1.0", features = ["derive"] }
schemars = "1.0.4"
spin-sdk = "4.0.0"

# Optional: for HTTP requests
tokio = { version = "1.0", features = ["rt"] }

# Optional: for additional serde formats
serde_json = "1.0"

[lints.rust]
unsafe_code = "forbid"

[lints.clippy]
panic = "forbid"
unwrap_used = "forbid"
expect_used = "forbid"
```

### 2. Define Your Tools

```rust
// src/lib.rs
use ftl_sdk::{tools, text, error, ToolResponse};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(Deserialize, JsonSchema)]
struct CalculatorInput {
    /// First number
    a: f64,
    /// Second number  
    b: f64,
    /// Operation to perform
    operation: String,
}

tools! {
    /// A simple calculator tool
    fn calculate(input: CalculatorInput) -> ToolResponse {
        match input.operation.as_str() {
            "add" => text!("{}", input.a + input.b),
            "subtract" => text!("{}", input.a - input.b),
            "multiply" => text!("{}", input.a * input.b),
            "divide" => {
                if input.b == 0.0 {
                    error!("Cannot divide by zero")
                } else {
                    text!("{}", input.a / input.b)
                }
            }
            _ => error!("Unknown operation: {}", input.operation)
        }
    }
}
```

### 3. Build for WebAssembly

```bash
# Add WASM target (one-time setup)
rustup target add wasm32-wasip1

# Build your tool
cargo build --target wasm32-wasip1 --release

# The output will be at:
# target/wasm32-wasip1/release/my_tool.wasm
```

### 4. Integrate with FTL

Create or update your `ftl.toml`:

```toml
[tools.my-tool]
path = "."
wasm = "target/wasm32-wasip1/release/my_tool.wasm"

[tools.my-tool.build]
command = "cargo build --target wasm32-wasip1 --release"
watch = ["src/**/*.rs", "Cargo.toml"]
```

Then build and run:

```bash
ftl build
ftl up
```

## Key Features

### 🚀 **Zero-Config Schema Generation**
Automatically generates MCP-compliant JSON schemas from your Rust types:

```rust
#[derive(Deserialize, JsonSchema)]
struct UserInput {
    /// User's email address
    email: String,
    /// Optional display name  
    name: Option<String>,
    /// Age in years
    age: u32,
}

// Automatically generates:
// {
//   "type": "object",
//   "properties": {
//     "email": { "type": "string", "description": "User's email address" },
//     "name": { "type": "string", "description": "Optional display name" },
//     "age": { "type": "integer", "description": "Age in years" }
//   },
//   "required": ["email", "age"]
// }
```

### ⚡ **Native Async Support**
The macro automatically detects and handles async functions:

```rust
tools! {
    /// Fetch user profile from external API
    async fn get_user_profile(user_id: String) -> ToolResponse {
        let url = format!("https://api.example.com/users/{}", user_id);
        
        match spin_sdk::http::send(
            spin_sdk::http::Request::builder()
                .method("GET")
                .uri(&url)
                .header("User-Agent", "FTL-Tool/1.0")
                .build()
        ).await {
            Ok(response) => {
                if response.status() == 200 {
                    text!("User data: {}", std::str::from_utf8(response.body()).unwrap_or(""))
                } else {
                    error!("API returned status: {}", response.status())
                }
            }
            Err(e) => error!("Request failed: {}", e)
        }
    }
}
```

### 🛡️ **Memory Safety & Security**
Built with Rust's ownership system and strict linting:

- **No `unsafe` code**: `unsafe_code = "forbid"`
- **No panics**: Clippy rules forbid `panic!()`, `unwrap()`, `expect()`  
- **Memory isolation**: Each tool runs in a WebAssembly sandbox
- **Type safety**: Compile-time validation of all data flows

### 🔧 **Powerful Macros**
Convenient macros for common response patterns:

```rust
// Text responses with formatting
text!("Hello, {}!", name)

// Error responses  
error!("Invalid input: {}", validation_error)

// Structured responses with both text and data
structured!(
    user_data,  // structured data
    "Found user: {}", user.name  // human-readable text
)
```

### 📦 **Multi-Tool Components**
Define multiple related tools in a single WebAssembly module:

```rust
tools! {
    /// Convert text to uppercase
    fn to_upper(input: TextInput) -> ToolResponse {
        text!("{}", input.text.to_uppercase())
    }
    
    /// Convert text to lowercase  
    fn to_lower(input: TextInput) -> ToolResponse {
        text!("{}", input.text.to_lowercase())
    }
    
    /// Count words in text
    fn word_count(input: TextInput) -> ToolResponse {
        let count = input.text.split_whitespace().count();
        text!("Word count: {}", count)
    }
}
```

## Performance Characteristics

- **Binary size**: ~100KB - 2MB depending on dependencies
- **Startup time**: 1-5ms (near-instantaneous)
- **Memory usage**: Minimal, precise garbage collection
- **Compilation time**: Fast incremental builds with cargo
- **Runtime performance**: Near-native speed via WebAssembly

## WebAssembly Constraints

When developing with the Rust SDK, be aware of WebAssembly limitations:

### ❌ **Not Available**
- `tokio` runtime (use Spin's async support)
- `std::thread` (single-threaded execution)
- File system access (unless explicitly granted)
- Network access (unless hosts are whitelisted)
- External processes or system calls

### ✅ **Available**
- Native `async/await` syntax
- `spin_sdk` for HTTP, variables, key-value storage
- Pure Rust libraries (most crates work)
- JSON, XML, and other data format libraries
- Cryptographic libraries
- Mathematical and scientific computing

## Best Practices

### 🎯 **Input Validation**
Let the gateway handle validation - define clear schemas:

```rust
#[derive(Deserialize, JsonSchema)]
struct EmailInput {
    /// Must be a valid email address
    #[schemars(regex = "^[^@]+@[^@]+\\.[^@]+$")]
    email: String,
    
    /// Optional subject line
    #[schemars(length(max = 100))]
    subject: Option<String>,
}
```

### 🔄 **Error Handling**
Use structured error responses:

```rust
fn validate_email(email: &str) -> Result<(), String> {
    if !email.contains('@') {
        return Err("Email must contain @ symbol".to_string());
    }
    if !email.contains('.') {
        return Err("Email must contain domain".to_string());
    }
    Ok(())
}

tools! {
    fn send_email(input: EmailInput) -> ToolResponse {
        if let Err(e) = validate_email(&input.email) {
            return error!("Invalid email: {}", e);
        }
        
        // Send email logic...
        text!("Email sent to {}", input.email)
    }
}
```

### 🚀 **Async Patterns**
Use proper error handling with async operations:

```rust
tools! {
    async fn fetch_with_retry(url: String) -> ToolResponse {
        for attempt in 1..=3 {
            match spin_sdk::http::send(
                spin_sdk::http::Request::builder()
                    .method("GET")
                    .uri(&url)
                    .build()
            ).await {
                Ok(response) if response.status() == 200 => {
                    return text!("Success on attempt {}: {}", attempt, 
                               std::str::from_utf8(response.body()).unwrap_or(""));
                }
                Ok(response) => {
                    if attempt == 3 {
                        return error!("Failed after 3 attempts. Last status: {}", response.status());
                    }
                    // Continue to next attempt
                }
                Err(e) => {
                    if attempt == 3 {
                        return error!("Request failed after 3 attempts: {}", e);
                    }
                    // Continue to next attempt
                }
            }
            
            // Simple backoff - in real code, use exponential backoff
            spin_sdk::variables::get("RETRY_DELAY_MS")
                .unwrap_or("1000".to_string())
                .parse::<u64>()
                .unwrap_or(1000);
        }
        
        error!("Retry logic failed unexpectedly")
    }
}
```

## Next Steps

- **[Core API Reference](./api.md)** - Complete function and macro documentation
- **[Examples](./examples.md)** - Working code for common patterns  
- **[Async Guide](./async.md)** - Master asynchronous tool development
- **[Migration Guide](./migration.md)** - Upgrade between SDK versions

The Rust SDK provides the most powerful and performant way to build FTL tools, with compile-time safety, zero-cost abstractions, and seamless WebAssembly integration.