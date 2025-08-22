package handlers

import (
	"encoding/json"
	"fmt"
	"log"
	"net/http"
	"os"
	"strings"
	"time"

	"github.com/fastertools/ftl/web-templates"
	"github.com/fastertools/ftl/internal/mcpclient"
	"github.com/fastertools/ftl/internal/models"
	"github.com/fastertools/ftl/internal/parser"
	"github.com/fastertools/ftl/internal/polling"
	"github.com/fastertools/ftl/internal/state"
)

// formatTimestamp returns a consistent timestamp format for all messages
func formatTimestamp() string {
	return time.Now().Format("15:04:05")
}

// formatMessage creates a consistently formatted message with timestamp
func formatMessage(color, message string) string {
	return fmt.Sprintf(`<div class="%s">[%s] %s</div>`, color, formatTimestamp(), message)
}

// Handler encapsulates all HTTP handlers
type Handler struct {
	mcpClient      *mcpclient.Client
	logPositions   *state.LogPositionTracker
	registry       *state.ProjectRegistry
	pollingManager *polling.Manager
}

// NewHandler creates a new handler instance
func NewHandler(mcpClient *mcpclient.Client) *Handler {
	// Get projects file from environment or use default
	projectsFile := os.Getenv("PROJECTS_FILE")
	if projectsFile == "" {
		projectsFile = "projects.json"
	}
	log.Printf("Using projects file: %s", projectsFile)
	
	// Create project registry
	registry := state.NewProjectRegistry(projectsFile)
	
	// Load existing projects
	if err := registry.LoadProjects(); err != nil {
		log.Printf("Warning: failed to load projects: %v", err)
	}
	
	// If no projects exist, add the default one
	allProjects := registry.GetAllProjects()
	log.Printf("After loading projects: found %d projects", len(allProjects))
	if len(allProjects) == 0 {
		log.Printf("No projects found, adding default project")
		if _, err := registry.AddProject("/Users/coreyryan/data/mashh/ftl-tool-think", "ftl-tool-think"); err != nil {
			log.Printf("Warning: failed to add default project: %v", err)
		}
	} else {
		log.Printf("Found existing projects, skipping default project creation")
	}
	
	// Create polling manager
	pollingManager := polling.NewManager(mcpClient, registry)
	
	// Start polling for all existing projects
	pollingManager.StartAllPolling()
	
	return &Handler{
		mcpClient:      mcpClient,
		logPositions:   state.NewLogPositionTracker(),
		registry:       registry,
		pollingManager: pollingManager,
	}
}

