package state

import (
	"testing"
	"time"

	"github.com/fastertools/ftl/internal/models"
)

func TestNewRingBuffer(t *testing.T) {
	rb := NewRingBuffer(5)
	
	if rb == nil {
		t.Fatal("Expected non-nil RingBuffer")
	}
	
	if rb.size != 5 {
		t.Errorf("Expected size 5, got %d", rb.size)
	}
	
	if rb.count != 0 {
		t.Errorf("Expected count 0, got %d", rb.count)
	}
	
	if len(rb.buffer) != 5 {
		t.Errorf("Expected buffer length 5, got %d", len(rb.buffer))
	}
}

func TestRingBufferAdd(t *testing.T) {
	rb := NewRingBuffer(3)
	
	// Add first item
	rb.Add("line1")
	if rb.count != 1 {
		t.Errorf("Expected count 1, got %d", rb.count)
	}
	
	// Add more items
	rb.Add("line2")
	rb.Add("line3")
	
	all := rb.GetAll()
	if len(all) != 3 {
		t.Errorf("Expected 3 lines, got %d", len(all))
	}
	
	// Verify order
	expected := []string{"line1", "line2", "line3"}
	for i, line := range all {
		if line != expected[i] {
			t.Errorf("Line %d: expected '%s', got '%s'", i, expected[i], line)
		}
	}
	
	// Add beyond capacity - should overwrite oldest
	rb.Add("line4")
	all = rb.GetAll()
	
	if len(all) != 3 {
		t.Errorf("Expected 3 lines after overflow, got %d", len(all))
	}
	
	// Should have dropped line1
	expected = []string{"line2", "line3", "line4"}
	for i, line := range all {
		if line != expected[i] {
			t.Errorf("After overflow line %d: expected '%s', got '%s'", i, expected[i], line)
		}
	}
}

func TestRingBufferGetRecent(t *testing.T) {
	rb := NewRingBuffer(5)
	
	// Add some lines
	for i := 1; i <= 5; i++ {
		rb.Add(string(rune('A' + i - 1)))
	}
	
	// Get recent 3
	recent := rb.GetRecent(3)
	if len(recent) != 3 {
		t.Errorf("Expected 3 recent lines, got %d", len(recent))
	}
	
	expected := []string{"C", "D", "E"}
	for i, line := range recent {
		if line != expected[i] {
			t.Errorf("Recent line %d: expected '%s', got '%s'", i, expected[i], line)
		}
	}
	
	// Get more than available
	recent = rb.GetRecent(10)
	if len(recent) != 5 {
		t.Errorf("Expected 5 lines when asking for 10, got %d", len(recent))
	}
	
	// Empty buffer
	rb2 := NewRingBuffer(5)
	recent = rb2.GetRecent(3)
	if len(recent) != 0 {
		t.Errorf("Expected 0 lines from empty buffer, got %d", len(recent))
	}
}

func TestRingBufferClear(t *testing.T) {
	rb := NewRingBuffer(3)
	
	rb.Add("line1")
	rb.Add("line2")
	rb.Add("line3")
	
	rb.Clear()
	
	if rb.count != 0 {
		t.Errorf("Expected count 0 after clear, got %d", rb.count)
	}
	
	if rb.head != 0 {
		t.Errorf("Expected head 0 after clear, got %d", rb.head)
	}
	
	all := rb.GetAll()
	if len(all) != 0 {
		t.Errorf("Expected 0 lines after clear, got %d", len(all))
	}
}

func TestRingBufferConcurrency(t *testing.T) {
	rb := NewRingBuffer(100)
	done := make(chan bool)
	
	// Writer goroutine
	go func() {
		for i := 0; i < 100; i++ {
			rb.Add("line")
		}
		done <- true
	}()
	
	// Reader goroutine
	go func() {
		for i := 0; i < 100; i++ {
			rb.GetAll()
		}
		done <- true
	}()
	
	// Wait for both
	<-done
	<-done
	
	// Should not panic or deadlock
}

func TestNewProjectState(t *testing.T) {
	project := Project{
		Name:       "test-project",
		Path:       "/test/path",
		AddedAt:    time.Now(),
		LastActive: time.Now(),
	}
	
	ps := NewProjectState(project)
	
	if ps == nil {
		t.Fatal("Expected non-nil ProjectState")
	}
	
	if ps.Project.Name != "test-project" {
		t.Errorf("Expected project name 'test-project', got %s", ps.Project.Name)
	}
	
	if ps.LogBuffer == nil {
		t.Fatal("Expected non-nil LogBuffer")
	}
	
	if ps.LogBuffer.size != 1000 {
		t.Errorf("Expected LogBuffer size 1000, got %d", ps.LogBuffer.size)
	}
	
	if ps.PollingActive {
		t.Error("Polling should not be active initially")
	}
	
	if len(ps.stopChannels) != 2 {
		t.Errorf("Expected 2 stop channels, got %d", len(ps.stopChannels))
	}
}

