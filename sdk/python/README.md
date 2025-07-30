# ftl-sdk (Python)

Python SDK for building Model Context Protocol (MCP) tools that compile to WebAssembly.

## Installation

```bash
pip install ftl-sdk
```

## Overview

This SDK provides:
- Zero-dependency implementation (only requires `spin-sdk`)
- Python type hints for the MCP protocol
- `create_tools` helper for building multiple tools per component
- Full compatibility with Spin WebAssembly components
- Seamless deployment to Fermyon Cloud

## Requirements

- Python 3.10 or later
- `componentize-py` for building WebAssembly components
- `spin-sdk` for Spin runtime integration
- Spin CLI for deployment

## Quick Start

### 1. Create a new Python tool

```python
from ftl_sdk import create_tools, ToolResponse

# Define your tool handler
Handler = create_tools({
    "echo": {
        "description": "Echo back the input",
        "inputSchema": {
            "type": "object",
            "properties": {
                "message": {"type": "string", "description": "The message to echo"}
            },
            "required": ["message"]
        },
        "handler": lambda input: ToolResponse.text(f"Echo: {input['message']}")
    }
})
```

### 2. Configure Spin

Create a `spin.toml` file:

```toml
spin_manifest_version = 2

[application]
name = "my-python-tool"
version = "0.1.0"
authors = ["Your Name <you@example.com>"]

[[trigger.http]]
route = "/..."
component = "my-tool"

[component.my-tool]
source = "app.wasm"
[component.my-tool.build]
command = "componentize-py -w spin-http componentize app -o app.wasm"
```

### 3. Build and Deploy

```bash
# Create virtual environment
python3 -m venv venv
source venv/bin/activate

# Install dependencies
pip install componentize-py spin-sdk ftl-sdk

# Build
spin build

# Deploy to Fermyon Cloud
spin aka deploy
```

## API Reference

### `create_tools(tools)`

Creates a Spin HTTP handler for multiple MCP tools.

```python
Handler = create_tools({
    "tool_name": {
        "description": "Tool description",
        "inputSchema": {...},  # JSON Schema
        "handler": tool_function,
        # Optional:
        "name": "override_name",  # Override tool name
        "outputSchema": {...},    # Output JSON Schema
        "annotations": {...},     # Tool behavior hints
        "_meta": {...}           # Tool metadata
    }
})
```

The returned Handler is a Spin `IncomingHandler` class that:
- Returns tool metadata on GET / requests
- Routes to specific tools on POST /{tool_name} requests
- Automatically converts camelCase to snake_case for tool names
- Handles errors gracefully

### `ToolResponse` Helper Methods

```python
# Simple text response
ToolResponse.text("Hello, world!")

# Error response
ToolResponse.error("Something went wrong")

# Response with structured content
ToolResponse.with_structured("Operation complete", {"result": 42})
```

### `ToolContent` Helper Methods

```python
# Text content
ToolContent.text("Some text", {"priority": 0.8})

# Image content
ToolContent.image(base64_data, "image/png")

# Audio content
ToolContent.audio(base64_data, "audio/wav")

# Resource reference
ToolContent.resource({"uri": "file:///example.txt"})
```

### Type Guards

```python
# Check content types
if is_text_content(content):
    print(content["text"])
```

## Examples

### Multi-Tool Component

```python
from ftl_sdk import create_tools, ToolResponse

Handler = create_tools({
    "echo": {
        "description": "Echo the input",
        "inputSchema": {
            "type": "object",
            "properties": {"message": {"type": "string"}},
            "required": ["message"]
        },
        "handler": lambda input: ToolResponse.text(f"Echo: {input['message']}")
    },
    
    "reverseText": {
        "name": "reverse",  # Override to keep it as "reverse"
        "description": "Reverse the input text",
        "inputSchema": {
            "type": "object",
            "properties": {"text": {"type": "string"}},
            "required": ["text"]
        },
        "handler": lambda input: ToolResponse.text(input["text"][::-1])
    },
    
    "wordCount": {
        "description": "Count words in text",
        "inputSchema": {
            "type": "object",
            "properties": {"text": {"type": "string"}},
            "required": ["text"]
        },
        "handler": lambda input: ToolResponse.with_structured(
            f"Word count: {len(input['text'].split())}",
            {"count": len(input["text"].split())}
        )
    }
})
```

### Error Handling

```python
def safe_divide(input):
    try:
        a = input["a"]
        b = input["b"]
        if b == 0:
            return ToolResponse.error("Cannot divide by zero")
        return ToolResponse.text(f"Result: {a / b}")
    except KeyError as e:
        return ToolResponse.error(f"Missing required field: {e}")
    except Exception as e:
        return ToolResponse.error(f"Unexpected error: {e}")

Handler = create_tools({
    "divide": {
        "description": "Divide two numbers",
        "inputSchema": {
            "type": "object",
            "properties": {
                "a": {"type": "number"},
                "b": {"type": "number"}
            },
            "required": ["a", "b"]
        },
        "handler": safe_divide
    }
})
```

## Building to WebAssembly

Tools must be compiled to WebAssembly to run on Spin:

1. **Install dependencies**:
   ```bash
   pip install componentize-py spin-sdk ftl-sdk
   ```

2. **Build with componentize-py**:
   ```bash
   componentize-py -w spin-http componentize app -o app.wasm
   ```

3. **Or use Spin's build command**:
   ```bash
   spin build
   ```

## Important Notes

1. **Python Version**: Requires Python 3.10 or later. Python 3.11+ recommended.

2. **Zero Dependencies**: This SDK has no external dependencies beyond `spin-sdk`, keeping the WASM bundle size minimal.

3. **Input Validation**: The FTL gateway handles input validation against your JSON Schema. Your handler can assume inputs are valid.

4. **Virtual Environments**: Always use a virtual environment to ensure consistent builds.

5. **WASM Size**: Python WASM components are larger than TypeScript/Rust equivalents (~37MB), but this is acceptable for cloud deployment.

## Deployment

Deploy to Fermyon Cloud:

```bash
# Deploy with auto-generated name
spin aka deploy --create-name my-app-name --no-confirm

# Check deployment status
spin aka app status

# View logs
spin aka logs
```

## License

Apache-2.0