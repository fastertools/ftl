// Package ftl - V3 Type-Safe Handlers
//
// This file adds idiomatic Go type-safe handlers on top of the existing
// SDK architecture without breaking existing functionality.
package ftl

import (
	"context"
	"fmt"
	"math"
	"reflect"
	"regexp"
	"time"
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

// executeTypedHandler executes a typed handler with proper type conversion and resource protection
func executeTypedHandler[In, Out any](name string, handler TypedHandler[In, Out], input map[string]interface{}) ToolResponse {
	// 1. Validate input parameters
	if err := validateExecutionInput(name, handler, input); err != nil {
		return convertError(err)
	}
	
	// 2. Create tool context with metadata and timeout protection
	toolCtx := NewToolContext(name)
	
	// 3. Convert input map to typed struct with validation
	var typedInput In
	if err := unmarshalInput(input, &typedInput); err != nil {
		return convertError(InvalidInput("input", "failed to parse input: invalid format"))
	}
	
	// 4. Validate the converted input against constraints
	if err := validateStructInput(typedInput); err != nil {
		return convertError(err)
	}
	
	// 5. Execute handler with timeout protection to prevent resource exhaustion
	result, err := executeWithTimeout(toolCtx, handler, typedInput)
	if err != nil {
		return convertError(err)
	}
	
	// 6. Convert result to ToolResponse
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
			return fmt.Errorf("failed to set field %s: invalid value type", field.Name)
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
		return setIntField(fieldVal, inputVal, fieldType)
	case reflect.Uint, reflect.Uint8, reflect.Uint16, reflect.Uint32, reflect.Uint64:
		return setUintField(fieldVal, inputVal, fieldType)
	case reflect.Float32, reflect.Float64:
		return setFloatField(fieldVal, inputVal, fieldType)
	case reflect.Bool:
		if b, ok := inputVal.(bool); ok {
			fieldVal.SetBool(b)
		} else {
			return fmt.Errorf("cannot convert %T to bool", inputVal)
		}
	case reflect.Struct:
		// For structs, attempt direct field mapping
		if mapVal, ok := inputVal.(map[string]interface{}); ok {
			// Create a new value of the field type and set it
			newVal := reflect.New(fieldType)
			if err := directMapToStruct(mapVal, newVal.Interface()); err != nil {
				return err
			}
			fieldVal.Set(newVal.Elem())
			return nil
		}
		return fmt.Errorf("cannot convert %T to struct %s", inputVal, fieldType.Name())
	case reflect.Slice:
		// Handle slice types
		if sliceVal, ok := inputVal.([]interface{}); ok {
			return setSliceField(fieldVal, sliceVal, fieldType)
		}
		return fmt.Errorf("cannot convert %T to slice %s", inputVal, fieldType)
	case reflect.Map:
		// Handle map types
		if mapVal, ok := inputVal.(map[string]interface{}); ok {
			return setMapField(fieldVal, mapVal, fieldType)
		}
		return fmt.Errorf("cannot convert %T to map %s", inputVal, fieldType)
	default:
		return fmt.Errorf("unsupported field type %s for field conversion", fieldType.Kind())
	}
	
	return nil
}

// setIntField safely sets integer fields with overflow checking
func setIntField(fieldVal reflect.Value, inputVal interface{}, fieldType reflect.Type) error {
	var intVal int64
	
	switch v := inputVal.(type) {
	case float64:
		if v != math.Trunc(v) {
			return fmt.Errorf("cannot convert float %g with decimal to integer", v)
		}
		if v > math.MaxInt64 || v < math.MinInt64 {
			return fmt.Errorf("value %g overflows int64", v)
		}
		intVal = int64(v)
	case int64:
		intVal = v
	case int:
		intVal = int64(v)
	default:
		return fmt.Errorf("cannot convert %T to %s", inputVal, fieldType.Kind())
	}
	
	// Check bounds for specific int types
	switch fieldType.Kind() {
	case reflect.Int8:
		if intVal > math.MaxInt8 || intVal < math.MinInt8 {
			return fmt.Errorf("value %d overflows int8", intVal)
		}
	case reflect.Int16:
		if intVal > math.MaxInt16 || intVal < math.MinInt16 {
			return fmt.Errorf("value %d overflows int16", intVal)
		}
	case reflect.Int32:
		if intVal > math.MaxInt32 || intVal < math.MinInt32 {
			return fmt.Errorf("value %d overflows int32", intVal)
		}
	}
	
	fieldVal.SetInt(intVal)
	return nil
}

