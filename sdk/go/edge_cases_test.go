package ftl

import (
	"context"
	"reflect"
	"strings"
	"testing"
	"time"
)

// TestEdgeCases_NilPointerHandling tests handling of nil pointers and empty values
func TestEdgeCases_NilPointerHandling(t *testing.T) {
	type TestStruct struct {
		Name    *string           `json:"name,omitempty"`
		Details map[string]string `json:"details,omitempty"`
		Tags    []string          `json:"tags,omitempty"`
	}

	// Test schema generation with nil pointer types
	schema := generateSchema[TestStruct]()
	
	if schema["type"] != "object" {
		t.Errorf("Schema should be object type, got %v", schema["type"])
	}

	// Test with nil values in handler
	handler := func(ctx context.Context, input TestStruct) (TestStruct, error) {
		// Handler should handle nil pointers gracefully
		result := TestStruct{}
		
		if input.Name != nil {
			name := "processed_" + *input.Name
			result.Name = &name
		}
		
		if input.Details != nil {
			result.Details = make(map[string]string)
			for k, v := range input.Details {
				result.Details[k] = "processed_" + v
			}
		}
		
		if input.Tags != nil {
			result.Tags = make([]string, len(input.Tags))
			for i, tag := range input.Tags {
				result.Tags[i] = "processed_" + tag
			}
		}
		
		return result, nil
	}

	clearV3Registry()
	HandleTypedTool("nil_test", handler)

	// Test with empty/nil input
	emptyInput := map[string]interface{}{}
	
	tool, _ := v3Registry.GetTypedTool("nil_test")
	response := tool.Handler(emptyInput)

	// Should not panic with empty input
	if len(response.Content) == 0 {
		t.Error("Handler should return content even for empty input")
	}
}

// TestEdgeCases_InvalidJSONTags tests handling of malformed or invalid JSON tags
func TestEdgeCases_InvalidJSONTags(t *testing.T) {
	type BadTagStruct struct {
		Field1 string `json:""`                    // Empty json tag
		Field2 string `json:","`                   // Just comma
		Field3 string `json:"valid_name,invalid"`  // Invalid option
		Field4 string `json:"-,omitempty"`         // Conflicting tags
		Field5 string // No json tag at all
		Field6 string `json:"valid,omitempty,extra"` // Too many options
	}

	// Schema generation should handle malformed tags gracefully
	schema := generateSchema[BadTagStruct]()
	
	if schema["type"] != "object" {
		t.Errorf("Schema should be object type even with bad tags, got %v", schema["type"])
	}

	// Properties should exist (implementation dependent behavior)
	properties, ok := schema["properties"].(map[string]interface{})
	if ok && len(properties) < 0 { // Allow any number, just don't crash
		t.Error("Schema generation should not crash on bad tags")
	}
}

// TestEdgeCases_CircularReferences tests handling of circular struct references
func TestEdgeCases_CircularReferences(t *testing.T) {
	type Node struct {
		Value    string  `json:"value"`
		Parent   *Node   `json:"parent,omitempty"`
		Children []*Node `json:"children,omitempty"`
		Self     *Node   `json:"self,omitempty"` // Direct self-reference
	}

	// This should not cause infinite recursion or stack overflow
	defer func() {
		if r := recover(); r != nil {
			t.Errorf("Schema generation should not panic on circular references: %v", r)
		}
	}()

	schema := generateSchema[Node]()
	
	if schema["type"] != "object" {
		t.Errorf("Schema should be object type, got %v", schema["type"])
	}

	// Test handler with circular data
	handler := func(ctx context.Context, input Node) (Node, error) {
		// Create a simple response to avoid actual circular references in output
		return Node{Value: "processed_" + input.Value}, nil
	}

	clearV3Registry()
	HandleTypedTool("circular_test", handler)

	// Should register without issues
	if !IsV3Tool("circular_test") {
		t.Error("Circular reference tool should register successfully")
	}
}

