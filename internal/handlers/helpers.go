package handlers

import (
	"github.com/fastertools/ftl/internal/state"
	"github.com/fastertools/ftl/ui/templates"
)

// ConvertProjectsToTemplates converts registry projects to template format
// This eliminates duplicate code that was in multiple handlers
func ConvertProjectsToTemplates(registry *state.ProjectRegistry) []templates.Project {
	allProjects := registry.GetAllProjects()
	projects := make([]templates.Project, len(allProjects))
	
	for i, p := range allProjects {
		// Get project state to access process info
		projectState, exists := registry.GetProject(p.Path)
		var ftlRunning, watchRunning bool
		
		if exists && projectState != nil && projectState.ProcessInfo.ActiveProcess != nil {
			ftlRunning = projectState.ProcessInfo.ActiveProcess.IsRunning && projectState.ProcessInfo.ActiveProcess.Type == "regular"
			watchRunning = projectState.ProcessInfo.ActiveProcess.IsRunning && projectState.ProcessInfo.ActiveProcess.Type == "watch"
		}
		
		projects[i] = templates.Project{
			Name:         p.Name,
			Path:         p.Path,
			Status:       "Running", // Simplified status
			LastUpdated:  p.LastActive,
			FTLRunning:   ftlRunning,
			WatchRunning: watchRunning,
		}
	}
	
	return projects
}

// GetProjectStatus returns a human-readable status string for a project
func GetProjectStatus(ftlRunning, watchRunning bool) string {
	if ftlRunning {
		return "FTL Running"
	}
	if watchRunning {
		return "Watch Mode"
	}
	return "Stopped"
}