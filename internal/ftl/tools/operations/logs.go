package operations

import (
	"context"
	"encoding/json"
	"fmt"
	"os"
	"strings"

	"github.com/modelcontextprotocol/go-sdk/mcp"
	"github.com/fastertools/ftl/internal/ftl/files"
	"github.com/fastertools/ftl/internal/ftl/ftl"
	"github.com/fastertools/ftl/internal/ftl/process"
	"github.com/fastertools/ftl/internal/ftl/types"
)

// LogsHandler handles log retrieval operations
type LogsHandler struct {
	fileManager    *files.Manager
	processManager *process.Manager
}

// NewLogsHandler creates a new logs handler
func NewLogsHandler(
	fileManager *files.Manager,
	processManager *process.Manager,
) *LogsHandler {
	return &LogsHandler{
		fileManager:    fileManager,
		processManager: processManager,
	}
}

// Handle processes the get logs request
func (h *LogsHandler) Handle(ctx context.Context, ss *mcp.ServerSession, params *mcp.CallToolParamsFor[types.GetLogsInput]) (*mcp.CallToolResultFor[struct{}], error) {
	fmt.Fprintf(os.Stderr, "=== getLogs FUNCTION CALLED ===\n")
	since := params.Arguments.Since
	projectPath := params.Arguments.ProjectPath

	// Validate project path exists
	if err := ftl.ValidateProjectPath(projectPath); err != nil {
		response := types.LogsResponse{
			ProjectPath: projectPath,
			Success:     false,
			Error:       err.Error(),
		}
		responseJSON, _ := json.Marshal(response)
		return &mcp.CallToolResultFor[struct{}]{
			Content: []mcp.Content{&mcp.TextContent{Text: string(responseJSON)}},
		}, nil
	}
	
	// Check for any active process using unified approach
	pid, port, mode, isRunning := h.processManager.ValidateAndCleanup(projectPath)
	
	var logFile string
	var processType string
	
	if isRunning {
		processType = mode
		logFile = h.fileManager.GetLogFilePath(projectPath, files.FTLProcess)
	}
	
	// If no processes found
	if !isRunning {
		response := types.LogsResponse{
			ProjectPath: projectPath,
			Success:     false,
			Error:       "No FTL processes found (neither watch nor regular)",
		}
		responseJSON, _ := json.Marshal(response)
		return &mcp.CallToolResultFor[struct{}]{
			Content: []mcp.Content{&mcp.TextContent{Text: string(responseJSON)}},
		}, nil
	}

	// Read log file
	content, err := os.ReadFile(logFile)
	if err != nil {
		response := types.LogsResponse{
			ProjectPath: projectPath,
			ProcessType: processType,
			IsRunning:   isRunning,
			PID:         pid,
			Port:        port,
			Success:     false,
			Error:       "No logs available",
		}
		responseJSON, _ := json.Marshal(response)
		return &mcp.CallToolResultFor[struct{}]{
			Content: []mcp.Content{&mcp.TextContent{Text: string(responseJSON)}},
		}, nil
	}

	lines := strings.Split(string(content), "\n")
	totalLines := len(lines)

	if since >= totalLines {
		// No new logs
		response := types.LogsResponse{
			ProjectPath:  projectPath,
			ProcessType:  processType,
			IsRunning:    isRunning,
			PID:          pid,
			Port:         port,
			Logs:         "",
			TotalLines:   totalLines,
			Since:        since,
			NewLogsCount: 0,
			Success:      true,
		}
		responseJSON, _ := json.Marshal(response)
		return &mcp.CallToolResultFor[struct{}]{
			Content: []mcp.Content{&mcp.TextContent{Text: string(responseJSON)}},
		}, nil
	}

	// Get new logs since the specified line
	newLines := lines[since:]
	logText := strings.Join(newLines, "\n")

	response := types.LogsResponse{
		ProjectPath:  projectPath,
		ProcessType:  processType,
		IsRunning:    isRunning,
		PID:          pid,
		Port:         port,
		Logs:         logText,
		TotalLines:   totalLines,
		Since:        since,
		NewLogsCount: len(newLines),
		Success:      true,
	}
	responseJSON, _ := json.Marshal(response)
	return &mcp.CallToolResultFor[struct{}]{
		Content: []mcp.Content{&mcp.TextContent{Text: string(responseJSON)}},
	}, nil
}