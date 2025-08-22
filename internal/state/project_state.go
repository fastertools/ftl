package state

import (
	"sync"
	"time"

	"github.com/fastertools/ftl/web-templates"
	"github.com/fastertools/ftl/internal/models"
)

// CommandOutput represents output from an executed command
type CommandOutput struct {
	Command   string
	Output    string
	Timestamp time.Time
	Success   bool
}

// RingBuffer implements a circular buffer for log storage
type RingBuffer struct {
	buffer   []string
	size     int
	head     int
	count    int
	mu       sync.RWMutex
}

// NewRingBuffer creates a new ring buffer with specified size
func NewRingBuffer(size int) *RingBuffer {
	return &RingBuffer{
		buffer: make([]string, size),
		size:   size,
		head:   0,
		count:  0,
	}
}

// Add adds a new line to the buffer
func (rb *RingBuffer) Add(line string) {
	rb.mu.Lock()
	defer rb.mu.Unlock()

	rb.buffer[rb.head] = line
	rb.head = (rb.head + 1) % rb.size
	if rb.count < rb.size {
		rb.count++
	}
}

// GetAll returns all lines in the buffer in order
func (rb *RingBuffer) GetAll() []string {
	rb.mu.RLock()
	defer rb.mu.RUnlock()

	if rb.count == 0 {
		return []string{}
	}

	result := make([]string, rb.count)
	if rb.count < rb.size {
		// Buffer not full yet, simple copy
		copy(result, rb.buffer[:rb.count])
	} else {
		// Buffer is full, need to reconstruct order
		tail := rb.size - rb.head
		copy(result, rb.buffer[rb.head:])
		copy(result[tail:], rb.buffer[:rb.head])
	}
	return result
}

// GetRecent returns the most recent n lines
func (rb *RingBuffer) GetRecent(n int) []string {
	all := rb.GetAll()
	if len(all) <= n {
		return all
	}
	return all[len(all)-n:]
}

// Clear empties the buffer
func (rb *RingBuffer) Clear() {
	rb.mu.Lock()
	defer rb.mu.Unlock()
	
	rb.head = 0
	rb.count = 0
	rb.buffer = make([]string, rb.size)
}

// ProjectState maintains the complete state for a single project
type ProjectState struct {
	Project         Project
	ProcessInfo     models.DetailedStatusInfo
	LogBuffer       *RingBuffer
	LogPosition     int
	CommandHistory  []CommandOutput
	PollingActive   bool
	stopChannels    map[string]chan bool
	mu              sync.RWMutex
}

// NewProjectState creates a new project state
func NewProjectState(project Project) *ProjectState {
	return &ProjectState{
		Project:      project,
		ProcessInfo:  models.DetailedStatusInfo{},
		LogBuffer:    NewRingBuffer(1000), // 1000 lines max
		LogPosition:  0,
		CommandHistory: make([]CommandOutput, 0),
		PollingActive: false,
		stopChannels: map[string]chan bool{
			"status": make(chan bool, 1),
			"logs":   make(chan bool, 1),
		},
	}
}

// StartPolling marks polling as active
func (ps *ProjectState) StartPolling() {
	ps.mu.Lock()
	defer ps.mu.Unlock()
	ps.PollingActive = true
}

// StopPolling stops all polling goroutines
func (ps *ProjectState) StopPolling() {
	ps.mu.Lock()
	defer ps.mu.Unlock()
	
	ps.PollingActive = false
	
	// Send stop signals
	for _, ch := range ps.stopChannels {
		select {
		case ch <- true:
		default:
			// Channel might be full, that's ok
		}
	}
}

// GetStopChannel returns the stop channel for a specific poller
func (ps *ProjectState) GetStopChannel(name string) chan bool {
	ps.mu.RLock()
	defer ps.mu.RUnlock()
	return ps.stopChannels[name]
}

// UpdateProcessInfo updates the process status information
func (ps *ProjectState) UpdateProcessInfo(info models.DetailedStatusInfo) {
	ps.mu.Lock()
	defer ps.mu.Unlock()
	ps.ProcessInfo = info
}

// GetProcessInfo returns the current process information
func (ps *ProjectState) GetProcessInfo() models.DetailedStatusInfo {
	ps.mu.RLock()
	defer ps.mu.RUnlock()
	return ps.ProcessInfo
}

// AddLogLine adds a new log line to the buffer
func (ps *ProjectState) AddLogLine(line string) {
	ps.LogBuffer.Add(line)
}

// AddLogLines adds multiple log lines to the buffer
func (ps *ProjectState) AddLogLines(lines []string) {
	for _, line := range lines {
		ps.LogBuffer.Add(line)
	}
}

// GetLogs returns all logs from the buffer
func (ps *ProjectState) GetLogs() []string {
	return ps.LogBuffer.GetAll()
}

