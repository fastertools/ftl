package testing

import (
	"context"
	"encoding/json"
	"testing"
	"time"

	"github.com/modelcontextprotocol/go-sdk/mcp"
	"github.com/fastertools/ftl/mcp-server/internal/files"
	"github.com/fastertools/ftl/mcp-server/internal/process"
	"github.com/fastertools/ftl/mcp-server/internal/types"
)

func TestWaitReadyHandler_Handle(t *testing.T) {
	// Create a temporary directory for valid test cases
	tempDir := t.TempDir()
	
	tests := []struct {
		name        string
		projectPath string
		timeoutSec  int
		intervalSec int
		maxAttempts int
		mockReady   bool
		wantReady   bool
		wantError   string
	}{
		{
			name:        "process not ready - timeout",
			projectPath: tempDir,
			timeoutSec:  1,
			intervalSec: 1,
			maxAttempts: 10,
			mockReady:   false,
			wantReady:   false,
			wantError:   "Timeout after 1 seconds",
		},
		{
			name:        "process not ready - max attempts",
			projectPath: tempDir,
			timeoutSec:  10,
			intervalSec: 1,
			maxAttempts: 2,
			mockReady:   false,
			wantReady:   false,
			wantError:   "Process not ready after 2 attempts",
		},
		{
			name:        "invalid project path",
			projectPath: "/nonexistent/path",
			wantReady:   false,
			wantError:   "Project directory does not exist",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Create dependencies
			fileManager := files.NewManager()
			processManager := process.NewManager(fileManager)
			
			// Create handler
			handler := NewWaitReadyHandler(processManager)
			
			// Create request
			params := &mcp.CallToolParamsFor[types.WaitReadyInput]{
				Arguments: types.WaitReadyInput{
					ProjectPath: tt.projectPath,
					TimeoutSec:  tt.timeoutSec,
					IntervalSec: tt.intervalSec,
					MaxAttempts: tt.maxAttempts,
				},
			}
			
			// Execute with timeout to prevent hanging tests
			ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
			defer cancel()
			
			result, err := handler.Handle(ctx, nil, params)
			if err != nil {
				t.Fatalf("unexpected error: %v", err)
			}
			
			// Parse response
			if len(result.Content) == 0 {
				t.Fatal("no content in response")
			}
			
			textContent, ok := result.Content[0].(*mcp.TextContent)
			if !ok {
				t.Fatal("expected TextContent")
			}
			
			var response types.WaitReadyResponse
			if err := json.Unmarshal([]byte(textContent.Text), &response); err != nil {
				t.Fatalf("failed to parse response: %v", err)
			}
			
			// Verify
			if response.Ready != tt.wantReady {
				t.Errorf("ready = %v, want %v", response.Ready, tt.wantReady)
			}
			
			if tt.wantError != "" && !contains(response.Error, tt.wantError) {
				t.Errorf("error = %v, want to contain %v", response.Error, tt.wantError)
			}
			
			if response.Attempts == 0 && tt.wantError != "Project directory does not exist" {
				t.Error("expected at least 1 attempt")
			}
		})
	}
}

func TestWaitReadyHandler_DefaultValues(t *testing.T) {
	// Create a temporary directory for testing
	tempDir := t.TempDir()
	
	// Test that default values are applied correctly
	fileManager := files.NewManager()
	processManager := process.NewManager(fileManager)
	handler := NewWaitReadyHandler(processManager)
	
	params := &mcp.CallToolParamsFor[types.WaitReadyInput]{
		Arguments: types.WaitReadyInput{
			ProjectPath: tempDir,
			// Leave all timing fields as zero to test defaults
		},
	}
	
	ctx, cancel := context.WithTimeout(context.Background(), 2*time.Second)
	defer cancel()
	
	result, err := handler.Handle(ctx, nil, params)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	
	textContent := result.Content[0].(*mcp.TextContent)
	var response types.WaitReadyResponse
	json.Unmarshal([]byte(textContent.Text), &response)
	
	// Should have made at least one attempt with defaults
	if response.Attempts == 0 {
		t.Error("expected at least 1 attempt with default values")
	}
}

// Using the contains function from health_check_test.go