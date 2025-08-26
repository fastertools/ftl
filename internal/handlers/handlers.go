package handlers

import (
	"encoding/json"
	"fmt"
	"log"
	"net/http"
	"os"
	"strings"
	"time"

	"github.com/fastertools/ftl/internal/config"
	"github.com/fastertools/ftl/internal/mcpclient"
	"github.com/fastertools/ftl/internal/models"
	"github.com/fastertools/ftl/internal/parser"
	"github.com/fastertools/ftl/internal/polling"
	"github.com/fastertools/ftl/internal/state"
	ui "github.com/fastertools/ftl/ui/gen"
	"github.com/fastertools/ftl/ui/templates"
)

// Handler encapsulates all HTTP handlers
type Handler struct {
	mcpClient      *mcpclient.Client
	logPositions   *state.LogPositionTracker
	registry       *state.ProjectRegistry
	pollingManager *polling.Manager
}

// NewHandler creates a new handler instance
func NewHandler(mcpClient *mcpclient.Client) *Handler {
	// Get projects file from environment or use centralized default
	projectsFile := os.Getenv("PROJECTS_FILE")
	if projectsFile == "" {
		// Use centralized data directory instead of current directory
		var err error
		projectsFile, err = config.UserDataPath("projects.json")
		if err != nil {
			log.Printf("Failed to get user data path, falling back to current dir: %v", err)
			projectsFile = "projects.json"
		}
	}
	log.Printf("Using projects file: %s", projectsFile)

	// Create project registry
	registry := state.NewProjectRegistry(projectsFile)

	// Load existing projects
	if err := registry.LoadProjects(); err != nil {
		log.Printf("Warning: failed to load projects: %v", err)
	}

	// Log projects loaded
	allProjects := registry.GetAllProjects()
	log.Printf("After loading projects: found %d projects", len(allProjects))

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
	// Ensure projects file exists (for new users or if deleted)
	if err := h.registry.EnsureProjectsFile(); err != nil {
		log.Printf("Warning: failed to ensure projects file: %v", err)
	}

	// Get all projects for sidebar using helper
	projects := ConvertProjectsToTemplates(h.registry)

	// Get current project from registry (may not exist for new users)
	currentProj, hasCurrentProject := h.registry.GetCurrentProject()

	// Initialize dashboard data with empty state
	dashboardData := templates.DashboardData{
		AllProjects:    projects,
		RecentLogs:     []templates.LogEntry{},
		CommandHistory: []templates.CommandOutput{},
		ServerStatus: templates.ServerStatus{
			Running:     false,
			ProcessID:   os.Getpid(),
			Port:        0,
			LastChecked: time.Now(),
		},
		ProcessStatus: templates.ProcessStatus{
			FTLRunning:   false,
			WatchRunning: false,
			LastActivity: time.Now(),
		},
	}

	// If we have a current project, populate its data
	if hasCurrentProject && currentProj != nil {
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
		dashboardData.CommandHistory = componentCommandHistory

		// Get the actual FTL process port if running
		var ftlPort int
		if currentProj.ProcessInfo.ActiveProcess != nil && currentProj.ProcessInfo.ActiveProcess.IsRunning {
			ftlPort = currentProj.ProcessInfo.ActiveProcess.Port
		}

		// Update dashboard data with current project info
		dashboardData.CurrentProject = templates.Project{
			Name:        currentProj.Project.Name,
			Path:        currentProj.Project.Path,
			Status:      "active",
			LastUpdated: currentProj.Project.LastActive,
		}
		dashboardData.ServerStatus = templates.ServerStatus{
			Running:     currentProj.ProcessInfo.ActiveProcess != nil && currentProj.ProcessInfo.ActiveProcess.IsRunning,
			ProcessID:   os.Getpid(),
			Port:        ftlPort,
			LastChecked: time.Now(),
		}
		dashboardData.ProcessStatus = templates.ProcessStatus{
			FTLRunning:   currentProj.ProcessInfo.ActiveProcess != nil && currentProj.ProcessInfo.ActiveProcess.IsRunning && currentProj.ProcessInfo.ActiveProcess.Type == "regular",
			WatchRunning: currentProj.ProcessInfo.ActiveProcess != nil && currentProj.ProcessInfo.ActiveProcess.IsRunning && currentProj.ProcessInfo.ActiveProcess.Type == "watch",
			LastActivity: time.Now(),
		}
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
		http.Error(w, ui.Handlers.Errors["method_not_allowed"], http.StatusMethodNotAllowed)
		return
	}

	// Parse form data
	if err := r.ParseForm(); err != nil {
		http.Error(w, ui.Handlers.Errors["parse_form_failed"], http.StatusBadRequest)
		return
	}

	// Get current project
	currentProj, exists := h.registry.GetCurrentProject()
	if !exists {
		http.Error(w, ui.Handlers.Errors["no_projects"], http.StatusInternalServerError)
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
			if err := templates.ErrorMessage(fmt.Sprintf("Build failed: %s", err.Error())).Render(r.Context(), w); err != nil {
				http.Error(w, err.Error(), http.StatusInternalServerError)
			}
			return
		}

		// Parse the build response JSON
		buildResponse := parser.ParseBuildResponse(result)

		w.Header().Set("Content-Type", "text/html")
		if buildResponse.Success {
			if err := templates.SuccessMessage(fmt.Sprintf("Build successful: %s", buildResponse.Output)).Render(r.Context(), w); err != nil {
				http.Error(w, err.Error(), http.StatusInternalServerError)
			}
		} else {
			if err := templates.ErrorMessage(fmt.Sprintf("Build failed: %s", buildResponse.Error)).Render(r.Context(), w); err != nil {
				http.Error(w, err.Error(), http.StatusInternalServerError)
			}
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
			if err := templates.ErrorMessage(fmt.Sprintf("Stop FTL failed: %s", err.Error())).Render(r.Context(), w); err != nil {
				http.Error(w, err.Error(), http.StatusInternalServerError)
			}
			return
		}

		w.Header().Set("Content-Type", "text/html")
		if err := templates.WarningMessage(fmt.Sprintf("FTL Process stopped: %s", result)).Render(r.Context(), w); err != nil {
			http.Error(w, err.Error(), http.StatusInternalServerError)
		}
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
			if err := templates.ErrorMessage(fmt.Sprintf("Stop Watch failed: %s", err.Error())).Render(r.Context(), w); err != nil {
				http.Error(w, err.Error(), http.StatusInternalServerError)
			}
			return
		}

		w.Header().Set("Content-Type", "text/html")
		if err := templates.WarningMessage(fmt.Sprintf("Watch Mode stopped: %s", result)).Render(r.Context(), w); err != nil {
			http.Error(w, err.Error(), http.StatusInternalServerError)
		}
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
			if err := templates.ErrorMessage(fmt.Sprintf("%s failed: %s", modeLabel, err.Error())).Render(r.Context(), w); err != nil {
				http.Error(w, err.Error(), http.StatusInternalServerError)
			}
			return
		}

		// Parse the up response JSON
		upResponse := parser.ParseUpResponse(result)

		w.Header().Set("Content-Type", "text/html")
		if upResponse.Success {
			// Start polling for this project if server started successfully
			h.pollingManager.StartProjectPolling(projectPath)
			if isWatchMode {
				if err := templates.InfoMessage(upResponse.Message).Render(r.Context(), w); err != nil {
					http.Error(w, err.Error(), http.StatusInternalServerError)
				}
			} else {
				if err := templates.SuccessMessage(upResponse.Message).Render(r.Context(), w); err != nil {
					http.Error(w, err.Error(), http.StatusInternalServerError)
				}
			}
		} else {
			if err := templates.ErrorMessage(upResponse.Error).Render(r.Context(), w); err != nil {
				http.Error(w, err.Error(), http.StatusInternalServerError)
			}
		}
		return
	}

	http.Error(w, "No valid action specified", http.StatusBadRequest)
}

