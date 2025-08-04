# {{project-name}}

An FTL MCP tool written in Python.

## Prerequisites

- Python 3.10 or higher
- pip

## Quick Start

1. **First-time setup** (optional - the build process handles this automatically):
   ```bash
   # The Makefile will automatically:
   # - Create a Python virtual environment
   # - Install componentize-py and dependencies
   # - Build the WebAssembly module
   
   # Or manually set up development environment:
   make install-dev
   ```

2. Build the WebAssembly module:
   ```bash
   ftl build
   # This runs `make build` which handles all Python dependencies
   
   # Or use make directly:
   make build
   ```

3. Run the MCP server:
   ```bash
   ftl up
   ```

### Windows Users

On Windows, you'll need to set up the environment manually:

```powershell
# Create virtual environment
python -m venv venv

# Activate it
.\venv\Scripts\Activate

# Install dependencies
pip install -e .
pip install componentize-py

# Build
componentize-py -w spin-http componentize src/main.py -o app.wasm
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
├── Makefile             # Development tasks and build automation
└── README.md
```

### Python Environment

The Makefile automatically manages the Python virtual environment during build:
- Creates a `venv/` directory if it doesn't exist
- Installs all required dependencies including `componentize-py`
- Builds the WebAssembly module

You can also manage the environment manually using the Makefile commands.

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

Edit `src/main.py` to add new tools using the decorator-based API:

```python
@ftl.tool
def my_new_tool(param: str, count: int = 1) -> dict:
    """
    Description of your tool.
    
    The SDK automatically generates JSON Schema from type hints.
    """
    # Implement your logic here
    return {
        "result": param * count,
        "count": count
    }

# For async operations (note: asyncio.sleep() not supported in WASM)
@ftl.tool
async def async_tool(items: list[str]) -> dict:
    """Async tool for concurrent processing."""
    import asyncio
    
    async def process(item: str) -> str:
        return f"Processed: {item}"
    
    # Create and await concurrent tasks
    tasks = [asyncio.create_task(process(item)) for item in items]
    results = await asyncio.gather(*tasks)
    
    return {"results": results}
```

### Testing

Write tests in `tests/test_main.py`:

```python
def test_my_new_tool():
    result = my_new_tool("hello", 3)
    assert result == {"result": "hellohellohello", "count": 3}

# For async tools
@pytest.mark.asyncio
async def test_async_tool():
    result = await async_tool(["item1", "item2"])
    assert result == {"results": ["Processed: item1", "Processed: item2"]}
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