package ftl

import (
	"context"
	"testing"
	"time"
)

// TestHandleTypedTool_BasicRegistration verifies tool registers correctly
func TestHandleTypedTool_BasicRegistration(t *testing.T) {
	type SimpleInput struct {
		Message string `json:"message"`
	}
	
	type SimpleOutput struct {
		Result string `json:"result"`
	}
	
	handler := func(ctx context.Context, input SimpleInput) (SimpleOutput, error) {
		return SimpleOutput{Result: "processed: " + input.Message}, nil
	}
	
	// Clear previous registrations
	clearV3Registry()
	
	// Register the tool
	HandleTypedTool("basic_test", handler)
	
	// Verify tool is registered
	if !IsV3Tool("basic_test") {
		t.Error("Tool should be registered as V3 tool")
	}
	
	// Verify tool appears in registry
	toolNames := GetV3ToolNames()
	found := false
	for _, name := range toolNames {
		if name == "basic_test" {
			found = true
			break
		}
	}
	
	if !found {
		t.Error("Tool not found in V3 registry")
	}
	
	// Verify tool definition has correct metadata
	if tool, exists := v3Registry.GetTypedTool("basic_test"); exists {
		if tool.Meta == nil {
			t.Error("Tool should have metadata")
		} else {
			if version, ok := tool.Meta["ftl_sdk_version"].(string); !ok || version != "v3" {
				t.Error("Tool should have V3 version metadata")
			}
			if typeSafe, ok := tool.Meta["type_safe"].(bool); !ok || !typeSafe {
				t.Error("Tool should be marked as type safe")
			}
		}
	} else {
		t.Error("Tool should exist in typed registry")
	}
}

// TestHandleTypedTool_SchemaGeneration tests schema auto-generation
func TestHandleTypedTool_SchemaGeneration(t *testing.T) {
	type TestInput struct {
		Name     string  `json:"name" jsonschema:"required,description=User name"`
		Age      int     `json:"age" jsonschema:"minimum=0,maximum=120"`
		Optional string  `json:"optional,omitempty"`
		Score    float64 `json:"score" jsonschema:"minimum=0.0,maximum=100.0"`
		Active   bool    `json:"active"`
	}
	
	type TestOutput struct {
		Summary string `json:"summary"`
		Valid   bool   `json:"valid"`
	}
	
	handler := func(ctx context.Context, input TestInput) (TestOutput, error) {
		return TestOutput{
			Summary: "processed " + input.Name,
			Valid:   true,
		}, nil
	}
	
	clearV3Registry()
	
	// Register the handler
	HandleTypedTool("schema_test", handler)
	
	// Get the tool definition
	tool, exists := v3Registry.GetTypedTool("schema_test")
	if !exists {
		t.Fatal("Tool should exist in registry")
	}
	
	// Verify schema structure
	schema := tool.InputSchema
	if schema["type"] != "object" {
		t.Errorf("Expected schema type 'object', got %v", schema["type"])
	}
	
	properties, ok := schema["properties"].(map[string]interface{})
	if !ok {
		t.Fatal("Schema should have properties as map")
	}
	
	// Test individual field schemas (these will fail in CRAWL, pass in RUN)
	
	// Test required fields
	required, ok := schema["required"].([]string)
	if !ok {
		t.Error("Schema should have required fields array")
	} else {
		// Name should be required (has "required" tag)
		nameRequired := false
		for _, field := range required {
			if field == "name" {
				nameRequired = true
				break
			}
		}
		if !nameRequired {
			t.Error("Name field should be required")
		}
		
		// Optional should not be required (has "omitempty" tag)
		optionalRequired := false
		for _, field := range required {
			if field == "optional" {
				optionalRequired = true
				break
			}
		}
		if optionalRequired {
			t.Error("Optional field should not be required")
		}
	}
	
	// Test field type mappings
	nameField, ok := properties["name"].(map[string]interface{})
	if ok {
		if nameField["type"] != "string" {
			t.Errorf("Name field should be string type, got %v", nameField["type"])
		}
		if description, ok := nameField["description"].(string); !ok || description != "User name" {
			t.Errorf("Name field should have description 'User name', got %v", nameField["description"])
		}
	} else {
		t.Error("Name field should exist in schema")
	}
	
	ageField, ok := properties["age"].(map[string]interface{})
	if ok {
		if ageField["type"] != "integer" {
			t.Errorf("Age field should be integer type, got %v", ageField["type"])
		}
		if min, ok := ageField["minimum"].(int); !ok || min != 0 {
			t.Errorf("Age field should have minimum 0, got %v", ageField["minimum"])
		}
		if max, ok := ageField["maximum"].(int); !ok || max != 120 {
			t.Errorf("Age field should have maximum 120, got %v", ageField["maximum"])
		}
	} else {
		t.Error("Age field should exist in schema")
	}
	
	scoreField, ok := properties["score"].(map[string]interface{})
	if ok {
		if scoreField["type"] != "number" {
			t.Errorf("Score field should be number type, got %v", scoreField["type"])
		}
	} else {
		t.Error("Score field should exist in schema")
	}
	
	activeField, ok := properties["active"].(map[string]interface{})
	if ok {
		if activeField["type"] != "boolean" {
			t.Errorf("Active field should be boolean type, got %v", activeField["type"])
		}
	} else {
		t.Error("Active field should exist in schema")
	}
}

