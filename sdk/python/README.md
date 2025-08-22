# ftl-sdk (Python)

Python SDK for building Model Context Protocol (MCP) tools that compile to WebAssembly.

[![PyPI Version](https://img.shields.io/pypi/v/ftl-sdk.svg)](https://pypi.org/project/ftl-sdk/)
[![Python Versions](https://img.shields.io/pypi/pyversions/ftl-sdk.svg)](https://pypi.org/project/ftl-sdk/)
[![License](https://img.shields.io/github/license/fastertools/ftl.svg)](https://github.com/fastertools/ftl/blob/main/LICENSE)
[![GitHub Actions](https://github.com/fastertools/ftl/workflows/Test%20Python%20SDK/badge.svg)](https://github.com/fastertools/ftl/actions)

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
pip install git+https://github.com/fastertools/ftl.git#subdirectory=sdk/python
```

### From TestPyPI (Pre-releases)

```bash
pip install --index-url https://test.pypi.org/simple/ --extra-index-url https://pypi.org/simple/ ftl-sdk
```

## Overview

This SDK provides:
- Decorator-based API for easy tool creation
- Automatic JSON Schema generation from type hints
- Support for both sync and async functions
- Automatic return value conversion to MCP format
- Zero-dependency implementation (only requires `spin-sdk`)
- Full compatibility with Spin WebAssembly components
- Seamless deployment to Fermyon Cloud

## Requirements

- Python 3.10 or later
- `componentize-py` for building WebAssembly components
- `spin-sdk` for Spin runtime integration

## Quick Start

### 1. Create a new Python tool

```python
from ftl_sdk import FTL

# Create FTL application instance
ftl = FTL()

@ftl.tool
def echo(message: str) -> str:
    """Echo back the input."""
    return f"Echo: {message}"

# Create the Spin handler
Handler = ftl.create_handler()
```

### 3. Build and Deploy

```bash
# Create virtual environment
python3 -m venv venv
source venv/bin/activate

# Install dependencies
pip install componentize-py spin-sdk ftl-sdk

# Build
ftl build

# Deploy to Fermyon Cloud
ftl deploy
```

## API Reference

### Decorator-based API

#### `FTL` Class

```python
ftl = FTL()
```

Creates an FTL application instance for registering tools.

#### `@ftl.tool` Decorator

```python
@ftl.tool
def my_tool(param: str) -> str:
    """Tool description."""
    return result

# With custom name and annotations
@ftl.tool(name="custom_name", annotations={"priority": "high"})
def another_tool(data: dict) -> dict:
    return {"processed": data}
```

The decorator:
- Automatically generates JSON Schema from type hints
- Extracts docstring as tool description
- Handles both sync and async functions
- Validates output against return type annotation

#### `ftl.create_handler()`

Creates a Spin HTTP handler that:
- Returns tool metadata on GET / requests
- Routes to specific tools on POST /{tool_name} requests
- Handles async/await for async tool functions
- Automatically converts return values to MCP format

### `ToolResponse` Helper Methods

```python
# Simple text response
ToolResponse.text("Hello, world!")

# Error response
ToolResponse.error("Something went wrong")

# Response with structured content
ToolResponse.with_structured("Operation complete", {"result": 42})
```

### `ToolResult` Class

For advanced response handling, you can use the `ToolResult` class:

```python
from ftl_sdk import ToolResult, ToolContent

# Create custom response with multiple content items
result = ToolResult(
    content=[
        ToolContent.text("Processing complete"),
        ToolContent.text("Results:", {"priority": 0.8})
    ],
    structured_content={"status": "success", "count": 5}
)

# Error result
error_result = ToolResult.error("Operation failed: invalid input")

# Text result (shorthand)
text_result = ToolResult.text("Simple text response")
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

### Basic Tools

```python
from ftl_sdk import FTL

ftl = FTL()

@ftl.tool
def echo(message: str) -> str:
    """Echo the input."""
    return f"Echo: {message}"

@ftl.tool
def reverse_text(text: str) -> str:
    """Reverse the input text."""
    return text[::-1]

@ftl.tool
def word_count(text: str) -> dict:
    """Count words in text."""
    count = len(text.split())
    return {"text": text, "word_count": count}

Handler = ftl.create_handler()
```

### Async Tools

```python
import asyncio
from ftl_sdk import FTL

ftl = FTL()

@ftl.tool
async def fetch_data(url: str) -> dict:
    """Fetch data from URL asynchronously."""
    # Simulate async HTTP request
    await asyncio.sleep(0.1)
    return {
        "url": url,
        "status": "success",
        "data": {"example": "data"}
    }

@ftl.tool
async def process_items(items: list[str]) -> dict:
    """Process items with async operations."""
    results = []
    for item in items:
        # Simulate async processing
        await asyncio.sleep(0.01)
        results.append(item.upper())
    
    return {
        "original": items,
        "processed": results,
        "count": len(results)
    }

# Mix sync and async tools
@ftl.tool
def sync_add(a: int, b: int) -> int:
    """Add two numbers synchronously."""
    return a + b

Handler = ftl.create_handler()
```

### Error Handling

```python
from ftl_sdk import FTL

ftl = FTL()

@ftl.tool
def divide(a: float, b: float) -> float:
    """Divide two numbers."""
    if b == 0:
        raise ValueError("Cannot divide by zero")
    return a / b

# Async error handling
@ftl.tool
async def validate_data(data: dict) -> dict:
    """Validate data asynchronously."""
    if "required_field" not in data:
        raise KeyError("Missing required field: required_field")
    
    # Simulate async validation
    await asyncio.sleep(0.05)
    
    if not isinstance(data["required_field"], str):
        raise TypeError("required_field must be a string")
    
    return {"status": "valid", "data": data}

Handler = ftl.create_handler()
```

The FTL framework automatically catches exceptions and returns them as error responses.


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

3. **Or use FTL CLIs build command**:
   ```bash
   ftl build
   ```

## Best Practices

### Type Hints

Always use type hints for better code clarity and automatic schema generation:

```python
from ftl_sdk import FTL

ftl = FTL()

@ftl.tool
def my_handler(message: str) -> str:
    """Handle a message with automatic type validation."""
    return f"Received: {message}"
```

### Error Handling

The FTL framework automatically catches exceptions and converts them to error responses. Use type hints for automatic validation:

```python
from ftl_sdk import FTL

ftl = FTL()

@ftl.tool
def safe_handler(required_field: str, optional_value: int = 10) -> dict:
    """Process data with automatic validation and error handling."""
    # Type validation is automatic - required_field is guaranteed to be a string
    # Framework automatically catches and converts exceptions to error responses
    
    if not required_field.strip():
        raise ValueError("required_field cannot be empty")
    
    # Process the validated input
    result = len(required_field) * optional_value
    
    return {
        "processed": required_field.upper(),
        "result": result,
        "status": "success"
    }
```

### Testing Your Tools

Write comprehensive tests for your decorated tools:

```python
import pytest
from ftl_sdk import FTL

# Your tool module
ftl = FTL()

@ftl.tool
def echo_message(message: str) -> str:
    """Echo back the input message."""
    if not message:
        raise ValueError("Message cannot be empty")
    return f"Echo: {message}"

# Test the tool function directly
def test_echo_success():
    result = echo_message("test")
    assert result == "Echo: test"

def test_echo_validation():
    with pytest.raises(ValueError, match="Message cannot be empty"):
        echo_message("")

# Test the HTTP handler integration
def test_handler_integration():
    handler = ftl.create_handler()
    # Test metadata endpoint
    metadata = handler.get_tools_metadata()
    assert len(metadata) == 1
    assert metadata[0]["name"] == "echo_message"
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

Deploy to FTL:

```bash
# Deploy with auto-generated name
ftl deploy
```

## Development

Contributions are welcome! Please feel free to submit a Pull Request.

### Setting Up Development Environment

```bash
# Clone the repository
git clone https://github.com/fastertools/ftl.git
cd ftl/sdk/python

# Create virtual environment
python -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate

# Install development dependencies
make install-dev
# or manually:
pip install -e ".[dev]"
pip install componentize-py tox
```

### Running Tests

```bash
# Run all tests
make test

# Run tests with coverage
make test-cov

# Run tests for all Python versions
tox

# Run tests for specific Python version
tox -e py311

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

# Or run individual tools:
black src tests
ruff check src tests
mypy src
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
make publish      # Publish to PyPI
```

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for a list of changes in each release.

## License

Apache-2.0 - see [LICENSE](../../LICENSE) for details.