func TestProjectStatePolling(t *testing.T) {
	project := Project{
		Name: "test-project",
		Path: "/test/path",
	}
	
	ps := NewProjectState(project)
	
	// Start polling
	ps.StartPolling()
	if !ps.PollingActive {
		t.Error("Polling should be active after StartPolling")
	}
	
	// Stop polling
	ps.StopPolling()
	if ps.PollingActive {
		t.Error("Polling should not be active after StopPolling")
	}
	
	// Verify stop signals were sent
	statusChan := ps.GetStopChannel("status")
	select {
	case <-statusChan:
		// Good, received stop signal
	default:
		t.Error("Expected stop signal in status channel")
	}
}

func TestProjectStateProcessInfo(t *testing.T) {
	project := Project{
		Name: "test-project",
		Path: "/test/path",
	}
	
	ps := NewProjectState(project)
	
	// Initial state
	info := ps.GetProcessInfo()
	if info.ActiveProcess != nil && info.ActiveProcess.IsRunning {
		t.Error("No process should be running initially")
	}
	
	// Update process info
	newInfo := models.DetailedStatusInfo{
		ActiveProcess: &models.ProcessInfo{
			IsRunning: true,
			PID:       12345,
			Port:      8891,
			Type:      "regular",
		},
	}
	
	ps.UpdateProcessInfo(newInfo)
	
	// Verify update
	info = ps.GetProcessInfo()
	if info.ActiveProcess == nil || !info.ActiveProcess.IsRunning {
		t.Error("Process should be running after update")
	}
	if info.ActiveProcess.PID != 12345 {
		t.Errorf("Expected PID 12345, got %d", info.ActiveProcess.PID)
	}
	if info.ActiveProcess.Type != "regular" {
		t.Errorf("Expected type 'regular', got %s", info.ActiveProcess.Type)
	}
}

func TestProjectStateLogs(t *testing.T) {
	project := Project{
		Name: "test-project",
		Path: "/test/path",
	}
	
	ps := NewProjectState(project)
	
	// Add single log line
	ps.AddLogLine("log line 1")
	
	logs := ps.GetLogs()
	if len(logs) != 1 {
		t.Errorf("Expected 1 log line, got %d", len(logs))
	}
	
	// Add multiple log lines
	ps.AddLogLines([]string{"log line 2", "log line 3", "log line 4"})
	
	logs = ps.GetLogs()
	if len(logs) != 4 {
		t.Errorf("Expected 4 log lines, got %d", len(logs))
	}
	
	// Get recent logs
	recent := ps.GetRecentLogs(2)
	if len(recent) != 2 {
		t.Errorf("Expected 2 recent logs, got %d", len(recent))
	}
	
	// Clear logs
	ps.ClearLogs()
	logs = ps.GetLogs()
	if len(logs) != 0 {
		t.Errorf("Expected 0 logs after clear, got %d", len(logs))
	}
	
	if ps.GetLogPosition() != 0 {
		t.Error("Log position should be 0 after clear")
	}
}

func TestProjectStateLogPosition(t *testing.T) {
	project := Project{
		Name: "test-project",
		Path: "/test/path",
	}
	
	ps := NewProjectState(project)
	
	// Initial position
	if ps.GetLogPosition() != 0 {
		t.Errorf("Expected initial log position 0, got %d", ps.GetLogPosition())
	}
	
	// Update position
	ps.UpdateLogPosition(42)
	if ps.GetLogPosition() != 42 {
		t.Errorf("Expected log position 42, got %d", ps.GetLogPosition())
	}
}

func TestProjectStateCommandHistory(t *testing.T) {
	project := Project{
		Name: "test-project",
		Path: "/test/path",
	}
	
	ps := NewProjectState(project)
	
	// Add command output
	cmd1 := CommandOutput{
		Command:   "ftl build",
		Output:    "Build successful",
		Timestamp: time.Now(),
		Success:   true,
	}
	
	ps.AddCommandOutput(cmd1)
	
	history := ps.GetCommandHistory()
	if len(history) != 1 {
		t.Errorf("Expected 1 command in history, got %d", len(history))
	}
	
	if history[0].Command != "ftl build" {
		t.Errorf("Expected command 'ftl build', got %s", history[0].Command)
	}
	
	// Add more commands to test limit
	for i := 0; i < 25; i++ {
		ps.AddCommandOutput(CommandOutput{
			Command:   "test",
			Output:    "output",
			Timestamp: time.Now(),
			Success:   true,
		})
	}
	
	history = ps.GetCommandHistory()
	if len(history) != 20 {
		t.Errorf("Expected history capped at 20, got %d", len(history))
	}
}