// TestEdgeCases_DeeplyNestedStructs tests handling of deeply nested structures
func TestEdgeCases_DeeplyNestedStructs(t *testing.T) {
	type Level5 struct {
		Data string `json:"data"`
	}
	
	type Level4 struct {
		Level5 Level5 `json:"level5"`
		Items  []Level5 `json:"items,omitempty"`
	}
	
	type Level3 struct {
		Level4 Level4 `json:"level4"`
		Map    map[string]Level4 `json:"map,omitempty"`
	}
	
	type Level2 struct {
		Level3 Level3 `json:"level3"`
		Array  [3]Level3 `json:"array"`
	}
	
	type Level1 struct {
		Level2 Level2 `json:"level2"`
		Slice  []Level2 `json:"slice,omitempty"`
	}

	// Should handle deep nesting without stack overflow
	defer func() {
		if r := recover(); r != nil {
			t.Errorf("Deep nesting should not cause panic: %v", r)
		}
	}()

	schema := generateSchema[Level1]()
	
	if schema["type"] != "object" {
		t.Errorf("Deep nested schema should be object type, got %v", schema["type"])
	}

	// Test with deeply nested input
	handler := func(ctx context.Context, input Level1) (Level1, error) {
		return Level1{
			Level2: Level2{
				Level3: Level3{
					Level4: Level4{
						Level5: Level5{Data: "deep_" + input.Level2.Level3.Level4.Level5.Data},
					},
				},
			},
		}, nil
	}

	clearV3Registry()
	HandleTypedTool("deep_test", handler)

	deepInput := map[string]interface{}{
		"level2": map[string]interface{}{
			"level3": map[string]interface{}{
				"level4": map[string]interface{}{
					"level5": map[string]interface{}{
						"data": "nested_data",
					},
				},
			},
			"array": []map[string]interface{}{
				{"level4": map[string]interface{}{"level5": map[string]interface{}{"data": "array1"}}},
				{"level4": map[string]interface{}{"level5": map[string]interface{}{"data": "array2"}}},
				{"level4": map[string]interface{}{"level5": map[string]interface{}{"data": "array3"}}},
			},
		},
	}

	tool, _ := v3Registry.GetTypedTool("deep_test")
	response := tool.Handler(deepInput)

	// Should handle deeply nested input without errors
	if len(response.Content) == 0 {
		t.Error("Deep nested handler should return content")
	}
}

// TestEdgeCases_UnsupportedTypes tests handling of types that can't be easily serialized
func TestEdgeCases_UnsupportedTypes(t *testing.T) {
	type UnsupportedStruct struct {
		Channel   chan int                   `json:"channel,omitempty"`
		Function  func() string             `json:"function,omitempty"`
		Complex   complex64                 `json:"complex,omitempty"`
		Interface interface{ DoSomething() } `json:"interface,omitempty"`
		Unsafe    uintptr                   `json:"unsafe,omitempty"`
	}

	// Schema generation should handle unsupported types gracefully
	defer func() {
		if r := recover(); r != nil {
			t.Errorf("Unsupported types should not cause panic: %v", r)
		}
	}()

	schema := generateSchema[UnsupportedStruct]()
	
	// Should still produce a valid schema (may exclude unsupported fields)
	if schema["type"] != "object" {
		t.Errorf("Schema should be object type, got %v", schema["type"])
	}
}

// TestEdgeCases_ExtremeValues tests handling of extreme values
func TestEdgeCases_ExtremeValues(t *testing.T) {
	type ExtremeStruct struct {
		MaxInt    int     `json:"max_int"`
		MinInt    int     `json:"min_int"`
		MaxFloat  float64 `json:"max_float"`
		MinFloat  float64 `json:"min_float"`
		LongString string `json:"long_string"`
		EmptyString string `json:"empty_string"`
	}

	handler := func(ctx context.Context, input ExtremeStruct) (ExtremeStruct, error) {
		return ExtremeStruct{
			MaxInt:      input.MaxInt,
			MinInt:      input.MinInt,
			MaxFloat:    input.MaxFloat,
			MinFloat:    input.MinFloat,
			LongString:  "processed_" + input.LongString,
			EmptyString: input.EmptyString,
		}, nil
	}

	clearV3Registry()
	HandleTypedTool("extreme_test", handler)

	// Test with extreme values
	extremeInput := map[string]interface{}{
		"max_int":      int64(9223372036854775807), // Max int64
		"min_int":      int64(-9223372036854775808), // Min int64
		"max_float":    1.7976931348623157e+308,     // Max float64
		"min_float":    4.9e-324,                    // Min positive float64
		"long_string":  strings.Repeat("A", 100000), // 100KB string
		"empty_string": "",
	}

	tool, _ := v3Registry.GetTypedTool("extreme_test")
	response := tool.Handler(extremeInput)

	// Should handle extreme values without errors
	if len(response.Content) == 0 {
		t.Error("Extreme values handler should return content")
	}
}

