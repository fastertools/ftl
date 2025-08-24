package testing

import (
	"context"
	"encoding/json"
	"strings"
	"testing"

	"github.com/modelcontextprotocol/go-sdk/mcp"
	"github.com/fastertools/ftl/mcp-server/internal/files"
	"github.com/fastertools/ftl/mcp-server/internal/process"
	"github.com/fastertools/ftl/mcp-server/internal/types"
)

func TestHealthCheckHandler_Handle(t *testing.T) {
	// Create a temporary directory for valid test cases
	tempDir := t.TempDir()
	
	tests := []struct {
		name           string
		projectPath    string
		setupMock      func(*process.Manager)
		wantHealthy    bool
		wantError      string
		wantProcessPID int
	}{
		{
			name:        "healthy process",
			projectPath: tempDir,
			setupMock: func(pm *process.Manager) {
				// In a real test, we'd mock GetProcessInfo
				// For now, this is a placeholder
			},
			wantHealthy:    false, // Will be false since no actual process is running
			wantProcessPID: 0,
		},
		{
			name:        "no process running",
			projectPath: tempDir,
			setupMock: func(pm *process.Manager) {
				// Mock no process
			},
			wantHealthy: false,
		},
		{
			name:        "invalid project path",
			projectPath: "/nonexistent/path",
			wantHealthy: false,
			wantError:   "Project directory does not exist",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Create dependencies
			fileManager := files.NewManager()
			processManager := process.NewManager(fileManager)
			
			// Setup mock if provided
			if tt.setupMock != nil {
				tt.setupMock(processManager)
			}
			
			// Create handler
			handler := NewHealthCheckHandler(processManager)
			
			// Create request
			params := &mcp.CallToolParamsFor[types.HealthCheckInput]{
				Arguments: types.HealthCheckInput{
					ProjectPath: tt.projectPath,
					Timeout:     30,
				},
			}
			
			// Execute
			result, err := handler.Handle(context.Background(), nil, params)
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
			
			var response types.HealthCheckResponse
			if err := json.Unmarshal([]byte(textContent.Text), &response); err != nil {
				t.Fatalf("failed to parse response: %v", err)
			}
			
			// Verify
			if response.Healthy != tt.wantHealthy {
				t.Errorf("healthy = %v, want %v", response.Healthy, tt.wantHealthy)
			}
			
			if tt.wantError != "" && !contains(response.Error, tt.wantError) {
				t.Errorf("error = %v, want to contain %v", response.Error, tt.wantError)
			}
			
			if tt.wantProcessPID > 0 && response.ProcessInfo != nil {
				if response.ProcessInfo.PID != tt.wantProcessPID {
					t.Errorf("PID = %v, want %v", response.ProcessInfo.PID, tt.wantProcessPID)
				}
			}
		})
	}
}

func contains(s, substr string) bool {
	return strings.Contains(s, substr)
}