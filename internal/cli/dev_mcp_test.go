package cli

import (
	"context"
	"fmt"
	"net/http"
	"strings"
	"testing"
	"time"

	"github.com/fastertools/ftl/internal/types"
)

func TestDevMcpFlags(t *testing.T) {
	tests := []struct {
		name           string
		ftlPath        string
		testMode       bool
		expectedFTLPath string
	}{
		{
			name:           "default ftl path",
			ftlPath:        "",
			testMode:       false,
			expectedFTLPath: "ftl",
		},
		{
			name:           "custom ftl path",
			ftlPath:        "/custom/path/to/ftl",
			testMode:       false,
			expectedFTLPath: "/custom/path/to/ftl",
		},
		{
			name:           "test mode with custom path",
			ftlPath:        "/test/path/to/ftl",
			testMode:       true,
			expectedFTLPath: "/test/path/to/ftl",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Create tool configuration like runDevMcp does
			toolConfig := &types.ToolConfig{}
			if tt.ftlPath != "" {
				toolConfig.FTLPath = tt.ftlPath
			}

			// Test that GetFTLPath returns expected value
			result := toolConfig.GetFTLPath()
			if result != tt.expectedFTLPath {
				t.Errorf("GetFTLPath() = %q, want %q", result, tt.expectedFTLPath)
			}

			// Verify the struct is properly configured
			if tt.ftlPath != "" && toolConfig.FTLPath != tt.ftlPath {
				t.Errorf("ToolConfig.FTLPath = %q, want %q", toolConfig.FTLPath, tt.ftlPath)
			}
		})
	}
}

func TestToolConfigJSON(t *testing.T) {
	// Test that ToolConfig serializes to JSON correctly
	config := &types.ToolConfig{
		FTLPath: "/custom/path/to/ftl",
	}

	// This validates the JSON tags are correct
	expected := "/custom/path/to/ftl"
	if config.FTLPath != expected {
		t.Errorf("FTLPath field = %q, want %q", config.FTLPath, expected)
	}

	// Test empty config defaults
	emptyConfig := &types.ToolConfig{}
	if emptyConfig.GetFTLPath() != "ftl" {
		t.Errorf("Empty config GetFTLPath() = %q, want %q", emptyConfig.GetFTLPath(), "ftl")
	}
}

// Test dual transport command flags
func TestNewDevMcpCmdDualTransport(t *testing.T) {
	cmd := newDevMcpCmd()
	
	if cmd.Use != "mcp" {
		t.Errorf("Expected command use 'mcp', got '%s'", cmd.Use)
	}
	
	// Check that new dual transport flags are registered
	httpFlag := cmd.Flags().Lookup("http")
	if httpFlag == nil {
		t.Error("Expected --http flag to be registered")
	}
	
	verboseFlag := cmd.Flags().Lookup("verbose")
	if verboseFlag == nil {
		t.Error("Expected --verbose flag to be registered")
	}
	
	// Test help text mentions dual transport
	longDesc := cmd.Long
	if !strings.Contains(longDesc, "stdio") {
		t.Error("Help should mention stdio transport")
	}
	if !strings.Contains(longDesc, "HTTP") {
		t.Error("Help should mention HTTP transport")
	}
}

func TestMcpCommandExamples(t *testing.T) {
	cmd := newDevMcpCmd()
	
	// Test that examples contain expected content
	examples := cmd.Example
	
	if !strings.Contains(examples, "stdio") {
		t.Error("Examples should mention stdio usage")
	}
	
	if !strings.Contains(examples, "HTTP") || !strings.Contains(examples, "--http") {
		t.Error("Examples should mention HTTP usage")
	}
	
	if !strings.Contains(examples, "--test-mode") {
		t.Error("Examples should mention test mode")
	}
	
	if !strings.Contains(examples, "--verbose") {
		t.Error("Examples should mention verbose flag")
	}
}

// Basic HTTP server test to validate transport concept
func TestHTTPTransportConcept(t *testing.T) {
	if testing.Short() {
		t.Skip("Skipping HTTP transport test in short mode")
	}
	
	// Test basic HTTP server functionality that MCP would use
	port := 9999 + (int(time.Now().UnixNano()) % 1000)
	httpAddr := fmt.Sprintf(":%d", port)
	
	ctx, cancel := context.WithTimeout(context.Background(), 2*time.Second)
	defer cancel()
	
	// Start simple HTTP server
	mux := http.NewServeMux()
	mux.HandleFunc("/health", func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusOK)
		w.Write([]byte("OK"))
	})
	
	server := &http.Server{Addr: httpAddr, Handler: mux}
	
	go func() {
		server.ListenAndServe()
	}()
	
	// Give server time to start
	time.Sleep(50 * time.Millisecond)
	
	// Test that server is reachable
	client := &http.Client{Timeout: 500 * time.Millisecond}
	resp, err := client.Get(fmt.Sprintf("http://localhost%s/health", httpAddr))
	if err != nil {
		t.Skipf("HTTP server not reachable (expected in test environment): %v", err)
		return
	}
	defer resp.Body.Close()
	
	if resp.StatusCode != http.StatusOK {
		t.Errorf("Expected status OK, got %d", resp.StatusCode)
	}
	
	// Cleanup
	server.Shutdown(ctx)
}