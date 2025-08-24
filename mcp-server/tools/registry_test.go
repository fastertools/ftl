package tools

import (
	"testing"

	"github.com/modelcontextprotocol/go-sdk/mcp"
)

func TestRegistry_TestMode(t *testing.T) {
	// Create a mock MCP server
	server := mcp.NewServer(&mcp.Implementation{
		Name:    "test-server",
		Version: "1.0.0",
	}, &mcp.ServerOptions{
		Instructions: "Test server",
	})
	
	// Create registry
	registry := NewRegistry(server)
	
	// Verify test mode is initially disabled
	if registry.testMode {
		t.Error("test mode should be disabled by default")
	}
	
	// Enable test mode
	registry.EnableTestMode()
	
	// Verify test mode is enabled
	if !registry.testMode {
		t.Error("test mode should be enabled after calling EnableTestMode()")
	}
	
	// Register all tools
	registry.RegisterAll()
	
	// Note: In a full test, we would verify that the test tools are registered
	// This would require either:
	// 1. Exposing a way to list registered tools from the MCP server
	// 2. Mocking the MCP server to track AddTool calls
	// 3. Testing the actual tool execution
}

func TestRegistry_Dependencies(t *testing.T) {
	// Create a mock MCP server
	server := mcp.NewServer(&mcp.Implementation{
		Name:    "test-server",
		Version: "1.0.0",
	}, &mcp.ServerOptions{
		Instructions: "Test server",
	})
	
	// Create registry
	registry := NewRegistry(server)
	
	// Verify all dependencies are initialized
	if registry.fileManager == nil {
		t.Error("fileManager should be initialized")
	}
	
	if registry.processManager == nil {
		t.Error("processManager should be initialized")
	}
	
	if registry.portManager == nil {
		t.Error("portManager should be initialized")
	}
	
	if registry.ftlCommander == nil {
		t.Error("ftlCommander should be initialized")
	}
	
	if registry.server == nil {
		t.Error("server should be set")
	}
}

func TestRegistry_RegisterAll_NoTestMode(t *testing.T) {
	// Create a mock MCP server
	server := mcp.NewServer(&mcp.Implementation{
		Name:    "test-server",
		Version: "1.0.0",
	}, &mcp.ServerOptions{
		Instructions: "Test server",
	})
	
	// Create registry without enabling test mode
	registry := NewRegistry(server)
	
	// This should register only production tools, not test tools
	registry.RegisterAll()
	
	// In production mode, only standard tools should be registered:
	// - mcp-server__up
	// - mcp-server__stop
	// - mcp-server__get_status
	// - mcp-server__build
	// - mcp-server__list_components
	// - mcp-server__get_logs
	// Test tools should NOT be registered
}

func TestRegistry_RegisterAll_WithTestMode(t *testing.T) {
	// Create a mock MCP server
	server := mcp.NewServer(&mcp.Implementation{
		Name:    "test-server",
		Version: "1.0.0",
	}, &mcp.ServerOptions{
		Instructions: "Test server",
	})
	
	// Create registry and enable test mode
	registry := NewRegistry(server)
	registry.EnableTestMode()
	
	// This should register both production and test tools
	registry.RegisterAll()
	
	// In test mode, both standard tools and test tools should be registered:
	// Standard tools:
	// - mcp-server__up
	// - mcp-server__stop
	// - mcp-server__get_status
	// - mcp-server__build
	// - mcp-server__list_components
	// - mcp-server__get_logs
	// Test tools:
	// - mcp-server__health_check
	// - mcp-server__process_info
	// - mcp-server__port_finder
	// - mcp-server__wait_ready
}