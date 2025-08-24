package testing

import (
	"context"
	"encoding/json"
	"fmt"

	"github.com/modelcontextprotocol/go-sdk/mcp"
	"github.com/fastertools/ftl/mcp-server/internal/ftl"
	"github.com/fastertools/ftl/mcp-server/internal/process"
	"github.com/fastertools/ftl/mcp-server/internal/types"
)

// HealthCheckHandler handles health check queries for FTL processes
type HealthCheckHandler struct {
	processManager *process.Manager
}

// NewHealthCheckHandler creates a new health check handler
func NewHealthCheckHandler(processManager *process.Manager) *HealthCheckHandler {
	return &HealthCheckHandler{
		processManager: processManager,
	}
}

// Handle processes the health check request
func (h *HealthCheckHandler) Handle(ctx context.Context, ss *mcp.ServerSession, params *mcp.CallToolParamsFor[types.HealthCheckInput]) (*mcp.CallToolResultFor[struct{}], error) {
	projectPath := params.Arguments.ProjectPath
	
	// Validate project path exists
	if err := ftl.ValidateProjectPath(projectPath); err != nil {
		response := types.HealthCheckResponse{
			ProjectPath: projectPath,
			Healthy:     false,
			Error:       fmt.Sprintf("Project directory does not exist: %s", projectPath),
		}
		responseJSON, _ := json.Marshal(response)
		return &mcp.CallToolResultFor[struct{}]{
			Content: []mcp.Content{&mcp.TextContent{Text: string(responseJSON)}},
		}, nil
	}
	
	// Get process information using the process manager
	pid, port, mode, isRunning := h.processManager.GetProcessInfo(projectPath)
	
	// Build response
	response := types.HealthCheckResponse{
		ProjectPath: projectPath,
		Healthy:     isRunning,
	}
	
	if isRunning {
		// Find deepest child for more accurate process info
		deepestPID := h.processManager.FindDeepestChild(pid)
		if deepestPID == 0 {
			deepestPID = pid
		}
		
		response.ProcessInfo = &types.ProcessInfo{
			PID:        pid,
			Port:       port,
			IsRunning:  true,
			Type:       mode,
			DeepestPID: deepestPID,
		}
	}
	
	responseJSON, _ := json.Marshal(response)
	return &mcp.CallToolResultFor[struct{}]{
		Content: []mcp.Content{&mcp.TextContent{Text: string(responseJSON)}},
	}, nil
}