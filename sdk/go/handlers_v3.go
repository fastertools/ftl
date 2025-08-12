// Package ftl - V3 Type-Safe Handlers
//
// This file adds idiomatic Go type-safe handlers on top of the existing
// SDK architecture without breaking existing functionality.
package ftl

import (
	"context"
	"encoding/json"
	"fmt"
	"reflect"
)

// TypedHandler is the V3 idiomatic handler interface.
// It provides type safety through Go generics and follows standard Go patterns.
type TypedHandler[In, Out any] func(context.Context, In) (Out, error)

// Note: V3 tool registry is now managed in types_v3.go via v3Registry

// HandleTypedTool registers a type-safe tool handler using the V3 API.
// This function generates JSON schema from struct tags and wraps the typed
// handler to work with the existing gateway infrastructure.
//
// Example:
//
//	type EchoInput struct {
//	    Message string `json:"message" jsonschema:"description=Message to echo,required"`
//	}
//	
//	type EchoOutput struct {
//	    Response string `json:"response"`
//	}
//	
//	func EchoHandler(ctx context.Context, input EchoInput) (EchoOutput, error) {
//	    return EchoOutput{Response: "Echo: " + input.Message}, nil
//	}
//	
//	HandleTypedTool("echo", EchoHandler)
func HandleTypedTool[In, Out any](name string, handler TypedHandler[In, Out]) {
	// Generate basic schema from input type (stub implementation for CRAWL phase)
	schema := generateBasicSchema[In]()
	
	// Wrap the typed handler to work with existing infrastructure
	wrappedHandler := func(input map[string]interface{}) ToolResponse {
		return executeTypedHandler(name, handler, input)
	}
	
	// Create V3 tool definition using existing structure
	definition := ToolDefinition{
		Description: fmt.Sprintf("V3 type-safe tool: %s", name),
		InputSchema: schema,
		Handler:     wrappedHandler,
		// Mark as V3 tool in metadata for debugging
		Meta: map[string]interface{}{
			"ftl_sdk_version": "v3",
			"type_safe":       true,
		},
	}
	
	// Register using existing infrastructure
	registerV3Tool(name, definition)
	
	// Tool tracking is now handled by the unified registry
	
	// Debug logging
	secureLogf("Registered V3 type-safe tool: %s", name)
}

// registerV3Tool registers a tool with the unified registry system
func registerV3Tool(name string, definition ToolDefinition) {
	// Create typed definition for V3 registry
	typedDef := TypedToolDefinition{
		ToolDefinition: definition,
		// TODO: Extract type information from generics in full implementation
		InputType:       "interface{}",
		OutputType:      "interface{}",
		SchemaGenerated: true,
	}
	
	// Register with unified V3 registry
	v3Registry.RegisterTypedTool(name, typedDef)
	
	// Also register with legacy system for backwards compatibility
	tools := map[string]ToolDefinition{
		name: definition,
	}
	createToolsIfAvailable(tools)
}


// GetV3ToolNames returns the names of all registered V3 tools (for testing/debugging)
func GetV3ToolNames() []string {
	allTools := v3Registry.GetAllTypedTools()
	names := make([]string, 0, len(allTools))
	for name := range allTools {
		names = append(names, name)
	}
	return names
}

// IsV3Tool checks if a tool was registered using the V3 API
func IsV3Tool(name string) bool {
	_, exists := v3Registry.GetTypedTool(name)
	return exists
}

// executeTypedHandler executes a typed handler with proper type conversion
func executeTypedHandler[In, Out any](name string, handler TypedHandler[In, Out], input map[string]interface{}) ToolResponse {
	// 1. Create tool context with metadata
	toolCtx := NewToolContext(name)
	
	// 2. Convert input map to typed struct
	var typedInput In
	if err := unmarshalInput(input, &typedInput); err != nil {
		return convertError(InvalidInput("input", fmt.Sprintf("failed to parse input: %v", err)))
	}
	
	// 3. Call the typed handler
	result, err := handler(toolCtx, typedInput)
	if err != nil {
		return convertError(err)
	}
	
	// 4. Convert result to ToolResponse
	return convertTypedOutput(result)
}

// unmarshalInput converts a map[string]interface{} to a typed struct
func unmarshalInput(input map[string]interface{}, target interface{}) error {
	// Convert to JSON and back to get proper type conversion
	jsonData, err := json.Marshal(input)
	if err != nil {
		return fmt.Errorf("failed to marshal input to JSON: %w", err)
	}
	
	if err := json.Unmarshal(jsonData, target); err != nil {
		return fmt.Errorf("failed to unmarshal JSON to target type: %w", err)
	}
	
	return nil
}

// convertTypedOutput converts a typed result to ToolResponse
func convertTypedOutput(output interface{}) ToolResponse {
	// Handle nil output
	if output == nil {
		return Text("null")
	}
	
	// Use reflection to check if output is a primitive type
	outputType := reflect.TypeOf(output)
	switch outputType.Kind() {
	case reflect.String:
		return Text(output.(string))
	case reflect.Int, reflect.Int8, reflect.Int16, reflect.Int32, reflect.Int64:
		return Text(fmt.Sprintf("%v", output))
	case reflect.Uint, reflect.Uint8, reflect.Uint16, reflect.Uint32, reflect.Uint64:
		return Text(fmt.Sprintf("%v", output))
	case reflect.Float32, reflect.Float64:
		return Text(fmt.Sprintf("%v", output))
	case reflect.Bool:
		return Text(fmt.Sprintf("%v", output))
	default:
		// For complex types, use structured response
		return StructuredResponse("", output)
	}
}

// generateBasicSchema creates a JSON schema from Go types using the schema generation system
func generateBasicSchema[T any]() map[string]interface{} {
	// Use the proper schema generation from schema_gen.go
	return generateSchema[T]()
}