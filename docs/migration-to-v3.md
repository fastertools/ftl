# Migration Guide: FTL Go SDK V3

This guide helps you migrate from the V1/V2 FTL Go SDK to the new V3 type-safe API.

## Overview

The V3 API introduces:
- **Type-safe handlers** using Go generics
- **Automatic JSON schema generation** from struct tags
- **Fluent response building** APIs
- **Standard Go error handling** patterns
- **Context support** for cancellation and metadata

**Important**: V3 is fully backward compatible. You can migrate gradually while both APIs coexist.

## Quick Start

### Before (V1/V2)

```go
func init() {
    ftl.CreateTools(map[string]ftl.ToolDefinition{
        "echo": {
            Description: "Echo a message",
            InputSchema: map[string]interface{}{
                "type": "object",
                "properties": map[string]interface{}{
                    "message": map[string]interface{}{
                        "type": "string",
                        "description": "Message to echo",
                    },
                },
                "required": []string{"message"},
            },
            Handler: func(input map[string]interface{}) ftl.ToolResponse {
                message, ok := input["message"].(string)
                if !ok {
                    return ftl.Error("Invalid message")
                }
                return ftl.Text(message)
            },
        },
    })
}
```

### After (V3)

```go
type EchoInput struct {
    Message string `json:"message" jsonschema:"required,description=Message to echo"`
}

func EchoHandler(ctx context.Context, input EchoInput) (string, error) {
    return input.Message, nil
}

func init() {
    ftl.HandleTypedTool("echo", EchoHandler)
}
```

## Step-by-Step Migration

### Step 1: Define Input/Output Types

Create structs for your tool's input and output:

```go
// Input struct with validation
type CalculatorInput struct {
    Operation string  `json:"operation" jsonschema:"required,enum=add|subtract|multiply|divide"`
    A         float64 `json:"a" jsonschema:"required,description=First operand"`
    B         float64 `json:"b" jsonschema:"required,description=Second operand"`
}

// Output struct
type CalculatorOutput struct {
    Result float64 `json:"result"`
    Formula string `json:"formula"`
}
```

### Step 2: Convert Handler to Type-Safe Function

Replace map-based handlers with typed functions:

```go
// Before
Handler: func(input map[string]interface{}) ftl.ToolResponse {
    op := input["operation"].(string)
    a := input["a"].(float64)
    b := input["b"].(float64)
    
    var result float64
    switch op {
    case "add":
        result = a + b
    // ...
    }
    
    return ftl.Text(fmt.Sprintf("Result: %f", result))
}

// After
func CalculatorHandler(ctx context.Context, input CalculatorInput) (CalculatorOutput, error) {
    var result float64
    var formula string
    
    switch input.Operation {
    case "add":
        result = input.A + input.B
        formula = fmt.Sprintf("%f + %f = %f", input.A, input.B, result)
    case "divide":
        if input.B == 0 {
            return CalculatorOutput{}, ftl.InvalidInput("b", "cannot divide by zero")
        }
        result = input.A / input.B
        formula = fmt.Sprintf("%f / %f = %f", input.A, input.B, result)
    // ...
    }
    
    return CalculatorOutput{
        Result: result,
        Formula: formula,
    }, nil
}
```

### Step 3: Register with V3 API

Replace `CreateTools` with `HandleTypedTool`:

```go
// Before
func init() {
    ftl.CreateTools(tools)
}

// After
func init() {
    ftl.HandleTypedTool("calculator", CalculatorHandler)
}
```

## Schema Tag Reference

### Basic Types

```go
type Example struct {
    // Required string field
    Name string `json:"name" jsonschema:"required,description=User name"`
    
    // Optional integer with constraints
    Age int `json:"age,omitempty" jsonschema:"minimum=0,maximum=120"`
    
    // String with pattern validation
    Email string `json:"email" jsonschema:"pattern=^[a-z]+@[a-z]+\\.[a-z]+$"`
    
    // Enum field
    Status string `json:"status" jsonschema:"enum=active|inactive|pending"`
    
    // Array field
    Tags []string `json:"tags" jsonschema:"description=List of tags"`
    
    // Nested object
    Address Address `json:"address,omitempty"`
}
```

### Validation Constraints

| Tag | Description | Example |
|-----|-------------|---------|
| `required` | Mark field as required | `jsonschema:"required"` |
| `description` | Field description | `jsonschema:"description=Field purpose"` |
| `minimum` | Minimum numeric value | `jsonschema:"minimum=0"` |
| `maximum` | Maximum numeric value | `jsonschema:"maximum=100"` |
| `minLength` | Minimum string length | `jsonschema:"minLength=3"` |
| `maxLength` | Maximum string length | `jsonschema:"maxLength=50"` |
| `pattern` | Regex pattern | `jsonschema:"pattern=^[A-Z].*"` |
| `enum` | Allowed values | `jsonschema:"enum=red|green|blue"` |

## Error Handling

### V3 Error Types

```go
// Input validation error
return Output{}, ftl.InvalidInput("field", "validation message")

// Tool execution error
return Output{}, ftl.ToolFailed("operation failed", err)

// Internal error
return Output{}, ftl.InternalError("unexpected condition")

// Custom error
return Output{}, ftl.NewToolError("custom_code", "error message")
```

### Error Conversion

V3 automatically converts Go errors to appropriate ToolResponse:

```go
func Handler(ctx context.Context, input Input) (Output, error) {
    // Standard Go error
    if err := validate(input); err != nil {
        return Output{}, err // Automatically converted
    }
    
    // Wrapped error with context
    if err := process(); err != nil {
        return Output{}, fmt.Errorf("processing failed: %w", err)
    }
    
    return Output{Result: "success"}, nil
}
```