// TestHandleTypedTool_TypedExecution tests input/output type safety
func TestHandleTypedTool_TypedExecution(t *testing.T) {
	type MathInput struct {
		A int `json:"a" jsonschema:"required"`
		B int `json:"b" jsonschema:"required"`
	}
	
	type MathOutput struct {
		Sum     int `json:"sum"`
		Product int `json:"product"`
	}
	
	handler := func(ctx context.Context, input MathInput) (MathOutput, error) {
		return MathOutput{
			Sum:     input.A + input.B,
			Product: input.A * input.B,
		}, nil
	}
	
	clearV3Registry()
	HandleTypedTool("math_test", handler)
	
	// Test with valid input
	input := map[string]interface{}{
		"a": 5,
		"b": 3,
	}
	
	// Get the wrapped handler from the tool definition
	tool, exists := v3Registry.GetTypedTool("math_test")
	if !exists {
		t.Fatal("Tool should exist")
	}
	
	// Call the handler (this will fail in CRAWL phase as it's stubbed)
	response := tool.Handler(input)
	
	// In RUN phase, this should return proper structured response
	// For now, we test that it returns something
	if response.IsError {
		t.Errorf("Handler should not return error for valid input")
	}
	
	if len(response.Content) == 0 {
		t.Error("Handler should return content")
	}
	
	// TODO: In RUN phase, verify structured content contains:
	// - Sum: 8
	// - Product: 15
}

// TestHandleTypedTool_ErrorHandling tests error propagation
func TestHandleTypedTool_ErrorHandling(t *testing.T) {
	type ValidationInput struct {
		Value int `json:"value" jsonschema:"required,minimum=1,maximum=100"`
	}
	
	type ValidationOutput struct {
		Result string `json:"result"`
	}
	
	handler := func(ctx context.Context, input ValidationInput) (ValidationOutput, error) {
		if input.Value < 1 {
			return ValidationOutput{}, InvalidInput("value", "value must be at least 1")
		}
		if input.Value > 100 {
			return ValidationOutput{}, InvalidInput("value", "value must be at most 100")
		}
		
		return ValidationOutput{Result: "valid"}, nil
	}
	
	clearV3Registry()
	HandleTypedTool("validation_test", handler)
	
	tool, _ := v3Registry.GetTypedTool("validation_test")
	
	// Test invalid input (too low)
	lowInput := map[string]interface{}{
		"value": 0,
	}
	
	response := tool.Handler(lowInput)
	
	// In RUN phase, this should return an error response
	// For CRAWL phase, we just verify handler doesn't panic
	if response.Content == nil {
		t.Error("Handler should return some content")
	}
	
	// Test invalid input (too high)
	highInput := map[string]interface{}{
		"value": 101,
	}
	
	response = tool.Handler(highInput)
	
	// Should handle error gracefully
	if response.Content == nil {
		t.Error("Handler should return some content")
	}
	
	// Test valid input
	validInput := map[string]interface{}{
		"value": 50,
	}
	
	response = tool.Handler(validInput)
	
	if response.IsError {
		t.Error("Valid input should not produce error")
	}
}

