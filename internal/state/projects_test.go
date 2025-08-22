package state

import (
	"os"
	"path/filepath"
	"testing"
	"time"
)

func TestNewProjectRegistry(t *testing.T) {
	registry := NewProjectRegistry("test_projects.json")
	if registry == nil {
		t.Fatal("Expected non-nil registry")
	}
	if registry.persistFile != "test_projects.json" {
		t.Errorf("Expected persistFile to be 'test_projects.json', got %s", registry.persistFile)
	}
	if len(registry.projects) != 0 {
		t.Errorf("Expected empty projects map, got %d projects", len(registry.projects))
	}
}

func TestAddProject(t *testing.T) {
	// Create temp file for testing
	tmpDir := t.TempDir()
	testFile := filepath.Join(tmpDir, "test_projects.json")
	
	registry := NewProjectRegistry(testFile)
	
	// Add a project
	ps, err := registry.AddProject("/test/path", "test-project")
	if err != nil {
		t.Fatalf("Failed to add project: %v", err)
	}
	
	if ps == nil {
		t.Fatal("Expected non-nil ProjectState")
	}
	
	if ps.Project.Name != "test-project" {
		t.Errorf("Expected project name 'test-project', got %s", ps.Project.Name)
	}
	
	if ps.Project.Path != "/test/path" {
		t.Errorf("Expected project path '/test/path', got %s", ps.Project.Path)
	}
	
	// Verify it's in the registry
	retrieved, exists := registry.GetProject("/test/path")
	if !exists {
		t.Fatal("Project should exist in registry")
	}
	
	if retrieved.Project.Name != "test-project" {
		t.Errorf("Retrieved project has wrong name: %s", retrieved.Project.Name)
	}
	
	// Try to add duplicate
	_, err = registry.AddProject("/test/path", "duplicate")
	if err == nil {
		t.Error("Expected error when adding duplicate project")
	}
}

func TestRemoveProject(t *testing.T) {
	tmpDir := t.TempDir()
	testFile := filepath.Join(tmpDir, "test_projects.json")
	
	registry := NewProjectRegistry(testFile)
	
	// Add a project
	_, err := registry.AddProject("/test/path", "test-project")
	if err != nil {
		t.Fatalf("Failed to add project: %v", err)
	}
	
	// Remove it
	err = registry.RemoveProject("/test/path")
	if err != nil {
		t.Fatalf("Failed to remove project: %v", err)
	}
	
	// Verify it's gone
	_, exists := registry.GetProject("/test/path")
	if exists {
		t.Error("Project should not exist after removal")
	}
	
	// Try to remove non-existent project
	err = registry.RemoveProject("/non/existent")
	if err == nil {
		t.Error("Expected error when removing non-existent project")
	}
}

func TestCurrentProjectManagement(t *testing.T) {
	tmpDir := t.TempDir()
	testFile := filepath.Join(tmpDir, "test_projects.json")
	
	registry := NewProjectRegistry(testFile)
	
	// No current project initially
	_, exists := registry.GetCurrentProject()
	if exists {
		t.Error("Should have no current project initially")
	}
	
	// Add first project - should become current
	_, err := registry.AddProject("/test/path1", "project1")
	if err != nil {
		t.Fatalf("Failed to add project: %v", err)
	}
	
	current, exists := registry.GetCurrentProject()
	if !exists {
		t.Fatal("Should have current project after adding first one")
	}
	
	if current.Project.Path != "/test/path1" {
		t.Errorf("Wrong current project: %s", current.Project.Path)
	}
	
	// Add second project
	_, err = registry.AddProject("/test/path2", "project2")
	if err != nil {
		t.Fatalf("Failed to add second project: %v", err)
	}
	
	// Switch current
	err = registry.SetCurrentProject("/test/path2")
	if err != nil {
		t.Fatalf("Failed to set current project: %v", err)
	}
	
	currentPath := registry.GetCurrentProjectPath()
	if currentPath != "/test/path2" {
		t.Errorf("Expected current path '/test/path2', got %s", currentPath)
	}
	
	// Try to set non-existent as current
	err = registry.SetCurrentProject("/non/existent")
	if err == nil {
		t.Error("Expected error when setting non-existent project as current")
	}
}