// HandleFTLStart handles starting FTL in regular mode
func (h *Handler) HandleFTLStart(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, ui.Handlers.Errors["method_not_allowed"], http.StatusMethodNotAllowed)
		return
	}

	projectPath := r.FormValue("project_path")
	if projectPath == "" {
		w.WriteHeader(http.StatusBadRequest)
		if err := templates.SimpleErrorDiv(ui.Handlers.Errors["project_path_required"]).Render(r.Context(), w); err != nil {
			http.Error(w, err.Error(), http.StatusInternalServerError)
		}
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
		if err := templates.SimpleErrorDiv(fmt.Sprintf("Error: %s", err.Error())).Render(r.Context(), w); err != nil {
			http.Error(w, err.Error(), http.StatusInternalServerError)
		}
	} else {
		if err := templates.SimpleSuccessDiv(result).Render(r.Context(), w); err != nil {
			http.Error(w, err.Error(), http.StatusInternalServerError)
		}
	}
}

// HandleFTLStop handles stopping FTL regular mode
func (h *Handler) HandleFTLStop(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, ui.Handlers.Errors["method_not_allowed"], http.StatusMethodNotAllowed)
		return
	}

	projectPath := r.FormValue("project_path")
	if projectPath == "" {
		w.WriteHeader(http.StatusBadRequest)
		if err := templates.SimpleErrorDiv(ui.Handlers.Errors["project_path_required"]).Render(r.Context(), w); err != nil {
			http.Error(w, err.Error(), http.StatusInternalServerError)
		}
		return
	}

	args := map[string]interface{}{
		"project_path": projectPath,
	}

	result, err := h.mcpClient.CallTool("mcp-server__stop", args)
	if err != nil {
		if err := templates.SimpleErrorDiv(fmt.Sprintf("Error: %s", err.Error())).Render(r.Context(), w); err != nil {
			http.Error(w, err.Error(), http.StatusInternalServerError)
		}
	} else {
		if err := templates.SimpleSuccessDiv(result).Render(r.Context(), w); err != nil {
			http.Error(w, err.Error(), http.StatusInternalServerError)
		}
	}
}