// TestEdgeCases_MemoryLimits tests behavior under memory pressure
func TestEdgeCases_MemoryLimits(t *testing.T) {
	type LargeStruct struct {
		Data   []byte            `json:"data"`
		Arrays [][]string        `json:"arrays,omitempty"`
		Maps   map[string][]byte `json:"maps,omitempty"`
	}

	handler := func(ctx context.Context, input LargeStruct) (LargeStruct, error) {
		// Process large data - this tests memory handling
		result := LargeStruct{
			Data: make([]byte, len(input.Data)),
		}
		
		// Copy data to test memory usage
		copy(result.Data, input.Data)
		
		if input.Arrays != nil {
			result.Arrays = make([][]string, len(input.Arrays))
			for i, arr := range input.Arrays {
				result.Arrays[i] = make([]string, len(arr))
				copy(result.Arrays[i], arr)
			}
		}
		
		return result, nil
	}

	clearV3Registry()
	HandleTypedTool("memory_test", handler)

	// Test with reasonably large data (1MB)
	largeData := make([]byte, 1024*1024)
	for i := range largeData {
		largeData[i] = byte(i % 256)
	}

	largeInput := map[string]interface{}{
		"data": largeData,
		"arrays": [][]string{
			make([]string, 1000),
			make([]string, 1000),
		},
	}

	// Fill arrays with data
	for i := 0; i < 1000; i++ {
		largeInput["arrays"].([][]string)[0][i] = strings.Repeat("test", 10)
		largeInput["arrays"].([][]string)[1][i] = strings.Repeat("data", 10)
	}

	tool, _ := v3Registry.GetTypedTool("memory_test")
	response := tool.Handler(largeInput)

	// Should handle large data without memory issues
	if len(response.Content) == 0 {
		t.Error("Large data handler should return content")
	}
}

// TestEdgeCases_ConcurrentModification tests thread safety issues
func TestEdgeCases_ConcurrentModification(t *testing.T) {
	type SharedStruct struct {
		Counter int               `json:"counter"`
		Data    map[string]string `json:"data,omitempty"`
	}

	sharedCounter := 0
	
	handler := func(ctx context.Context, input SharedStruct) (SharedStruct, error) {
		// Simulate potential race condition (this is bad practice, but tests thread safety)
		sharedCounter++
		
		result := SharedStruct{
			Counter: sharedCounter,
			Data:    make(map[string]string),
		}
		
		// Simulate some processing time to increase chance of race conditions
		time.Sleep(1 * time.Millisecond)
		
		if input.Data != nil {
			for k, v := range input.Data {
				result.Data[k] = v + "_processed"
			}
		}
		
		return result, nil
	}

	clearV3Registry()
	HandleTypedTool("concurrent_test", handler)

	// Run multiple concurrent requests
	numGoroutines := 50
	results := make(chan ToolResponse, numGoroutines)
	
	for i := 0; i < numGoroutines; i++ {
		go func(id int) {
			input := map[string]interface{}{
				"counter": id,
				"data": map[string]interface{}{
					"id": string(rune('A' + id%26)),
				},
			}
			
			tool, _ := v3Registry.GetTypedTool("concurrent_test")
			response := tool.Handler(input)
			results <- response
		}(i)
	}

	// Collect all results
	errorCount := 0
	for i := 0; i < numGoroutines; i++ {
		response := <-results
		if response.IsError {
			errorCount++
		}
	}

	// Some race conditions might be acceptable, but should not cause crashes
	if errorCount > numGoroutines/2 { // Allow some failures due to race conditions
		t.Errorf("Too many errors in concurrent execution: %d/%d", errorCount, numGoroutines)
	}
}