// HandleIndex serves the main page
func (h *Handler) HandleIndex(w http.ResponseWriter, r *http.Request) {
	// Get current project from registry
	currentProj, exists := h.registry.GetCurrentProject()
	if !exists {
		http.Error(w, "No projects configured", http.StatusInternalServerError)
		return
	}
	
	// Get all projects for sidebar
	allProjects := h.registry.GetAllProjects()
	
	// Convert to components.Project with process status
	projects := make([]templates.Project, len(allProjects))
	for i, p := range allProjects {
		// Get project state to access process info
		projectState, exists := h.registry.GetProject(p.Path)
		var ftlRunning, watchRunning bool
		if exists && projectState != nil && projectState.ProcessInfo.ActiveProcess != nil {
			ftlRunning = projectState.ProcessInfo.ActiveProcess.IsRunning && projectState.ProcessInfo.ActiveProcess.Type == "regular"
			watchRunning = projectState.ProcessInfo.ActiveProcess.IsRunning && projectState.ProcessInfo.ActiveProcess.Type == "watch"
		}
		
		projects[i] = templates.Project{
			Name:         p.Name,
			Path:         p.Path,
			Status:       "active",
			LastUpdated:  p.LastActive,
			FTLRunning:   ftlRunning,
			WatchRunning: watchRunning,
		}
	}
	
	// Get command history from current project state
	stateCommandHistory := currentProj.GetCommandHistory()
	componentCommandHistory := make([]templates.CommandOutput, len(stateCommandHistory))
	for i, cmd := range stateCommandHistory {
		componentCommandHistory[i] = templates.CommandOutput{
			Command:   cmd.Command,
			Output:    cmd.Output,
			Timestamp: cmd.Timestamp,
			Success:   cmd.Success,
		}
	}
	
	// Get the actual FTL process port if running
	var ftlPort int
	if currentProj.ProcessInfo.ActiveProcess != nil && currentProj.ProcessInfo.ActiveProcess.IsRunning {
		ftlPort = currentProj.ProcessInfo.ActiveProcess.Port
	}
	
	// Use templ components for rendering
	dashboardData := templates.DashboardData{
		CurrentProject: templates.Project{
			Name:        currentProj.Project.Name,
			Path:        currentProj.Project.Path,
			Status:      "active",
			LastUpdated: currentProj.Project.LastActive,
		},
		AllProjects:    projects,
		ServerStatus: templates.ServerStatus{
			Running:     currentProj.ProcessInfo.ActiveProcess != nil && currentProj.ProcessInfo.ActiveProcess.IsRunning,
			ProcessID:   os.Getpid(),
			Port:        ftlPort,
			LastChecked: time.Now(),
		},
		ProcessStatus: templates.ProcessStatus{
			FTLRunning:   currentProj.ProcessInfo.ActiveProcess != nil && currentProj.ProcessInfo.ActiveProcess.IsRunning && currentProj.ProcessInfo.ActiveProcess.Type == "regular",
			WatchRunning: currentProj.ProcessInfo.ActiveProcess != nil && currentProj.ProcessInfo.ActiveProcess.IsRunning && currentProj.ProcessInfo.ActiveProcess.Type == "watch",
			LastActivity: time.Now(),
		},
		RecentLogs:     []templates.LogEntry{},
		CommandHistory: componentCommandHistory,
	}
	
	w.Header().Set("Content-Type", "text/html")
	if err := templates.Layout(dashboardData).Render(r.Context(), w); err != nil {
		http.Error(w, fmt.Sprintf("Template error: %v", err), http.StatusInternalServerError)
		return
	}
}

