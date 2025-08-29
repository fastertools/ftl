package testing

import (
	"context"
	"encoding/json"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
)

// TestProjectSetupRequest represents input for setting up test projects
type TestProjectSetupRequest struct {
	CreateEmpty      bool     `json:"create_empty"`
	ProjectNames     []string `json:"project_names,omitempty"`
	ProjectsFilePath string   `json:"projects_file_path,omitempty"`
}

// TestProjectResponse represents the result of test project operations
type TestProjectResponse struct {
	Success      bool                   `json:"success"`
	ProjectsFile string                 `json:"projects_file,omitempty"`
	Projects     []TestProjectInfo      `json:"projects,omitempty"`
	Error        string                 `json:"error,omitempty"`
}

// TestProjectInfo represents information about a test project
type TestProjectInfo struct {
	Name string `json:"name"`
	Path string `json:"path"`
}

// TestDirectoryRequest represents input for ensuring test directories
type TestDirectoryRequest struct {
	ProjectName  string `json:"project_name"`
	ProjectPath  string `json:"project_path,omitempty"`
	Language     string `json:"language,omitempty"`
	ToolName     string `json:"tool_name,omitempty"`
	FTLBinaryPath string `json:"ftl_binary_path,omitempty"`
}

// SetupTestProjects replicates TestHelpers.resetTestProjectsFile functionality
func SetupTestProjects(ctx context.Context, input json.RawMessage) (interface{}, error) {
	var req TestProjectSetupRequest
	if err := json.Unmarshal(input, &req); err != nil {
		return nil, fmt.Errorf("failed to parse test project setup request: %v", err)
	}

	// Determine projects file path
	projectsFile := req.ProjectsFilePath
	if projectsFile == "" {
		projectsFile = filepath.Join(".", ".e2e-projects", ".e2e-projects.json")
	}

	// Ensure directory exists
	projectsDir := filepath.Dir(projectsFile)
	if err := os.MkdirAll(projectsDir, 0755); err != nil {
		return TestProjectResponse{
			Success: false,
			Error:   fmt.Sprintf("failed to create projects directory: %v", err),
		}, nil
	}

	var projects []TestProjectInfo

	if req.CreateEmpty {
		// Create empty projects file (for new-user tests)
		projects = []TestProjectInfo{}
	} else {
		// Create with default projects if names provided
		for _, name := range req.ProjectNames {
			projects = append(projects, TestProjectInfo{
				Name: name,
				Path: generateTestProjectPath(name),
			})
		}

		// If no names provided, create a default single project
		if len(projects) == 0 {
			projects = append(projects, TestProjectInfo{
				Name: "test-project",
				Path: generateTestProjectPath("test-project"),
			})
		}
	}

	// Write projects file
	projectsJSON, err := json.MarshalIndent(projects, "", "  ")
	if err != nil {
		return TestProjectResponse{
			Success: false,
			Error:   fmt.Sprintf("failed to marshal projects JSON: %v", err),
		}, nil
	}

	if err := os.WriteFile(projectsFile, projectsJSON, 0644); err != nil {
		return TestProjectResponse{
			Success: false,
			Error:   fmt.Sprintf("failed to write projects file: %v", err),
		}, nil
	}

	return TestProjectResponse{
		Success:      true,
		ProjectsFile: projectsFile,
		Projects:     projects,
	}, nil
}

// EnsureTestDirectory replicates TestHelpers.ensureTestDirectory functionality
func EnsureTestDirectory(ctx context.Context, input json.RawMessage) (interface{}, error) {
	var req TestDirectoryRequest
	if err := json.Unmarshal(input, &req); err != nil {
		return nil, fmt.Errorf("failed to parse test directory request: %v", err)
	}

	if req.ProjectName == "" {
		return nil, fmt.Errorf("project_name is required")
	}

	// Generate project path if not provided
	projectPath := req.ProjectPath
	if projectPath == "" {
		projectPath = generateTestProjectPath(req.ProjectName)
	}

	// Determine FTL binary path
	ftlBinary := req.FTLBinaryPath
	if ftlBinary == "" {
		ftlBinary = "ftl" // Default to "ftl" binary in PATH
	}

	// Create directory if it doesn't exist
	if _, err := os.Stat(projectPath); os.IsNotExist(err) {
		if err := os.MkdirAll(projectPath, 0755); err != nil {
			return TestProjectResponse{
				Success: false,
				Error:   fmt.Sprintf("failed to create test directory: %v", err),
			}, nil
		}

		// Initialize FTL project
		initCmd := exec.CommandContext(ctx, ftlBinary, "init", req.ProjectName)
		initCmd.Dir = projectPath
		if output, err := initCmd.CombinedOutput(); err != nil {
			return TestProjectResponse{
				Success: false,
				Error:   fmt.Sprintf("ftl init failed: %v\nOutput: %s", err, string(output)),
			}, nil
		}

		// Add a tool to the project
		toolName := req.ToolName
		if toolName == "" {
			toolName = fmt.Sprintf("%s-tool", req.ProjectName)
		}

		language := req.Language
		if language == "" {
			language = "rust" // Default to Rust
		}

		// Change to the created project directory for ftl add
		projectDir := filepath.Join(projectPath, req.ProjectName)
		addCmd := exec.CommandContext(ctx, ftlBinary, "add", toolName, "-l", language)
		addCmd.Dir = projectDir
		if output, err := addCmd.CombinedOutput(); err != nil {
			return TestProjectResponse{
				Success: false,
				Error:   fmt.Sprintf("ftl add failed: %v\nOutput: %s", err, string(output)),
			}, nil
		}
	}

	return TestProjectResponse{
		Success: true,
		Projects: []TestProjectInfo{
			{
				Name: req.ProjectName,
				Path: projectPath,
			},
		},
	}, nil
}