func TestProjectPersistence(t *testing.T) {
	tmpDir := t.TempDir()
	testFile := filepath.Join(tmpDir, "test_projects.json")
	
	// Create first registry and add projects
	registry1 := NewProjectRegistry(testFile)
	_, err := registry1.AddProject("/test/path1", "project1")
	if err != nil {
		t.Fatalf("Failed to add project1: %v", err)
	}
	
	_, err = registry1.AddProject("/test/path2", "project2")
	if err != nil {
		t.Fatalf("Failed to add project2: %v", err)
	}
	
	// Set current project
	err = registry1.SetCurrentProject("/test/path2")
	if err != nil {
		t.Fatalf("Failed to set current project: %v", err)
	}
	
	// Create second registry and load
	registry2 := NewProjectRegistry(testFile)
	err = registry2.LoadProjects()
	if err != nil {
		t.Fatalf("Failed to load projects: %v", err)
	}
	
	// Verify projects were loaded
	projects := registry2.GetAllProjects()
	if len(projects) != 2 {
		t.Fatalf("Expected 2 projects, got %d", len(projects))
	}
	
	// Verify project data
	p1, exists := registry2.GetProject("/test/path1")
	if !exists {
		t.Fatal("Project1 should exist after loading")
	}
	if p1.Project.Name != "project1" {
		t.Errorf("Project1 has wrong name: %s", p1.Project.Name)
	}
	
	// Note: Current project is not persisted, so it will be the first one
	currentPath := registry2.GetCurrentProjectPath()
	if currentPath == "" {
		t.Error("Should have a current project after loading")
	}
}

func TestRemoveCurrentProject(t *testing.T) {
	tmpDir := t.TempDir()
	testFile := filepath.Join(tmpDir, "test_projects.json")
	
	registry := NewProjectRegistry(testFile)
	
	// Add two projects
	_, err := registry.AddProject("/test/path1", "project1")
	if err != nil {
		t.Fatalf("Failed to add project1: %v", err)
	}
	
	_, err = registry.AddProject("/test/path2", "project2")
	if err != nil {
		t.Fatalf("Failed to add project2: %v", err)
	}
	
	// Set project1 as current
	err = registry.SetCurrentProject("/test/path1")
	if err != nil {
		t.Fatalf("Failed to set current project: %v", err)
	}
	
	// Remove current project
	err = registry.RemoveProject("/test/path1")
	if err != nil {
		t.Fatalf("Failed to remove current project: %v", err)
	}
	
	// Should have switched to the other project
	currentPath := registry.GetCurrentProjectPath()
	if currentPath != "/test/path2" {
		t.Errorf("Expected current to switch to '/test/path2', got %s", currentPath)
	}
	
	// Remove last project
	err = registry.RemoveProject("/test/path2")
	if err != nil {
		t.Fatalf("Failed to remove last project: %v", err)
	}
	
	// Should have no current project
	_, exists := registry.GetCurrentProject()
	if exists {
		t.Error("Should have no current project after removing all")
	}
}

func TestLoadNonExistentFile(t *testing.T) {
	tmpDir := t.TempDir()
	testFile := filepath.Join(tmpDir, "non_existent.json")
	
	registry := NewProjectRegistry(testFile)
	err := registry.LoadProjects()
	
	// Should not error on non-existent file
	if err != nil {
		t.Errorf("LoadProjects should not error on non-existent file: %v", err)
	}
	
	// Should have empty projects
	if len(registry.GetAllProjects()) != 0 {
		t.Error("Should have no projects when file doesn't exist")
	}
}

func TestSaveProjects(t *testing.T) {
	tmpDir := t.TempDir()
	testFile := filepath.Join(tmpDir, "save_test.json")
	
	registry := NewProjectRegistry(testFile)
	
	// Add projects with time delay to test LastActive
	_, err := registry.AddProject("/test/path1", "project1")
	if err != nil {
		t.Fatalf("Failed to add project: %v", err)
	}
	
	time.Sleep(10 * time.Millisecond)
	
	_, err = registry.AddProject("/test/path2", "project2")
	if err != nil {
		t.Fatalf("Failed to add project: %v", err)
	}
	
	// Save should happen automatically on add, but call it explicitly
	err = registry.SaveProjects()
	if err != nil {
		t.Fatalf("Failed to save projects: %v", err)
	}
	
	// Verify file exists
	if _, err := os.Stat(testFile); os.IsNotExist(err) {
		t.Fatal("Projects file should exist after save")
	}
	
	// Load in new registry to verify
	registry2 := NewProjectRegistry(testFile)
	err = registry2.LoadProjects()
	if err != nil {
		t.Fatalf("Failed to load saved projects: %v", err)
	}
	
	projects := registry2.GetAllProjects()
	if len(projects) != 2 {
		t.Fatalf("Expected 2 projects after load, got %d", len(projects))
	}
}