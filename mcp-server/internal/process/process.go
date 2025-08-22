package process

import (
	"bufio"
	"fmt"
	"os"
	"os/exec"
	"strconv"
	"strings"
	"syscall"
	"time"

	"github.com/modelcontextprotocol/mcp-server/internal/files"
)

// Manager handles process lifecycle operations
type Manager struct {
	fileManager *files.Manager
}

// NewManager creates a new process manager
func NewManager(fileManager *files.Manager) *Manager {
	return &Manager{
		fileManager: fileManager,
	}
}

// IsRunning checks if a process with the given PID is running
func (m *Manager) IsRunning(pid int) bool {
	// Invalid PID
	if pid <= 0 {
		return false
	}
	
	// Send signal 0 to check if process exists
	process, err := os.FindProcess(pid)
	if err != nil {
		fmt.Fprintf(os.Stderr, "DEBUG: isProcessRunning - FindProcess failed for PID %d: %v\n", pid, err)
		return false
	}
	
	// On Unix systems, signal 0 can be used to check if process exists
	err = process.Signal(syscall.Signal(0))
	if err != nil {
		fmt.Fprintf(os.Stderr, "DEBUG: isProcessRunning - Signal check failed for PID %d: %v\n", pid, err)
		return false
	}
	
	fmt.Fprintf(os.Stderr, "DEBUG: PID %d is CONFIRMED RUNNING\n", pid)
	return true
}

// GetProcessInfo retrieves process information from PID file and validates if running
// Returns (pid, port, mode, isRunning) or (0, 0, "", false) if no valid process
func (m *Manager) GetProcessInfo(projectPath string) (int, int, string, bool) {
	pid, port, mode, err := m.fileManager.ReadPidFile(projectPath)
	if err != nil {
		return 0, 0, "", false
	}
	
	if m.IsRunning(pid) {
		fmt.Fprintf(os.Stderr, "DEBUG: GetProcessInfo - FTL process %d is running on port %d in %s mode\n", pid, port, mode)
		return pid, port, mode, true
	}
	
	// Process not running, clean up stale PID file
	fmt.Fprintf(os.Stderr, "DEBUG: GetProcessInfo - FTL process %d not running, removing stale PID file\n", pid)
	m.fileManager.RemovePidFile(projectPath)
	return 0, 0, "", false
}

// ValidateAndCleanup checks if a PID file exists and the process is running
// If the process is not running, it removes the stale PID file
// Returns (pid, port, mode, isRunning) or (0, 0, "", false) if no valid process
// This is an alias for GetProcessInfo for backward compatibility
func (m *Manager) ValidateAndCleanup(projectPath string) (int, int, string, bool) {
	return m.GetProcessInfo(projectPath)
}

// IsProcessRunning checks if a specific process is running for a project
func (m *Manager) IsProcessRunning(projectPath string) bool {
	_, _, _, isRunning := m.GetProcessInfo(projectPath)
	return isRunning
}

// StopProcess stops a process and cleans up its PID file
func (m *Manager) StopProcess(projectPath string) error {
	pid, _, mode, isRunning := m.GetProcessInfo(projectPath)
	if !isRunning {
		return fmt.Errorf("FTL process is not running")
	}
	
	if err := m.Kill(pid); err != nil {
		return fmt.Errorf("failed to stop FTL process: %v", err)
	}
	
	// Clean up PID file
	m.fileManager.RemovePidFile(projectPath)
	fmt.Fprintf(os.Stderr, "DEBUG: Stopped FTL process %d (%s mode) and cleaned up PID file\n", pid, mode)
	return nil
}

// Kill stops a process by PID using SIGTERM first, then SIGKILL if needed
func (m *Manager) Kill(pid int) error {
	process, err := os.FindProcess(pid)
	if err != nil {
		return fmt.Errorf("failed to find process %d: %v", pid, err)
	}
	
	// First try SIGTERM (graceful termination)
	fmt.Fprintf(os.Stderr, "DEBUG: Sending SIGTERM to PID %d\n", pid)
	if err := process.Signal(syscall.SIGTERM); err != nil {
		fmt.Fprintf(os.Stderr, "DEBUG: SIGTERM failed for PID %d, trying SIGKILL: %v\n", pid, err)
		// If SIGTERM fails, use SIGKILL
		if err := process.Kill(); err != nil {
			return fmt.Errorf("failed to kill process %d: %v", pid, err)
		}
		return nil
	}
	
	// Wait a moment for graceful shutdown
	time.Sleep(2 * time.Second)
	
	// Check if process is still running
	if err := process.Signal(syscall.Signal(0)); err == nil {
		// Process still running, force kill
		fmt.Fprintf(os.Stderr, "DEBUG: Process %d still running after SIGTERM, sending SIGKILL\n", pid)
		if err := process.Kill(); err != nil {
			return fmt.Errorf("failed to force kill process %d: %v", pid, err)
		}
	} else {
		fmt.Fprintf(os.Stderr, "DEBUG: Process %d terminated gracefully with SIGTERM\n", pid)
	}
	
	return nil
}

