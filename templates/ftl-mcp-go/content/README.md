# {{project-name}}

An FTL MCP tool written in Go.

## Prerequisites

- Go 1.21 or higher
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
make help         # Show all available commands
make fmt          # Format code with gofmt
make lint         # Run linting with golangci-lint
make test         # Run tests
make test-cov     # Run tests with coverage report
make clean        # Clean build artifacts
make build        # Build WebAssembly module
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

## Deployment

After building with `make build` or `ftl build`, deploy to FTL Engine:

```bash
ftl eng deploy
```

## Troubleshooting

### Build Errors

If you encounter build errors:

1. Ensure TinyGo is installed: `tinygo version`
2. Check Go version: `go version` (must be 1.21+)
3. Run `go mod tidy` to fix dependencies
4. Check for TinyGo-incompatible packages

### Runtime Errors

1. Check Spin logs: `ftl logs`
2. Validate your input schemas match handler expectations
3. Ensure all required fields are handled

## License

Apache License 2.0