// GetRecentLogs returns the most recent n log lines
func (ps *ProjectState) GetRecentLogs(n int) []string {
	return ps.LogBuffer.GetRecent(n)
}

// UpdateLogPosition updates the log polling position
func (ps *ProjectState) UpdateLogPosition(position int) {
	ps.mu.Lock()
	defer ps.mu.Unlock()
	ps.LogPosition = position
}

// GetLogPosition returns the current log position
func (ps *ProjectState) GetLogPosition() int {
	ps.mu.RLock()
	defer ps.mu.RUnlock()
	return ps.LogPosition
}

// AddCommandOutput adds a command output to history
func (ps *ProjectState) AddCommandOutput(cmd CommandOutput) {
	ps.mu.Lock()
	defer ps.mu.Unlock()
	
	ps.CommandHistory = append(ps.CommandHistory, cmd)
	
	// Keep only last 20 commands
	if len(ps.CommandHistory) > 20 {
		ps.CommandHistory = ps.CommandHistory[len(ps.CommandHistory)-20:]
	}
}

// GetCommandHistory returns the command history
func (ps *ProjectState) GetCommandHistory() []CommandOutput {
	ps.mu.RLock()
	defer ps.mu.RUnlock()
	
	result := make([]CommandOutput, len(ps.CommandHistory))
	copy(result, ps.CommandHistory)
	return result
}

// ClearLogs clears the log buffer
func (ps *ProjectState) ClearLogs() {
	ps.LogBuffer.Clear()
	ps.mu.Lock()
	ps.LogPosition = 0
	ps.mu.Unlock()
}


// ToComponentTypes converts ProjectState to templates types directly
func (ps *ProjectState) ToComponentTypes() (templates.Project, templates.ProcessStatus, templates.ServerStatus, []templates.LogEntry) {
	ps.mu.RLock()
	defer ps.mu.RUnlock()
	
	// Convert state.Project to templates.Project
	project := templates.Project{
		Name:        ps.Project.Name,
		Path:        ps.Project.Path,
		Status:      ps.deriveStatus(),
		LastUpdated: ps.Project.LastActive,
	}
	
	// Convert DetailedStatusInfo to templates.ProcessStatus
	processStatus := templates.ProcessStatus{
		FTLRunning:   ps.ProcessInfo.ActiveProcess != nil && ps.ProcessInfo.ActiveProcess.IsRunning && ps.ProcessInfo.ActiveProcess.Type == "regular",
		WatchRunning: ps.ProcessInfo.ActiveProcess != nil && ps.ProcessInfo.ActiveProcess.IsRunning && ps.ProcessInfo.ActiveProcess.Type == "watch",
		LastActivity: time.Now(), // Use current time or last update time
	}
	
	// Create templates.ServerStatus
	serverStatus := templates.ServerStatus{
		Running:     ps.ProcessInfo.ActiveProcess != nil && ps.ProcessInfo.ActiveProcess.IsRunning,
		ProcessID:   ps.getActiveProcessID(),
		Port:        ps.getActivePort(),
		LastChecked: time.Now(),
	}
	
	// Convert string logs to templates.LogEntry structs
	logs := ps.convertLogsToEntries()
	
	return project, processStatus, serverStatus, logs
}

// Helper method to derive simple status string
func (ps *ProjectState) deriveStatus() string {
	if ps.ProcessInfo.ActiveProcess != nil && ps.ProcessInfo.ActiveProcess.IsRunning {
		return "Running"
	}
	return "Stopped"
}

// Helper method to get active port
func (ps *ProjectState) getActivePort() int {
	if ps.ProcessInfo.ActiveProcess != nil && ps.ProcessInfo.ActiveProcess.IsRunning && ps.ProcessInfo.ActiveProcess.Port > 0 {
		return ps.ProcessInfo.ActiveProcess.Port
	}
	return 0
}

// Helper method to get active process ID
func (ps *ProjectState) getActiveProcessID() int {
	if ps.ProcessInfo.ActiveProcess != nil && ps.ProcessInfo.ActiveProcess.IsRunning && ps.ProcessInfo.ActiveProcess.PID > 0 {
		return ps.ProcessInfo.ActiveProcess.PID
	}
	return 0
}

// Helper method to convert logs to templates.LogEntry structs
func (ps *ProjectState) convertLogsToEntries() []templates.LogEntry {
	logs := ps.LogBuffer.GetRecent(20) // Get recent 20 logs
	entries := make([]templates.LogEntry, len(logs))
	
	for i, log := range logs {
		entries[i] = templates.LogEntry{
			ID:        i,
			Timestamp: time.Now(), // Could parse timestamp from log if available
			Level:     "INFO",     // Could parse level from log if available
			Message:   log,
		}
	}
	
	return entries
}