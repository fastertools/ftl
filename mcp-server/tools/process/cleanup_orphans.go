package process

import (
	"context"
	"encoding/json"
	"fmt"
	"os"
	"os/exec"
	"strconv"
	"strings"

	"github.com/modelcontextprotocol/go-sdk/mcp"
)

// CleanupOrphansHandler handles orphaned process cleanup
type CleanupOrphansHandler struct{}

// NewCleanupOrphansHandler creates a new cleanup orphans handler
func NewCleanupOrphansHandler() *CleanupOrphansHandler {
	return &CleanupOrphansHandler{}
}

// CleanupOrphansArgs holds the arguments for cleanup_orphans
type CleanupOrphansArgs struct {
	Pattern string `json:"pattern,omitempty"` // Process name pattern to search for
	Kill    bool   `json:"kill,omitempty"`    // Whether to kill found processes
}

// OrphanedProcess represents an orphaned process
type OrphanedProcess struct {
	PID     int    `json:"pid"`
	Command string `json:"command"`
	Killed  bool   `json:"killed,omitempty"`
}

// CleanupOrphansResult holds the result
type CleanupOrphansResult struct {
	Found   []OrphanedProcess `json:"found"`
	Message string            `json:"message"`
}

// Handle handles the cleanup orphans request
func (h *CleanupOrphansHandler) Handle(ctx context.Context, ss *mcp.ServerSession, params *mcp.CallToolParamsFor[CleanupOrphansArgs]) (*mcp.CallToolResultFor[struct{}], error) {
	args := params.Arguments

	// Default pattern
	if args.Pattern == "" {
		args.Pattern = "ftl"
	}

	// Find processes matching pattern
	orphans, err := h.findOrphanedProcesses(args.Pattern)
	if err != nil {
		return nil, fmt.Errorf("failed to find processes: %v", err)
	}

	// Kill processes if requested
	if args.Kill {
		for i := range orphans {
			proc, err := os.FindProcess(orphans[i].PID)
			if err == nil {
				if err := proc.Kill(); err == nil {
					orphans[i].Killed = true
				}
			}
		}
	}

	message := fmt.Sprintf("Found %d orphaned process(es)", len(orphans))
	if args.Kill {
		killedCount := 0
		for _, o := range orphans {
			if o.Killed {
				killedCount++
			}
		}
		message = fmt.Sprintf("Found %d orphaned process(es), killed %d", len(orphans), killedCount)
	}

	resultJSON, _ := json.Marshal(CleanupOrphansResult{
		Found:   orphans,
		Message: message,
	})
	return &mcp.CallToolResultFor[struct{}]{
		Content: []mcp.Content{&mcp.TextContent{Text: string(resultJSON)}},
	}, nil
}

// findOrphanedProcesses finds processes matching the pattern
func (h *CleanupOrphansHandler) findOrphanedProcesses(pattern string) ([]OrphanedProcess, error) {
	var orphans []OrphanedProcess

	// Use ps to find processes
	cmd := exec.Command("ps", "aux")
	output, err := cmd.Output()
	if err != nil {
		return orphans, err
	}

	lines := strings.Split(string(output), "\n")
	for _, line := range lines {
		if strings.Contains(line, pattern) && !strings.Contains(line, "ps aux") {
			fields := strings.Fields(line)
			if len(fields) >= 11 {
				pidStr := fields[1]
				pid, err := strconv.Atoi(pidStr)
				if err != nil {
					continue
				}

				// Get command (fields 10 onwards)
				command := strings.Join(fields[10:], " ")
				
				// Skip if it's this process or the grep process
				if strings.Contains(command, "cleanup_orphans") {
					continue
				}

				orphans = append(orphans, OrphanedProcess{
					PID:     pid,
					Command: command,
				})
			}
		}
	}

	return orphans, nil
}