// setUintField safely sets unsigned integer fields with overflow checking
func setUintField(fieldVal reflect.Value, inputVal interface{}, fieldType reflect.Type) error {
	var uintVal uint64
	
	switch v := inputVal.(type) {
	case float64:
		if v < 0 {
			return fmt.Errorf("cannot convert negative value %g to unsigned integer", v)
		}
		if v != math.Trunc(v) {
			return fmt.Errorf("cannot convert float %g with decimal to unsigned integer", v)
		}
		if v > math.MaxUint64 {
			return fmt.Errorf("value %g overflows uint64", v)
		}
		uintVal = uint64(v)
	case uint64:
		uintVal = v
	case int64:
		if v < 0 {
			return fmt.Errorf("cannot convert negative value %d to unsigned integer", v)
		}
		uintVal = uint64(v)
	default:
		return fmt.Errorf("cannot convert %T to %s", inputVal, fieldType.Kind())
	}
	
	// Check bounds for specific uint types
	switch fieldType.Kind() {
	case reflect.Uint8:
		if uintVal > math.MaxUint8 {
			return fmt.Errorf("value %d overflows uint8", uintVal)
		}
	case reflect.Uint16:
		if uintVal > math.MaxUint16 {
			return fmt.Errorf("value %d overflows uint16", uintVal)
		}
	case reflect.Uint32:
		if uintVal > math.MaxUint32 {
			return fmt.Errorf("value %d overflows uint32", uintVal)
		}
	}
	
	fieldVal.SetUint(uintVal)
	return nil
}

// setFloatField safely sets float fields with overflow checking
func setFloatField(fieldVal reflect.Value, inputVal interface{}, fieldType reflect.Type) error {
	var floatVal float64
	
	switch v := inputVal.(type) {
	case float64:
		floatVal = v
	case float32:
		floatVal = float64(v)
	case int64:
		floatVal = float64(v)
	case int:
		floatVal = float64(v)
	default:
		return fmt.Errorf("cannot convert %T to %s", inputVal, fieldType.Kind())
	}
	
	// Check bounds for float32
	if fieldType.Kind() == reflect.Float32 {
		if floatVal > math.MaxFloat32 || floatVal < -math.MaxFloat32 {
			return fmt.Errorf("value %g overflows float32", floatVal)
		}
	}
	
	fieldVal.SetFloat(floatVal)
	return nil
}

// setSliceField sets a slice field from a slice of interface{}
func setSliceField(fieldVal reflect.Value, sliceVal []interface{}, fieldType reflect.Type) error {
	newSlice := reflect.MakeSlice(fieldType, len(sliceVal), len(sliceVal))
	
	for i, item := range sliceVal {
		if err := setFieldValue(newSlice.Index(i), item); err != nil {
			return fmt.Errorf("failed to set slice element %d: %w", i, err)
		}
	}
	
	fieldVal.Set(newSlice)
	return nil
}

