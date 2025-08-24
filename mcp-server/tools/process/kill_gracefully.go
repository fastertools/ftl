package process

import (
	"context"
	"encoding/json"
	"fmt"
	"os"
	"syscall"
	"time"

	"github.com/modelcontextprotocol/go-sdk/mcp"
)

// KillGracefullyHandler handles graceful process termination
type KillGracefullyHandler struct{}

// NewKillGracefullyHandler creates a new kill gracefully handler
func NewKillGracefullyHandler() *KillGracefullyHandler {
	return &KillGracefullyHandler{}
}

// marshalResult converts result to JSON string
func marshalResult(v interface{}) string {
	data, _ := json.Marshal(v)
	return string(data)
}

// KillGracefullyArgs holds the arguments for kill_gracefully
type KillGracefullyArgs struct {
	PID        int `json:"pid"`
	GraceTime  int `json:"grace_time,omitempty"`  // Time to wait before SIGKILL (seconds)
}

// KillGracefullyResult holds the result
type KillGracefullyResult struct {
	Success   bool   `json:"success"`
	PID       int    `json:"pid"`
	Signal    string `json:"signal_used"`
	Message   string `json:"message"`
}

// Handle handles the kill gracefully request
func (h *KillGracefullyHandler) Handle(ctx context.Context, ss *mcp.ServerSession, params *mcp.CallToolParamsFor[KillGracefullyArgs]) (*mcp.CallToolResultFor[struct{}], error) {
	args := params.Arguments

	if args.PID == 0 {
		return nil, fmt.Errorf("PID is required")
	}

	// Default grace time
	if args.GraceTime == 0 {
		args.GraceTime = 5
	}

	// Find the process
	proc, err := os.FindProcess(args.PID)
	if err != nil {
		return &mcp.CallToolResultFor[struct{}]{
		Content: []mcp.Content{&mcp.TextContent{Text: marshalResult(KillGracefullyResult{
			Success: false,
			PID:     args.PID,
			Message: fmt.Sprintf("Process %d not found", args.PID),
		})}},
	}, nil
	}

	// Try SIGTERM first
	err = proc.Signal(syscall.SIGTERM)
	if err != nil {
		// Process might already be dead
		return &mcp.CallToolResultFor[struct{}]{
		Content: []mcp.Content{&mcp.TextContent{Text: marshalResult(KillGracefullyResult{
			Success: true,
			PID:     args.PID,
			Message: "Process already stopped",
		})}},
	}, nil
	}

	// Wait for grace period
	gracePeriod := time.Duration(args.GraceTime) * time.Second
	deadline := time.Now().Add(gracePeriod)
	stopped := false

	for time.Now().Before(deadline) {
		// Check if process still exists
		err = proc.Signal(syscall.Signal(0))
		if err != nil {
			stopped = true
			break
		}
		time.Sleep(100 * time.Millisecond)
	}

	if stopped {
		return &mcp.CallToolResultFor[struct{}]{
		Content: []mcp.Content{&mcp.TextContent{Text: marshalResult(KillGracefullyResult{
			Success:   true,
			PID:       args.PID,
			Signal:    "SIGTERM",
			Message:   fmt.Sprintf("Process %d terminated gracefully", args.PID),
		})}},
	}, nil
	}

	// Force kill with SIGKILL
	err = proc.Signal(syscall.SIGKILL)
	if err != nil {
		// Might have died between checks
		return &mcp.CallToolResultFor[struct{}]{
		Content: []mcp.Content{&mcp.TextContent{Text: marshalResult(KillGracefullyResult{
			Success:   true,
			PID:       args.PID,
			Signal:    "SIGTERM",
			Message:   "Process stopped after SIGTERM",
		})}},
	}, nil
	}

	// Wait a moment for SIGKILL to take effect
	time.Sleep(500 * time.Millisecond)

	return &mcp.CallToolResultFor[struct{}]{
		Content: []mcp.Content{&mcp.TextContent{Text: marshalResult(KillGracefullyResult{
			Success:   true,
			PID:       args.PID,
			Signal:    "SIGKILL",
			Message:   fmt.Sprintf("Process %d force killed after %d seconds", args.PID, args.GraceTime),
		})}},
	}, nil
}