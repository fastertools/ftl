package testing

import (
	"context"
	"encoding/json"
	"fmt"
	"net"
	"net/http"
	"os"
	"os/exec"
	"path/filepath"
	"strconv"
	"syscall"
	"time"
)

// ServerStartRequest represents the input for starting a test server
type ServerStartRequest struct {
	Port             int               `json:"port,omitempty"`
	UseTestProjects  bool              `json:"use_test_projects"`
	Timeout          int               `json:"timeout,omitempty"`
	Retries          int               `json:"retries,omitempty"`
	Environment      map[string]string `json:"environment,omitempty"`
	FTLBinaryPath    string            `json:"ftl_binary_path,omitempty"`
}

// ServerStopRequest represents the input for stopping a test server
type ServerStopRequest struct {
	Port      int  `json:"port,omitempty"`
	PID       int  `json:"pid,omitempty"`
	Graceful  bool `json:"graceful"`
	Timeout   int  `json:"timeout,omitempty"`
}

// ServerStatusResponse represents server status information
type ServerStatusResponse struct {
	Running    bool   `json:"running"`
	Port       int    `json:"port,omitempty"`
	PID        int    `json:"pid,omitempty"`
	BaseURL    string `json:"base_url,omitempty"`
	Error      string `json:"error,omitempty"`
}

// StartServer starts an FTL development server for testing
func StartServer(ctx context.Context, input json.RawMessage) (interface{}, error) {
	var req ServerStartRequest
	if err := json.Unmarshal(input, &req); err != nil {
		return nil, fmt.Errorf("failed to parse server start request: %v", err)
	}

	// Set defaults
	if req.Port == 0 {
		port, err := findAvailablePort(8080, 10)
		if err != nil {
			return nil, fmt.Errorf("failed to find available port: %v", err)
		}
		req.Port = port
	}
	if req.Timeout == 0 {
		req.Timeout = 30000 // 30 seconds
	}
	if req.Retries == 0 {
		req.Retries = 3
	}

	// Determine FTL binary path
	ftlBinary := req.FTLBinaryPath
	if ftlBinary == "" {
		ftlBinary = "ftl" // Default to "ftl" binary in PATH
	}

	// Setup environment
	env := os.Environ()
	if req.UseTestProjects {
		// Add test projects file environment variable
		testProjectsDir := filepath.Join(".", ".e2e-projects")
		testProjectsFile := filepath.Join(testProjectsDir, ".e2e-projects.json")
		env = append(env, fmt.Sprintf("PROJECTS_FILE=%s", testProjectsFile))
		
		// Ensure test projects directory and file exist
		if err := os.MkdirAll(testProjectsDir, 0755); err != nil {
			return nil, fmt.Errorf("failed to create test projects directory: %v", err)
		}
		if _, err := os.Stat(testProjectsFile); os.IsNotExist(err) {
			if err := os.WriteFile(testProjectsFile, []byte("[]"), 0644); err != nil {
				return nil, fmt.Errorf("failed to create test projects file: %v", err)
			}
		}
	}
	env = append(env, "FTL_TEST_MODE=true")

	// Add custom environment variables
	for key, value := range req.Environment {
		env = append(env, fmt.Sprintf("%s=%s", key, value))
	}

	// Start the server process
	cmd := exec.CommandContext(ctx, ftlBinary, "dev", "console", "--port", strconv.Itoa(req.Port))
	cmd.Env = env
	cmd.SysProcAttr = &syscall.SysProcAttr{Setpgid: true}

	if err := cmd.Start(); err != nil {
		return nil, fmt.Errorf("failed to start FTL server: %v", err)
	}

	baseURL := fmt.Sprintf("http://localhost:%d", req.Port)

	// Wait for server to be ready
	ready, err := waitForServerReady(ctx, baseURL, time.Duration(req.Timeout)*time.Millisecond, req.Retries)
	if err != nil {
		cmd.Process.Kill()
		return nil, fmt.Errorf("server failed to become ready: %v", err)
	}

	if !ready {
		cmd.Process.Kill()
		return nil, fmt.Errorf("server did not become ready within timeout")
	}

	return ServerStatusResponse{
		Running: true,
		Port:    req.Port,
		PID:     cmd.Process.Pid,
		BaseURL: baseURL,
	}, nil
}

