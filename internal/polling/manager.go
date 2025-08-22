package polling

import (
	"log"
	"strings"
	"time"

	"github.com/fastertools/ftl/internal/parser"
	"github.com/fastertools/ftl/internal/state"
)

// MCPClient interface for MCP operations
type MCPClient interface {
	CallTool(tool string, args map[string]interface{}) (string, error)
}

// Manager handles background polling for all projects
type Manager struct {
	mcpClient MCPClient
	registry  *state.ProjectRegistry
}

// NewManager creates a new polling manager
func NewManager(mcpClient MCPClient, registry *state.ProjectRegistry) *Manager {
	return &Manager{
		mcpClient: mcpClient,
		registry:  registry,
	}
}

// StartProjectPolling starts background polling for a project
func (m *Manager) StartProjectPolling(projectPath string) {
	ps, exists := m.registry.GetProject(projectPath)
	if !exists {
		log.Printf("Project not found for polling: %s", projectPath)
		return
	}

	// Mark polling as active
	ps.StartPolling()

	// Add delay to allow MCP client initialization to complete
	go func() {
		time.Sleep(5 * time.Second)
		m.pollStatus(ps)
	}()

	go func() {
		time.Sleep(5 * time.Second)
		m.pollLogs(ps)
	}()

	log.Printf("Started polling for project: %s", projectPath)
}

// StopProjectPolling stops background polling for a project
func (m *Manager) StopProjectPolling(projectPath string) {
	ps, exists := m.registry.GetProject(projectPath)
	if !exists {
		return
	}

	ps.StopPolling()
	log.Printf("Stopped polling for project: %s", projectPath)
}

// pollStatus continuously polls for process status
func (m *Manager) pollStatus(ps *state.ProjectState) {
	ticker := time.NewTicker(3 * time.Second)
	defer ticker.Stop()

	stopChan := ps.GetStopChannel("status")

	for {
		select {
		case <-stopChan:
			log.Printf("Status polling stopped for: %s", ps.Project.Path)
			return
		case <-ticker.C:
			if !ps.PollingActive {
				return
			}

			// Call MCP to get status
			args := map[string]interface{}{
				"project_path": ps.Project.Path,
				"detailed":     true,
			}

			result, err := m.mcpClient.CallTool("mcp-server__get_status", args)
			if err != nil {
				// Don't log every error to avoid spam
				continue
			}

			// Parse the detailed status response
			detailedStatus := parser.ParseDetailedStatusResponse(result, ps.Project.Path)
			
			// Update project state
			ps.UpdateProcessInfo(detailedStatus)
		}
	}
}

// pollLogs continuously polls for new logs
func (m *Manager) pollLogs(ps *state.ProjectState) {
	ticker := time.NewTicker(2 * time.Second)
	defer ticker.Stop()

	stopChan := ps.GetStopChannel("logs")

	for {
		select {
		case <-stopChan:
			log.Printf("Log polling stopped for: %s", ps.Project.Path)
			return
		case <-ticker.C:
			if !ps.PollingActive {
				return
			}

			// Get current log position
			position := ps.GetLogPosition()

			// Call MCP to get logs
			args := map[string]interface{}{
				"project_path": ps.Project.Path,
				"since":        position,
			}

			result, err := m.mcpClient.CallTool("mcp-server__get_logs", args)
			if err != nil {
				// Don't log every error to avoid spam
				continue
			}

			// Parse the log response
			logResponse := parser.ParseLogResponse(result, ps.Project.Path, position)

			// If we have new logs, add them to the buffer
			if logResponse.HasNewContent && logResponse.NewLogs != "" {
				// Split logs into lines and add to buffer
				lines := strings.Split(logResponse.NewLogs, "\n")
				for _, line := range lines {
					if line != "" {
						ps.AddLogLine(line)
					}
				}
				
				// Update position
				ps.UpdateLogPosition(logResponse.LogPosition)
			}
		}
	}
}

// StartAllPolling starts polling for all existing projects
func (m *Manager) StartAllPolling() {
	projects := m.registry.GetAllProjects()
	for _, project := range projects {
		m.StartProjectPolling(project.Path)
	}
}