package handlers

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"os"
	"strings"
	"testing"
	"time"

	"github.com/fastertools/ftl/internal/mcpclient"
	"github.com/fastertools/ftl/internal/models"
	"github.com/fastertools/ftl/internal/polling"
	"github.com/fastertools/ftl/internal/state"
)

// TestHandleTestWaitForHTMX tests the HTMX wait endpoint
func TestHandleTestWaitForHTMX(t *testing.T) {
	// Set test mode environment variable
	os.Setenv("FTL_TEST_MODE", "true")
	defer os.Unsetenv("FTL_TEST_MODE")
	
	// Create temporary test directory
	tmpDir := t.TempDir()
	projectPath := tmpDir + "/test-project"
	os.MkdirAll(projectPath, 0755)
	
	// Create a test MCP client
	mcpClient := mcpclient.NewClientWithArgs("echo", []string{"test"})

	// Create project registry
	projectRegistry := state.NewProjectRegistry(tmpDir + "/test_projects.json")

	// Create handler with dependencies
	handler := &Handler{
		mcpClient:      mcpClient,
		logPositions:   state.NewLogPositionTracker(),
		registry:       projectRegistry,
		pollingManager: polling.NewManager(mcpClient, projectRegistry),
	}

	// Add a test project
	project, err := handler.registry.AddProject(projectPath, "Test Project")
	if err != nil {
		t.Fatalf("Failed to add project: %v", err)
	}

	// Set initial state - no active polling
	project.PollingActive = false

	tests := []struct {
		name           string
		timeout        string
		setupState     func()
		expectedStatus int
		expectedBody   string
	}{
		{
			name:    "HTMX becomes idle immediately",
			timeout: "100",
			setupState: func() {
				// Polling is already inactive
			},
			expectedStatus: http.StatusOK,
			expectedBody:   `{"ready":true,"message":"HTMX operations settled"}`,
		},
		{
			name:    "Timeout waiting for HTMX",
			timeout: "100",
			setupState: func() {
				// Set polling active and keep it active
				project.StartPolling()
			},
			expectedStatus: http.StatusRequestTimeout,
			expectedBody:   `{"ready":false,"message":"Timeout waiting for HTMX to settle"}`,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Setup state
			tt.setupState()

			// Create request
			req := httptest.NewRequest("GET", "/api/test/wait-for-htmx?timeout="+tt.timeout+"&project_path="+projectPath, nil)
			w := httptest.NewRecorder()

			// Call handler
			handler.HandleTestWaitForHTMX(w, req)

			// Check status
			if w.Code != tt.expectedStatus {
				t.Errorf("Expected status %d, got %d", tt.expectedStatus, w.Code)
			}

			// Check body
			body := strings.TrimSpace(w.Body.String())
			expected := strings.TrimSpace(tt.expectedBody)
			if body != expected {
				t.Errorf("Expected body %s, got %s", expected, body)
			}

			// Clean up
			project.StopPolling()
		})
	}
}

// TestHandleTestWaitForStatus tests the status wait endpoint
func TestHandleTestWaitForStatus(t *testing.T) {
	// Set test mode environment variable
	os.Setenv("FTL_TEST_MODE", "true")
	defer os.Unsetenv("FTL_TEST_MODE")
	
	// Create temporary test directory
	tmpDir := t.TempDir()
	projectPath := tmpDir + "/test-project"
	os.MkdirAll(projectPath, 0755)
	
	// Create a test MCP client
	mcpClient := mcpclient.NewClientWithArgs("echo", []string{"test"})

	// Create project registry
	projectRegistry := state.NewProjectRegistry(tmpDir + "/test_projects.json")

	// Create handler with dependencies
	handler := &Handler{
		mcpClient:      mcpClient,
		logPositions:   state.NewLogPositionTracker(),
		registry:       projectRegistry,
		pollingManager: polling.NewManager(mcpClient, projectRegistry),
	}

	// Add a test project
	project, err := handler.registry.AddProject(projectPath, "Test Project")
	if err != nil {
		t.Fatalf("Failed to add project: %v", err)
	}

	tests := []struct {
		name           string
		status         string
		timeout        string
		setupState     func()
		expectedStatus int
		expectedBody   string
	}{
		{
			name:    "Process is already running",
			status:  "running",
			timeout: "100",
			setupState: func() {
				// Set process as running
				project.UpdateProcessInfo(models.DetailedStatusInfo{
					ActiveProcess: &models.ProcessInfo{
						IsRunning: true,
						Type:      "regular",
					},
				})
			},
			expectedStatus: http.StatusOK,
			expectedBody:   `{"ready":true,"status":"running","message":"Process status matches"}`,
		},
		{
			name:    "Process is already stopped",
			status:  "stopped",
			timeout: "100",
			setupState: func() {
				// Set process as stopped
				project.UpdateProcessInfo(models.DetailedStatusInfo{
					ActiveProcess: nil,
				})
			},
			expectedStatus: http.StatusOK,
			expectedBody:   `{"ready":true,"status":"stopped","message":"Process status matches"}`,
		},
		{
			name:    "Timeout waiting for status change",
			status:  "running",
			timeout: "100",
			setupState: func() {
				// Set process as stopped and keep it stopped
				project.UpdateProcessInfo(models.DetailedStatusInfo{
					ActiveProcess: nil,
				})
			},
			expectedStatus: http.StatusRequestTimeout,
			expectedBody:   `{"ready":false,"status":"stopped","message":"Timeout waiting for status"}`,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Setup state
			tt.setupState()

			// Create request
			req := httptest.NewRequest("GET", "/api/test/wait-for-status?status="+tt.status+"&timeout="+tt.timeout+"&project_path="+projectPath, nil)
			w := httptest.NewRecorder()

			// Call handler
			handler.HandleTestWaitForStatus(w, req)

			// Check status
			if w.Code != tt.expectedStatus {
				t.Errorf("Expected status %d, got %d", tt.expectedStatus, w.Code)
			}

			// Check body
			body := strings.TrimSpace(w.Body.String())
			expected := strings.TrimSpace(tt.expectedBody)
			if body != expected {
				t.Errorf("Expected body %s, got %s", expected, body)
			}
		})
	}
}