## Response Building

### Simple Responses

```go
// Text response
return "Hello, World!", nil

// Numeric response
return 42, nil

// Boolean response
return true, nil
```

### Complex Responses

```go
// Struct response (automatically serialized)
return MyOutput{
    Field1: "value",
    Field2: 123,
}, nil

// Using response builder for mixed content
response := ftl.NewResponse().
    AddText("Processing complete").
    AddStructured(result).
    AddImage(imageData, "image/png").
    Build()
```

## Context Usage

### Accessing Tool Context

```go
func Handler(ctx context.Context, input Input) (Output, error) {
    // Type assert to access tool context
    if toolCtx, ok := ctx.(*ftl.ToolContext); ok {
        // Log with context
        toolCtx.Log("INFO", "Processing request %s", toolCtx.RequestID)
        
        // Access metadata
        fmt.Printf("Tool: %s, Request: %s\n", toolCtx.ToolName, toolCtx.RequestID)
    }
    
    // Handle cancellation
    select {
    case <-ctx.Done():
        return Output{}, ctx.Err()
    default:
        // Continue processing
    }
    
    return Output{}, nil
}
```

## Testing

### Unit Testing V3 Handlers

```go
func TestEchoHandler(t *testing.T) {
    ctx := context.Background()
    
    input := EchoInput{
        Message: "test",
        Count:   2,
    }
    
    output, err := EchoHandler(ctx, input)
    if err != nil {
        t.Fatalf("Unexpected error: %v", err)
    }
    
    if output.Response != "test test" {
        t.Errorf("Expected 'test test', got '%s'", output.Response)
    }
}
```

### Integration Testing

```go
func TestToolIntegration(t *testing.T) {
    // Register handler
    ftl.HandleTypedTool("test", TestHandler)
    
    // Verify registration
    if !ftl.IsV3Tool("test") {
        t.Error("Tool not registered")
    }
    
    // Test through HTTP interface (if available)
    // ...
}
```

## Gradual Migration Strategy

1. **Phase 1**: Keep existing V1/V2 tools running
2. **Phase 2**: Add new tools using V3 API
3. **Phase 3**: Gradually convert existing tools one at a time
4. **Phase 4**: Remove V1/V2 code once all tools are migrated

### Coexistence Example

```go
func init() {
    // V1/V2 tools continue to work
    ftl.CreateTools(legacyTools)
    
    // New V3 tools
    ftl.HandleTypedTool("new_tool", NewHandler)
    
    // Migrated tools
    ftl.HandleTypedTool("migrated_tool", MigratedHandler)
}
```

## Common Migration Patterns

### Pattern 1: Simple Input/Output

```go
// V1/V2
Handler: func(input map[string]interface{}) ftl.ToolResponse {
    name := input["name"].(string)
    return ftl.Text("Hello, " + name)
}

// V3
func Handler(ctx context.Context, name string) (string, error) {
    return "Hello, " + name, nil
}
```

### Pattern 2: Complex Validation

```go
// V1/V2
Handler: func(input map[string]interface{}) ftl.ToolResponse {
    val, ok := input["value"].(float64)
    if !ok || val < 0 || val > 100 {
        return ftl.Error("Invalid value")
    }
    // ...
}

// V3
type Input struct {
    Value float64 `json:"value" jsonschema:"required,minimum=0,maximum=100"`
}

func Handler(ctx context.Context, input Input) (Output, error) {
    // Validation happens automatically
    // ...
}
```

### Pattern 3: Multiple Return Types

```go
// V1/V2
Handler: func(input map[string]interface{}) ftl.ToolResponse {
    if simple {
        return ftl.Text("simple response")
    }
    return ftl.ToolResponse{
        Content: []ftl.ToolContent{
            {Type: "text", Text: "complex"},
            {Type: "data", Data: encoded},
        },
    }
}

// V3
func Handler(ctx context.Context, input Input) (Output, error) {
    if input.Simple {
        return Output{Text: "simple response"}, nil
    }
    // Use response builder for complex responses
    return Output{Complex: true}, nil
}
```

## Troubleshooting

### Issue: Schema not generating correctly

**Solution**: Ensure struct tags are properly formatted:
```go
// Correct
`json:"field" jsonschema:"required,description=Field description"`

// Incorrect
`json:"field", jsonschema:"required, description=Field description"` // No spaces
```

### Issue: Handler not accepting primitive types

**Solution**: V3 handlers work with any type, including primitives:
```go
// String input/output
func Handler(ctx context.Context, input string) (string, error)

// Struct input, primitive output
func Handler(ctx context.Context, input Input) (int, error)
```

### Issue: Tests failing with undefined functions

**Solution**: Use the `test` build tag:
```go
go test -tags test ./...
```

## Performance Considerations

- **Schema Generation**: Happens once at registration time (not per request)
- **Type Conversion**: Minimal overhead from JSON marshaling/unmarshaling
- **Memory Usage**: Similar to V1/V2 (no additional allocations)
- **WASM Compatibility**: Fully maintained with zero external dependencies

## Getting Help

- Check the [examples/echo_v3](../examples/echo_v3) directory for working examples
- Review the package documentation: `go doc github.com/fastertools/ftl-cli/sdk/go`
- File issues at: https://github.com/fastertools/ftl-cli/issues

## Summary

The V3 API makes FTL tools more idiomatic, type-safe, and maintainable while preserving the architectural simplicity that makes FTL powerful. Migration can be done gradually, allowing you to benefit from V3 features immediately while maintaining existing tools.