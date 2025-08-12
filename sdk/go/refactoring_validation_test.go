//go:build !test

package ftl

import (
	"testing"
)

// TestRefactoredFunctionality validates that the refactored code maintains compatibility
func TestRefactoredFunctionality(t *testing.T) {
	// Test validateAndCopyTools function
	t.Run("ValidateAndCopyTools", func(t *testing.T) {
		// Test with nil input
		result := validateAndCopyTools(nil)
		if result == nil {
			t.Error("Expected non-nil result for nil input")
		}
		if len(result) != 0 {
			t.Error("Expected empty map for nil input")
		}
		
		// Test with valid tools
		input := map[string]ToolDefinition{
			"test_tool": {
				Description: "Test tool",
				Handler: func(input map[string]interface{}) ToolResponse {
					return Text("test")
				},
			},
			"": { // Invalid empty key should be filtered out
				Description: "Invalid tool",
			},
		}
		
		result = validateAndCopyTools(input)
		if len(result) != 1 {
			t.Errorf("Expected 1 tool, got %d", len(result))
		}
		
		if _, exists := result["test_tool"]; !exists {
			t.Error("Expected test_tool to exist in result")
		}
		
		if _, exists := result[""]; exists {
			t.Error("Expected empty key to be filtered out")
		}
	})
	
	// Test buildToolMetadata function
	t.Run("BuildToolMetadata", func(t *testing.T) {
		tools := map[string]ToolDefinition{
			"test_tool": {
				Description: "Test tool",
				Handler: func(input map[string]interface{}) ToolResponse {
					return Text("test")
				},
			},
		}
		
		metadata := buildToolMetadata(tools)
		if len(metadata) != 1 {
			t.Errorf("Expected 1 metadata entry, got %d", len(metadata))
		}
		
		if metadata[0].Name != "test_tool" {
			t.Errorf("Expected name 'test_tool', got '%s'", metadata[0].Name)
		}
		
		if metadata[0].Description != "Test tool" {
			t.Errorf("Expected description 'Test tool', got '%s'", metadata[0].Description)
		}
		
		// Verify default input schema is set
		if metadata[0].InputSchema == nil {
			t.Error("Expected non-nil InputSchema")
		}
	})
	
	// Test findToolByName function
	t.Run("FindToolByName", func(t *testing.T) {
		tools := map[string]ToolDefinition{
			"TestTool": { // CamelCase key
				Name: "test_tool", // Explicit snake_case name
				Description: "Test tool",
			},
			"another_tool": { // No explicit name
				Description: "Another tool",
			},
		}
		
		// Should find by explicit name
		tool := findToolByName(tools, "test_tool")
		if tool == nil {
			t.Error("Expected to find tool by explicit name")
		}
		
		// Should find by converted key name
		tool = findToolByName(tools, "another_tool")
		if tool == nil {
			t.Error("Expected to find tool by converted key name")
		}
		
		// Should not find non-existent tool
		tool = findToolByName(tools, "nonexistent")
		if tool != nil {
			t.Error("Expected not to find non-existent tool")
		}
	})
}

// TestV3RegistryConsolidation validates the consolidated registry system
func TestV3RegistryConsolidation(t *testing.T) {
	// Clear the registry for clean test
	v3Registry.tools = make(map[string]TypedToolDefinition)
	
	// Test V3 tool registration
	definition := ToolDefinition{
		Description: "Test V3 tool",
		Handler: func(input map[string]interface{}) ToolResponse {
			return Text("v3 test")
		},
		Meta: map[string]interface{}{
			"ftl_sdk_version": "v3",
		},
	}
	
	registerV3Tool("test_v3_tool", definition)
	
	// Verify tool was registered in V3 registry
	if !IsV3Tool("test_v3_tool") {
		t.Error("Expected tool to be registered as V3 tool")
	}
	
	// Verify tool can be retrieved
	tool, exists := v3Registry.GetTypedTool("test_v3_tool")
	if !exists {
		t.Error("Expected tool to exist in V3 registry")
	}
	
	if tool.Description != "Test V3 tool" {
		t.Errorf("Expected description 'Test V3 tool', got '%s'", tool.Description)
	}
	
	// Test GetV3ToolNames
	names := GetV3ToolNames()
	found := false
	for _, name := range names {
		if name == "test_v3_tool" {
			found = true
			break
		}
	}
	if !found {
		t.Error("Expected test_v3_tool in GetV3ToolNames result")
	}
}