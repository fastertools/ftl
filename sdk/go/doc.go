// Package ftl provides the FasterTools Tool Language (FTL) SDK for Go.
//
// The FTL SDK enables developers to create type-safe, composable tools that
// can be discovered and executed by any FTL-compatible gateway or runtime.
//
// # V3 API - Type-Safe Handlers
//
// The V3 API introduces idiomatic Go patterns with compile-time type safety:
//
//	type EchoInput struct {
//	    Message string `json:"message" jsonschema:"required,description=Message to echo"`
//	    Count   int    `json:"count,omitempty" jsonschema:"minimum=1,maximum=10"`
//	}
//	
//	type EchoOutput struct {
//	    Response string `json:"response"`
//	    Length   int    `json:"length"`
//	}
//	
//	func EchoHandler(ctx context.Context, input EchoInput) (EchoOutput, error) {
//	    // Type-safe implementation
//	    return EchoOutput{
//	        Response: strings.Repeat(input.Message, input.Count),
//	        Length:   len(input.Message) * input.Count,
//	    }, nil
//	}
//	
//	func init() {
//	    ftl.HandleTypedTool("echo", EchoHandler)
//	}
//
// # Automatic Schema Generation
//
// The V3 API automatically generates JSON schemas from struct tags:
//
//	- `json` tags define field names
//	- `jsonschema` tags define validation constraints
//	- Required fields, descriptions, and validation rules are extracted
//
// Supported jsonschema tags:
//	- required: Mark field as required
//	- description: Field description
//	- minimum/maximum: Numeric constraints
//	- minLength/maxLength: String length constraints
//	- pattern: Regex pattern for strings
//	- enum: Allowed values
//
// # Response Building
//
// The V3 API provides a fluent response builder:
//
//	response := ftl.NewResponse().
//	    AddText("Processing complete").
//	    AddStructured(result).
//	    Build()
//
// # Error Handling
//
// The V3 API uses standard Go error patterns:
//
//	if input.Value < 0 {
//	    return Output{}, ftl.InvalidInput("value", "must be positive")
//	}
//	
//	if err := externalCall(); err != nil {
//	    return Output{}, ftl.ToolFailed("external call failed", err)
//	}
//
// # Context Support
//
// All V3 handlers receive a context.Context for cancellation and metadata:
//
//	func Handler(ctx context.Context, input Input) (Output, error) {
//	    // Check for cancellation
//	    if ctx.Err() != nil {
//	        return Output{}, ctx.Err()
//	    }
//	    
//	    // Access tool context
//	    if toolCtx, ok := ctx.(*ftl.ToolContext); ok {
//	        toolCtx.Log("INFO", "Processing request %s", toolCtx.RequestID)
//	    }
//	    
//	    return Output{}, nil
//	}
//
// # Migration from V1/V2
//
// The V3 API is fully backward compatible. Existing tools continue to work
// while you gradually migrate to type-safe handlers. Both APIs can coexist
// in the same codebase.
//
// # Architecture
//
// The FTL SDK follows a "simple SDK, powerful gateway" philosophy:
//	- SDK provides type safety and developer ergonomics
//	- Gateway handles discovery, routing, and execution
//	- Tools are stateless and composable
//	- Zero external dependencies for WASM compatibility
//
package ftl