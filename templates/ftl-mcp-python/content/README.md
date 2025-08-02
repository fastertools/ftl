# {{project-name}}

An FTL MCP tool written in Python.

## Prerequisites

- Python 3.10 or higher
- pip

## Quick Start

1. Set up development environment:
   ```bash
   make install-dev
   ```

2. Run tests:
   ```bash
   make test
   ```

3. Build the WebAssembly module:
   ```bash
   make build
   # or use FTL directly:
   ftl build
   ```

4. Run the MCP server:
   ```bash
   ftl up
   ```

## Development

### Project Structure

```
{{project-name}}/
├── src/
│   ├── __init__.py
│   └── main.py          # Tool implementation
├── tests/
│   ├── __init__.py
│   └── test_main.py     # Unit tests
├── pyproject.toml       # Project configuration
├── Makefile             # Development tasks
└── README.md
```

### Available Commands

```bash
make help         # Show all available commands
make format       # Format code with black
make lint         # Run linting with ruff
make type-check   # Run type checking with mypy
make test         # Run tests
make test-cov     # Run tests with coverage report
make clean        # Clean build artifacts
make build        # Build WebAssembly module
```

### Adding New Tools

Edit `src/main.py` to add new tools:

```python
def my_new_tool(input_data: Dict[str, Any]) -> ToolResponse:
    """Your tool description."""
    # Implement your logic here
    return ToolResponse.text("Result")

# Add to the Handler
Handler = create_tools({
    "myNewTool": {
        "description": "Description of your tool",
        "inputSchema": {
            "type": "object",
            "properties": {
                "param": {"type": "string", "description": "Parameter description"}
            },
            "required": ["param"]
        },
        "handler": my_new_tool
    }
})
```

### Testing

Write tests in `tests/test_main.py`:

```python
def test_my_new_tool():
    result = my_new_tool({"param": "value"})
    assert result.content == "Expected result"
```

### Code Quality

This project uses:
- **Black** for code formatting
- **Ruff** for fast linting
- **MyPy** for type checking
- **Pytest** for testing

Run all checks:
```bash
make format lint type-check test
```

## Deployment

After building with `make build` or `ftl build`, deploy to FTL Engine:

```bash
ftl eng deploy
```