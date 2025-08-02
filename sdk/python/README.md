# ftl-sdk (Python)

Python SDK for building Model Context Protocol (MCP) tools that compile to WebAssembly.

[![PyPI Version](https://img.shields.io/pypi/v/ftl-sdk.svg)](https://pypi.org/project/ftl-sdk/)
[![Python Versions](https://img.shields.io/pypi/pyversions/ftl-sdk.svg)](https://pypi.org/project/ftl-sdk/)
[![License](https://img.shields.io/pypi/l/ftl-sdk.svg)](https://github.com/fastertools/ftl-cli/blob/main/LICENSE)
[![GitHub Actions](https://github.com/fastertools/ftl-cli/workflows/Test%20Python%20SDK/badge.svg)](https://github.com/fastertools/ftl-cli/actions)

## Installation

### Latest Stable Version

```bash
pip install ftl-sdk
```

### Specific Version

```bash
pip install ftl-sdk==0.1.0
```

### Development Version

```bash
pip install git+https://github.com/fastertools/ftl-cli.git#subdirectory=sdk/python
```

### From TestPyPI (Pre-releases)

```bash
pip install --index-url https://test.pypi.org/simple/ --extra-index-url https://pypi.org/simple/ ftl-sdk
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

## Development

### Setting Up Development Environment

1. **Clone the repository**:
   ```bash
   git clone https://github.com/fastertools/ftl-cli.git
   cd ftl-cli/sdk/python
   ```

2. **Create virtual environment**:
   ```bash
   python -m venv venv
   source venv/bin/activate  # On Windows: venv\Scripts\activate
   ```

3. **Install development dependencies**:
   ```bash
   make install-dev
   # or manually:
   pip install -e ".[dev]"
   pip install componentize-py
   ```

### Running Tests

```bash
# Run all tests
make test

# Run tests with coverage
make test-cov

# Run specific test
pytest tests/test_ftl_sdk.py::test_tool_response_text
```

### Code Quality

```bash
# Format code
make format

# Run linting
make lint

# Type checking
make type-check

# Run all quality checks
make quality
```

### Available Make Commands

```bash
make help         # Show all available commands
make install      # Install SDK
make install-dev  # Install with dev dependencies
make format       # Format code with black
make lint         # Run linting with ruff
make type-check   # Run type checking with mypy
make test         # Run tests
make test-cov     # Run tests with coverage
make clean        # Clean build artifacts
make build        # Build distribution packages
make publish      # Publish to PyPI
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

## Best Practices

### Type Hints

Always use type hints for better code clarity and IDE support:

```python
from typing import Dict, Any
from ftl_sdk import ToolResponse

def my_handler(input_data: Dict[str, Any]) -> Dict[str, Any]:
    message: str = input_data.get("message", "")
    return ToolResponse.text(f"Received: {message}")
```

### Error Handling

Handle errors gracefully and return informative error messages:

```python
def safe_handler(input_data: Dict[str, Any]) -> Dict[str, Any]:
    try:
        # Validate required fields
        if "required_field" not in input_data:
            return ToolResponse.error("Missing required field: required_field")
        
        # Process input
        result = process_data(input_data["required_field"])
        return ToolResponse.text(f"Success: {result}")
        
    except ValueError as e:
        return ToolResponse.error(f"Invalid value: {e}")
    except Exception as e:
        return ToolResponse.error(f"Unexpected error: {e}")
```

### Testing Your Tools

Write comprehensive tests for your tools:

```python
import pytest
from your_module import your_handler

def test_handler_success():
    result = your_handler({"message": "test"})
    assert result["content"][0]["text"] == "Expected output"

def test_handler_missing_field():
    result = your_handler({})
    assert result.get("isError") is True
    assert "Missing required field" in result["content"][0]["text"]
```

## Important Notes

1. **Python Version**: Requires Python 3.10 or later. Python 3.11+ recommended.

2. **Zero Dependencies**: This SDK has no external dependencies beyond `spin-sdk`, keeping the WASM bundle size minimal.

3. **Input Validation**: The FTL gateway handles input validation against your JSON Schema. Your handler can assume inputs are valid.

4. **Virtual Environments**: Always use a virtual environment to ensure consistent builds.

5. **WASM Size**: Python WASM components are larger than TypeScript/Rust equivalents (~37MB), but this is acceptable for cloud deployment.

6. **Type Safety**: Use type hints and mypy for better code quality and fewer runtime errors.

7. **Code Quality**: The SDK includes development tools (black, ruff, mypy, pytest) to maintain high code quality standards.

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

## Development

Contributions are welcome! Please feel free to submit a Pull Request.

### Setting Up Development Environment

```bash
# Clone the repository
git clone https://github.com/fastertools/ftl-cli.git
cd ftl-cli/sdk/python

# Create virtual environment
python -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate

# Install development dependencies
pip install -e ".[dev]"
pip install tox
```

### Running Tests

```bash
# Run tests for all Python versions
tox

# Run tests for specific Python version
tox -e py311

# Run linting and type checking
tox -e lint,type

# Run tests with coverage
tox -e py311 -- --cov-report=html
```

### Code Quality

```bash
# Format code
black src tests

# Run linter
ruff check src tests

# Type checking
mypy src
```

## Version Support

| SDK Version | Python Version | Status |
|-------------|----------------|--------|
| 0.1.x       | 3.10+          | Active |

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for a list of changes in each release.

## License

Apache-2.0 - see [LICENSE](../../LICENSE) for details.