// setMapField sets a map field from a map[string]interface{}
func setMapField(fieldVal reflect.Value, mapVal map[string]interface{}, fieldType reflect.Type) error {
	if fieldType.Key().Kind() != reflect.String {
		return fmt.Errorf("only maps with string keys are supported")
	}
	
	newMap := reflect.MakeMap(fieldType)
	elemType := fieldType.Elem()
	
	for k, v := range mapVal {
		keyVal := reflect.ValueOf(k)
		elemVal := reflect.New(elemType).Elem()
		
		if err := setFieldValue(elemVal, v); err != nil {
			return fmt.Errorf("failed to set map value for key %s: %w", k, err)
		}
		
		newMap.SetMapIndex(keyVal, elemVal)
	}
	
	fieldVal.Set(newMap)
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

// Input validation functions

// validateExecutionInput validates the basic parameters for handler execution
func validateExecutionInput[In, Out any](name string, handler TypedHandler[In, Out], input map[string]interface{}) error {
	// Validate tool name
	if name == "" {
		return InvalidInput("name", "tool name cannot be empty")
	}
	
	// Validate tool name format (alphanumeric + underscore/hyphen)
	if !isValidToolName(name) {
		return InvalidInput("name", "tool name must contain only alphanumeric characters, underscores, and hyphens")
	}
	
	// Validate handler is not nil
	if handler == nil {
		return InternalError("handler cannot be nil")
	}
	
	// Validate input map
	if input == nil {
		return InvalidInput("input", "input cannot be nil")
	}
	
	// Check input size limits (protect against resource exhaustion)
	if err := validateInputSize(input); err != nil {
		return err
	}
	
	return nil
}

// validateStructInput validates struct field constraints using reflection and struct tags
func validateStructInput(input interface{}) error {
	if input == nil {
		return nil
	}
	
	val := reflect.ValueOf(input)
	typ := reflect.TypeOf(input)
	
	// Handle pointer types
	if val.Kind() == reflect.Ptr {
		if val.IsNil() {
			return nil
		}
		val = val.Elem()
		typ = typ.Elem()
	}
	
	if val.Kind() != reflect.Struct {
		return nil // Only validate structs
	}
	
	// Validate each field
	for i := 0; i < typ.NumField(); i++ {
		field := typ.Field(i)
		fieldVal := val.Field(i)
		
		// Skip unexported fields
		if !field.IsExported() {
			continue
		}
		
		// Validate field constraints from struct tags
		if err := validateFieldConstraints(field, fieldVal); err != nil {
			return err
		}
	}
	
	return nil
}

// validateFieldConstraints validates individual field constraints from jsonschema tags
func validateFieldConstraints(field reflect.StructField, value reflect.Value) error {
	schemaTag := field.Tag.Get("jsonschema")
	if schemaTag == "" {
		return nil
	}
	
	// Parse schema constraints
	constraints := parseSchemaTag(schemaTag)
	
	// Check required fields
	if _, required := constraints["required"]; required {
		if isZeroValue(value) {
			return ValidationError{
				Field:   field.Name,
				Message: "field is required but was not provided or is empty",
			}
		}
	}
	
	// Validate string constraints
	if value.Kind() == reflect.String {
		if err := validateStringConstraints(field.Name, value.String(), constraints); err != nil {
			return err
		}
	}
	
	// Validate numeric constraints
	if isNumericType(value.Kind()) {
		if err := validateNumericConstraints(field.Name, value, constraints); err != nil {
			return err
		}
	}
	
	// Validate array/slice constraints
	if value.Kind() == reflect.Slice || value.Kind() == reflect.Array {
		if err := validateArrayConstraints(field.Name, value, constraints); err != nil {
			return err
		}
	}
	
	return nil
}

// validateStringConstraints validates string-specific constraints
func validateStringConstraints(fieldName, value string, constraints map[string]interface{}) error {
	// Validate minimum length
	if minLen, exists := constraints["minLength"]; exists {
		if min, ok := minLen.(float64); ok && len(value) < int(min) {
			return ValidationError{
				Field:   fieldName,
				Message: fmt.Sprintf("string length %d is less than minimum %d", len(value), int(min)),
			}
		}
	}
	
	// Validate maximum length
	if maxLen, exists := constraints["maxLength"]; exists {
		if max, ok := maxLen.(float64); ok && len(value) > int(max) {
			return ValidationError{
				Field:   fieldName,
				Message: fmt.Sprintf("string length %d exceeds maximum %d", len(value), int(max)),
			}
		}
	}
	
	// Validate pattern (if provided)
	if pattern, exists := constraints["pattern"]; exists {
		if patternStr, ok := pattern.(string); ok {
			if matched, err := regexp.MatchString(patternStr, value); err != nil {
				return ValidationError{
					Field:   fieldName,
					Message: "invalid pattern configuration",
				}
			} else if !matched {
				return ValidationError{
					Field:   fieldName,
					Message: "value does not match required pattern",
				}
			}
		}
	}
	
	return nil
}

// validateNumericConstraints validates numeric constraints
func validateNumericConstraints(fieldName string, value reflect.Value, constraints map[string]interface{}) error {
	var numVal float64
	
	// Convert to float64 for comparison
	switch value.Kind() {
	case reflect.Int, reflect.Int8, reflect.Int16, reflect.Int32, reflect.Int64:
		numVal = float64(value.Int())
	case reflect.Uint, reflect.Uint8, reflect.Uint16, reflect.Uint32, reflect.Uint64:
		numVal = float64(value.Uint())
	case reflect.Float32, reflect.Float64:
		numVal = value.Float()
	default:
		return nil // Not a numeric type
	}
	
	// Validate minimum
	if min, exists := constraints["minimum"]; exists {
		if minVal, ok := min.(float64); ok && numVal < minVal {
			return ValidationError{
				Field:   fieldName,
				Message: fmt.Sprintf("value %g is less than minimum %g", numVal, minVal),
			}
		}
	}
	
	// Validate maximum
	if max, exists := constraints["maximum"]; exists {
		if maxVal, ok := max.(float64); ok && numVal > maxVal {
			return ValidationError{
				Field:   fieldName,
				Message: fmt.Sprintf("value %g exceeds maximum %g", numVal, maxVal),
			}
		}
	}
	
	return nil
}

// validateArrayConstraints validates array/slice constraints
func validateArrayConstraints(fieldName string, value reflect.Value, constraints map[string]interface{}) error {
	length := value.Len()
	
	// Validate minimum items
	if minItems, exists := constraints["minItems"]; exists {
		if min, ok := minItems.(float64); ok && length < int(min) {
			return ValidationError{
				Field:   fieldName,
				Message: fmt.Sprintf("array has %d items, minimum required is %d", length, int(min)),
			}
		}
	}
	
	// Validate maximum items
	if maxItems, exists := constraints["maxItems"]; exists {
		if max, ok := maxItems.(float64); ok && length > int(max) {
			return ValidationError{
				Field:   fieldName,
				Message: fmt.Sprintf("array has %d items, maximum allowed is %d", length, int(max)),
			}
		}
	}
	
	return nil
}

// validateInputSize protects against resource exhaustion attacks
func validateInputSize(input map[string]interface{}) error {
	const (
		maxMapSize       = 1000   // Maximum number of top-level fields
		maxStringLength  = 10000  // Maximum string field length
		maxNestingDepth  = 10     // Maximum nesting depth
		maxArraySize     = 10000  // Maximum array size
		maxTotalElements = 100000 // Maximum total elements across all arrays/objects
	)
	
	// Check map size
	if len(input) > maxMapSize {
		return InvalidInput("input", fmt.Sprintf("input has %d fields, maximum allowed is %d", len(input), maxMapSize))
	}
	
	// Check string lengths, nesting depth, and array sizes with element counting
	elementCounter := &elementCounter{count: 0, maxElements: maxTotalElements}
	return validateInputDepth(input, 0, maxNestingDepth, maxStringLength, maxArraySize, elementCounter)
}

// elementCounter tracks total elements to prevent excessive memory usage
type elementCounter struct {
	count       int
	maxElements int
}

// validateInputDepth recursively validates input depth, string lengths, and array sizes
func validateInputDepth(obj interface{}, depth, maxDepth, maxStringLen, maxArraySize int, counter *elementCounter) error {
	if depth > maxDepth {
		return InvalidInput("input", fmt.Sprintf("input nesting depth %d exceeds maximum %d", depth, maxDepth))
	}
	
	// Check total element count to prevent memory exhaustion
	counter.count++
	if counter.count > counter.maxElements {
		return InvalidInput("input", fmt.Sprintf("total input elements %d exceeds maximum %d", counter.count, counter.maxElements))
	}
	
	switch v := obj.(type) {
	case string:
		if len(v) > maxStringLen {
			return InvalidInput("input", fmt.Sprintf("string length %d exceeds maximum %d", len(v), maxStringLen))
		}
	case map[string]interface{}:
		// Check map size at each level
		if len(v) > maxArraySize {
			return InvalidInput("input", fmt.Sprintf("map size %d exceeds maximum %d", len(v), maxArraySize))
		}
		for _, value := range v {
			if err := validateInputDepth(value, depth+1, maxDepth, maxStringLen, maxArraySize, counter); err != nil {
				return err
			}
		}
	case []interface{}:
		// Check array size
		if len(v) > maxArraySize {
			return InvalidInput("input", fmt.Sprintf("array size %d exceeds maximum %d", len(v), maxArraySize))
		}
		for _, value := range v {
			if err := validateInputDepth(value, depth+1, maxDepth, maxStringLen, maxArraySize, counter); err != nil {
				return err
			}
		}
	}
	
	return nil
}

// executeWithTimeout executes a handler with timeout protection to prevent resource exhaustion
func executeWithTimeout[In, Out any](toolCtx *ToolContext, handler TypedHandler[In, Out], input In) (Out, error) {
	const (
		defaultHandlerTimeout = 30 * time.Second // Maximum execution time for any handler
	)
	
	var result Out
	var err error
	
	// Create timeout context to prevent long-running handlers from exhausting resources
	timeoutCtx, cancel := context.WithTimeout(toolCtx.Context, defaultHandlerTimeout)
	defer cancel()
	
	// Execute handler in a goroutine with timeout protection
	done := make(chan struct{})
	go func() {
		defer func() {
			if r := recover(); r != nil {
				// Convert panic to error to prevent crashes
				err = InternalError("handler execution failed due to panic")
			}
			close(done)
		}()
		
		result, err = handler(timeoutCtx, input)
	}()
	
	// Wait for completion or timeout
	select {
	case <-done:
		// Handler completed normally
		return result, err
	case <-timeoutCtx.Done():
		// Handler timed out
		return result, ToolError{
			Code:    "timeout_error",
			Message: fmt.Sprintf("handler execution exceeded timeout of %v", defaultHandlerTimeout),
			Cause:   timeoutCtx.Err(),
		}
	}
}

// Helper functions

// isValidToolName checks if a tool name contains only allowed characters
func isValidToolName(name string) bool {
	if len(name) == 0 || len(name) > 100 {
		return false
	}
	
	for _, r := range name {
		if !((r >= 'a' && r <= 'z') || (r >= 'A' && r <= 'Z') || (r >= '0' && r <= '9') || r == '_' || r == '-') {
			return false
		}
	}
	return true
}

// isZeroValue checks if a reflect.Value represents a zero value
func isZeroValue(value reflect.Value) bool {
	switch value.Kind() {
	case reflect.String:
		return value.String() == ""
	case reflect.Int, reflect.Int8, reflect.Int16, reflect.Int32, reflect.Int64:
		return value.Int() == 0
	case reflect.Uint, reflect.Uint8, reflect.Uint16, reflect.Uint32, reflect.Uint64:
		return value.Uint() == 0
	case reflect.Float32, reflect.Float64:
		return value.Float() == 0
	case reflect.Bool:
		return !value.Bool()
	case reflect.Ptr, reflect.Interface, reflect.Slice, reflect.Map, reflect.Chan, reflect.Func:
		return value.IsNil()
	default:
		return false
	}
}

// isNumericType checks if a reflect.Kind represents a numeric type
func isNumericType(kind reflect.Kind) bool {
	return kind >= reflect.Int && kind <= reflect.Float64
}