// StartProcess starts a new FTL process with the given configuration
type StartConfig struct {
	ProjectPath string
	Command     string
	Args        []string
	LogFile     *os.File
}

// Start starts a new process with the given configuration
func (m *Manager) Start(config StartConfig) (*exec.Cmd, error) {
	cmd := exec.Command(config.Command, config.Args...)
	cmd.Dir = config.ProjectPath
	
	if config.LogFile != nil {
		cmd.Stdout = config.LogFile
		cmd.Stderr = config.LogFile
	}
	
	// Log the command being executed
	fmt.Fprintf(os.Stderr, "Starting command: %s %s in directory %s\n", 
		config.Command, strings.Join(config.Args, " "), config.ProjectPath)
	
	if err := cmd.Start(); err != nil {
		return nil, err
	}
	
	fmt.Fprintf(os.Stderr, "Process started with PID: %d\n", cmd.Process.Pid)
	return cmd, nil
}

// MonitorProcess monitors a process and performs cleanup when it exits
func (m *Manager) MonitorProcess(cmd *exec.Cmd, projectPath string, logFile *os.File) {
	go func() {
		parentPID := cmd.Process.Pid
		cmd.Wait()
		if logFile != nil {
			logFile.Close()
		}
		
		// Read the actual tracked PID from file (this is the child process we found)
		trackedPID, _, mode, err := m.fileManager.ReadPidFile(projectPath)
		if err != nil {
			fmt.Fprintf(os.Stderr, "DEBUG: MonitorProcess - Could not read PID file for cleanup check: %v\n", err)
			return
		}
		
		// Only clean up PID file if the tracked process is actually dead
		if !m.IsRunning(trackedPID) {
			m.fileManager.RemovePidFile(projectPath)
			fmt.Fprintf(os.Stderr, "FTL process (PID: %d, %s mode) has stopped, cleaned up PID file\n", trackedPID, mode)
		} else {
			fmt.Fprintf(os.Stderr, "FTL parent process (PID: %d) stopped, but tracked process (PID: %d, %s mode) still running\n", parentPID, trackedPID, mode)
		}
	}()
}

// WaitForStartup waits for a process to start up and returns initial output
func (m *Manager) WaitForStartup(logFilePath string, duration time.Duration) (string, error) {
	time.Sleep(duration)
	
	content, err := os.ReadFile(logFilePath)
	if err != nil {
		return "", err
	}
	
	return string(content), nil
}

// FindDeepestChild finds the deepest child process in the process tree
// that is most likely to be the actual HTTP trigger process
func (m *Manager) FindDeepestChild(parentPID int) int {
	fmt.Fprintf(os.Stderr, "DEBUG: FindDeepestChild - Starting search for parent PID %d\n", parentPID)
	
	// Get all descendants recursively
	allDescendants := m.getAllDescendants(parentPID)
	if len(allDescendants) == 0 {
		fmt.Fprintf(os.Stderr, "DEBUG: FindDeepestChild - No descendants found for PID %d, returning parent\n", parentPID)
		return parentPID
	}
	
	fmt.Fprintf(os.Stderr, "DEBUG: FindDeepestChild - Found %d total descendants for PID %d: %v\n", len(allDescendants), parentPID, allDescendants)
	
	// Look for spin watch processes first (for watch mode)
	for _, pid := range allDescendants {
		if m.isSpinWatchProcess(pid) {
			fmt.Fprintf(os.Stderr, "DEBUG: FindDeepestChild - Found spin watch process PID %d\n", pid)
			return pid
		}
	}
	
	// Look for HTTP trigger processes specifically
	for _, pid := range allDescendants {
		if m.isLikelyHTTPProcess(pid) {
			fmt.Fprintf(os.Stderr, "DEBUG: FindDeepestChild - Found HTTP trigger process PID %d\n", pid)
			return pid
		}
	}
	
	// If no special process found, return the deepest (last) descendant
	deepestPID := allDescendants[len(allDescendants)-1]
	fmt.Fprintf(os.Stderr, "DEBUG: FindDeepestChild - No special process found, returning deepest PID %d\n", deepestPID)
	return deepestPID
}

// getAllDescendants recursively gets all descendant processes of a parent PID
func (m *Manager) getAllDescendants(parentPID int) []int {
	var allDescendants []int
	directChildren := m.getDirectChildren(parentPID)
	
	for _, childPID := range directChildren {
		fmt.Fprintf(os.Stderr, "DEBUG: Found child PID %d of parent %d\n", childPID, parentPID)
		allDescendants = append(allDescendants, childPID)
		// Recursively get grandchildren
		grandchildren := m.getAllDescendants(childPID)
		allDescendants = append(allDescendants, grandchildren...)
	}
	
	return allDescendants
}

