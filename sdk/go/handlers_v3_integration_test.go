//go:build test

package ftl

import (
	"context"
	"testing"
)

// TestHandlerExecutionIntegration tests the complete handler execution flow
func TestHandlerExecutionIntegration(t *testing.T) {
	// Clear registry for clean test
	v3Registry = &V3ToolRegistry{
		tools: make(map[string]TypedToolDefinition),
	}
	registeredV3ToolsMu.Lock()
	registeredV3Tools = make(map[string]bool)
	registeredV3ToolsMu.Unlock()
	
	// Define test input/output types
	type TestInput struct {
		Message string `json:"message" jsonschema:"required,description=Test message"`
		Count   int    `json:"count,omitempty" jsonschema:"minimum=1,maximum=5"`
	}
	
	type TestOutput struct {
		Response string `json:"response"`
		Total    int    `json:"total"`
	}
	
	// Create a test handler
	testHandler := func(ctx context.Context, input TestInput) (TestOutput, error) {
		// Validate input
		if input.Message == "" {
			return TestOutput{}, InvalidInput("message", "message is required")
		}
		
		// Default count to 1
		count := input.Count
		if count <= 0 {
			count = 1
		}
		
		// Build response
		response := ""
		for i := 0; i < count; i++ {
			if i > 0 {
				response += " "
			}
			response += input.Message
		}
		
		return TestOutput{
			Response: response,
			Total:    count,
		}, nil
	}
	
	// Register the handler
	HandleTypedTool("test_tool", testHandler)
	
	// Verify registration
	if !IsV3Tool("test_tool") {
		t.Fatal("Tool should be registered as V3 tool")
	}
	
	// Get the registered tool
	toolDef, exists := v3Registry.GetTypedTool("test_tool")
	if !exists {
		t.Fatal("Tool should exist in registry")
	}
	
	// Test successful execution
	t.Run("successful_execution", func(t *testing.T) {
		input := map[string]interface{}{
			"message": "hello",
			"count":   3,
		}
		
		response := toolDef.Handler(input)
		
		if response.IsError {
			t.Fatalf("Expected success, got error: %v", response.Content)
		}
		
		// Response should contain structured data for complex output type
		if len(response.Content) == 0 {
			t.Fatal("Expected content in response")
		}
	})
	
	// Test validation error
	t.Run("validation_error", func(t *testing.T) {
		input := map[string]interface{}{
			"message": "",
			"count":   1,
		}
		
		response := toolDef.Handler(input)
		
		if !response.IsError {
			t.Fatal("Expected validation error for empty message")
		}
	})
	
	// Test with missing optional field
	t.Run("missing_optional_field", func(t *testing.T) {
		input := map[string]interface{}{
			"message": "test",
			// count is omitted
		}
		
		response := toolDef.Handler(input)
		
		if response.IsError {
			t.Fatalf("Should handle missing optional field, got error: %v", response.Content)
		}
	})
	
	// Test schema generation
	t.Run("schema_generation", func(t *testing.T) {
		schema := toolDef.InputSchema
		if schema == nil {
			t.Fatal("Expected schema to be generated")
		}
		
		// Check schema has correct type
		if schemaType, ok := schema["type"].(string); !ok || schemaType != "object" {
			t.Errorf("Expected schema type to be 'object', got %v", schema["type"])
		}
		
		// Check required fields
		if required, ok := schema["required"].([]string); ok {
			found := false
			for _, field := range required {
				if field == "message" {
					found = true
					break
				}
			}
			if !found {
				t.Error("Expected 'message' to be in required fields")
			}
		} else {
			t.Error("Expected 'required' field in schema")
		}
	})
}

// TestMultipleHandlers tests registering multiple V3 handlers
func TestMultipleHandlers(t *testing.T) {
	// Clear registry
	v3Registry = &V3ToolRegistry{
		tools: make(map[string]TypedToolDefinition),
	}
	registeredV3ToolsMu.Lock()
	registeredV3Tools = make(map[string]bool)
	registeredV3ToolsMu.Unlock()
	
	// Define input/output structs for handler1
	type Handler1Input struct {
		Message string `json:"message" jsonschema:"required,description=Input message"`
	}
	type Handler1Output struct {
		Response string `json:"response"`
	}
	
	// Define input/output structs for handler2
	type Handler2Input struct {
		Number int `json:"number" jsonschema:"required,description=Input number"`
	}
	type Handler2Output struct {
		Result int `json:"result"`
	}
	
	// Register first handler with struct types
	HandleTypedTool("handler1", func(ctx context.Context, input Handler1Input) (Handler1Output, error) {
		return Handler1Output{Response: "response1: " + input.Message}, nil
	})
	
	// Register second handler with struct types
	HandleTypedTool("handler2", func(ctx context.Context, input Handler2Input) (Handler2Output, error) {
		return Handler2Output{Result: input.Number * 2}, nil
	})
	
	// Both should be registered
	if !IsV3Tool("handler1") || !IsV3Tool("handler2") {
		t.Error("Both handlers should be registered")
	}
	
	// Test that both work independently
	tool1, ok1 := v3Registry.GetTypedTool("handler1")
	if !ok1 {
		t.Fatal("handler1 not found in registry")
	}
	response1 := tool1.Handler(map[string]interface{}{"message": "test"})
	
	tool2, ok2 := v3Registry.GetTypedTool("handler2")
	if !ok2 {
		t.Fatal("handler2 not found in registry")
	}
	response2 := tool2.Handler(map[string]interface{}{"number": 5})
	
	// Both should execute (even if stubbed)
	if response1.IsError {
		t.Errorf("Handler1 returned error: %v", response1.Content)
	}
	if response2.IsError {
		t.Errorf("Handler2 returned error: %v", response2.Content)
	}
}