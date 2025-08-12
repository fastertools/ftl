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
	// Validate input type - must be a struct for proper schema generation
	if err := validateHandlerInputType[In](name); err != nil {
		secureLogf("Failed to register tool '%s': %v", name, err)
		return
	}
	
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

// validateHandlerInputType ensures the input type is valid for V3 handlers
func validateHandlerInputType[In any](toolName string) error {
	var zero In
	inputType := reflect.TypeOf(zero)
	
	// Handle pointer types by getting the underlying type
	if inputType != nil && inputType.Kind() == reflect.Ptr {
		inputType = inputType.Elem()
	}
	
	// V3 handlers require struct inputs for automatic schema generation
	if inputType == nil || inputType.Kind() != reflect.Struct {
		return fmt.Errorf("input type for tool '%s' must be a struct, got %v. "+
			"V3 handlers require struct types to enable automatic JSON schema generation. "+
			"Wrap primitive types in a struct (e.g., type Input struct { Value %v `json:\"value\"` })",
			toolName, inputType, inputType)
	}
	
	return nil
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
	// Use direct conversion with the mapstructure-like approach for better performance
	return directMapToStruct(input, target)
}

// directMapToStruct converts a map[string]interface{} directly to a struct using reflection
// This is more efficient than double JSON marshaling/unmarshaling
func directMapToStruct(input map[string]interface{}, target interface{}) error {
	targetVal := reflect.ValueOf(target)
	if targetVal.Kind() != reflect.Ptr {
		return fmt.Errorf("target must be a pointer to struct")
	}
	
	targetElem := targetVal.Elem()
	if targetElem.Kind() != reflect.Struct {
		return fmt.Errorf("target must point to a struct")
	}
	
	targetType := targetElem.Type()
	
	for i := 0; i < targetType.NumField(); i++ {
		field := targetType.Field(i)
		fieldVal := targetElem.Field(i)
		
		// Skip unexported fields
		if !field.IsExported() || !fieldVal.CanSet() {
			continue
		}
		
		// Get JSON field name
		jsonName := getJSONFieldName(field)
		if jsonName == "-" || jsonName == "" {
			continue
		}
		
		// Get value from input map
		inputVal, exists := input[jsonName]
		if !exists {
			continue
		}
		
		// Convert and set the field value
		if err := setFieldValue(fieldVal, inputVal); err != nil {
			return fmt.Errorf("failed to set field %s: %w", field.Name, err)
		}
	}
	
	return nil
}

// setFieldValue sets a struct field value from an interface{} with type conversion
func setFieldValue(fieldVal reflect.Value, inputVal interface{}) error {
	if inputVal == nil {
		return nil // Skip nil values
	}
	
	inputValue := reflect.ValueOf(inputVal)
	fieldType := fieldVal.Type()
	
	// Handle direct type matches
	if inputValue.Type() == fieldType {
		fieldVal.Set(inputValue)
		return nil
	}
	
	// Handle convertible types
	if inputValue.Type().ConvertibleTo(fieldType) {
		fieldVal.Set(inputValue.Convert(fieldType))
		return nil
	}
	
	// Handle special cases for common JSON->Go type conversions
	switch fieldType.Kind() {
	case reflect.String:
		if str, ok := inputVal.(string); ok {
			fieldVal.SetString(str)
		} else {
			fieldVal.SetString(fmt.Sprintf("%v", inputVal))
		}
	case reflect.Int, reflect.Int8, reflect.Int16, reflect.Int32, reflect.Int64:
		if num, ok := inputVal.(float64); ok {
			fieldVal.SetInt(int64(num))
		} else if num, ok := inputVal.(int64); ok {
			fieldVal.SetInt(num)
		} else {
			return fmt.Errorf("cannot convert %T to %s", inputVal, fieldType.Kind())
		}
	case reflect.Uint, reflect.Uint8, reflect.Uint16, reflect.Uint32, reflect.Uint64:
		if num, ok := inputVal.(float64); ok {
			fieldVal.SetUint(uint64(num))
		} else if num, ok := inputVal.(uint64); ok {
			fieldVal.SetUint(num)
		} else {
			return fmt.Errorf("cannot convert %T to %s", inputVal, fieldType.Kind())
		}
	case reflect.Float32, reflect.Float64:
		if num, ok := inputVal.(float64); ok {
			fieldVal.SetFloat(num)
		} else {
			return fmt.Errorf("cannot convert %T to %s", inputVal, fieldType.Kind())
		}
	case reflect.Bool:
		if b, ok := inputVal.(bool); ok {
			fieldVal.SetBool(b)
		} else {
			return fmt.Errorf("cannot convert %T to bool", inputVal)
		}
	default:
		// For complex types, fall back to JSON marshaling (still more efficient than double marshaling)
		jsonData, err := json.Marshal(inputVal)
		if err != nil {
			return fmt.Errorf("failed to marshal field value: %w", err)
		}
		
		// Create a new instance of the field type
		newVal := reflect.New(fieldType)
		if err := json.Unmarshal(jsonData, newVal.Interface()); err != nil {
			return fmt.Errorf("failed to unmarshal field value: %w", err)
		}
		
		fieldVal.Set(newVal.Elem())
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
		if str, ok := output.(string); ok {
			return Text(str)
		}
		return Text(fmt.Sprintf("%v", output))
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