// HandleMCP handles MCP form submissions (legacy endpoint)
func (h *Handler) HandleMCP(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}
	
	// Parse form data
	if err := r.ParseForm(); err != nil {
		http.Error(w, "Failed to parse form", http.StatusBadRequest)
		return
	}
	
	// Get current project
	currentProj, exists := h.registry.GetCurrentProject()
	if !exists {
		http.Error(w, "No projects configured", http.StatusInternalServerError)
		return
	}
	
	// Handle build action
	if r.FormValue("action") == "build" {
		projectPath := r.FormValue("project-path")
		if projectPath == "" {
			projectPath = currentProj.Project.Path
		}
		
		args := map[string]interface{}{
			"project_path": projectPath,
			"clean":        false,
		}
		
		result, err := h.mcpClient.CallTool("mcp-server__build", args)
		if err != nil {
			fmt.Fprint(w, formatMessage("text-red-400 mb-1", fmt.Sprintf("Build failed: %s", err.Error())))
			return
		}
		
		// Parse the build response JSON
		buildResponse := parser.ParseBuildResponse(result)
		
		w.Header().Set("Content-Type", "text/html")
		if buildResponse.Success {
			fmt.Fprint(w, formatMessage("text-green-400 mb-1", fmt.Sprintf("Build successful: %s", buildResponse.Output)))
		} else {
			fmt.Fprint(w, formatMessage("text-red-400 mb-1", fmt.Sprintf("Build failed: %s", buildResponse.Error)))
		}
		return
	}
	
	// Handle stop actions for specific processes
	if r.FormValue("stop_regular") == "true" {
		projectPath := r.FormValue("project-path")
		if projectPath == "" {
			projectPath = currentProj.Project.Path
		}
		
		result, err := h.mcpClient.CallTool("mcp-server__stop", map[string]interface{}{
			"project_path": projectPath,
		})
		if err != nil {
			fmt.Fprint(w, formatMessage("text-red-400 mb-1", fmt.Sprintf("Stop FTL failed: %s", err.Error())))
			return
		}
		
		w.Header().Set("Content-Type", "text/html")
		fmt.Fprint(w, formatMessage("text-orange-400 mb-1", fmt.Sprintf("FTL Process stopped: %s", result)))
		return
	}
	
	if r.FormValue("stop_watch") == "true" {
		projectPath := r.FormValue("project-path")
		if projectPath == "" {
			projectPath = currentProj.Project.Path
		}
		
		result, err := h.mcpClient.CallTool("mcp-server__stop", map[string]interface{}{
			"project_path": projectPath,
		})
		if err != nil {
			fmt.Fprint(w, formatMessage("text-red-400 mb-1", fmt.Sprintf("Stop Watch failed: %s", err.Error())))
			return
		}
		
		w.Header().Set("Content-Type", "text/html")
		fmt.Fprint(w, formatMessage("text-orange-400 mb-1", fmt.Sprintf("Watch Mode stopped: %s", result)))
		return
	}
	
	// Handle unified FTL up with mode parameter
	if projectPath := r.FormValue("project-path"); projectPath != "" {
		mode := r.FormValue("mode")
		if mode == "" {
			mode = "regular" // Default to regular mode
		}
		
		isWatchMode := mode == "watch"
		
		args := map[string]interface{}{
			"project_path": projectPath,
			"watch":        isWatchMode,
			"build":        false,
			"listen":       "",
		}
		
		result, err := h.mcpClient.CallTool("mcp-server__up", args)
		if err != nil {
			modeLabel := "Start"
			if isWatchMode {
				modeLabel = "Watch start"
			}
			fmt.Fprint(w, formatMessage("text-red-400 mb-1", fmt.Sprintf("%s failed: %s", modeLabel, err.Error())))
			return
		}
		
		// Parse the up response JSON
		upResponse := parser.ParseUpResponse(result)
		
		w.Header().Set("Content-Type", "text/html")
		if upResponse.Success {
			// Start polling for this project if server started successfully
			h.pollingManager.StartProjectPolling(projectPath)
			if isWatchMode {
				fmt.Fprint(w, formatMessage("text-blue-400 mb-1", upResponse.Message))
			} else {
				fmt.Fprint(w, formatMessage("text-green-400 mb-1", upResponse.Message))
			}
		} else {
			fmt.Fprint(w, formatMessage("text-red-400 mb-1", upResponse.Error))
		}
		return
	}
	
	http.Error(w, "No valid action specified", http.StatusBadRequest)
}

// HandleFTLStart handles starting FTL in regular mode
func (h *Handler) HandleFTLStart(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}
	
	projectPath := r.FormValue("project_path")
	if projectPath == "" {
		w.WriteHeader(http.StatusBadRequest)
		fmt.Fprint(w, `<div class="text-red-400">Error: project_path required</div>`)
		return
	}
	
	args := map[string]interface{}{
		"project_path": projectPath,
		"watch":        false,
		"build":        false,
		"listen":       "",
	}
	
	result, err := h.mcpClient.CallTool("mcp-server__up", args)
	if err != nil {
		fmt.Fprintf(w, `<div class="text-red-400">Error: %s</div>`, err.Error())
	} else {
		fmt.Fprintf(w, `<div class="text-green-400">%s</div>`, result)
	}
}

// HandleFTLStop handles stopping FTL regular mode
func (h *Handler) HandleFTLStop(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}
	
	projectPath := r.FormValue("project_path")
	if projectPath == "" {
		w.WriteHeader(http.StatusBadRequest)
		fmt.Fprint(w, `<div class="text-red-400">Error: project_path required</div>`)
		return
	}
	
	args := map[string]interface{}{
		"project_path": projectPath,
	}
	
	result, err := h.mcpClient.CallTool("mcp-server__stop", args)
	if err != nil {
		fmt.Fprintf(w, `<div class="text-red-400">Error: %s</div>`, err.Error())
	} else {
		fmt.Fprintf(w, `<div class="text-green-400">%s</div>`, result)
	}
}

