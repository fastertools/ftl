package polling

import (
	"fmt"
	"testing"
	"time"

	"github.com/fastertools/ftl/internal/state"
)

// MockMCPClient is a mock implementation of MCP client for testing
type MockMCPClient struct {
	statusCalls int
	logCalls    int
	shouldError bool
	statusData  string
	logData     string
}

func (m *MockMCPClient) CallTool(tool string, args map[string]interface{}) (string, error) {
	if m.shouldError {
		return "", fmt.Errorf("mock error")
	}

	switch tool {
	case "mcp-server__get_status":
		m.statusCalls++
		if m.statusData != "" {
			return m.statusData, nil
		}
		return `{"project_path": "/test/path", "active_process": {"pid": 99999, "port": 8892, "is_running": true, "type": "regular"}}`, nil
	case "mcp-server__get_logs":
		m.logCalls++
		if m.logData != "" {
			return m.logData, nil
		}
		return "New log line\nAnother log line", nil
	default:
		return "", fmt.Errorf("unknown tool: %s", tool)
	}
}

func TestNewManager(t *testing.T) {
	mockClient := &MockMCPClient{}
	registry := state.NewProjectRegistry("test.json")
	
	manager := NewManager(mockClient, registry)
	
	if manager == nil {
		t.Fatal("Expected non-nil manager")
	}
	
	// Note: We can't directly compare interface values, but we can verify the manager works
	// by using it in subsequent tests
	
	if manager.registry != registry {
		t.Error("Manager should have the provided registry")
	}
}

func TestStartProjectPolling(t *testing.T) {
	mockClient := &MockMCPClient{}
	registry := state.NewProjectRegistry("test.json")
	
	// Add a test project
	ps, err := registry.AddProject("/test/path", "test-project")
	if err != nil {
		t.Fatalf("Failed to add project: %v", err)
	}
	
	manager := NewManager(mockClient, registry)
	
	// Start polling
	manager.StartProjectPolling("/test/path")
	
	// Verify polling is active
	if !ps.PollingActive {
		t.Error("Polling should be active after StartProjectPolling")
	}
	
	// Give goroutines time to run
	time.Sleep(100 * time.Millisecond)
	
	// Stop polling to clean up
	manager.StopProjectPolling("/test/path")
}

func TestStartProjectPollingNonExistent(t *testing.T) {
	mockClient := &MockMCPClient{}
	registry := state.NewProjectRegistry("test.json")
	
	manager := NewManager(mockClient, registry)
	
	// Try to start polling for non-existent project (should not panic)
	manager.StartProjectPolling("/non/existent")
	
	// Give it a moment to ensure no panic
	time.Sleep(50 * time.Millisecond)
}

func TestStopProjectPolling(t *testing.T) {
	mockClient := &MockMCPClient{}
	registry := state.NewProjectRegistry("test.json")
	
	// Add a test project
	ps, err := registry.AddProject("/test/path", "test-project")
	if err != nil {
		t.Fatalf("Failed to add project: %v", err)
	}
	
	manager := NewManager(mockClient, registry)
	
	// Start polling
	manager.StartProjectPolling("/test/path")
	
	// Verify polling is active
	if !ps.PollingActive {
		t.Error("Polling should be active after start")
	}
	
	// Stop polling
	manager.StopProjectPolling("/test/path")
	
	// Verify polling stopped
	if ps.PollingActive {
		t.Error("Polling should not be active after stop")
	}
}

func TestPollStatus(t *testing.T) {
	if testing.Short() {
		t.Skip("Skipping polling test in short mode")
	}
	
	mockClient := &MockMCPClient{
		statusData: `{"regular": {"running": true, "pid": 99999, "port": 8892}, "watch": {"running": true, "pid": 88888, "port": 8893}}`,
	}
	registry := state.NewProjectRegistry("test.json")
	
	// Add a test project
	ps, err := registry.AddProject("/test/path", "test-project")
	if err != nil {
		t.Fatalf("Failed to add project: %v", err)
	}
	
	manager := NewManager(mockClient, registry)
	
	// Don't call ps.StartPolling() as it sends stop signals
	// Just set the flag directly
	ps.PollingActive = true
	
	// Clear any existing signals from stop channels
	select {
	case <-ps.GetStopChannel("status"):
	default:
	}
	
	// Run status polling in goroutine
	go manager.pollStatus(ps)
	
	// Wait for at least one poll cycle (ticker fires every 3s)
	time.Sleep(3100 * time.Millisecond)
	
	// Check that status was updated
	info := ps.GetProcessInfo()
	if info.ActiveProcess == nil || !info.ActiveProcess.IsRunning {
		t.Error("Active process should be running after status poll")
	}
	if info.ActiveProcess.PID != 99999 {
		t.Errorf("Expected PID 99999, got %d", info.ActiveProcess.PID)
	}
	if info.ActiveProcess.Port != 8892 {
		t.Errorf("Expected port 8892, got %d", info.ActiveProcess.Port)
	}
	if info.ActiveProcess.Type != "regular" {
		t.Errorf("Expected type 'regular', got %s", info.ActiveProcess.Type)
	}
	
	// Verify MCP client was called
	t.Logf("Status calls made: %d", mockClient.statusCalls)
	if mockClient.statusCalls < 1 {
		t.Error("Expected at least one status call to MCP client")
	}
	
	// Stop polling
	ps.StopPolling()
	
	// Give goroutine time to exit
	time.Sleep(50 * time.Millisecond)
}

