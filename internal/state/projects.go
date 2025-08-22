package state

import (
	"encoding/json"
	"fmt"
	"log"
	"os"
	"sort"
	"sync"
	"time"
)

// Project represents a single FTL project
type Project struct {
	Name        string    `json:"name"`
	Path        string    `json:"path"`
	AddedAt     time.Time `json:"added_at"`
	LastActive  time.Time `json:"last_active"`
}

// ProjectRegistry manages all projects and their states
type ProjectRegistry struct {
	projects       map[string]*ProjectState // All projects by path
	currentProject string                   // Currently viewing project path
	persistFile    string                   // Path to projects.json
	mu             sync.RWMutex
	stopSync       chan bool                // Stop signal for file sync goroutine
	syncOnce       sync.Once                // Ensure sync only starts once
}

// NewProjectRegistry creates a new project registry
func NewProjectRegistry(persistFile string) *ProjectRegistry {
	pr := &ProjectRegistry{
		projects:    make(map[string]*ProjectState),
		persistFile: persistFile,
		stopSync:    make(chan bool),
	}
	
	// Only start automatic file sync in production mode
	// In test mode (test_projects.json), use manual reload for deterministic behavior
	if persistFile != "test_projects.json" {
		go pr.startFileSync()
	}
	
	return pr
}

// LoadProjects loads projects from persistent storage
func (pr *ProjectRegistry) LoadProjects() error {
	pr.mu.Lock()
	defer pr.mu.Unlock()

	// Check if file exists
	if _, err := os.Stat(pr.persistFile); os.IsNotExist(err) {
		// No projects file yet, that's ok
		return nil
	}

	data, err := os.ReadFile(pr.persistFile)
	if err != nil {
		return fmt.Errorf("failed to read projects file: %w", err)
	}

	var savedProjects []Project
	if err := json.Unmarshal(data, &savedProjects); err != nil {
		return fmt.Errorf("failed to unmarshal projects: %w", err)
	}

	// Create project states for each saved project
	for _, proj := range savedProjects {
		ps := NewProjectState(proj)
		pr.projects[proj.Path] = ps
	}

	// Set first project as current if we have any
	if len(savedProjects) > 0 {
		pr.currentProject = savedProjects[0].Path
	}

	return nil
}

// ReloadProjects clears current state and reloads from disk
// This is used by tests to ensure server state matches file state
func (pr *ProjectRegistry) ReloadProjects() error {
	pr.mu.Lock()
	defer pr.mu.Unlock()

	log.Printf("Reloading projects from %s", pr.persistFile)
	
	// Clear current state
	pr.projects = make(map[string]*ProjectState)
	pr.currentProject = ""

	// Check if file exists
	if _, err := os.Stat(pr.persistFile); os.IsNotExist(err) {
		// No projects file yet, that's ok
		log.Printf("No projects file found at %s", pr.persistFile)
		return nil
	}

	data, err := os.ReadFile(pr.persistFile)
	if err != nil {
		return fmt.Errorf("failed to read projects file: %w", err)
	}

	var savedProjects []Project
	if err := json.Unmarshal(data, &savedProjects); err != nil {
		return fmt.Errorf("failed to unmarshal projects: %w", err)
	}

	log.Printf("Loaded %d projects from disk", len(savedProjects))

	// Create project states for each saved project
	for _, proj := range savedProjects {
		ps := NewProjectState(proj)
		pr.projects[proj.Path] = ps
	}

	// Set first project as current if we have any
	if len(savedProjects) > 0 {
		pr.currentProject = savedProjects[0].Path
	}

	return nil
}

// SaveProjects persists the project list to disk
func (pr *ProjectRegistry) SaveProjects() error {
	pr.mu.RLock()
	defer pr.mu.RUnlock()
	return pr.saveProjectsLocked()
}

// saveProjectsLocked is an internal version that assumes lock is already held
func (pr *ProjectRegistry) saveProjectsLocked() error {
	// Extract just the Project data (not the full state)
	var projects []Project
	for _, ps := range pr.projects {
		projects = append(projects, ps.Project)
	}

	log.Printf("Saving %d projects to %s", len(projects), pr.persistFile)
	data, err := json.MarshalIndent(projects, "", "  ")
	if err != nil {
		return fmt.Errorf("failed to marshal projects: %w", err)
	}

	if err := os.WriteFile(pr.persistFile, data, 0644); err != nil {
		return fmt.Errorf("failed to write projects file: %w", err)
	}
	log.Printf("Successfully saved projects to %s", pr.persistFile)

	return nil
}

// AddProject adds a new project to the registry
func (pr *ProjectRegistry) AddProject(path string, name string) (*ProjectState, error) {
	pr.mu.Lock()
	defer pr.mu.Unlock()

	// Check if project already exists
	if _, exists := pr.projects[path]; exists {
		return nil, fmt.Errorf("project already exists: %s", path)
	}

	// Create new project
	project := Project{
		Name:       name,
		Path:       path,
		AddedAt:    time.Now(),
		LastActive: time.Now(),
	}

	// Create project state
	ps := NewProjectState(project)
	pr.projects[path] = ps

	// If this is the first project, make it current
	if pr.currentProject == "" {
		pr.currentProject = path
	}

	// Save to disk
	pr.saveProjectsLocked()

	return ps, nil
}