// HandleWatchStart handles starting FTL watch mode
func (h *Handler) HandleWatchStart(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}
	
	projectPath := r.FormValue("project_path")
	if projectPath == "" {
		w.WriteHeader(http.StatusBadRequest)
		fmt.Fprint(w, `<div class="text-red-400">Error: project_path required</div>`)
		return
	}
	
	args := map[string]interface{}{
		"project_path": projectPath,
		"watch":        true,
		"build":        false,
		"listen":       "",
	}
	
	result, err := h.mcpClient.CallTool("mcp-server__up", args)
	if err != nil {
		fmt.Fprintf(w, `<div class="text-red-400">Error: %s</div>`, err.Error())
	} else {
		fmt.Fprintf(w, `<div class="text-green-400">%s</div>`, result)
	}
}

// HandleWatchStop handles stopping FTL watch mode
func (h *Handler) HandleWatchStop(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}
	
	projectPath := r.FormValue("project_path")
	if projectPath == "" {
		w.WriteHeader(http.StatusBadRequest)
		fmt.Fprint(w, `<div class="text-red-400">Error: project_path required</div>`)
		return
	}
	
	args := map[string]interface{}{
		"project_path": projectPath,
	}
	
	result, err := h.mcpClient.CallTool("mcp-server__stop", args)
	if err != nil {
		fmt.Fprintf(w, `<div class="text-red-400">Error: %s</div>`, err.Error())
	} else {
		fmt.Fprintf(w, `<div class="text-green-400">%s</div>`, result)
	}
}

// HandleLogsPoll polls for logs using background-collected data
func (h *Handler) HandleLogsPoll(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}
	
	projectPath := r.FormValue("project_path")
	if projectPath == "" {
		w.WriteHeader(http.StatusBadRequest)
		fmt.Fprint(w, `<div class="text-red-400">Error: project_path required</div>`)
		return
	}
	
	// Use background-collected project state instead of fresh MCP calls
	projectState, exists := h.registry.GetProject(projectPath)
	if !exists || projectState == nil {
		log.Printf("WARNING: No project state found for path: %s, returning empty logs", projectPath)
		// Return empty content when no project state - don't append anything
		fmt.Fprint(w, "")
		return
	}
	
	// Get current log position for this project
	since := h.logPositions.GetPosition(projectPath)
	
	// Get all cached logs from background polling
	allLogs := projectState.GetLogs()
	totalLines := len(allLogs)
	
	// Check if we have new logs since the last position
	if since >= totalLines {
		// No new logs - return empty content
		fmt.Fprint(w, "")
		return
	}
	
	// Get new logs since the specified position
	newLogs := allLogs[since:]
	newLogsText := strings.Join(newLogs, "\n")
	
	// Update log position to current total
	h.logPositions.SetPosition(projectPath, totalLines)
	
	// Return new logs with proper HTML formatting if we have content
	if len(newLogs) > 0 && strings.TrimSpace(newLogsText) != "" {
		escapedLogs := strings.ReplaceAll(newLogsText, "\n", "<br>")
		fmt.Fprintf(w, `<div class="text-green-400">%s</div>`, escapedLogs)
	} else {
		// Return empty content when no meaningful new logs - don't append anything
		fmt.Fprint(w, "")
	}
}