// StopServer stops a running test server
func StopServer(ctx context.Context, input json.RawMessage) (interface{}, error) {
	var req ServerStopRequest
	if err := json.Unmarshal(input, &req); err != nil {
		return nil, fmt.Errorf("failed to parse server stop request: %v", err)
	}

	// Set defaults
	if req.Timeout == 0 {
		req.Timeout = 5000 // 5 seconds
	}

	// Find process by PID if provided
	if req.PID > 0 {
		process, err := os.FindProcess(req.PID)
		if err != nil {
			return ServerStatusResponse{Running: false, Error: fmt.Sprintf("Process not found: %v", err)}, nil
		}

		var signal os.Signal = syscall.SIGKILL
		if req.Graceful {
			signal = syscall.SIGTERM
		}

		if err := process.Signal(signal); err != nil {
			return ServerStatusResponse{Running: false, Error: fmt.Sprintf("Failed to signal process: %v", err)}, nil
		}

		// Wait for process to exit if graceful
		if req.Graceful {
			timeout := time.After(time.Duration(req.Timeout) * time.Millisecond)
			ticker := time.NewTicker(100 * time.Millisecond)
			defer ticker.Stop()

			for {
				select {
				case <-timeout:
					// Force kill after timeout
					process.Signal(syscall.SIGKILL)
					return ServerStatusResponse{Running: false}, nil
				case <-ticker.C:
					// Check if process still exists
					if err := process.Signal(syscall.Signal(0)); err != nil {
						// Process doesn't exist anymore
						return ServerStatusResponse{Running: false}, nil
					}
				}
			}
		}

		return ServerStatusResponse{Running: false}, nil
	}

	// If no PID provided, try to find by port
	if req.Port > 0 {
		// Implementation would need platform-specific process finding by port
		// For now, return success
		return ServerStatusResponse{Running: false}, nil
	}

	return ServerStatusResponse{Running: false, Error: "No PID or port provided"}, nil
}

// GetServerStatus checks if a server is running on a specific port
func GetServerStatus(ctx context.Context, input json.RawMessage) (interface{}, error) {
	var req struct {
		Port int `json:"port"`
	}
	if err := json.Unmarshal(input, &req); err != nil {
		return nil, fmt.Errorf("failed to parse server status request: %v", err)
	}

	if req.Port == 0 {
		return ServerStatusResponse{Running: false, Error: "Port required"}, nil
	}

	baseURL := fmt.Sprintf("http://localhost:%d", req.Port)
	
	client := &http.Client{Timeout: 2 * time.Second}
	resp, err := client.Get(baseURL)
	if err != nil {
		return ServerStatusResponse{Running: false, Port: req.Port, Error: err.Error()}, nil
	}
	resp.Body.Close()

	running := resp.StatusCode < 500 // Consider 4xx as running but maybe not fully ready

	return ServerStatusResponse{
		Running: running,
		Port:    req.Port,
		BaseURL: baseURL,
	}, nil
}

// Helper functions

func findAvailablePort(startPort, maxAttempts int) (int, error) {
	for i := 0; i < maxAttempts; i++ {
		port := startPort + i
		if isPortAvailable(port) {
			return port, nil
		}
	}
	return 0, fmt.Errorf("no available ports found in range %d-%d", startPort, startPort+maxAttempts)
}

func isPortAvailable(port int) bool {
	ln, err := net.Listen("tcp", fmt.Sprintf(":%d", port))
	if err != nil {
		return false
	}
	ln.Close()
	return true
}

func waitForServerReady(ctx context.Context, baseURL string, timeout time.Duration, retries int) (bool, error) {
	client := &http.Client{Timeout: 2 * time.Second}
	deadline := time.Now().Add(timeout)

	for time.Now().Before(deadline) {
		select {
		case <-ctx.Done():
			return false, ctx.Err()
		default:
		}

		resp, err := client.Get(baseURL)
		if err == nil && resp.StatusCode < 500 {
			resp.Body.Close()
			return true, nil
		}
		if resp != nil {
			resp.Body.Close()
		}

		time.Sleep(1 * time.Second)
	}

	return false, nil
}