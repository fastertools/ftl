package lifecycle

import (
	"context"
	"fmt"
	"os"

	"github.com/modelcontextprotocol/go-sdk/mcp"
	"github.com/fastertools/ftl/mcp-server/internal/files"
	"github.com/fastertools/ftl/mcp-server/internal/ftl"
	"github.com/fastertools/ftl/mcp-server/internal/process"
	"github.com/fastertools/ftl/mcp-server/internal/types"
)

// StatusHandler handles status queries for FTL processes
type StatusHandler struct {
	fileManager    *files.Manager
	processManager *process.Manager
}

// NewStatusHandler creates a new status handler
func NewStatusHandler(
	fileManager *files.Manager,
	processManager *process.Manager,
) *StatusHandler {
	return &StatusHandler{
		fileManager:    fileManager,
		processManager: processManager,
	}
}

// Handle processes the get status request
func (h *StatusHandler) Handle(ctx context.Context, ss *mcp.ServerSession, params *mcp.CallToolParamsFor[types.GetStatusInput]) (*mcp.CallToolResultFor[struct{}], error) {
	projectPath := params.Arguments.ProjectPath
	detailed := params.Arguments.Detailed
	fmt.Fprintf(os.Stderr, "=== getStatus FUNCTION CALLED ===\n")
	fmt.Fprintf(os.Stderr, "DEBUG: getStatus called with project path: %s, detailed: %t\n", projectPath, detailed)
	
	// Validate project path exists
	if err := ftl.ValidateProjectPath(projectPath); err != nil {
		statusJSON := fmt.Sprintf(`{"error": "Project directory does not exist", "project_path": "%s"}`, projectPath)
		return &mcp.CallToolResultFor[struct{}]{
			Content: []mcp.Content{&mcp.TextContent{Text: statusJSON}},
		}, nil
	}
	
	// Check for any active process using unified approach
	pid, port, mode, isRunning := h.processManager.GetProcessInfo(projectPath)
	
	if detailed {
		// Return unified status for active process
		var statusJSON string
		if isRunning {
			statusJSON = fmt.Sprintf(`{
				"project_path": "%s",
				"active_process": {
					"is_running": true,
					"pid": %d,
					"port": %d,
					"type": "%s"
				}
			}`, projectPath, pid, port, mode)
		} else {
			statusJSON = fmt.Sprintf(`{
				"project_path": "%s",
				"active_process": null
			}`, projectPath)
		}
		return &mcp.CallToolResultFor[struct{}]{
			Content: []mcp.Content{&mcp.TextContent{Text: statusJSON}},
		}, nil
	}
	
	// Non-detailed unified format
	if isRunning {
		statusJSON := fmt.Sprintf(`{"process_type": "%s", "is_running": true, "pid": %d, "port": %d, "project_path": "%s"}`, 
			mode, pid, port, projectPath)
		return &mcp.CallToolResultFor[struct{}]{
			Content: []mcp.Content{&mcp.TextContent{Text: statusJSON}},
		}, nil
	}
	
	// No processes running
	statusJSON := fmt.Sprintf(`{"process_type": "none", "is_running": false, "pid": 0, "port": 0, "project_path": "%s"}`, projectPath)
	return &mcp.CallToolResultFor[struct{}]{
		Content: []mcp.Content{&mcp.TextContent{Text: statusJSON}},
	}, nil
}