// HandleStatusPoll polls for process status using background-collected data
func (h *Handler) HandleStatusPoll(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}
	
	projectPath := r.FormValue("project_path")
	if projectPath == "" {
		w.WriteHeader(http.StatusBadRequest)
		fmt.Fprint(w, `<div class="text-red-400">Error: project_path required</div>`)
		return
	}
	
	// Use background-collected project state instead of fresh MCP calls
	projectState, exists := h.registry.GetProject(projectPath)
	if !exists || projectState == nil {
		log.Printf("WARNING: No project state found for path: %s, returning empty status", projectPath)
		// Return empty status instead of making fresh MCP calls to avoid competition
		detailedStatus := models.DetailedStatusInfo{
			ProjectPath:   projectPath,
			ActiveProcess: nil, // No active process
		}
		
		htmlContent := generateControlCenterStatusHTML(detailedStatus)
		fmt.Fprint(w, htmlContent)
		return
	}
	
	// Convert cached project state to DetailedStatusInfo
	detailedStatus := models.DetailedStatusInfo{
		ProjectPath:   projectPath,
		ActiveProcess: projectState.ProcessInfo.ActiveProcess, // Use the cached active process
	}
	
	// Debug logging
	if detailedStatus.ActiveProcess != nil {
		log.Printf("DEBUG: Using cached state - ActiveProcess: Type=%s Running=%t PID=%d Port=%d", 
			detailedStatus.ActiveProcess.Type, detailedStatus.ActiveProcess.IsRunning, 
			detailedStatus.ActiveProcess.PID, detailedStatus.ActiveProcess.Port)
	} else {
		log.Printf("DEBUG: Using cached state - No active process")
	}
	
	// Generate status HTML compatible with Control Center template
	htmlContent := generateControlCenterStatusHTML(detailedStatus)
	
	// Also generate Info Panel port update if process is running
	if detailedStatus.ActiveProcess != nil && detailedStatus.ActiveProcess.IsRunning && detailedStatus.ActiveProcess.Port > 0 {
		// Include OOB swap to update the info panel port
		htmlContent += fmt.Sprintf(`
		<div hx-swap-oob="innerHTML:#port-display">
			<span class="text-sm" style="color: #a0a0a0;">Port</span>
			<span class="text-sm text-white font-mono">%d</span>
		</div>`, detailedStatus.ActiveProcess.Port)
	} else {
		// Clear port display when not running
		htmlContent += `<div hx-swap-oob="innerHTML:#port-display"></div>`
	}
	
	fmt.Fprint(w, htmlContent)
}

// HandleProcessStop handles universal process stopping
func (h *Handler) HandleProcessStop(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}
	
	projectPath := r.FormValue("project_path")
	processType := r.FormValue("process_type")
	
	if projectPath == "" || processType == "" {
		w.WriteHeader(http.StatusBadRequest)
		fmt.Fprint(w, `<div class="text-red-400">Error: project_path and process_type required</div>`)
		return
	}
	
	// Use unified stop tool for all process types
	toolName := "mcp-server__stop"
	
	args := map[string]interface{}{
		"project_path": projectPath,
	}
	
	result, err := h.mcpClient.CallTool(toolName, args)
	if err != nil {
		fmt.Fprintf(w, `<div class="text-red-400">Stop failed: %s</div>`, err.Error())
		// Auto-clear error message after 3 seconds
		fmt.Fprint(w, `<script>setTimeout(() => { document.getElementById('operation-feedback').innerHTML = ''; }, 3000);</script>`)
	} else {
		// Parse the stop response but don't show success messages
		stopResponse := parser.ParseStopResponse(result)
		if !stopResponse.Success {
			// Only show error messages, not success messages
			errorMsg := stopResponse.Message
			if stopResponse.Error != "" {
				errorMsg = fmt.Sprintf("%s: %s", stopResponse.Message, stopResponse.Error)
			}
			fmt.Fprintf(w, `<div class="text-red-400">%s</div>`, errorMsg)
			// Auto-clear error message after 3 seconds
			fmt.Fprint(w, `<script>setTimeout(() => { document.getElementById('operation-feedback').innerHTML = ''; }, 3000);</script>`)
		}
		// Success case: return empty response for operation-feedback, but also append to command output
		// Send success message to command output via OOB swap
		fmt.Fprintf(w, `<div hx-swap-oob="beforeend:#ftl-output">%s</div>`, formatMessage("text-yellow-400", "Process stopped successfully"))
	}
}

// HandleProjectAddForm returns the add project form
func (h *Handler) HandleProjectAddForm(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "text/html")
	if err := templates.AddProjectForm().Render(r.Context(), w); err != nil {
		http.Error(w, "Failed to render form", http.StatusInternalServerError)
	}
}

