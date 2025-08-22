package state

import "sync"

// LogPositionTracker tracks log positions for each project
type LogPositionTracker struct {
	mu        sync.RWMutex
	positions map[string]int // projectPath -> lastLogPosition
}

// NewLogPositionTracker creates a new log position tracker
func NewLogPositionTracker() *LogPositionTracker {
	return &LogPositionTracker{
		positions: make(map[string]int),
	}
}

// GetPosition returns the current log position for a project
func (t *LogPositionTracker) GetPosition(projectPath string) int {
	t.mu.RLock()
	defer t.mu.RUnlock()
	return t.positions[projectPath]
}

// SetPosition updates the log position for a project
func (t *LogPositionTracker) SetPosition(projectPath string, position int) {
	t.mu.Lock()
	defer t.mu.Unlock()
	t.positions[projectPath] = position
}