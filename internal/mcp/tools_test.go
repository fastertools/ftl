package mcp

import (
	"context"
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestInitTool(t *testing.T) {
	tool := &InitTool{}
	
	// Test definition
	def := tool.Definition()
	assert.Equal(t, "ftl-init", def.Name)
	assert.Equal(t, "Initialize a new FTL project", def.Description)
	assert.NotNil(t, def.Schema)
	
	// Test execution with missing name (should fail)
	ctx := context.Background()
	args := map[string]interface{}{}
	
	result, err := tool.Execute(ctx, args)
	assert.Error(t, err)
	assert.Empty(t, result)
	
	// Test execution with valid args (will fail due to no ftl command, but should handle gracefully)
	args = map[string]interface{}{
		"name":     "test-project",
		"template": "rust",
	}
	
	result, err = tool.Execute(ctx, args)
	// Expect error since ftl command doesn't exist in test environment
	assert.Error(t, err)
	assert.Empty(t, result)
}

func TestBuildTool(t *testing.T) {
	tool := &BuildTool{}
	
	def := tool.Definition()
	assert.Equal(t, "ftl-build", def.Name)
	assert.Equal(t, "Build the current FTL project", def.Description)
	assert.NotNil(t, def.Schema)
	
	// Test execution
	ctx := context.Background()
	args := map[string]interface{}{
		"watch": true,
	}
	
	result, err := tool.Execute(ctx, args)
	// Expect error since ftl command doesn't exist in test environment
	assert.Error(t, err)
	assert.Empty(t, result)
}

func TestUpTool(t *testing.T) {
	tool := &UpTool{}
	
	def := tool.Definition()
	assert.Equal(t, "ftl-up", def.Name)
	assert.Equal(t, "Start the FTL development server", def.Description)
	assert.NotNil(t, def.Schema)
	
	// Test execution
	ctx := context.Background()
	args := map[string]interface{}{
		"watch": false,
	}
	
	result, err := tool.Execute(ctx, args)
	// Expect error since ftl command doesn't exist in test environment
	assert.Error(t, err)
	assert.Empty(t, result)
}

func TestStatusTool(t *testing.T) {
	tool := &StatusTool{}
	
	def := tool.Definition()
	assert.Equal(t, "ftl-status", def.Name)
	assert.Equal(t, "Get the current status of FTL applications", def.Description)
	assert.NotNil(t, def.Schema)
	
	// Test execution
	ctx := context.Background()
	args := map[string]interface{}{}
	
	result, err := tool.Execute(ctx, args)
	// Expect error since ftl command doesn't exist in test environment
	assert.Error(t, err)
	assert.Empty(t, result)
}

func TestLogsTool(t *testing.T) {
	tool := &LogsTool{}
	
	def := tool.Definition()
	assert.Equal(t, "ftl-logs", def.Name)
	assert.Equal(t, "Get logs from FTL applications", def.Description)
	assert.NotNil(t, def.Schema)
	
	// Test execution
	ctx := context.Background()
	args := map[string]interface{}{
		"follow": false,
		"lines":  float64(20), // JSON numbers come as float64
	}
	
	result, err := tool.Execute(ctx, args)
	// Expect error since ftl command doesn't exist in test environment
	assert.Error(t, err)
	assert.Empty(t, result)
}