package mcpclient

import (
	"context"
	"testing"
	"time"
)

func TestNewClient(t *testing.T) {
	client := NewClient("ftl")
	
	if client.serverPath != "ftl" {
		t.Errorf("Expected serverPath 'ftl', got '%s'", client.serverPath)
	}
	
	if client.impl.Name != "ftl-web-client" {
		t.Errorf("Expected implementation name 'ftl-web-client', got '%s'", client.impl.Name)
	}
	
	if client.impl.Version != "1.0.0" {
		t.Errorf("Expected version '1.0.0', got '%s'", client.impl.Version)
	}
}

func TestNewClientWithArgs(t *testing.T) {
	args := []string{"dev", "mcp", "--test-mode"}
	client := NewClientWithArgs("ftl", args)
	
	if client.serverPath != "ftl" {
		t.Errorf("Expected serverPath 'ftl', got '%s'", client.serverPath)
	}
	
	if len(client.serverArgs) != 3 {
		t.Errorf("Expected 3 server args, got %d", len(client.serverArgs))
	}
	
	if client.serverArgs[0] != "dev" {
		t.Errorf("Expected first arg 'dev', got '%s'", client.serverArgs[0])
	}
}

func TestClientLifecycle(t *testing.T) {
	// Test client lifecycle without actual MCP server
	client := NewClient("echo") // Use 'echo' as a simple command that will fail gracefully
	
	// Test connection attempt (should fail gracefully)
	ctx, cancel := context.WithTimeout(context.Background(), 1*time.Second)
	defer cancel()
	
	err := client.Connect(ctx)
	// We expect this to fail since 'echo' isn't an MCP server
	if err == nil {
		t.Error("Expected connection to fail with 'echo' command")
	}
	
	// Test cleanup doesn't panic even if not connected
	err = client.Close()
	if err != nil {
		t.Errorf("Close should not error when not connected: %v", err)
	}
	
	// Test legacy cleanup method
	client.Cleanup() // Should not panic
}

func TestEnsureConnectionBackwardCompatibility(t *testing.T) {
	client := NewClient("echo")
	
	// Test that EnsureConnection calls Connect with timeout
	err := client.EnsureConnection()
	// Should fail gracefully since echo isn't MCP server
	if err == nil {
		t.Error("Expected EnsureConnection to fail with 'echo' command")
	}
}

func TestCallToolWithoutConnection(t *testing.T) {
	client := NewClient("nonexistent-command")
	
	// Test tool call when connection fails
	result, err := client.CallTool("test-tool", map[string]interface{}{})
	if err == nil {
		t.Error("Expected CallTool to fail with nonexistent command")
	}
	if result != "" {
		t.Errorf("Expected empty result on error, got '%s'", result)
	}
}

func TestListToolsWithoutConnection(t *testing.T) {
	client := NewClient("nonexistent-command")
	
	ctx, cancel := context.WithTimeout(context.Background(), 1*time.Second)
	defer cancel()
	
	// Test tool listing when connection fails
	tools, err := client.ListTools(ctx)
	if err == nil {
		t.Error("Expected ListTools to fail with nonexistent command")
	}
	if tools != nil {
		t.Error("Expected nil tools on error")
	}
}

// Integration test that requires actual FTL binary
func TestIntegrationWithFTL(t *testing.T) {
	if testing.Short() {
		t.Skip("Skipping integration test in short mode")
	}
	
	// This test requires the FTL binary to be built
	client := NewClientWithArgs("../../../bin/ftl", []string{"dev", "mcp", "--test-mode"})
	
	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()
	
	// Test connection
	err := client.Connect(ctx)
	if err != nil {
		t.Skipf("Skipping integration test - FTL binary not available: %v", err)
	}
	defer client.Close()
	
	// Test listing tools
	tools, err := client.ListTools(ctx)
	if err != nil {
		t.Errorf("Failed to list tools: %v", err)
		return
	}
	
	if len(tools) == 0 {
		t.Error("Expected at least one tool to be available")
	}
	
	// Test calling a simple tool (health check if available)
	for _, tool := range tools {
		if tool.Name == "mcp-server__health_check" {
			result, err := client.CallToolWithContext(ctx, "mcp-server__health_check", map[string]interface{}{})
			if err != nil {
				t.Errorf("Failed to call health check tool: %v", err)
			}
			if result == "" {
				t.Error("Expected non-empty result from health check")
			}
			break
		}
	}
}

func TestConcurrentAccess(t *testing.T) {
	client := NewClient("echo")
	
	// Test that concurrent access doesn't cause data races
	done := make(chan bool)
	
	// Start multiple goroutines trying to connect
	for i := 0; i < 10; i++ {
		go func() {
			defer func() { done <- true }()
			
			ctx, cancel := context.WithTimeout(context.Background(), 100*time.Millisecond)
			defer cancel()
			
			// These will all fail, but should not cause data races
			client.Connect(ctx)
			client.Close()
		}()
	}
	
	// Wait for all goroutines to complete
	for i := 0; i < 10; i++ {
		<-done
	}
	
	// Final cleanup should not panic
	client.Cleanup()
}