// HandleProjectCancelForm returns the add project button
func (h *Handler) HandleProjectCancelForm(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "text/html")
	if err := templates.AddProjectButton().Render(r.Context(), w); err != nil {
		http.Error(w, "Failed to render button", http.StatusInternalServerError)
	}
}

// HandleProjectAdd adds a new project and returns updated project list
func (h *Handler) HandleProjectAdd(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}
	
	// Parse form data
	if err := r.ParseForm(); err != nil {
		http.Error(w, "Failed to parse form", http.StatusBadRequest)
		return
	}
	
	name := r.FormValue("name")
	path := r.FormValue("path")
	
	if name == "" || path == "" {
		http.Error(w, "Name and path are required", http.StatusBadRequest)
		return
	}
	
	// Validate that the directory exists
	if _, err := os.Stat(path); os.IsNotExist(err) {
		log.Printf("Project path does not exist: %s", path)
		http.Error(w, fmt.Sprintf("Directory does not exist: %s", path), http.StatusBadRequest)
		return
	}
	
	// Validate that it's an FTL project by calling mcp-server__list_components
	log.Printf("Validating FTL project by calling list_components for: %s", path)
	componentArgs := map[string]interface{}{
		"project_path": path,
	}
	
	componentResult, err := h.mcpClient.CallTool("mcp-server__list_components", componentArgs)
	if err != nil {
		log.Printf("Failed to validate FTL project via MCP: %v", err)
		http.Error(w, fmt.Sprintf("Failed to validate project: %v", err), http.StatusBadRequest)
		return
	}
	
	// Parse the MCP response to check if it succeeded
	var componentResponse map[string]interface{}
	if err := json.Unmarshal([]byte(componentResult), &componentResponse); err != nil {
		log.Printf("Failed to parse component list response: %v", err)
		http.Error(w, "Failed to validate project structure", http.StatusBadRequest)
		return
	}
	
	// Check if the component list call was successful
	if success, ok := componentResponse["success"].(bool); !ok || !success {
		errorMsg := "Unknown validation error"
		if errStr, ok := componentResponse["error"].(string); ok {
			errorMsg = errStr
		}
		log.Printf("FTL project validation failed: %s", errorMsg)
		http.Error(w, fmt.Sprintf("Not a valid FTL project: %s", errorMsg), http.StatusBadRequest)
		return
	}
	
	log.Printf("FTL project validation successful for: %s", path)
	
	// Add project to registry
	log.Printf("Adding validated FTL project: name=%s, path=%s", name, path)
	_, err = h.registry.AddProject(path, name)
	if err != nil {
		log.Printf("Failed to add project: %v", err)
		http.Error(w, fmt.Sprintf("Failed to add project: %v", err), http.StatusBadRequest)
		return
	}
	log.Printf("Successfully added project: %s", name)
	
	// Start polling for the new project
	h.pollingManager.StartProjectPolling(path)
	
	// Get all projects and render updated list
	allProjects := h.registry.GetAllProjects()
	currentPath := h.registry.GetCurrentProjectPath()
	
	// Convert to components.Project with process status
	projects := make([]templates.Project, len(allProjects))
	for i, p := range allProjects {
		// Get project state to access process info
		projectState, exists := h.registry.GetProject(p.Path)
		var ftlRunning, watchRunning bool
		if exists && projectState != nil && projectState.ProcessInfo.ActiveProcess != nil {
			ftlRunning = projectState.ProcessInfo.ActiveProcess.IsRunning && projectState.ProcessInfo.ActiveProcess.Type == "regular"
			watchRunning = projectState.ProcessInfo.ActiveProcess.IsRunning && projectState.ProcessInfo.ActiveProcess.Type == "watch"
		}
		
		projects[i] = templates.Project{
			Name:         p.Name,
			Path:         p.Path,
			Status:       "active",
			LastUpdated:  p.LastActive,
			FTLRunning:   ftlRunning,
			WatchRunning: watchRunning,
		}
	}
	
	w.Header().Set("Content-Type", "text/html")
	if err := templates.ProjectListContents(projects, currentPath).Render(r.Context(), w); err != nil {
		http.Error(w, "Failed to render project list", http.StatusInternalServerError)
	}
}

