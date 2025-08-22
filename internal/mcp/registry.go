package mcp

import (
	"context"
)

// ToolDefinition represents the definition of an MCP tool
type ToolDefinition struct {
	Name        string
	Description string
	Schema      map[string]interface{}
}

// Tool represents an FTL tool that can be exposed via MCP
type Tool interface {
	Definition() ToolDefinition
	Execute(ctx context.Context, args map[string]interface{}) (string, error)
}

// ToolRegistry manages all available MCP tools
type ToolRegistry struct {
	tools map[string]Tool
}

// NewToolRegistry creates a new tool registry with default FTL tools
func NewToolRegistry() *ToolRegistry {
	registry := &ToolRegistry{
		tools: make(map[string]Tool),
	}
	
	// Register default FTL tools
	registry.registerTool("ftl-init", &InitTool{})
	registry.registerTool("ftl-build", &BuildTool{})
	registry.registerTool("ftl-up", &UpTool{})
	registry.registerTool("ftl-status", &StatusTool{})
	registry.registerTool("ftl-logs", &LogsTool{})
	
	return registry
}

// registerTool adds a tool to the registry
func (r *ToolRegistry) registerTool(name string, tool Tool) {
	r.tools[name] = tool
}

// GetTool retrieves a tool by name
func (r *ToolRegistry) GetTool(name string) (Tool, bool) {
	tool, exists := r.tools[name]
	return tool, exists
}

// ListTools returns all registered tools
func (r *ToolRegistry) ListTools() []Tool {
	tools := make([]Tool, 0, len(r.tools))
	for _, tool := range r.tools {
		tools = append(tools, tool)
	}
	return tools
}