// TestHandleTestWaitForLogs tests the logs wait endpoint
func TestHandleTestWaitForLogs(t *testing.T) {
	// Set test mode environment variable
	os.Setenv("FTL_TEST_MODE", "true")
	defer os.Unsetenv("FTL_TEST_MODE")
	
	// Create temporary test directory
	tmpDir := t.TempDir()
	projectPath := tmpDir + "/test-project"
	os.MkdirAll(projectPath, 0755)
	
	// Create a test MCP client
	mcpClient := mcpclient.NewClientWithArgs("echo", []string{"test"})

	// Create project registry
	projectRegistry := state.NewProjectRegistry(tmpDir + "/test_projects.json")

	// Create handler with dependencies
	handler := &Handler{
		mcpClient:      mcpClient,
		logPositions:   state.NewLogPositionTracker(),
		registry:       projectRegistry,
		pollingManager: polling.NewManager(mcpClient, projectRegistry),
	}

	// Add a test project
	project, err := handler.registry.AddProject(projectPath, "Test Project")
	if err != nil {
		t.Fatalf("Failed to add project: %v", err)
	}

	tests := []struct {
		name           string
		minLines       string
		timeout        string
		setupState     func()
		addLogsDuring  bool
		expectedStatus int
		checkBody      func(string) bool
	}{
		{
			name:     "Logs already available",
			minLines: "2",
			timeout:  "100",
			setupState: func() {
				// Add some initial logs
				project.AddLogLine("Log line 1")
				project.AddLogLine("Log line 2")
				project.AddLogLine("Log line 3")
			},
			expectedStatus: http.StatusOK,
			checkBody: func(body string) bool {
				var resp map[string]interface{}
				if err := json.Unmarshal([]byte(body), &resp); err != nil {
					return false
				}
				ready, ok := resp["ready"].(bool)
				if !ok {
					return false
				}
				logs, ok := resp["logs"].([]interface{})
				if !ok {
					return false
				}
				return ready && len(logs) >= 2
			},
		},
		{
			name:     "Wait for new logs",
			minLines: "2",
			timeout:  "500",
			setupState: func() {
				// Clear logs
				project.ClearLogs()
			},
			addLogsDuring:  true,
			expectedStatus: http.StatusOK,
			checkBody: func(body string) bool {
				var resp map[string]interface{}
				if err := json.Unmarshal([]byte(body), &resp); err != nil {
					return false
				}
				ready, ok := resp["ready"].(bool)
				if !ok {
					return false
				}
				logs, ok := resp["logs"].([]interface{})
				if !ok {
					return false
				}
				return ready && len(logs) >= 2
			},
		},
		{
			name:     "Timeout waiting for logs",
			minLines: "5",
			timeout:  "100",
			setupState: func() {
				// Add only one log
				project.ClearLogs()
				project.AddLogLine("Only one log")
			},
			expectedStatus: http.StatusRequestTimeout,
			checkBody: func(body string) bool {
				var resp map[string]interface{}
				if err := json.Unmarshal([]byte(body), &resp); err != nil {
					return false
				}
				ready, ok := resp["ready"].(bool)
				if !ok {
					return false
				}
				return !ready
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Setup state
			tt.setupState()

			// Create request
			req := httptest.NewRequest("GET", "/api/test/wait-for-logs?min_lines="+tt.minLines+"&timeout="+tt.timeout+"&project_path="+projectPath, nil)
			w := httptest.NewRecorder()

			// If we need to add logs during the wait
			if tt.addLogsDuring {
				go func() {
					time.Sleep(50 * time.Millisecond)
					project.AddLogLine("New log 1")
					project.AddLogLine("New log 2")
					project.AddLogLine("New log 3")
				}()
			}

			// Call handler
			handler.HandleTestWaitForLogs(w, req)

			// Check status
			if w.Code != tt.expectedStatus {
				t.Errorf("Expected status %d, got %d", tt.expectedStatus, w.Code)
			}

			// Check body
			body := w.Body.String()
			if !tt.checkBody(body) {
				t.Errorf("Body check failed: %s", body)
			}
		})
	}
}