// RemoveProject removes a project from the registry
func (pr *ProjectRegistry) RemoveProject(path string) error {
	pr.mu.Lock()
	defer pr.mu.Unlock()

	ps, exists := pr.projects[path]
	if !exists {
		return fmt.Errorf("project not found: %s", path)
	}

	// Stop polling for this project
	ps.StopPolling()

	// Remove from map
	delete(pr.projects, path)

	// If this was the current project, select another
	if pr.currentProject == path {
		pr.currentProject = ""
		for p := range pr.projects {
			pr.currentProject = p
			break
		}
	}

	// Save to disk
	pr.saveProjectsLocked()

	return nil
}

// GetProject returns a specific project state
func (pr *ProjectRegistry) GetProject(path string) (*ProjectState, bool) {
	pr.mu.RLock()
	defer pr.mu.RUnlock()
	ps, exists := pr.projects[path]
	return ps, exists
}

// GetCurrentProject returns the currently selected project
func (pr *ProjectRegistry) GetCurrentProject() (*ProjectState, bool) {
	pr.mu.RLock()
	defer pr.mu.RUnlock()
	
	if pr.currentProject == "" {
		return nil, false
	}
	
	ps, exists := pr.projects[pr.currentProject]
	return ps, exists
}

// SetCurrentProject sets the currently selected project
func (pr *ProjectRegistry) SetCurrentProject(path string) error {
	pr.mu.Lock()
	defer pr.mu.Unlock()

	if _, exists := pr.projects[path]; !exists {
		return fmt.Errorf("project not found: %s", path)
	}

	pr.currentProject = path
	pr.projects[path].Project.LastActive = time.Now()
	
	return nil
}

// GetAllProjects returns all projects in consistent order (sorted by name)
func (pr *ProjectRegistry) GetAllProjects() []Project {
	pr.mu.RLock()
	defer pr.mu.RUnlock()

	var projects []Project
	for _, ps := range pr.projects {
		projects = append(projects, ps.Project)
	}
	
	// Sort projects by name for consistent ordering
	sort.Slice(projects, func(i, j int) bool {
		return projects[i].Name < projects[j].Name
	})
	
	return projects
}

// GetCurrentProjectPath returns the current project path
func (pr *ProjectRegistry) GetCurrentProjectPath() string {
	pr.mu.RLock()
	defer pr.mu.RUnlock()
	return pr.currentProject
}

// startFileSync periodically syncs with the filesystem to detect external changes
func (pr *ProjectRegistry) startFileSync() {
	ticker := time.NewTicker(1 * time.Second) // Check every second
	defer ticker.Stop()
	
	for {
		select {
		case <-ticker.C:
			pr.syncWithFile()
		case <-pr.stopSync:
			return
		}
	}
}

// syncWithFile checks if the file has changed and syncs if needed
func (pr *ProjectRegistry) syncWithFile() {
	// Check if file exists
	_, err := os.Stat(pr.persistFile)
	if err != nil {
		if !os.IsNotExist(err) {
			// File exists but we can't read it
			return
		}
		// File doesn't exist, clear all projects if we have any
		pr.mu.Lock()
		if len(pr.projects) > 0 {
			pr.projects = make(map[string]*ProjectState)
			pr.currentProject = ""
		}
		pr.mu.Unlock()
		return
	}
	
	// Read the file
	data, err := os.ReadFile(pr.persistFile)
	if err != nil {
		return
	}
	
	var fileProjects []Project
	if err := json.Unmarshal(data, &fileProjects); err != nil {
		return
	}
	
	// Check if file content differs from memory
	pr.mu.Lock()
	defer pr.mu.Unlock()
	
	// Quick check: if counts differ, we need to sync
	if len(fileProjects) != len(pr.projects) {
		pr.syncProjectsFromFile(fileProjects)
		return
	}
	
	// Detailed check: see if any projects are different
	for _, fp := range fileProjects {
		if _, exists := pr.projects[fp.Path]; !exists {
			pr.syncProjectsFromFile(fileProjects)
			return
		}
	}
}

// syncProjectsFromFile updates in-memory state from file projects
func (pr *ProjectRegistry) syncProjectsFromFile(fileProjects []Project) {
	// Clear current state
	pr.projects = make(map[string]*ProjectState)
	pr.currentProject = ""
	
	// Create project states for each saved project
	for _, proj := range fileProjects {
		ps := NewProjectState(proj)
		pr.projects[proj.Path] = ps
	}
	
	// Set first project as current if we have any
	if len(fileProjects) > 0 {
		pr.currentProject = fileProjects[0].Path
	}
	
	log.Printf("Synced %d projects from file", len(fileProjects))
}