// HandleWatchStart handles starting FTL watch mode
func (h *Handler) HandleWatchStart(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, ui.Handlers.Errors["method_not_allowed"], http.StatusMethodNotAllowed)
		return
	}

	projectPath := r.FormValue("project_path")
	if projectPath == "" {
		w.WriteHeader(http.StatusBadRequest)
		if err := templates.SimpleErrorDiv(ui.Handlers.Errors["project_path_required"]).Render(r.Context(), w); err != nil {
			http.Error(w, err.Error(), http.StatusInternalServerError)
		}
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
		if err := templates.SimpleErrorDiv(fmt.Sprintf("Error: %s", err.Error())).Render(r.Context(), w); err != nil {
			http.Error(w, err.Error(), http.StatusInternalServerError)
		}
	} else {
		if err := templates.SimpleSuccessDiv(result).Render(r.Context(), w); err != nil {
			http.Error(w, err.Error(), http.StatusInternalServerError)
		}
	}
}

// HandleWatchStop handles stopping FTL watch mode
func (h *Handler) HandleWatchStop(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, ui.Handlers.Errors["method_not_allowed"], http.StatusMethodNotAllowed)
		return
	}

	projectPath := r.FormValue("project_path")
	if projectPath == "" {
		w.WriteHeader(http.StatusBadRequest)
		if err := templates.SimpleErrorDiv(ui.Handlers.Errors["project_path_required"]).Render(r.Context(), w); err != nil {
			http.Error(w, err.Error(), http.StatusInternalServerError)
		}
		return
	}

	args := map[string]interface{}{
		"project_path": projectPath,
	}

	result, err := h.mcpClient.CallTool("mcp-server__stop", args)
	if err != nil {
		if err := templates.SimpleErrorDiv(fmt.Sprintf("Error: %s", err.Error())).Render(r.Context(), w); err != nil {
			http.Error(w, err.Error(), http.StatusInternalServerError)
		}
	} else {
		if err := templates.SimpleSuccessDiv(result).Render(r.Context(), w); err != nil {
			http.Error(w, err.Error(), http.StatusInternalServerError)
		}
	}
}

// HandleLogsPoll polls for logs using background-collected data
func (h *Handler) HandleLogsPoll(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, ui.Handlers.Errors["method_not_allowed"], http.StatusMethodNotAllowed)
		return
	}

	projectPath := r.FormValue("project_path")
	if projectPath == "" {
		w.WriteHeader(http.StatusBadRequest)
		if err := templates.SimpleErrorDiv(ui.Handlers.Errors["project_path_required"]).Render(r.Context(), w); err != nil {
			http.Error(w, err.Error(), http.StatusInternalServerError)
		}
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
		if err := templates.LogString(newLogsText).Render(r.Context(), w); err != nil {
			http.Error(w, err.Error(), http.StatusInternalServerError)
		}
	} else {
		// Return empty content when no meaningful new logs - don't append anything
		fmt.Fprint(w, "")
	}
}