// TestHandleTypedTool_ContextPassing tests context.Context usage
func TestHandleTypedTool_ContextPassing(t *testing.T) {
	type ContextInput struct {
		Delay int `json:"delay_ms,omitempty"`
	}
	
	type ContextOutput struct {
		Message   string `json:"message"`
		Cancelled bool   `json:"cancelled"`
	}
	
	handler := func(ctx context.Context, input ContextInput) (ContextOutput, error) {
		// Test context cancellation
		if input.Delay > 0 {
			select {
			case <-time.After(time.Duration(input.Delay) * time.Millisecond):
				return ContextOutput{Message: "completed", Cancelled: false}, nil
			case <-ctx.Done():
				return ContextOutput{Message: "cancelled", Cancelled: true}, ctx.Err()
			}
		}
		
		return ContextOutput{Message: "immediate", Cancelled: false}, nil
	}
	
	clearV3Registry()
	HandleTypedTool("context_test", handler)
	
	// Test immediate response
	input := map[string]interface{}{}
	
	tool, _ := v3Registry.GetTypedTool("context_test")
	response := tool.Handler(input)
	
	// Should not error on basic execution
	if response.IsError {
		t.Error("Basic context test should not error")
	}
	
	// TODO: In RUN phase, test actual context cancellation
	// TODO: Test context timeout behavior
	// TODO: Verify context values are passed through
}

// TestHandleTypedTool_MultipleTool tests multiple tool registration
func TestHandleTypedTool_MultipleTool(t *testing.T) {
	type Tool1Input struct {
		Message string `json:"message"`
	}
	
	type Tool1Output struct {
		Echo string `json:"echo"`
	}
	
	type Tool2Input struct {
		Number int `json:"number"`
	}
	
	type Tool2Output struct {
		Double int `json:"double"`
	}
	
	handler1 := func(ctx context.Context, input Tool1Input) (Tool1Output, error) {
		return Tool1Output{Echo: "Echo: " + input.Message}, nil
	}
	
	handler2 := func(ctx context.Context, input Tool2Input) (Tool2Output, error) {
		return Tool2Output{Double: input.Number * 2}, nil
	}
	
	clearV3Registry()
	
	// Register multiple tools
	HandleTypedTool("tool1", handler1)
	HandleTypedTool("tool2", handler2)
	
	// Verify both are registered
	if !IsV3Tool("tool1") {
		t.Error("Tool1 should be registered")
	}
	
	if !IsV3Tool("tool2") {
		t.Error("Tool2 should be registered")
	}
	
	// Verify registry contains both
	toolNames := GetV3ToolNames()
	if len(toolNames) != 2 {
		t.Errorf("Expected 2 tools, got %d", len(toolNames))
	}
	
	// Verify each tool has different schemas
	tool1, exists1 := v3Registry.GetTypedTool("tool1")
	tool2, exists2 := v3Registry.GetTypedTool("tool2")
	
	if !exists1 || !exists2 {
		t.Fatal("Both tools should exist in registry")
	}
	
	// Schemas should be different
	if tool1.InputSchema["type"] != "object" || tool2.InputSchema["type"] != "object" {
		t.Error("Both tools should have object schemas")
	}
	
	// Test that handlers are independent
	input1 := map[string]interface{}{"message": "test"}
	input2 := map[string]interface{}{"number": 5}
	
	response1 := tool1.Handler(input1)
	response2 := tool2.Handler(input2)
	
	// Should not interfere with each other
	if response1.IsError || response2.IsError {
		t.Error("Independent tools should not interfere")
	}
}

