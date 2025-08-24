package lifecycle

import (
	"context"
	"encoding/json"
	"fmt"
	"os"
	"os/exec"
	"strconv"
	"strings"
	"time"

	"github.com/fastertools/ftl/mcp-server/internal/files"
	"github.com/fastertools/ftl/mcp-server/internal/ftl"
	"github.com/fastertools/ftl/mcp-server/internal/port"
	"github.com/fastertools/ftl/mcp-server/internal/process"
	"github.com/fastertools/ftl/mcp-server/internal/types"
	"github.com/modelcontextprotocol/go-sdk/mcp"
)

// UpHandler handles ftl up operations
type UpHandler struct {
	fileManager    *files.Manager
	processManager *process.Manager
	portManager    *port.Manager
	ftlCommander   *ftl.Commander
}

// NewUpHandler creates a new up handler
func NewUpHandler(
	fileManager *files.Manager,
	processManager *process.Manager,
	portManager *port.Manager,
	ftlCommander *ftl.Commander,
) *UpHandler {
	return &UpHandler{
		fileManager:    fileManager,
		processManager: processManager,
		portManager:    portManager,
		ftlCommander:   ftlCommander,
	}
}

// Helper function for error responses
func (h *UpHandler) createErrorResponse(projectPath, mode, errorType string, err error) (*mcp.CallToolResultFor[struct{}], error) {
	response := types.UpResponse{
		Success:     false,
		ProjectPath: projectPath,
		ProcessType: mode,
		Error:       fmt.Sprintf("Failed to %s: %s", errorType, err.Error()),
	}
	responseJSON, _ := json.Marshal(response)
	return &mcp.CallToolResultFor[struct{}]{
		Content: []mcp.Content{&mcp.TextContent{Text: string(responseJSON)}},
	}, nil
}

