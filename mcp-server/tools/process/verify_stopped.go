package process

import (
	"context"
	"encoding/json"
	"fmt"
	"os"
	"syscall"
	"time"

	"github.com/modelcontextprotocol/go-sdk/mcp"
	"github.com/fastertools/ftl/mcp-server/internal/files"
	"github.com/fastertools/ftl/mcp-server/internal/process"
)

// VerifyStoppedHandler handles process verification
type VerifyStoppedHandler struct {
	fileManager    *files.Manager
	processManager *process.Manager
}

// NewVerifyStoppedHandler creates a new verify stopped handler
func NewVerifyStoppedHandler() *VerifyStoppedHandler {
	fileManager := files.NewManager()
	processManager := process.NewManager(fileManager)
	return &VerifyStoppedHandler{
		fileManager:    fileManager,
		processManager: processManager,
	}
}

// VerifyStoppedArgs holds the arguments for verify_stopped
type VerifyStoppedArgs struct {
	PID         int    `json:"pid,omitempty"`
	ProjectPath string `json:"project_path,omitempty"`
	Timeout     int    `json:"timeout,omitempty"`
	CleanupPID  bool   `json:"cleanup_pid,omitempty"`
}

// VerifyStoppedResult holds the result
type VerifyStoppedResult struct {
	Stopped        bool   `json:"stopped"`
	PID            int    `json:"pid"`
	Message        string `json:"message"`
	TimeElapsed    int    `json:"time_elapsed_ms"`
	PIDFileCleaned bool   `json:"pid_file_cleaned"`
}

// Handle handles the verify stopped request
func (h *VerifyStoppedHandler) Handle(ctx context.Context, ss *mcp.ServerSession, params *mcp.CallToolParamsFor[VerifyStoppedArgs]) (*mcp.CallToolResultFor[struct{}], error) {
	args := params.Arguments

	// Default timeout
	if args.Timeout == 0 {
		args.Timeout = 10
	}

	startTime := time.Now()

	// Get PID from project path if not provided
	pid := args.PID
	if pid == 0 && args.ProjectPath != "" {
		// Try to get PID from project
		projectPID, _, _, _ := h.processManager.GetProcessInfo(args.ProjectPath)
		if projectPID > 0 {
			pid = projectPID
		}
	}

	if pid == 0 {
		return nil, fmt.Errorf("no PID provided or found")
	}

	// Wait for process to stop
	timeout := time.Duration(args.Timeout) * time.Second
	deadline := time.Now().Add(timeout)
	stopped := false

	for time.Now().Before(deadline) {
		// Check if process exists
		proc, err := os.FindProcess(pid)
		if err != nil {
			stopped = true
			break
		}

		// Try sending signal 0 to check if process is alive
		err = proc.Signal(syscall.Signal(0))
		if err != nil {
			stopped = true
			break
		}

		// Wait a bit before checking again
		time.Sleep(100 * time.Millisecond)
	}

	elapsed := int(time.Since(startTime).Milliseconds())

	// Clean up PID file if requested and process is stopped
	pidFileCleaned := false
	if stopped && args.CleanupPID && args.ProjectPath != "" {
		h.fileManager.RemovePidFile(args.ProjectPath)
		pidFileCleaned = true
	}

	result := VerifyStoppedResult{
		Stopped:        stopped,
		PID:            pid,
		TimeElapsed:    elapsed,
		PIDFileCleaned: pidFileCleaned,
	}

	if stopped {
		result.Message = fmt.Sprintf("Process %d has stopped", pid)
	} else {
		result.Message = fmt.Sprintf("Process %d is still running after %d ms", pid, elapsed)
	}

	resultJSON, _ := json.Marshal(result)
	return &mcp.CallToolResultFor[struct{}]{
		Content: []mcp.Content{&mcp.TextContent{Text: string(resultJSON)}},
	}, nil
}