// HandleStatusPoll polls for process status using background-collected data
func (h *Handler) HandleStatusPoll(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, ui.Handlers.Errors["method_not_allowed"], http.StatusMethodNotAllowed)
		return
	}

	projectPath := r.FormValue("project_path")
	if projectPath == "" {
		w.WriteHeader(http.StatusBadRequest)
		if err := templates.SimpleErrorDiv(ui.Handlers.Errors["project_path_required"]).Render(r.Context(), w); err != nil {
			http.Error(w, err.Error(), http.StatusInternalServerError)
		}
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

		if err := templates.ProcessStatusDisplay(detailedStatus, projectPath).Render(r.Context(), w); err != nil {
			http.Error(w, err.Error(), http.StatusInternalServerError)
		}
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

	// Render status using templ component
	if err := templates.ProcessStatusDisplay(detailedStatus, projectPath).Render(r.Context(), w); err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
}

// HandleProcessStop handles universal process stopping
func (h *Handler) HandleProcessStop(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, ui.Handlers.Errors["method_not_allowed"], http.StatusMethodNotAllowed)
		return
	}

	projectPath := r.FormValue("project_path")

	if projectPath == "" {
		w.WriteHeader(http.StatusBadRequest)
		if err := templates.SimpleErrorDiv("Error: project_path required").Render(r.Context(), w); err != nil {
			http.Error(w, err.Error(), http.StatusInternalServerError)
		}
		return
	}

	// Use unified stop tool for all process types
	toolName := "mcp-server__stop"

	args := map[string]interface{}{
		"project_path": projectPath,
	}

	result, err := h.mcpClient.CallTool(toolName, args)
	if err != nil {
		// Use auto-clearing error message
		if err := templates.AutoClearErrorMessage(fmt.Sprintf("Stop failed: %s", err.Error()), 3000).Render(r.Context(), w); err != nil {
			http.Error(w, err.Error(), http.StatusInternalServerError)
		}
	} else {
		// Parse the stop response but don't show success messages
		stopResponse := parser.ParseStopResponse(result)
		if !stopResponse.Success {
			// Only show error messages, not success messages
			errorMsg := stopResponse.Message
			if stopResponse.Error != "" {
				errorMsg = fmt.Sprintf("%s: %s", stopResponse.Message, stopResponse.Error)
			}
			// Use auto-clearing error message
			if err := templates.AutoClearErrorMessage(errorMsg, 3000).Render(r.Context(), w); err != nil {
				http.Error(w, err.Error(), http.StatusInternalServerError)
			}
		}
		// Success case: return empty response for operation-feedback, but also append to command output
		// Send success message to command output via OOB swap
		if err := templates.MessageWithOOB("Process stopped successfully", ui.CSSClasses.Text["warning"], "#ftl-output").Render(r.Context(), w); err != nil {
			http.Error(w, err.Error(), http.StatusInternalServerError)
		}
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
		http.Error(w, ui.Handlers.Errors["method_not_allowed"], http.StatusMethodNotAllowed)
		return
	}

	// Parse form data
	if err := r.ParseForm(); err != nil {
		http.Error(w, ui.Handlers.Errors["parse_form_failed"], http.StatusBadRequest)
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

	// Get all projects and render updated list using helper
	projects := ConvertProjectsToTemplates(h.registry)
	currentPath := h.registry.GetCurrentProjectPath()

	w.Header().Set("Content-Type", "text/html")
	if err := templates.ProjectListContents(projects, currentPath).Render(r.Context(), w); err != nil {
		http.Error(w, "Failed to render project list", http.StatusInternalServerError)
	}
}