// Helper function for success responses
func (h *UpHandler) createSuccessResponse(projectPath, mode string, pid, port int, message string) (*mcp.CallToolResultFor[struct{}], error) {
	response := types.UpResponse{
		Success:     true,
		Message:     message,
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

// Helper function for already running process responses
func (h *UpHandler) createAlreadyRunningResponse(projectPath, mode string, pid, port int) (*mcp.CallToolResultFor[struct{}], error) {
	response := types.UpResponse{
		Success:     false,
		Message:     fmt.Sprintf("FTL process already running in %s mode", mode),
		ProjectPath: projectPath,
		ProcessType: mode,
		PID:         pid,
		Port:        port,
		Error:       "Process already exists",
	}
	responseJSON, _ := json.Marshal(response)
	return &mcp.CallToolResultFor[struct{}]{
		Content: []mcp.Content{&mcp.TextContent{Text: string(responseJSON)}},
	}, nil
}

// parseListenAddress extracts port number from listen address, returns port and error
func parseListenAddress(listen string) (int, error) {
	if listen == "" {
		return 0, fmt.Errorf("empty listen address")
	}

	// Handle formats like "localhost:3000", ":3000", "127.0.0.1:3000"
	parts := strings.Split(listen, ":")
	if len(parts) < 2 {
		return 0, fmt.Errorf("invalid listen address format: %s", listen)
	}

	portStr := parts[len(parts)-1] // Get the last part as port
	port, err := strconv.Atoi(portStr)
	if err != nil {
		return 0, fmt.Errorf("invalid port in listen address %s: %v", listen, err)
	}

	if port <= 0 || port > 65535 {
		return 0, fmt.Errorf("port %d is out of valid range (1-65535)", port)
	}

	return port, nil
}

// Handle processes the ftl up request
func (h *UpHandler) Handle(ctx context.Context, ss *mcp.ServerSession, params *mcp.CallToolParamsFor[types.FTLUpInput]) (*mcp.CallToolResultFor[struct{}], error) {
	fmt.Fprintf(os.Stderr, "DEBUG: ftlUp function called\n")
	projectPath := params.Arguments.ProjectPath
	watch := params.Arguments.Watch
	build := params.Arguments.Build
	listen := params.Arguments.Listen

	// Determine mode
	mode := "regular"
	if watch {
		mode = "watch"
	}

	// Log the incoming payload to stderr
	fmt.Fprintf(os.Stderr, "DEBUG: ftl_up received payload: project_path='%s', mode='%s', build='%t', listen='%s'\n",
		projectPath, mode, build, listen)

	// Log current working directory and validate project path
	if cwd, err := os.Getwd(); err == nil {
		fmt.Fprintf(os.Stderr, "DEBUG: Current working directory: %s\n", cwd)
	}
	fmt.Fprintf(os.Stderr, "DEBUG: About to validate project path: %s\n", projectPath)

	// Validate project directory
	if err := ftl.ValidateProjectPath(projectPath); err != nil {
		fmt.Fprintf(os.Stderr, "DEBUG: Project path validation failed: %v\n", err)
		return h.createErrorResponse(projectPath, mode, "validate project path", err)
	}
	fmt.Fprintf(os.Stderr, "DEBUG: Project path validation successful\n")

	// Start unified process
	return h.startProcess(ctx, projectPath, mode, build, listen)
}

func (h *UpHandler) startProcess(ctx context.Context, projectPath, mode string, build bool, listen string) (*mcp.CallToolResultFor[struct{}], error) {
	fmt.Fprintf(os.Stderr, "DEBUG: startProcess called with project path: %s, mode: %s, build: %t, listen: %s\n", projectPath, mode, build, listen)

	// Check if there's already a running process
	if pid, port, existingMode, isRunning := h.processManager.ValidateAndCleanup(projectPath); isRunning {
		fmt.Fprintf(os.Stderr, "DEBUG: startProcess - process %d is still running on port %d in %s mode\n", pid, port, existingMode)
		return h.createAlreadyRunningResponse(projectPath, existingMode, pid, port)
	}

	var availablePort int
	var cmd *exec.Cmd
	var err error

	var listenAddr string
	if listen != "" {
		fmt.Fprintf(os.Stderr, "DEBUG: Using custom listen address: %s\n", listen)
		// Use custom listen address
		availablePort, err = parseListenAddress(listen)
		if err != nil {
			fmt.Fprintf(os.Stderr, "DEBUG: Failed to parse listen address %s: %v\n", listen, err)
			return h.createErrorResponse(projectPath, mode, "parse listen address", err)
		}
		fmt.Fprintf(os.Stderr, "DEBUG: Parsed port %d from listen address %s\n", availablePort, listen)
		listenAddr = listen
	} else {
		fmt.Fprintf(os.Stderr, "DEBUG: Finding available port automatically\n")
		// Find an available port automatically
		availablePort, err = h.portManager.FindAvailable()
		if err != nil {
			fmt.Fprintf(os.Stderr, "DEBUG: Failed to find available port: %v\n", err)
			return h.createErrorResponse(projectPath, mode, "find available port", err)
		}
		fmt.Fprintf(os.Stderr, "DEBUG: Found available port: %d\n", availablePort)
		listenAddr = fmt.Sprintf("localhost:%d", availablePort)
	}
	
	// Get command using unified method
	cmd = h.ftlCommander.UpCommand(projectPath, listenAddr, build, mode == "watch")

	// Log the command that will be executed
	fmt.Fprintf(os.Stderr, "DEBUG: Executing command: %s %v in directory: %s\n",
		cmd.Path, cmd.Args, cmd.Dir)

	// Create log file for capturing output
	logFile := h.fileManager.GetLogFilePath(projectPath, files.FTLProcess)
	fmt.Fprintf(os.Stderr, "DEBUG: Creating log file: %s\n", logFile)
	logWriter, err := os.Create(logFile)
	if err != nil {
		fmt.Fprintf(os.Stderr, "DEBUG: Failed to create log file %s: %v\n", logFile, err)
		return h.createErrorResponse(projectPath, mode, "create log file", err)
	}

	// Set command output
	cmd.Stdout = logWriter
	cmd.Stderr = logWriter

	// Start the command
	fmt.Fprintf(os.Stderr, "DEBUG: About to start command\n")
	if err := cmd.Start(); err != nil {
		fmt.Fprintf(os.Stderr, "DEBUG: Failed to start command: %v\n", err)
		logWriter.Close()
		modeFlag := ""
		if mode == "watch" {
			modeFlag = " --watch"
		}
		return h.createErrorResponse(projectPath, mode, fmt.Sprintf("start 'ftl up%s'", modeFlag), err)
	}
	fmt.Fprintf(os.Stderr, "DEBUG: Command started successfully\n")

	mainPID := cmd.Process.Pid
	fmt.Fprintf(os.Stderr, "FTL process started with PID: %d on port %d in %s mode\n", mainPID, availablePort, mode)

	// Wait for child processes to spawn, then find the correct stop PID
	if mode == "watch" {
		time.Sleep(3 * time.Second)
	} else {
		time.Sleep(5 * time.Second)
	}

	stopPID := h.processManager.FindDeepestChild(mainPID)
	fmt.Fprintf(os.Stderr, "DEBUG: Found stop PID %d for main PID %d\n", stopPID, mainPID)

	// Write the worker PID to file (not the parent PID)
	if err := h.fileManager.WritePidFile(projectPath, stopPID, availablePort, mode); err != nil {
		cmd.Process.Kill()
		logWriter.Close()
		return h.createErrorResponse(projectPath, mode, "write PID file", err)
	}

	// Monitor process in background
	h.processManager.MonitorProcess(cmd, projectPath, logWriter)

	// For regular mode, wait for startup and get initial output
	var output string
	if mode == "regular" {
		output, _ = h.processManager.WaitForStartup(logFile, 2*time.Second)
	}

	buildFlag := ""
	if build {
		buildFlag = " --build"
	}
	
	modeFlag := ""
	if mode == "watch" {
		modeFlag = " --watch"
	}
	
	message := fmt.Sprintf("Started 'ftl up%s%s' in project: %s", modeFlag, buildFlag, projectPath)
	if output != "" {
		message += fmt.Sprintf("\n\nInitial output:\n%s", output)
	}
	
	return h.createSuccessResponse(projectPath, mode, stopPID, availablePort, message)
}