// HandleProjectRemove removes a project and returns updated project list
func (h *Handler) HandleProjectRemove(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodDelete {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}
	
	path := r.URL.Query().Get("path")
	if path == "" {
		http.Error(w, "Path is required", http.StatusBadRequest)
		return
	}
	
	// Stop polling first
	h.pollingManager.StopProjectPolling(path)
	
	// Check if we're removing the current project
	currentPath := h.registry.GetCurrentProjectPath()
	removingCurrentProject := (currentPath == path)
	
	// Remove from registry
	if err := h.registry.RemoveProject(path); err != nil {
		http.Error(w, fmt.Sprintf("Failed to remove project: %v", err), http.StatusBadRequest)
		return
	}
	
	// If we removed the current project, switch to another one
	allProjects := h.registry.GetAllProjects()
	if removingCurrentProject && len(allProjects) > 0 {
		// Switch to the first available project
		newCurrentProject := allProjects[0]
		h.registry.SetCurrentProject(newCurrentProject.Path)
		currentPath = newCurrentProject.Path
	} else {
		currentPath = h.registry.GetCurrentProjectPath()
	}
	
	// Convert to components.Project with process status
	projects := make([]templates.Project, len(allProjects))
	for i, p := range allProjects {
		// Get project state to access process info
		projectState, exists := h.registry.GetProject(p.Path)
		var ftlRunning, watchRunning bool
		if exists && projectState != nil && projectState.ProcessInfo.ActiveProcess != nil {
			ftlRunning = projectState.ProcessInfo.ActiveProcess.IsRunning && projectState.ProcessInfo.ActiveProcess.Type == "regular"
			watchRunning = projectState.ProcessInfo.ActiveProcess.IsRunning && projectState.ProcessInfo.ActiveProcess.Type == "watch"
		}
		
		projects[i] = templates.Project{
			Name:         p.Name,
			Path:         p.Path,
			Status:       "active",
			LastUpdated:  p.LastActive,
			FTLRunning:   ftlRunning,
			WatchRunning: watchRunning,
		}
	}
	
	w.Header().Set("Content-Type", "text/html")
	if err := templates.ProjectListContents(projects, currentPath).Render(r.Context(), w); err != nil {
		http.Error(w, "Failed to render project list", http.StatusInternalServerError)
	}
}

// HandleProjectSwitch switches to a different project
func (h *Handler) HandleProjectSwitch(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}
	
	path := r.URL.Query().Get("path")
	if path == "" {
		http.Error(w, "Path is required", http.StatusBadRequest)
		return
	}
	
	// Get the project state (pre-collected from background polling)
	projectState, exists := h.registry.GetProject(path)
	if !exists || projectState == nil {
		http.Error(w, "Project not found", http.StatusNotFound)
		return
	}

	// Convert complex state to component types directly  
	project, processStatus, serverStatus, recentLogs := projectState.ToComponentTypes()
	
	// Get command history from project state
	stateCommandHistory := projectState.GetCommandHistory()
	componentCommandHistory := make([]templates.CommandOutput, len(stateCommandHistory))
	for i, cmd := range stateCommandHistory {
		componentCommandHistory[i] = templates.CommandOutput{
			Command:   cmd.Command,
			Output:    cmd.Output,
			Timestamp: cmd.Timestamp,
			Success:   cmd.Success,
		}
	}
	
	// Get all projects for sidebar with process status
	allProjects := h.registry.GetAllProjects()
	sidebarProjects := make([]templates.Project, len(allProjects))
	for i, p := range allProjects {
		// Get project state to access process info
		projectState, exists := h.registry.GetProject(p.Path)
		var ftlRunning, watchRunning bool
		if exists && projectState != nil && projectState.ProcessInfo.ActiveProcess != nil {
			ftlRunning = projectState.ProcessInfo.ActiveProcess.IsRunning && projectState.ProcessInfo.ActiveProcess.Type == "regular"
			watchRunning = projectState.ProcessInfo.ActiveProcess.IsRunning && projectState.ProcessInfo.ActiveProcess.Type == "watch"
		}
		
		sidebarProjects[i] = templates.Project{
			Name:         p.Name,
			Path:         p.Path,
			Status:       "Active", // All projects in registry are active
			LastUpdated:  p.LastActive,
			FTLRunning:   ftlRunning,
			WatchRunning: watchRunning,
		}
	}
	
	// Create dashboard data with converted types
	data := templates.DashboardData{
		CurrentProject:  project,
		AllProjects:     sidebarProjects,
		ServerStatus:    serverStatus,
		ProcessStatus:   processStatus,
		RecentLogs:      recentLogs,
		CommandHistory:  componentCommandHistory,
	}
	
	// Set current project in registry
	h.registry.SetCurrentProject(path)
	
	// Render complete main content area with updated project data using existing Layout
	w.Header().Set("Content-Type", "text/html")
	if err := templates.Layout(data).Render(r.Context(), w); err != nil {
		http.Error(w, "Failed to render main content", http.StatusInternalServerError)
	}
}

