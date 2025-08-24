package testing

import (
	"context"
	"encoding/json"
	"fmt"
	"time"

	"github.com/modelcontextprotocol/go-sdk/mcp"
	"github.com/fastertools/ftl/mcp-server/internal/ftl"
	"github.com/fastertools/ftl/mcp-server/internal/process"
	"github.com/fastertools/ftl/mcp-server/internal/types"
)

// WaitReadyHandler handles wait-for-ready requests with polling
type WaitReadyHandler struct {
	processManager *process.Manager
}

// NewWaitReadyHandler creates a new wait ready handler
func NewWaitReadyHandler(processManager *process.Manager) *WaitReadyHandler {
	return &WaitReadyHandler{
		processManager: processManager,
	}
}

// Handle processes the wait ready request with polling
func (h *WaitReadyHandler) Handle(ctx context.Context, ss *mcp.ServerSession, params *mcp.CallToolParamsFor[types.WaitReadyInput]) (*mcp.CallToolResultFor[struct{}], error) {
	projectPath := params.Arguments.ProjectPath
	timeoutSec := params.Arguments.TimeoutSec
	intervalSec := params.Arguments.IntervalSec
	maxAttempts := params.Arguments.MaxAttempts
	
	// Set defaults if not provided
	if timeoutSec == 0 {
		timeoutSec = 30
	}
	if intervalSec == 0 {
		intervalSec = 1
	}
	if maxAttempts == 0 {
		maxAttempts = 30
	}
	
	// Validate project path exists
	if err := ftl.ValidateProjectPath(projectPath); err != nil {
		response := types.WaitReadyResponse{
			ProjectPath: projectPath,
			Ready:       false,
			Attempts:    0,
			Error:       fmt.Sprintf("Project directory does not exist: %s", projectPath),
		}
		responseJSON, _ := json.Marshal(response)
		return &mcp.CallToolResultFor[struct{}]{
			Content: []mcp.Content{&mcp.TextContent{Text: string(responseJSON)}},
		}, nil
	}
	
	// Set up timeout context
	ctxWithTimeout, cancel := context.WithTimeout(ctx, time.Duration(timeoutSec)*time.Second)
	defer cancel()
	
	// Poll for process readiness
	attempts := 0
	ticker := time.NewTicker(time.Duration(intervalSec) * time.Second)
	defer ticker.Stop()
	
	for {
		attempts++
		
		// Check process status
		pid, port, mode, isRunning := h.processManager.GetProcessInfo(projectPath)
		
		if isRunning {
			// Process is running and ready
			deepestPID := h.processManager.FindDeepestChild(pid)
			if deepestPID == 0 {
				deepestPID = pid
			}
			
			response := types.WaitReadyResponse{
				ProjectPath: projectPath,
				Ready:       true,
				Attempts:    attempts,
				ProcessInfo: &types.ProcessInfo{
					PID:        pid,
					Port:       port,
					IsRunning:  true,
					Type:       mode,
					DeepestPID: deepestPID,
				},
			}
			responseJSON, _ := json.Marshal(response)
			return &mcp.CallToolResultFor[struct{}]{
				Content: []mcp.Content{&mcp.TextContent{Text: string(responseJSON)}},
			}, nil
		}
		
		// Check if we've exceeded max attempts
		if attempts >= maxAttempts {
			response := types.WaitReadyResponse{
				ProjectPath: projectPath,
				Ready:       false,
				Attempts:    attempts,
				Error:       fmt.Sprintf("Process not ready after %d attempts", maxAttempts),
			}
			responseJSON, _ := json.Marshal(response)
			return &mcp.CallToolResultFor[struct{}]{
				Content: []mcp.Content{&mcp.TextContent{Text: string(responseJSON)}},
			}, nil
		}
		
		// Wait for next interval or timeout
		select {
		case <-ctxWithTimeout.Done():
			response := types.WaitReadyResponse{
				ProjectPath: projectPath,
				Ready:       false,
				Attempts:    attempts,
				Error:       fmt.Sprintf("Timeout after %d seconds", timeoutSec),
			}
			responseJSON, _ := json.Marshal(response)
			return &mcp.CallToolResultFor[struct{}]{
				Content: []mcp.Content{&mcp.TextContent{Text: string(responseJSON)}},
			}, nil
		case <-ticker.C:
			// Continue polling
		}
	}
}