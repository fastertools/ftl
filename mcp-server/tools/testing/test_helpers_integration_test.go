package testing

import (
	"context"
	"encoding/json"
	"os"
	"path/filepath"
	"testing"
)

func TestSetupTestProjects(t *testing.T) {
	tests := []struct {
		name           string
		input          TestProjectSetupRequest
		expectSuccess  bool
		expectProjects int
	}{
		{
			name: "create empty projects",
			input: TestProjectSetupRequest{
				CreateEmpty:      true,
				ProjectsFilePath: filepath.Join(t.TempDir(), "test-projects.json"),
			},
			expectSuccess:  true,
			expectProjects: 0,
		},
		{
			name: "create with single project",
			input: TestProjectSetupRequest{
				CreateEmpty:      false,
				ProjectNames:     []string{"test-project-1"},
				ProjectsFilePath: filepath.Join(t.TempDir(), "test-projects.json"),
			},
			expectSuccess:  true,
			expectProjects: 1,
		},
		{
			name: "create with multiple projects",
			input: TestProjectSetupRequest{
				CreateEmpty:      false,
				ProjectNames:     []string{"project-1", "project-2", "project-3"},
				ProjectsFilePath: filepath.Join(t.TempDir(), "test-projects.json"),
			},
			expectSuccess:  true,
			expectProjects: 3,
		},
		{
			name: "create default project",
			input: TestProjectSetupRequest{
				CreateEmpty:      false,
				ProjectsFilePath: filepath.Join(t.TempDir(), "test-projects.json"),
			},
			expectSuccess:  true,
			expectProjects: 1, // Should create one default project
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			inputJSON, err := json.Marshal(tt.input)
			if err != nil {
				t.Fatalf("Failed to marshal input: %v", err)
			}

			result, err := SetupTestProjects(context.Background(), inputJSON)
			if err != nil {
				if tt.expectSuccess {
					t.Fatalf("Expected success but got error: %v", err)
				}
				return
			}

			if !tt.expectSuccess {
				t.Fatal("Expected error but got success")
			}

			// Verify result structure
			response, ok := result.(TestProjectResponse)
			if !ok {
				t.Fatalf("Expected TestProjectResponse but got %T", result)
			}

			if !response.Success {
				t.Fatalf("Expected success but got error: %s", response.Error)
			}

			if len(response.Projects) != tt.expectProjects {
				t.Fatalf("Expected %d projects but got %d", tt.expectProjects, len(response.Projects))
			}

			// Verify projects file was created
			if _, err := os.Stat(tt.input.ProjectsFilePath); err != nil {
				t.Fatalf("Projects file was not created: %v", err)
			}

			// Verify file contents
			data, err := os.ReadFile(tt.input.ProjectsFilePath)
			if err != nil {
				t.Fatalf("Failed to read projects file: %v", err)
			}

			var projects []TestProjectInfo
			if err := json.Unmarshal(data, &projects); err != nil {
				t.Fatalf("Failed to parse projects JSON: %v", err)
			}

			if len(projects) != tt.expectProjects {
				t.Fatalf("Expected %d projects in file but got %d", tt.expectProjects, len(projects))
			}
		})
	}
}

func TestGenerateTestProjectName(t *testing.T) {
	tests := []struct {
		name   string
		prefix string
	}{
		{
			name:   "default prefix",
			prefix: "",
		},
		{
			name:   "custom prefix",
			prefix: "CustomProject",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			input := struct {
				Prefix string `json:"prefix,omitempty"`
			}{
				Prefix: tt.prefix,
			}

			inputJSON, err := json.Marshal(input)
			if err != nil {
				t.Fatalf("Failed to marshal input: %v", err)
			}

			result, err := GenerateTestProjectName(context.Background(), inputJSON)
			if err != nil {
				t.Fatalf("GenerateTestProjectName failed: %v", err)
			}

			// Verify result structure
			response, ok := result.(map[string]interface{})
			if !ok {
				t.Fatalf("Expected map[string]interface{} but got %T", result)
			}

			name, exists := response["name"]
			if !exists {
				t.Fatal("Result missing 'name' field")
			}

			nameStr, ok := name.(string)
			if !ok {
				t.Fatalf("Expected string name but got %T", name)
			}

			expectedPrefix := tt.prefix
			if expectedPrefix == "" {
				expectedPrefix = "TestProject"
			}

			if nameStr == "" {
				t.Fatal("Generated name is empty")
			}

			// Name should contain prefix
			if len(nameStr) <= len(expectedPrefix) {
				t.Fatalf("Generated name '%s' should be longer than prefix '%s'", nameStr, expectedPrefix)
			}

			path, exists := response["path"]
			if !exists {
				t.Fatal("Result missing 'path' field")
			}

			pathStr, ok := path.(string)
			if !ok {
				t.Fatalf("Expected string path but got %T", path)
			}

			if pathStr == "" {
				t.Fatal("Generated path is empty")
			}
		})
	}
}