// HandleProjectReload forces a reload of projects from disk
// This is primarily used by tests to ensure server state matches file state
func (h *Handler) HandleProjectReload(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}
	
	log.Println("Reloading projects from disk (test mode)")
	
	// Reload projects from disk
	if err := h.registry.ReloadProjects(); err != nil {
		log.Printf("Failed to reload projects: %v", err)
		http.Error(w, fmt.Sprintf("Failed to reload projects: %v", err), http.StatusInternalServerError)
		return
	}
	
	// Return success
	w.WriteHeader(http.StatusOK)
	fmt.Fprint(w, "Projects reloaded successfully")
}

// generateControlCenterStatusHTML generates HTML compatible with Control Center status div
func generateControlCenterStatusHTML(status models.DetailedStatusInfo) string {
	if status.ActiveProcess != nil && status.ActiveProcess.IsRunning {
		// Display the active process status
		dotColor := "bg-green-400"
		statusText := "Up"
		
		if status.ActiveProcess.Type == "watch" {
			dotColor = "bg-blue-400"
			statusText = "Watching"
		}
		
		return fmt.Sprintf(`<div class="space-y-2">
			<div class="flex items-center justify-between p-2 rounded" style="background-color: #333333;">
				<div class="flex items-center space-x-2">
					<div class="w-2 h-2 %s rounded-full"></div>
					<span class="text-sm font-medium text-white">%s</span>
				</div>
				<div class="flex items-center">
					%s
				</div>
			</div>
		</div>`, dotColor, statusText, generateStopButtonHTML(status.ActiveProcess.Type, true, status.ProjectPath))
	}
	
	// No active process
	return `<div class="space-y-2">
		<div class="flex items-center justify-between p-2 rounded" style="background-color: #333333;">
			<div class="flex items-center space-x-2">
				<div class="w-2 h-2 bg-gray-500 rounded-full"></div>
				<span class="text-sm font-medium" style="color: #666666;">Stopped</span>
			</div>
		</div>
	</div>`
}

// generateStopButtonHTML generates stop button HTML if process is running
func generateStopButtonHTML(processType string, isRunning bool, projectPath string) string {
	if !isRunning {
		return ""
	}
	
	return fmt.Sprintf(`
		<button 
			class="text-xs px-2 py-1 bg-red-500 text-white rounded hover:bg-red-600"
			hx-post="/htmx/process/stop"
			hx-vals='{"project_path": "%s", "process_type": "%s"}'
			hx-target="#operation-feedback"
			hx-on::before-request="document.getElementById('ftl-output').insertAdjacentHTML('beforeend', '<div class=\'text-blue-400\'>[' + new Date().toTimeString().slice(0,8) + '] Sending stop command...</div>')"
		>
			Stop
		</button>
	`, projectPath, processType)
}