// getDirectChildren gets immediate children of a process using pgrep
func (m *Manager) getDirectChildren(parentPID int) []int {
	var children []int
	cmd := exec.Command("pgrep", "-P", fmt.Sprintf("%d", parentPID))
	output, err := cmd.Output()
	if err != nil {
		fmt.Fprintf(os.Stderr, "DEBUG: getDirectChildren - Failed to get children for PID %d: %v\n", parentPID, err)
		return children
	}

	scanner := bufio.NewScanner(strings.NewReader(string(output)))
	for scanner.Scan() {
		if childPid, err := strconv.Atoi(strings.TrimSpace(scanner.Text())); err == nil {
			children = append(children, childPid)
		}
	}
	
	fmt.Fprintf(os.Stderr, "DEBUG: getDirectChildren - Found %d direct children for PID %d: %v\n", len(children), parentPID, children)
	return children
}

// isSpinWatchProcess checks if a process is running "spin watch"
func (m *Manager) isSpinWatchProcess(pid int) bool {
	// Get the command line for this process
	cmd := exec.Command("ps", "-p", fmt.Sprintf("%d", pid), "-o", "command=")
	output, err := cmd.Output()
	if err != nil {
		fmt.Fprintf(os.Stderr, "DEBUG: isSpinWatchProcess - Failed to get command for PID %d: %v\n", pid, err)
		return false
	}
	
	command := strings.TrimSpace(string(output))
	fmt.Fprintf(os.Stderr, "DEBUG: isSpinWatchProcess - PID %d command: %s\n", pid, command)
	
	// Look for "spin watch" in the command line
	isSpinWatch := strings.Contains(command, "spin watch")
	fmt.Fprintf(os.Stderr, "DEBUG: isSpinWatchProcess - PID %d is spin watch process: %t\n", pid, isSpinWatch)
	
	return isSpinWatch
}

// isLikelyHTTPProcess checks if a process is likely the HTTP trigger process
func (m *Manager) isLikelyHTTPProcess(pid int) bool {
	// Get the command line for this process
	cmd := exec.Command("ps", "-p", fmt.Sprintf("%d", pid), "-o", "command=")
	output, err := cmd.Output()
	if err != nil {
		fmt.Fprintf(os.Stderr, "DEBUG: isLikelyHTTPProcess - Failed to get command for PID %d: %v\n", pid, err)
		return false
	}
	
	command := strings.TrimSpace(string(output))
	fmt.Fprintf(os.Stderr, "DEBUG: isLikelyHTTPProcess - PID %d command: %s\n", pid, command)
	
	// Look for "trigger http" in the command line
	isHTTP := strings.Contains(command, "trigger http")
	fmt.Fprintf(os.Stderr, "DEBUG: isLikelyHTTPProcess - PID %d is HTTP process: %t\n", pid, isHTTP)
	
	return isHTTP
}

// getChildProcesses returns a list of child process PIDs for a given parent PID
func (m *Manager) getChildProcesses(parentPID int) []int {
	cmd := exec.Command("ps", "-o", "pid=", "--ppid", strconv.Itoa(parentPID))
	output, err := cmd.Output()
	if err != nil {
		fmt.Fprintf(os.Stderr, "DEBUG: getChildProcesses - Failed to get children for PID %d: %v\n", parentPID, err)
		return []int{}
	}
	
	var children []int
	lines := strings.Split(strings.TrimSpace(string(output)), "\n")
	for _, line := range lines {
		line = strings.TrimSpace(line)
		if line == "" {
			continue
		}
		if pid, err := strconv.Atoi(line); err == nil {
			children = append(children, pid)
		}
	}
	
	return children
}

// KillDeepestChild finds and kills the deepest child process of a given parent PID
func (m *Manager) KillDeepestChild(parentPID int) error {
	deepestPID := m.FindDeepestChild(parentPID)
	if deepestPID == parentPID {
		fmt.Fprintf(os.Stderr, "DEBUG: KillDeepestChild - No children found, killing parent PID %d\n", parentPID)
	} else {
		fmt.Fprintf(os.Stderr, "DEBUG: KillDeepestChild - Killing deepest child PID %d (parent was %d)\n", deepestPID, parentPID)
	}
	
	return m.Kill(deepestPID)
}

// KillProcessGroup kills an entire process group by PID
func (m *Manager) KillProcessGroup(pid int) error {
	fmt.Fprintf(os.Stderr, "DEBUG: KillProcessGroup - Attempting to kill process group for PID %d\n", pid)
	
	// First try to kill the process group
	process, err := os.FindProcess(pid)
	if err != nil {
		return fmt.Errorf("failed to find process %d: %v", pid, err)
	}
	
	// Kill the process group (negative PID)
	err = syscall.Kill(-pid, syscall.SIGTERM)
	if err != nil {
		fmt.Fprintf(os.Stderr, "DEBUG: KillProcessGroup - SIGTERM failed for process group %d, trying SIGKILL: %v\n", pid, err)
		// If SIGTERM fails, try SIGKILL
		err = syscall.Kill(-pid, syscall.SIGKILL)
		if err != nil {
			fmt.Fprintf(os.Stderr, "DEBUG: KillProcessGroup - SIGKILL failed for process group %d, falling back to individual kill: %v\n", pid, err)
			// Fall back to killing just the process
			return process.Kill()
		}
	}
	
	fmt.Fprintf(os.Stderr, "DEBUG: KillProcessGroup - Successfully killed process group for PID %d\n", pid)
	return nil
}