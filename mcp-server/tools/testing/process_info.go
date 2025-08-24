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

// ProcessInfoHandler handles detailed process info queries
type ProcessInfoHandler struct {
	processManager *process.Manager
}

// NewProcessInfoHandler creates a new process info handler
func NewProcessInfoHandler(processManager *process.Manager) *ProcessInfoHandler {
	return &ProcessInfoHandler{
		processManager: processManager,
	}
}

// Handle processes the process info request
func (h *ProcessInfoHandler) Handle(ctx context.Context, ss *mcp.ServerSession, params *mcp.CallToolParamsFor[types.ProcessInfoInput]) (*mcp.CallToolResultFor[struct{}], error) {
	projectPath := params.Arguments.ProjectPath
	includeChildren := params.Arguments.IncludeChildren
	
	// Validate project path exists
	if err := ftl.ValidateProjectPath(projectPath); err != nil {
		response := types.ProcessTreeResponse{
			ProjectPath: projectPath,
			Error:       fmt.Sprintf("Project directory does not exist: %s", projectPath),
		}
		responseJSON, _ := json.Marshal(response)
		return &mcp.CallToolResultFor[struct{}]{
			Content: []mcp.Content{&mcp.TextContent{Text: string(responseJSON)}},
		}, nil
	}
	
	// Get process information
	pid, port, mode, isRunning := h.processManager.GetProcessInfo(projectPath)
	
	// Build response
	response := types.ProcessTreeResponse{
		ProjectPath: projectPath,
	}
	
	if isRunning {
		// Find deepest child for accurate process tree
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
		
		// Include child PIDs if requested
		if includeChildren && deepestPID != pid {
			// Build a simple child list (in real implementation, would walk the process tree)
			// For now, just include the deepest PID as a child
			response.Children = []int{deepestPID}
		}
	}
	
	responseJSON, _ := json.Marshal(response)
	return &mcp.CallToolResultFor[struct{}]{
		Content: []mcp.Content{&mcp.TextContent{Text: string(responseJSON)}},
	}, nil
}