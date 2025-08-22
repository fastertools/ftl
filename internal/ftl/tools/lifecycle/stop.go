package lifecycle

import (
	"context"
	"encoding/json"
	"fmt"

	"github.com/modelcontextprotocol/go-sdk/mcp"
	"github.com/fastertools/ftl/internal/ftl/files"
	"github.com/fastertools/ftl/internal/ftl/process"
	"github.com/fastertools/ftl/internal/ftl/types"
)

// StopHandler handles stop operations for FTL processes
type StopHandler struct {
	fileManager    *files.Manager
	processManager *process.Manager
}

// NewStopHandler creates a new stop handler
func NewStopHandler(
	fileManager *files.Manager,
	processManager *process.Manager,
) *StopHandler {
	return &StopHandler{
		fileManager:    fileManager,
		processManager: processManager,
	}
}

// HandleStop stops any running FTL process (unified handler)
func (h *StopHandler) HandleStop(ctx context.Context, ss *mcp.ServerSession, params *mcp.CallToolParamsFor[types.StopInput]) (*mcp.CallToolResultFor[struct{}], error) {
	projectPath := params.Arguments.ProjectPath

	// Check if there's a running process
	pid, port, mode, isRunning := h.processManager.ValidateAndCleanup(projectPath)
	if !isRunning {
		response := types.StopResponse{
			Success:     false,
			Message:     "No running FTL process found",
			ProjectPath: projectPath,
			ProcessType: "none",
			Error:       "Process not found",
		}
		responseJSON, _ := json.Marshal(response)
		return &mcp.CallToolResultFor[struct{}]{
			Content: []mcp.Content{&mcp.TextContent{Text: string(responseJSON)}},
		}, nil
	}

	// Kill the process
	if err := h.processManager.Kill(pid); err != nil {
		response := types.StopResponse{
			Success:     false,
			Message:     fmt.Sprintf("Failed to kill FTL process (%s mode)", mode),
			ProjectPath: projectPath,
			ProcessType: mode,
			PID:         pid,
			Port:        port,
			Error:       err.Error(),
		}
		responseJSON, _ := json.Marshal(response)
		return &mcp.CallToolResultFor[struct{}]{
			Content: []mcp.Content{&mcp.TextContent{Text: string(responseJSON)}},
		}, nil
	}

	// Clean up PID file
	h.fileManager.RemovePidFile(projectPath)

	response := types.StopResponse{
		Success:     true,
		Message:     fmt.Sprintf("FTL process (%s mode) stopped successfully", mode),
		ProjectPath: projectPath,
		ProcessType: mode,
		PID:         pid,
		Port:        port,
	}
	responseJSON, _ := json.Marshal(response)
	return &mcp.CallToolResultFor[struct{}]{
		Content: []mcp.Content{&mcp.TextContent{Text: string(responseJSON)}},
	}, nil
}

// HandleStopWatch stops a running watch process (legacy compatibility)
func (h *StopHandler) HandleStopWatch(ctx context.Context, ss *mcp.ServerSession, params *mcp.CallToolParamsFor[types.StopInput]) (*mcp.CallToolResultFor[struct{}], error) {
	return h.HandleStop(ctx, ss, params)
}

// HandleStopRegular stops a running regular FTL server (legacy compatibility)
func (h *StopHandler) HandleStopRegular(ctx context.Context, ss *mcp.ServerSession, params *mcp.CallToolParamsFor[types.StopInput]) (*mcp.CallToolResultFor[struct{}], error) {
	return h.HandleStop(ctx, ss, params)
}