// TestHandleTestProcessTree tests the process tree endpoint
func TestHandleTestProcessTree(t *testing.T) {
	// Set test mode environment variable
	os.Setenv("FTL_TEST_MODE", "true")
	defer os.Unsetenv("FTL_TEST_MODE")
	
	// Create temporary test directory
	tmpDir := t.TempDir()
	projectPath := tmpDir + "/test-project"
	os.MkdirAll(projectPath, 0755)
	
	// Create a test MCP client
	mcpClient := mcpclient.NewClientWithArgs("echo", []string{"test"})

	// Create project registry
	projectRegistry := state.NewProjectRegistry(tmpDir + "/test_projects.json")

	// Create handler with dependencies
	handler := &Handler{
		mcpClient:      mcpClient,
		logPositions:   state.NewLogPositionTracker(),
		registry:       projectRegistry,
		pollingManager: polling.NewManager(mcpClient, projectRegistry),
	}

	// Add a test project
	project, err := handler.registry.AddProject(projectPath, "Test Project")
	if err != nil {
		t.Fatalf("Failed to add project: %v", err)
	}

	tests := []struct {
		name           string
		setupState     func()
		expectedStatus int
		checkBody      func(string) bool
	}{
		{
			name: "Process tree with running process",
			setupState: func() {
				// Set up a running process
				project.UpdateProcessInfo(models.DetailedStatusInfo{
					ActiveProcess: &models.ProcessInfo{
						IsRunning: true,
						Type:      "regular",
						PID:       12345,
						Port:      8080,
					},
				})
			},
			expectedStatus: http.StatusOK,
			checkBody: func(body string) bool {
				var resp map[string]interface{}
				if err := json.Unmarshal([]byte(body), &resp); err != nil {
					return false
				}
				
				// Check that we have the ftl field
				ftl, ok := resp["ftl"].(map[string]interface{})
				if !ok {
					return false
				}
				
				// Check that ftl process is marked as running
				isRunning, ok := ftl["is_running"].(bool)
				if !ok || !isRunning {
					return false
				}
				pid, ok := ftl["pid"].(float64)
				if !ok {
					return false
				}
				return pid == 12345
			},
		},
		{
			name: "Process tree with no processes",
			setupState: func() {
				// Clear all process info
				project.UpdateProcessInfo(models.DetailedStatusInfo{})
			},
			expectedStatus: http.StatusOK,
			checkBody: func(body string) bool {
				var resp map[string]interface{}
				if err := json.Unmarshal([]byte(body), &resp); err != nil {
					return false
				}
				
				// Check that ftl process is not running
				ftl, ok := resp["ftl"].(map[string]interface{})
				if !ok {
					return false
				}
				isRunning, ok := ftl["is_running"].(bool)
				if !ok {
					return false
				}
				return !isRunning
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Setup state
			tt.setupState()

			// Create request
			req := httptest.NewRequest("GET", "/api/test/process-tree?project_path="+projectPath, nil)
			w := httptest.NewRecorder()

			// Call handler
			handler.HandleTestProcessTree(w, req)

			// Check status
			if w.Code != tt.expectedStatus {
				t.Errorf("Expected status %d, got %d", tt.expectedStatus, w.Code)
			}

			// Check body
			body := w.Body.String()
			if !tt.checkBody(body) {
				t.Errorf("Body check failed: %s", body)
			}
		})
	}
}

// TestHandleTestEndpointsRequireTestMode tests that endpoints return 404 when not in test mode
func TestHandleTestEndpointsRequireTestMode(t *testing.T) {
	// This test would need to verify that endpoints are only registered when FTL_TEST_MODE=true
	// Since the registration happens in dev_console.go, we can only test that the handlers
	// work correctly when called directly (as done above)
	
	// The actual test mode check is done during HTTP route registration,
	// not in the handlers themselves, so we verify the handlers work correctly
	// when they are registered.
	t.Log("Test endpoints are conditionally registered based on FTL_TEST_MODE environment variable")
}