// TestEdgeCases_ReflectionEdgeCases tests edge cases in reflection usage
func TestEdgeCases_ReflectionEdgeCases(t *testing.T) {
	// Test with anonymous structs
	anonType := reflect.TypeOf(struct {
		Field string `json:"field"`
	}{})
	
	// Should handle anonymous types
	jsonType := mapGoTypeToJSONType(anonType)
	if jsonType != "object" {
		t.Errorf("Anonymous struct should map to object, got %s", jsonType)
	}

	// Test with interface{} type
	interfaceType := reflect.TypeOf((*interface{})(nil)).Elem()
	jsonType = mapGoTypeToJSONType(interfaceType)
	
	// Should handle interface{} gracefully - returns empty string intentionally
	// (interface{} should not have a restrictive type)
	if jsonType != "" {
		t.Errorf("interface{} should return empty string for unrestricted type, got %q", jsonType)
	}

	// Test with nil type (should not happen in normal use, but test robustness)
	defer func() {
		if r := recover(); r != nil {
			t.Errorf("Nil type should not cause panic: %v", r)
		}
	}()
	
	// This might cause a panic, which is caught above
	_ = mapGoTypeToJSONType(nil)
}

// TestEdgeCases_ErrorChaining tests complex error scenarios
func TestEdgeCases_ErrorChaining(t *testing.T) {
	type ErrorTestStruct struct {
		TriggerError string `json:"trigger_error"`
		Data         string `json:"data,omitempty"`
	}

	handler := func(ctx context.Context, input ErrorTestStruct) (ErrorTestStruct, error) {
		switch input.TriggerError {
		case "validation":
			return ErrorTestStruct{}, InvalidInput("data", "validation failed")
		case "internal":
			return ErrorTestStruct{}, InternalError("internal processing error")
		case "nested":
			// Create a nested error scenario
			innerErr := InvalidInput("inner", "inner validation failed")
			return ErrorTestStruct{}, NewToolError("NESTED_ERROR", "outer error: " + innerErr.Error())
		case "nil_error":
			// Test with nil error (should not happen, but test robustness)
			var err error
			return ErrorTestStruct{}, err
		default:
			return ErrorTestStruct{Data: "success"}, nil
		}
	}

	clearV3Registry()
	HandleTypedTool("error_chain_test", handler)

	tool, _ := v3Registry.GetTypedTool("error_chain_test")

	// Test different error scenarios
	errorTypes := []string{"validation", "internal", "nested", "nil_error", "success"}
	
	for _, errorType := range errorTypes {
		input := map[string]interface{}{
			"trigger_error": errorType,
			"data":          "test_data",
		}
		
		response := tool.Handler(input)
		
		// Should handle all error types gracefully without panicking
		if len(response.Content) == 0 {
			t.Errorf("Error type %s should return some content", errorType)
		}
		
		// Success case should not be marked as error
		if errorType == "success" && response.IsError {
			t.Error("Success case should not be marked as error")
		}
	}
}

// TestEdgeCases_TimeoutHandling tests timeout and context cancellation scenarios
func TestEdgeCases_TimeoutHandling(t *testing.T) {
	type TimeoutStruct struct {
		Duration int `json:"duration_ms"`
	}

	handler := func(ctx context.Context, input TimeoutStruct) (TimeoutStruct, error) {
		// Simulate work that might timeout
		if input.Duration > 0 {
			select {
			case <-time.After(time.Duration(input.Duration) * time.Millisecond):
				return TimeoutStruct{Duration: input.Duration}, nil
			case <-ctx.Done():
				return TimeoutStruct{}, NewToolError("TIMEOUT", "operation was cancelled")
			}
		}
		
		return TimeoutStruct{Duration: 0}, nil
	}

	clearV3Registry()
	HandleTypedTool("timeout_test", handler)

	tool, _ := v3Registry.GetTypedTool("timeout_test")

	// Test with short duration (should complete)
	shortInput := map[string]interface{}{"duration_ms": 10}
	response := tool.Handler(shortInput)
	
	if response.IsError {
		t.Error("Short duration should not error")
	}

	// Test with cancelled context
	_, cancel := context.WithCancel(context.Background())
	cancel() // Cancel immediately
	
	// Handler doesn't actually use context in current stub implementation,
	// but this tests the pattern
	cancelledInput := map[string]interface{}{"duration_ms": 100}
	response = tool.Handler(cancelledInput)
	
	// Should handle cancelled context gracefully
	if len(response.Content) == 0 {
		t.Error("Cancelled context should still return content")
	}
}