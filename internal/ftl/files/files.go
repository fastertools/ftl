package files

import (
	"fmt"
	"os"
	"path/filepath"
	"strconv"
	"strings"
)

// ProcessType represents the type of FTL process
type ProcessType string

const (
	FTLProcess ProcessType = "ftl"
)

// Manager handles file operations for FTL processes
type Manager struct{}

// NewManager creates a new file manager
func NewManager() *Manager {
	return &Manager{}
}

// GetPidFilePath returns the PID file path for a process type
func (m *Manager) GetPidFilePath(projectPath string, processType ProcessType) string {
	return filepath.Join(projectPath, ".ftl.pid")
}

// GetLogFilePath returns the log file path for a process type
func (m *Manager) GetLogFilePath(projectPath string, processType ProcessType) string {
	return filepath.Join(projectPath, ".ftl.log")
}

// WritePidFile writes process ID, port, and mode to a PID file
func (m *Manager) WritePidFile(projectPath string, pid int, port int, mode string) error {
	pidFile := m.GetPidFilePath(projectPath, FTLProcess)
	// Format: PID\nPORT\nMODE\n
	content := fmt.Sprintf("%d\n%d\n%s\n", pid, port, mode)
	fmt.Fprintf(os.Stderr, "DEBUG: Writing PID file: %s with content: %s\n", pidFile, content)
	err := os.WriteFile(pidFile, []byte(content), 0644)
	if err != nil {
		fmt.Fprintf(os.Stderr, "DEBUG: Failed to write PID file: %v\n", err)
	} else {
		fmt.Fprintf(os.Stderr, "DEBUG: Successfully wrote PID file: %s\n", pidFile)
	}
	return err
}

// ReadPidFile reads process ID, port, and mode from a PID file
// Returns (pid, port, mode, error)
func (m *Manager) ReadPidFile(projectPath string) (int, int, string, error) {
	pidFile := m.GetPidFilePath(projectPath, FTLProcess)
	fmt.Fprintf(os.Stderr, "DEBUG: Attempting to read PID file: %s\n", pidFile)
	
	content, err := os.ReadFile(pidFile)
	if err != nil {
		fmt.Fprintf(os.Stderr, "DEBUG: Failed to read PID file: %v\n", err)
		return 0, 0, "", err
	}
	
	fmt.Fprintf(os.Stderr, "DEBUG: PID file content: %s\n", string(content))
	
	lines := strings.Split(strings.TrimSpace(string(content)), "\n")
	if len(lines) < 3 {
		fmt.Fprintf(os.Stderr, "DEBUG: Invalid PID file format - only %d lines, expected 3\n", len(lines))
		return 0, 0, "", fmt.Errorf("invalid PID file format")
	}
	
	pid, err := strconv.Atoi(lines[0])
	if err != nil {
		fmt.Fprintf(os.Stderr, "DEBUG: Invalid PID in file: %v\n", err)
		return 0, 0, "", fmt.Errorf("invalid PID: %v", err)
	}
	
	port, err := strconv.Atoi(lines[1])
	if err != nil {
		fmt.Fprintf(os.Stderr, "DEBUG: Invalid port in file: %v\n", err)
		return 0, 0, "", fmt.Errorf("invalid port: %v", err)
	}
	
	mode := lines[2]
	fmt.Fprintf(os.Stderr, "DEBUG: Successfully read PID file - PID: %d, Port: %d, Mode: %s\n", pid, port, mode)
	
	return pid, port, mode, nil
}

// RemovePidFile removes a PID file
func (m *Manager) RemovePidFile(projectPath string) {
	pidFile := m.GetPidFilePath(projectPath, FTLProcess)
	fmt.Fprintf(os.Stderr, "DEBUG: Removing PID file: %s\n", pidFile)
	err := os.Remove(pidFile)
	if err != nil {
		fmt.Fprintf(os.Stderr, "DEBUG: Failed to remove PID file: %v\n", err)
	} else {
		fmt.Fprintf(os.Stderr, "DEBUG: Successfully removed PID file: %s\n", pidFile)
	}
}

// PidFileExists checks if a PID file exists (basic file existence check)
func (m *Manager) PidFileExists(projectPath string) bool {
	pidFile := m.GetPidFilePath(projectPath, FTLProcess)
	_, err := os.Stat(pidFile)
	return err == nil
}