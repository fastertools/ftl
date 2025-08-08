# {{project-name}}

An FTL MCP tool written in Rust.

## Prerequisites

- Rust 1.86 or higher
- WebAssembly target: `rustup target add wasm32-wasip1`

## Quick Start

1. **Build the WebAssembly module:**
   ```bash
   ftl build
   # This runs `make build` which compiles to wasm32-wasip1 target
   
   # Or use make directly:
   make build
   ```

2. **Run the MCP server:**
   ```bash
   ftl up
   ```

## Development

### Project Structure

```
{{project-name}}/
├── src/
│   └── lib.rs           # Tool implementation
├── Cargo.toml           # Project configuration and dependencies
├── Makefile             # Development tasks and build automation
└── README.md
```

### Available Commands

```bash
make build       # Build WebAssembly module (wasm32-wasip1 target)
make clean       # Clean build artifacts
make test        # Run tests
make check       # Check code without building
make format      # Format code with rustfmt
make lint        # Run Clippy linter with strict warnings
make dev         # Run format, lint, and test (full development check)
```

### Adding New Tools

Edit `src/lib.rs` to add new tools using the `tools!` macro:

```rust
use ftl_sdk::{tools, text, json, ToolResponse};
use serde::Deserialize;
use schemars::JsonSchema;

#[derive(Deserialize, JsonSchema)]
struct CalculatorInput {
    /// First number
    a: f64,
    /// Second number
    b: f64,
}

tools! {
    /// Adds two numbers together
    fn add(input: CalculatorInput) -> ToolResponse {
        let result = input.a + input.b;
        json!({ "result": result })
    }
    
    /// Multiplies two numbers
    fn multiply(input: CalculatorInput) -> ToolResponse {
        let result = input.a * input.b;
        text!("Result: {}", result)
    }
}
```

**Key Points:**
- Use `#[derive(Deserialize, JsonSchema)]` on input structs
- Document functions with `///` - these become tool descriptions
- Use `json!()` for structured responses or `text!()` for simple text
- The SDK automatically generates MCP tool schemas from your Rust types

### Testing

Write tests in `src/lib.rs` or separate test files:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        let input = CalculatorInput { a: 2.0, b: 3.0 };
        let response = add(input);
        // Test your tool logic here
        assert!(response.is_success());
    }
}
```

Run tests with:
```bash
make test
# Or directly: cargo test
```

### Code Quality

This project uses:
- **rustfmt** for code formatting
- **Clippy** for linting with strict warnings (`-D warnings`)
- **Cargo check** for fast compilation checking

Run all quality checks:
```bash
make dev
# This runs: format, lint, and test
```

### WASM Constraints

Since this compiles to WebAssembly:
- ❌ No `tokio` or async runtimes (use `spin-sdk` for async)
- ❌ No `std::thread` or system threads
- ❌ No file system access beyond what Spin provides
- ✅ Use `spin-sdk` for HTTP, async, and other runtime features
- ✅ Pure Rust libraries work well
- ✅ `serde` and JSON processing work normally

## Running Your Tool

After building, start the local development server:

```bash
ftl up
```

Your MCP server will be available at `http://localhost:3000/` and can be used with any MCP-compatible client.

## Troubleshooting

**Build fails with "target not found":**
```bash
rustup target add wasm32-wasip1
```

**Clippy warnings as errors:**
Fix all warnings or temporarily allow them:
```rust
#[allow(clippy::some_lint)]
```

**Async code not working:**
Use `spin-sdk` async features instead of tokio:
```rust
use spin_sdk::http::{Request, Response};
// Don't use tokio::time::sleep - not available in WASM
```