// GenerateTestProjectName replicates TestHelpers.generateTestProjectName
func GenerateTestProjectName(ctx context.Context, input json.RawMessage) (interface{}, error) {
	var req struct {
		Prefix string `json:"prefix,omitempty"`
	}
	if err := json.Unmarshal(input, &req); err != nil {
		return nil, fmt.Errorf("failed to parse test project name request: %v", err)
	}

	prefix := req.Prefix
	if prefix == "" {
		prefix = "TestProject"
	}

	// Generate unique name with timestamp
	timestamp := fmt.Sprintf("%d", os.Getpid())
	name := fmt.Sprintf("%s_%s", prefix, timestamp)

	return map[string]interface{}{
		"name": name,
		"path": generateTestProjectPath(name),
	}, nil
}

// CleanupTestProjects provides comprehensive test cleanup
func CleanupTestProjects(ctx context.Context, input json.RawMessage) (interface{}, error) {
	var req struct {
		ProjectsFilePath string `json:"projects_file_path,omitempty"`
		RemoveDirectories bool   `json:"remove_directories"`
		KeepProjectsFile bool   `json:"keep_projects_file"`
	}
	if err := json.Unmarshal(input, &req); err != nil {
		return nil, fmt.Errorf("failed to parse cleanup request: %v", err)
	}

	projectsFile := req.ProjectsFilePath
	if projectsFile == "" {
		projectsFile = filepath.Join(".", ".e2e-projects", ".e2e-projects.json")
	}

	var cleanedProjects []string

	// Read existing projects file if it exists
	if _, err := os.Stat(projectsFile); err == nil {
		data, err := os.ReadFile(projectsFile)
		if err != nil {
			return TestProjectResponse{
				Success: false,
				Error:   fmt.Sprintf("failed to read projects file: %v", err),
			}, nil
		}

		var projects []TestProjectInfo
		if err := json.Unmarshal(data, &projects); err == nil {
			// Remove project directories if requested
			if req.RemoveDirectories {
				for _, project := range projects {
					if project.Path != "" && strings.Contains(project.Path, "test") {
						if err := os.RemoveAll(project.Path); err != nil {
							// Log but don't fail on cleanup errors
							fmt.Printf("Warning: failed to remove project directory %s: %v\n", project.Path, err)
						} else {
							cleanedProjects = append(cleanedProjects, project.Path)
						}
					}
				}
			}
		}
	}

	// Remove or reset projects file
	if !req.KeepProjectsFile {
		if err := os.Remove(projectsFile); err != nil && !os.IsNotExist(err) {
			return TestProjectResponse{
				Success: false,
				Error:   fmt.Sprintf("failed to remove projects file: %v", err),
			}, nil
		}
	} else {
		// Reset to empty array
		if err := os.WriteFile(projectsFile, []byte("[]"), 0644); err != nil {
			return TestProjectResponse{
				Success: false,
				Error:   fmt.Sprintf("failed to reset projects file: %v", err),
			}, nil
		}
	}

	return map[string]interface{}{
		"success":          true,
		"cleaned_projects": cleanedProjects,
		"projects_file":    projectsFile,
	}, nil
}

// Helper function to generate standardized test project paths
func generateTestProjectPath(name string) string {
	// Sanitize name for filesystem use
	safeName := strings.ToLower(strings.ReplaceAll(name, " ", "_"))
	return filepath.Join(".", ".e2e-projects", safeName)
}