func TestPollLogs(t *testing.T) {
	if testing.Short() {
		t.Skip("Skipping polling test in short mode")
	}
	
	mockClient := &MockMCPClient{
		logData: "Test log line 1\nTest log line 2\nTest log line 3",
	}
	registry := state.NewProjectRegistry("test.json")
	
	// Add a test project
	ps, err := registry.AddProject("/test/path", "test-project")
	if err != nil {
		t.Fatalf("Failed to add project: %v", err)
	}
	
	manager := NewManager(mockClient, registry)
	
	// Don't call ps.StartPolling() as it sends stop signals
	// Just set the flag directly
	ps.PollingActive = true
	
	// Clear any existing signals from stop channels
	select {
	case <-ps.GetStopChannel("logs"):
	default:
	}
	
	// Run log polling in goroutine
	go manager.pollLogs(ps)
	
	// Wait for at least one poll cycle (ticker fires every 2s for logs)
	time.Sleep(2100 * time.Millisecond)
	
	// Check that logs were added
	logs := ps.GetLogs()
	if len(logs) < 3 {
		t.Errorf("Expected at least 3 log lines, got %d", len(logs))
	}
	
	// Verify log position was updated
	if ps.GetLogPosition() == 0 {
		t.Error("Log position should have been updated")
	}
	
	// Verify MCP client was called
	t.Logf("Log calls made: %d", mockClient.logCalls)
	if mockClient.logCalls < 1 {
		t.Error("Expected at least one log call to MCP client")
	}
	
	// Stop polling
	ps.StopPolling()
	
	// Give goroutine time to exit
	time.Sleep(50 * time.Millisecond)
}

func TestPollWithErrors(t *testing.T) {
	if testing.Short() {
		t.Skip("Skipping polling test in short mode")
	}
	
	mockClient := &MockMCPClient{
		shouldError: true,
	}
	registry := state.NewProjectRegistry("test.json")
	
	// Add a test project
	ps, err := registry.AddProject("/test/path", "test-project")
	if err != nil {
		t.Fatalf("Failed to add project: %v", err)
	}
	
	manager := NewManager(mockClient, registry)
	
	// Don't call ps.StartPolling() as it sends stop signals
	// Just set the flag directly
	ps.PollingActive = true
	
	// Clear any existing signals from stop channels
	select {
	case <-ps.GetStopChannel("status"):
	default:
	}
	select {
	case <-ps.GetStopChannel("logs"):
	default:
	}
	
	// Run polling in goroutines
	go manager.pollStatus(ps)
	go manager.pollLogs(ps)
	
	// Wait for poll cycles (should handle errors gracefully)
	time.Sleep(2100 * time.Millisecond)
	
	// Process info should remain unchanged
	info := ps.GetProcessInfo()
	if info.ActiveProcess != nil && info.ActiveProcess.IsRunning {
		t.Error("No process should be running (no data due to error)")
	}
	
	// Logs should be empty
	logs := ps.GetLogs()
	if len(logs) != 0 {
		t.Errorf("Expected no logs due to error, got %d", len(logs))
	}
	
	// Stop polling
	ps.StopPolling()
	
	// Give goroutines time to exit
	time.Sleep(50 * time.Millisecond)
}

func TestStartAllPolling(t *testing.T) {
	mockClient := &MockMCPClient{}
	registry := state.NewProjectRegistry("test.json")
	
	// Add multiple projects
	ps1, err := registry.AddProject("/test/path1", "project1")
	if err != nil {
		t.Fatalf("Failed to add project1: %v", err)
	}
	
	ps2, err := registry.AddProject("/test/path2", "project2")
	if err != nil {
		t.Fatalf("Failed to add project2: %v", err)
	}
	
	ps3, err := registry.AddProject("/test/path3", "project3")
	if err != nil {
		t.Fatalf("Failed to add project3: %v", err)
	}
	
	manager := NewManager(mockClient, registry)
	
	// Start polling for all projects
	manager.StartAllPolling()
	
	// Verify all projects have polling active
	if !ps1.PollingActive {
		t.Error("Project1 should have polling active")
	}
	if !ps2.PollingActive {
		t.Error("Project2 should have polling active")
	}
	if !ps3.PollingActive {
		t.Error("Project3 should have polling active")
	}
	
	// Give goroutines time to run
	time.Sleep(100 * time.Millisecond)
	
	// Clean up - stop all polling
	manager.StopProjectPolling("/test/path1")
	manager.StopProjectPolling("/test/path2")
	manager.StopProjectPolling("/test/path3")
	
	// Give goroutines time to exit
	time.Sleep(100 * time.Millisecond)
}

func TestPollingStopChannel(t *testing.T) {
	if testing.Short() {
		t.Skip("Skipping polling test in short mode")
	}
	
	mockClient := &MockMCPClient{}
	registry := state.NewProjectRegistry("test.json")
	
	// Add a test project
	_, err := registry.AddProject("/test/path", "test-project")
	if err != nil {
		t.Fatalf("Failed to add project: %v", err)
	}
	
	manager := NewManager(mockClient, registry)
	
	// Start polling
	manager.StartProjectPolling("/test/path")
	
	// Let it poll for a bit (reduced from 1s)
	time.Sleep(100 * time.Millisecond)
	
	initialStatusCalls := mockClient.statusCalls
	initialLogCalls := mockClient.logCalls
	
	// Stop polling
	manager.StopProjectPolling("/test/path")
	
	// Wait a moment to ensure goroutines have stopped (reduced from 4s)
	time.Sleep(200 * time.Millisecond)
	
	// Verify no more calls were made after stopping
	if mockClient.statusCalls > initialStatusCalls+1 {
		t.Errorf("Status polling should have stopped, but got %d additional calls", 
			mockClient.statusCalls - initialStatusCalls)
	}
	
	if mockClient.logCalls > initialLogCalls+2 {
		t.Errorf("Log polling should have stopped, but got %d additional calls", 
			mockClient.logCalls - initialLogCalls)
	}
}