// TestHandleTypedTool_DuplicateNames tests duplicate name handling
func TestHandleTypedTool_DuplicateNames(t *testing.T) {
	type Input struct {
		Value string `json:"value"`
	}
	
	type Output struct {
		Result string `json:"result"`
	}
	
	handler1 := func(ctx context.Context, input Input) (Output, error) {
		return Output{Result: "first"}, nil
	}
	
	handler2 := func(ctx context.Context, input Input) (Output, error) {
		return Output{Result: "second"}, nil
	}
	
	clearV3Registry()
	
	// Register first tool
	HandleTypedTool("duplicate", handler1)
	
	// Register second tool with same name (should overwrite or error)
	HandleTypedTool("duplicate", handler2)
	
	// Should still only have one tool registered
	toolNames := GetV3ToolNames()
	count := 0
	for _, name := range toolNames {
		if name == "duplicate" {
			count++
		}
	}
	
	if count != 1 {
		t.Errorf("Expected exactly 1 tool named 'duplicate', got %d", count)
	}
	
	// TODO: In RUN phase, decide if we should:
	// - Overwrite the previous registration (current behavior)
	// - Return an error for duplicate names
	// - Support multiple handlers per name
}

// Helper function to clear V3 registry for testing
func clearV3Registry() {
	registeredV3ToolsMu.Lock()
	registeredV3Tools = make(map[string]bool)
	registeredV3ToolsMu.Unlock()
	v3Registry = &V3ToolRegistry{
		tools: make(map[string]TypedToolDefinition),
	}
}

// TestHandleTypedTool_ComplexTypes tests complex nested types
func TestHandleTypedTool_ComplexTypes(t *testing.T) {
	type Address struct {
		Street string `json:"street" jsonschema:"required"`
		City   string `json:"city" jsonschema:"required"`
		Zip    string `json:"zip" jsonschema:"pattern=^[0-9]{5}$"`
	}
	
	type PersonInput struct {
		Name      string    `json:"name" jsonschema:"required,description=Full name"`
		Age       int       `json:"age" jsonschema:"minimum=0,maximum=150"`
		Addresses []Address `json:"addresses,omitempty"`
		Metadata  map[string]interface{} `json:"metadata,omitempty"`
	}
	
	type PersonOutput struct {
		ID       string `json:"id"`
		Summary  string `json:"summary"`
		Valid    bool   `json:"valid"`
	}
	
	handler := func(ctx context.Context, input PersonInput) (PersonOutput, error) {
		summary := input.Name
		if input.Age > 0 {
			summary += " (age " + string(rune(input.Age)) + ")"
		}
		
		return PersonOutput{
			ID:      "person_123",
			Summary: summary,
			Valid:   input.Name != "",
		}, nil
	}
	
	clearV3Registry()
	HandleTypedTool("complex_test", handler)
	
	tool, exists := v3Registry.GetTypedTool("complex_test")
	if !exists {
		t.Fatal("Complex tool should be registered")
	}
	
	// Test schema generation for nested types
	schema := tool.InputSchema
	properties, ok := schema["properties"].(map[string]interface{})
	if !ok {
		t.Fatal("Schema should have properties")
	}
	
	// Test addresses array field
	addressesField, ok := properties["addresses"].(map[string]interface{})
	if ok {
		if addressesField["type"] != "array" {
			t.Errorf("Addresses field should be array type, got %v", addressesField["type"])
		}
		
		// TODO: In RUN phase, test items schema for nested Address type
		// TODO: Test that nested object schemas are generated correctly
	} else {
		t.Error("Addresses field should exist in schema")
	}
	
	// Test metadata map field  
	metadataField, ok := properties["metadata"].(map[string]interface{})
	if ok {
		if metadataField["type"] != "object" {
			t.Errorf("Metadata field should be object type, got %v", metadataField["type"])
		}
	} else {
		t.Error("Metadata field should exist in schema")
	}
	
	// Test handler with complex input
	complexInput := map[string]interface{}{
		"name": "John Doe",
		"age":  30,
		"addresses": []map[string]interface{}{
			{
				"street": "123 Main St",
				"city":   "New York",
				"zip":    "10001",
			},
		},
		"metadata": map[string]interface{}{
			"source": "test",
			"tags":   []string{"important"},
		},
	}
	
	response := tool.Handler(complexInput)
	
	// Should handle complex input without panicking
	if len(response.Content) == 0 {
		t.Error("Handler should return content for complex input")
	}
}