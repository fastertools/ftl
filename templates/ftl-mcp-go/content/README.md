# {{project-name}}

An FTL MCP tool written in Go.

## Prerequisites

- Go 1.23 or higher
- TinyGo 0.30.0+ (for WebAssembly compilation)
- golangci-lint (for development)

## Quick Start

1. Set up development environment:
   ```bash
   make dev-setup
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
├── main.go              # Tool implementation
├── main_test.go         # Unit tests
├── go.mod               # Go module definition
├── go.sum               # Dependency lock file
├── Makefile             # Development tasks
├── .gitignore           # Git ignore rules
├── .golangci.yml        # Linter configuration
└── README.md
```

### Available Commands

```bash
make build        # Build WebAssembly module
make clean        # Clean build artifacts
make test         # Run tests
make test-cov     # Run tests with coverage report
make fmt          # Format code with gofmt
make lint         # Run linting with golangci-lint
make dev          # Run format, lint, and test (full development check)
make help         # Show all available commands (optional setup guidance)
```

### Adding New Tools

Edit `main.go` to add new tools:

```go
func myNewToolHandler(input map[string]interface{}) ftl.ToolResponse {
    // Validate input
    param, ok := input["param"].(string)
    if !ok {
        return ftl.Error("Invalid input: param must be a string")
    }
    
    // Implement your logic here
    result := processParam(param)
    
    return ftl.Text(result)
}

// Add to the tools map in main()
"myNewTool": {
    Description: "Description of your tool",
    InputSchema: map[string]interface{}{
        "type": "object",
        "properties": map[string]interface{}{
            "param": map[string]interface{}{
                "type":        "string",
                "description": "Parameter description",
            },
        },
        "required": []string{"param"},
    },
    Handler: myNewToolHandler,
},
```

### Testing

Write tests in `main_test.go`:

```go
func TestMyNewTool(t *testing.T) {
    input := map[string]interface{}{
        "param": "test value",
    }
    
    response := myNewToolHandler(input)
    
    // Check response type
    content, ok := response["content"].([]map[string]interface{})
    if !ok || len(content) == 0 {
        t.Fatal("Invalid response format")
    }
    
    // Check response text
    text, ok := content[0]["text"].(string)
    if !ok {
        t.Fatal("Response should contain text")
    }
    
    if text != "Expected result" {
        t.Errorf("Expected 'Expected result', got '%s'", text)
    }
}
```

### Code Quality

This project uses:
- **gofmt** for code formatting
- **golangci-lint** for comprehensive linting
- **go test** with coverage for testing

Run all checks:
```bash
make quality
```

## Building for WebAssembly

The project is configured to build with TinyGo for WebAssembly:

```bash
# Build with make
make build

# Or build directly with TinyGo
tinygo build -target=wasip1 -gc=leaking -scheduler=none -no-debug -o app.wasm main.go
```

### TinyGo Limitations

When using TinyGo, be aware of these limitations:
- No goroutines (scheduler disabled for WASI)
- Limited reflection support
- Some standard library packages not available
- Memory management differences (using -gc=leaking)

## Best Practices

1. **Input Validation**: Always validate and type-assert inputs
   ```go
   value, ok := input["key"].(string)
   if !ok {
       return ftl.Error("key must be a string")
   }
   ```

2. **Error Handling**: Return clear, actionable error messages
   ```go
   if err != nil {
       return ftl.Errorf("Failed to process: %v", err)
   }
   ```

3. **Testing**: Write comprehensive tests for all handlers
4. **Memory Usage**: Be mindful of memory constraints in WASI
5. **Documentation**: Document all tools and parameters clearly

## Running Your Tool

After building, start the local development server:

```bash
ftl up
```

Your MCP server will be available at `http://localhost:3000/` and can be used with any MCP-compatible client.

## Troubleshooting

**Build fails with "tinygo not found":**
```bash
# Install TinyGo
go install tinygo.org/x/tinygo@latest
# Or follow official installation: https://tinygo.org/getting-started/install/
tinygo version
```

**Build fails with Go version error:**
```bash
# Check Go version (must be 1.23+)
go version
# Update Go if necessary
```

**Dependency issues:**
```bash
# Clean and rebuild dependencies
go mod tidy
go mod download
make clean
make build
```

**TinyGo compilation errors:**
- Check if your dependencies are TinyGo-compatible
- Avoid packages that use CGO or unsupported features
- Use `tinygo build -target=wasip1` to test compilation directly

**Runtime errors in WASM:**
- No goroutines available (scheduler disabled)
- Limited reflection support - avoid reflect package
- Memory leaks possible (-gc=leaking) - be mindful of allocations
- Some standard library packages not available

**Input validation errors:**
```go
// Always validate and type-assert inputs
value, ok := input["key"].(string)
if !ok {
    return ftl.Error("key must be a string")
}
```

**Testing failures:**
```bash
# Run tests with verbose output
make test
# Run with coverage
make test-cov
```