// HandleProjectRemove removes a project and returns updated project list
func (h *Handler) HandleProjectRemove(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodDelete {
		http.Error(w, ui.Handlers.Errors["method_not_allowed"], http.StatusMethodNotAllowed)
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

	// Convert to components.Project with process status using helper
	projects := ConvertProjectsToTemplates(h.registry)

	w.Header().Set("Content-Type", "text/html")
	if err := templates.ProjectListContents(projects, currentPath).Render(r.Context(), w); err != nil {
		http.Error(w, "Failed to render project list", http.StatusInternalServerError)
	}
}

// HandleProjectSwitch switches to a different project
func (h *Handler) HandleProjectSwitch(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, ui.Handlers.Errors["method_not_allowed"], http.StatusMethodNotAllowed)
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

	// Get all projects for sidebar with process status using helper
	sidebarProjects := ConvertProjectsToTemplates(h.registry)

	// Create dashboard data with converted types
	data := templates.DashboardData{
		CurrentProject: project,
		AllProjects:    sidebarProjects,
		ServerStatus:   serverStatus,
		ProcessStatus:  processStatus,
		RecentLogs:     recentLogs,
		CommandHistory: componentCommandHistory,
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
		http.Error(w, ui.Handlers.Errors["method_not_allowed"], http.StatusMethodNotAllowed)
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


// ============================================================================
// Test-Specific HTTP Endpoints (Phase 2)
// ============================================================================

// HandleTestWaitForHTMX blocks until HTMX operations complete
func (h *Handler) HandleTestWaitForHTMX(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodGet {
		http.Error(w, ui.Handlers.Errors["method_not_allowed"], http.StatusMethodNotAllowed)
		return
	}

	projectPath := r.URL.Query().Get("project_path")
	timeoutStr := r.URL.Query().Get("timeout")

	if projectPath == "" {
		http.Error(w, "project_path parameter required", http.StatusBadRequest)
		return
	}

	timeout := 30 * time.Second
	if timeoutStr != "" {
		if t, err := time.ParseDuration(timeoutStr + "ms"); err == nil {
			timeout = t
		}
	}

	ticker := time.NewTicker(100 * time.Millisecond)
	defer ticker.Stop()

	timeoutChan := time.After(timeout)

	for {
		// Check if HTMX operations have settled (no active polling)
		projectState, exists := h.registry.GetProject(projectPath)
		if exists && projectState != nil && !projectState.PollingActive {
			response := map[string]interface{}{
				"ready":   true,
				"message": "HTMX operations settled",
			}
			w.Header().Set("Content-Type", "application/json")
			json.NewEncoder(w).Encode(response)
			return
		}

		// Wait for next check or timeout
		select {
		case <-ticker.C:
			continue
		case <-timeoutChan:
			response := map[string]interface{}{
				"ready":   false,
				"message": "Timeout waiting for HTMX to settle",
			}
			w.WriteHeader(http.StatusRequestTimeout)
			w.Header().Set("Content-Type", "application/json")
			json.NewEncoder(w).Encode(response)
			return
		}
	}
}

// HandleTestWaitForStatus blocks until process status changes to expected value
func (h *Handler) HandleTestWaitForStatus(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodGet {
		http.Error(w, ui.Handlers.Errors["method_not_allowed"], http.StatusMethodNotAllowed)
		return
	}

	projectPath := r.URL.Query().Get("project_path")
	expectedStatus := r.URL.Query().Get("status")
	timeoutStr := r.URL.Query().Get("timeout")

	if projectPath == "" {
		response := models.APIResponse{
			Success: false,
			Error:   "project_path parameter required",
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(response)
		return
	}

	timeout := 30 * time.Second
	if timeoutStr != "" {
		if t, err := time.ParseDuration(timeoutStr + "ms"); err == nil {
			timeout = t
		}
	}

	ticker := time.NewTicker(500 * time.Millisecond)
	defer ticker.Stop()

	timeoutChan := time.After(timeout)

	for {
		// Check current status
		projectState, exists := h.registry.GetProject(projectPath)
		if exists && projectState != nil {
			currentStatus := "stopped"
			if projectState.ProcessInfo.ActiveProcess != nil && projectState.ProcessInfo.ActiveProcess.IsRunning {
				currentStatus = "running"
			}

			// Check if status matches expected
			if expectedStatus == "" || currentStatus == expectedStatus {
				response := map[string]interface{}{
					"ready":   true,
					"status":  currentStatus,
					"message": "Process status matches",
				}
				w.Header().Set("Content-Type", "application/json")
				json.NewEncoder(w).Encode(response)
				return
			}
		}

		// Wait for next check or timeout
		select {
		case <-ticker.C:
			// Continue to next iteration
			continue
		case <-timeoutChan:
			// Get current status for response
			currentStatus := "stopped"
			if projectState, exists := h.registry.GetProject(projectPath); exists && projectState != nil {
				if projectState.ProcessInfo.ActiveProcess != nil && projectState.ProcessInfo.ActiveProcess.IsRunning {
					currentStatus = "running"
				}
			}
			response := map[string]interface{}{
				"ready":   false,
				"status":  currentStatus,
				"message": "Timeout waiting for status",
			}
			w.WriteHeader(http.StatusRequestTimeout)
			w.Header().Set("Content-Type", "application/json")
			json.NewEncoder(w).Encode(response)
			return
		}
	}
}

// HandleTestWaitForLogs blocks until sufficient logs are available
func (h *Handler) HandleTestWaitForLogs(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodGet {
		http.Error(w, ui.Handlers.Errors["method_not_allowed"], http.StatusMethodNotAllowed)
		return
	}

	projectPath := r.URL.Query().Get("project_path")
	minLinesStr := r.URL.Query().Get("min_lines")
	timeoutStr := r.URL.Query().Get("timeout")

	if projectPath == "" {
		http.Error(w, "project_path parameter required", http.StatusBadRequest)
		return
	}

	minLines := 1
	if minLinesStr != "" {
		fmt.Sscanf(minLinesStr, "%d", &minLines)
	}

	timeout := 30 * time.Second
	if timeoutStr != "" {
		if t, err := time.ParseDuration(timeoutStr + "ms"); err == nil {
			timeout = t
		}
	}

	ticker := time.NewTicker(100 * time.Millisecond)
	defer ticker.Stop()

	timeoutChan := time.After(timeout)

	for {
		// Check for sufficient logs
		projectState, exists := h.registry.GetProject(projectPath)
		if exists && projectState != nil {
			logs := projectState.LogBuffer.GetAll()
			if len(logs) >= minLines {
				response := map[string]interface{}{
					"ready": true,
					"logs":  logs,
				}
				w.Header().Set("Content-Type", "application/json")
				json.NewEncoder(w).Encode(response)
				return
			}
		}

		// Wait for next check or timeout
		select {
		case <-ticker.C:
			continue
		case <-timeoutChan:
			// Get current logs for timeout response
			currentLogs := []string{}
			if projectState, exists := h.registry.GetProject(projectPath); exists && projectState != nil {
				currentLogs = projectState.LogBuffer.GetAll()
			}
			response := map[string]interface{}{
				"ready": false,
				"logs":  currentLogs,
			}
			w.WriteHeader(http.StatusRequestTimeout)
			w.Header().Set("Content-Type", "application/json")
			json.NewEncoder(w).Encode(response)
			return
		}
	}
}

// HandleTestProcessTree returns JSON process hierarchy
func (h *Handler) HandleTestProcessTree(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodGet {
		http.Error(w, ui.Handlers.Errors["method_not_allowed"], http.StatusMethodNotAllowed)
		return
	}

	projectPath := r.URL.Query().Get("project_path")
	if projectPath == "" {
		http.Error(w, "project_path parameter required", http.StatusBadRequest)
		return
	}

	// Get process info from project state
	projectState, exists := h.registry.GetProject(projectPath)
	if !exists || projectState == nil {
		http.Error(w, "Project not found", http.StatusNotFound)
		return
	}

	// Create ftl process info
	ftlInfo := map[string]interface{}{
		"is_running": false,
	}

	if projectState.ProcessInfo.ActiveProcess != nil && projectState.ProcessInfo.ActiveProcess.IsRunning {
		ftlInfo = map[string]interface{}{
			"is_running": true,
			"pid":        projectState.ProcessInfo.ActiveProcess.PID,
			"port":       projectState.ProcessInfo.ActiveProcess.Port,
			"type":       projectState.ProcessInfo.ActiveProcess.Type,
		}
	}

	response := map[string]interface{}{
		"ftl": ftlInfo,
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(response)
}