func TestCleanupTestProjects(t *testing.T) {
	// Setup test directory structure
	tempDir := t.TempDir()
	projectsFile := filepath.Join(tempDir, "test-projects.json")
	
	// Create test projects file
	projects := []TestProjectInfo{
		{Name: "test-project-1", Path: filepath.Join(tempDir, "project-1")},
		{Name: "test-project-2", Path: filepath.Join(tempDir, "project-2")},
	}
	
	projectsJSON, err := json.MarshalIndent(projects, "", "  ")
	if err != nil {
		t.Fatalf("Failed to marshal test projects: %v", err)
	}
	
	if err := os.WriteFile(projectsFile, projectsJSON, 0644); err != nil {
		t.Fatalf("Failed to create test projects file: %v", err)
	}

	// Create test project directories
	for _, project := range projects {
		if err := os.MkdirAll(project.Path, 0755); err != nil {
			t.Fatalf("Failed to create test project directory: %v", err)
		}
		// Create a test file to verify cleanup
		testFile := filepath.Join(project.Path, "test.txt")
		if err := os.WriteFile(testFile, []byte("test"), 0644); err != nil {
			t.Fatalf("Failed to create test file: %v", err)
		}
	}

	tests := []struct {
		name                 string
		removeDirectories    bool
		keepProjectsFile     bool
		expectDirectoriesGone bool
		expectProjectsFileExists bool
	}{
		{
			name:                     "remove directories and file",
			removeDirectories:        true,
			keepProjectsFile:         false,
			expectDirectoriesGone:    true,
			expectProjectsFileExists: false,
		},
		{
			name:                     "remove directories keep file",
			removeDirectories:        true,
			keepProjectsFile:         true,
			expectDirectoriesGone:    true,
			expectProjectsFileExists: true,
		},
		{
			name:                     "keep directories remove file",
			removeDirectories:        false,
			keepProjectsFile:         false,
			expectDirectoriesGone:    false,
			expectProjectsFileExists: false,
		},
		{
			name:                     "keep directories and file",
			removeDirectories:        false,
			keepProjectsFile:         true,
			expectDirectoriesGone:    false,
			expectProjectsFileExists: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Create fresh setup for each test
			testProjectsFile := filepath.Join(t.TempDir(), "test-projects.json")
			if err := os.WriteFile(testProjectsFile, projectsJSON, 0644); err != nil {
				t.Fatalf("Failed to create test projects file: %v", err)
			}

			input := struct {
				ProjectsFilePath  string `json:"projects_file_path,omitempty"`
				RemoveDirectories bool   `json:"remove_directories"`
				KeepProjectsFile  bool   `json:"keep_projects_file"`
			}{
				ProjectsFilePath:  testProjectsFile,
				RemoveDirectories: tt.removeDirectories,
				KeepProjectsFile:  tt.keepProjectsFile,
			}

			inputJSON, err := json.Marshal(input)
			if err != nil {
				t.Fatalf("Failed to marshal input: %v", err)
			}

			result, err := CleanupTestProjects(context.Background(), inputJSON)
			if err != nil {
				t.Fatalf("CleanupTestProjects failed: %v", err)
			}

			// Verify result structure
			response, ok := result.(map[string]interface{})
			if !ok {
				t.Fatalf("Expected map[string]interface{} but got %T", result)
			}

			success, exists := response["success"]
			if !exists {
				t.Fatal("Result missing 'success' field")
			}

			successBool, ok := success.(bool)
			if !ok {
				t.Fatalf("Expected bool success but got %T", success)
			}

			if !successBool {
				t.Fatal("Expected cleanup to succeed")
			}

			// Check projects file state
			_, err = os.Stat(testProjectsFile)
			if tt.expectProjectsFileExists {
				if os.IsNotExist(err) {
					t.Fatal("Expected projects file to exist but it was removed")
				}
				
				// If file should exist and be kept, verify it was reset to empty
				if tt.keepProjectsFile {
					data, err := os.ReadFile(testProjectsFile)
					if err != nil {
						t.Fatalf("Failed to read projects file: %v", err)
					}
					
					var resetProjects []TestProjectInfo
					if err := json.Unmarshal(data, &resetProjects); err != nil {
						t.Fatalf("Failed to parse reset projects file: %v", err)
					}
					
					if len(resetProjects) != 0 {
						t.Fatalf("Expected empty projects file but got %d projects", len(resetProjects))
					}
				}
			} else {
				if !os.IsNotExist(err) {
					t.Fatal("Expected projects file to be removed but it still exists")
				}
			}
		})
	}
}

func TestGenerateTestProjectPath(t *testing.T) {
	tests := []struct {
		name     string
		input    string
		expected string
	}{
		{
			name:     "simple name",
			input:    "TestProject",
			expected: filepath.Join(".", ".e2e-projects", "testproject"),
		},
		{
			name:     "name with spaces",
			input:    "Test Project Name",
			expected: filepath.Join(".", ".e2e-projects", "test_project_name"),
		},
		{
			name:     "mixed case",
			input:    "MixedCaseProject",
			expected: filepath.Join(".", ".e2e-projects", "mixedcaseproject"),
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := generateTestProjectPath(tt.input)
			if result != tt.expected {
				t.Fatalf("Expected path '%s' but got '%s'", tt.expected, result)
			}
		})
	}
}