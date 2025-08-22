package mcp

import (
	"context"
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestNewToolRegistry(t *testing.T) {
	registry := NewToolRegistry()
	
	assert.NotNil(t, registry)
	assert.NotNil(t, registry.tools)
	assert.Equal(t, 5, len(registry.tools))
	
	// Check all expected tools are registered
	expectedTools := []string{
		"ftl-init",
		"ftl-build", 
		"ftl-up",
		"ftl-status",
		"ftl-logs",
	}
	
	for _, toolName := range expectedTools {
		tool, exists := registry.GetTool(toolName)
		assert.True(t, exists, "Tool %s should exist", toolName)
		assert.NotNil(t, tool)
		
		// Test tool definition
		def := tool.Definition()
		assert.Equal(t, toolName, def.Name)
		assert.NotEmpty(t, def.Description)
		assert.NotNil(t, def.Schema)
	}
}

func TestToolRegistryGetTool(t *testing.T) {
	registry := NewToolRegistry()
	
	// Test getting existing tool
	tool, exists := registry.GetTool("ftl-init")
	assert.True(t, exists)
	assert.NotNil(t, tool)
	
	// Test getting non-existent tool
	tool, exists = registry.GetTool("non-existent")
	assert.False(t, exists)
	assert.Nil(t, tool)
}

func TestToolRegistryListTools(t *testing.T) {
	registry := NewToolRegistry()
	
	tools := registry.ListTools()
	assert.Equal(t, 5, len(tools))
	
	// Verify all tools implement the Tool interface correctly
	for _, tool := range tools {
		def := tool.Definition()
		assert.NotEmpty(t, def.Name)
		assert.NotEmpty(t, def.Description)
		assert.NotNil(t, def.Schema)
		
		// Test execution (will fail due to no ftl command, but should not panic)
		ctx := context.Background()
		args := map[string]interface{}{}
		
		_, err := tool.Execute(ctx, args)
		// We expect errors since ftl command doesn't exist, but no panics
		assert.Error(t, err)
	}
}

func TestToolRegistryRegisterTool(t *testing.T) {
	registry := &ToolRegistry{
		tools: make(map[string]Tool),
	}
	
	// Test registering a new tool
	mockTool := &InitTool{}
	registry.registerTool("test-tool", mockTool)
	
	tool, exists := registry.GetTool("test-tool")
	assert.True(t, exists)
	assert.Equal(t, mockTool, tool)
	
	// Test overwriting existing tool
	newMockTool := &BuildTool{}
	registry.registerTool("test-tool", newMockTool)
	
	tool, exists = registry.GetTool("test-tool")
	assert.True(t, exists)
	assert.Equal(t, newMockTool, tool)
}