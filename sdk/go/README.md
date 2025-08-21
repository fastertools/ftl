# FTL Go SDK

Version: 0.1.1

A lightweight SDK for building MCP (Model Context Protocol) tools with Go using the Spin framework.

[![Go Reference](https://pkg.go.dev/badge/github.com/fastertools/ftl/sdk/go.svg)](https://pkg.go.dev/github.com/fastertools/ftl/sdk/go)
[![Go Report Card](https://goreportcard.com/badge/github.com/fastertools/ftl/sdk/go)](https://goreportcard.com/report/github.com/fastertools/ftl/sdk/go)
![GitHub release (latest SemVer)](https://img.shields.io/github/v/release/fastertools/ftl)

## Installation

### Latest Version

```bash
go get github.com/fastertools/ftl/sdk/go@latest
```

### Specific Version

```bash
go get github.com/fastertools/ftl/sdk/go@v0.1.0
```

### Development Version

```bash
go get github.com/fastertools/ftl/sdk/go@main
```

## Requirements

- Go 1.23+
- TinyGo 0.30.0+ (for WASI compilation)
- Spin CLI (for running tools)

## Quick Start

Create a tool with the FTL CLI:

```bash
ftl init my-app
ftl add my-go-tool -l go
```

```go
package main

import (
    ftl "github.com/fastertools/ftl/sdk/go"
)

func init() {
    ftl.CreateTools(map[string]ftl.ToolDefinition{
        "echo": {
            Description: "Echo the input message",
            InputSchema: map[string]interface{}{
                "type": "object",
                "properties": map[string]interface{}{
                    "message": map[string]interface{}{
                        "type":        "string",
                        "description": "The message to echo",
                    },
                },
                "required": []string{"message"},
            },
            Handler: func(input map[string]interface{}) ftl.ToolResponse {
                message, _ := input["message"].(string)
                return ftl.Text("Echo: " + message)
            },
        },
    })
}

func main() {}
```

Build with TinyGo:

```bash
tinygo build -o echo.wasm -target=wasi main.go
```

## API Reference

### Creating Tools

```go
ftl.CreateTools(tools map[string]ftl.ToolDefinition)
```

Creates a Spin HTTP handler that implements the MCP protocol for the provided tools.

### Tool Definition

```go
type ToolDefinition struct {
    Name         string                   // Optional explicit tool name
    Title        string                   // Optional human-readable title
    Description  string                   // Tool description
    InputSchema  map[string]interface{}   // JSON Schema for input
    OutputSchema map[string]interface{}   // Optional output schema
    Annotations  *ToolAnnotations         // Optional behavior hints
    Meta         map[string]interface{}   // Optional metadata
    Handler      ToolHandler              // Handler function
}
```

### Response Helpers

```go
// Simple text response
ftl.Text("Hello, world!")

// Formatted text response
ftl.Textf("Hello, %s!", name)

// Error response
ftl.Error("Something went wrong")

// Formatted error response
ftl.Errorf("Failed to process: %v", err)

// Response with structured data
ftl.WithStructured("Success", map[string]interface{}{
    "result": 42,
})
```

### Content Types

```go
// Text content
ftl.TextContent("Hello", nil)

// Image content
ftl.ImageContent(base64Data, "image/png", nil)

// Audio content
ftl.AudioContent(base64Data, "audio/wav", nil)

// Resource content
ftl.ResourceContent(&ftl.ResourceContents{
    URI:      "file://example.txt",
    MimeType: "text/plain",
    Text:     "File contents",
}, nil)
```

## Advanced Example

```go
package main

import (
    "encoding/json"
    "fmt"
    "strings"
    
    ftl "github.com/fastertools/ftl/sdk/go"
)

func init() {
    ftl.CreateTools(map[string]ftl.ToolDefinition{
        "text_tools": {
            Name:        "text_tools",
            Description: "Various text manipulation tools",
            InputSchema: map[string]interface{}{
                "type": "object",
                "properties": map[string]interface{}{
                    "operation": map[string]interface{}{
                        "type": "string",
                        "enum": []string{"uppercase", "lowercase", "reverse"},
                        "description": "The operation to perform",
                    },
                    "text": map[string]interface{}{
                        "type":        "string",
                        "description": "The text to manipulate",
                    },
                },
                "required": []string{"operation", "text"},
            },
            Handler: textToolsHandler,
        },
        "json_formatter": {
            Description: "Format JSON data",
            InputSchema: map[string]interface{}{
                "type": "object",
                "properties": map[string]interface{}{
                    "json": map[string]interface{}{
                        "type":        "string",
                        "description": "JSON string to format",
                    },
                    "indent": map[string]interface{}{
                        "type":        "boolean",
                        "description": "Whether to indent the output",
                        "default":     true,
                    },
                },
                "required": []string{"json"},
            },
            Handler: jsonFormatterHandler,
        },
    })
}

func textToolsHandler(input map[string]interface{}) ftl.ToolResponse {
    operation, _ := input["operation"].(string)
    text, _ := input["text"].(string)
    
    var result string
    switch operation {
    case "uppercase":
        result = strings.ToUpper(text)
    case "lowercase":
        result = strings.ToLower(text)
    case "reverse":
        runes := []rune(text)
        for i, j := 0, len(runes)-1; i < j; i, j = i+1, j-1 {
            runes[i], runes[j] = runes[j], runes[i]
        }
        result = string(runes)
    default:
        return ftl.Errorf("Unknown operation: %s", operation)
    }
    
    return ftl.WithStructured(
        fmt.Sprintf("Applied %s operation", operation),
        map[string]interface{}{
            "original": text,
            "result":   result,
            "operation": operation,
        },
    )
}

func jsonFormatterHandler(input map[string]interface{}) ftl.ToolResponse {
    jsonStr, _ := input["json"].(string)
    indent := true
    if val, ok := input["indent"].(bool); ok {
        indent = val
    }
    
    var data interface{}
    if err := json.Unmarshal([]byte(jsonStr), &data); err != nil {
        return ftl.Errorf("Invalid JSON: %v", err)
    }
    
    var formatted []byte
    var err error
    if indent {
        formatted, err = json.MarshalIndent(data, "", "  ")
    } else {
        formatted, err = json.Marshal(data)
    }
    
    if err != nil {
        return ftl.Errorf("Failed to format JSON: %v", err)
    }
    
    return ftl.Text(string(formatted))
}

func main() {}
```

## Building and Running

1. Build your tool with TinyGo:
   ```bash
   tinygo build -o tool.wasm -target=wasi main.go
   ```

2. Add to your `spin.toml`:
   ```toml
   [[component]]
   id = "my-go-tool"
   source = "tool.wasm"
   [component.trigger]
   route = "/tools/my-go-tool/..."
   ```

3. Run with Spin:
   ```bash
   spin up
   ```

## Best Practices

1. **Error Handling**: Always validate input and return clear error messages
2. **Schema Definition**: Provide complete JSON schemas for better tool discovery
3. **Memory Usage**: Be mindful of memory constraints in WASI environments
4. **Tool Naming**: Use descriptive names that clearly indicate the tool's purpose
5. **Documentation**: Include descriptions for all tools and parameters

## Limitations

- Must use TinyGo for WASI compilation (standard Go compiler not supported)
- Limited to packages compatible with TinyGo and WASI
- No goroutines or certain runtime features due to WASI constraints

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Development

```bash
# Clone the repository
git clone https://github.com/fastertools/ftl.git
cd ftl-cli/sdk/go

# Install development dependencies
make dev-deps

# Run tests
make test

# Run linting
make lint

# Run all quality checks
make quality
```

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for a list of changes in each release.

## License

Apache License 2.0 - see [LICENSE](../../LICENSE) for details.
