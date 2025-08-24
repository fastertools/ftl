package testing

import (
	"context"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"

	"github.com/fastertools/ftl/internal/testing/config"
)

// CreateTestProjectArgs holds the arguments for the create_test_project tool
type CreateTestProjectArgs struct {
	Name      string                 `json:"name"`               // Project name
	Language  string                 `json:"language,omitempty"` // Project language (default from config)
	Type      string                 `json:"type,omitempty"`     // Project type (default from config)
	Overrides map[string]interface{} `json:"overrides,omitempty"` // Additional overrides
	CreateDir bool                   `json:"create_dir,omitempty"` // Create project directory
}

// CreateTestProject creates a new test project with standardized configuration
func CreateTestProject(ctx context.Context, args json.RawMessage) (interface{}, error) {
	var params CreateTestProjectArgs
	if err := json.Unmarshal(args, &params); err != nil {
		return nil, fmt.Errorf("invalid arguments: %v", err)
	}

	if params.Name == "" {
		return nil, fmt.Errorf("project name is required")
	}

	// Get the test configuration
	testConfig := config.GetTestConfig()

	// Create project using test config
	project := testConfig.CreateTestProject(params.Name, params.Overrides)

	// Override language and type if provided
	if params.Language != "" {
		project["language"] = params.Language
	}
	if params.Type != "" {
		project["type"] = params.Type
	}

	// Create project directory if requested
	if params.CreateDir {
		projectPath := project["path"].(string)
		if err := os.MkdirAll(projectPath, 0755); err != nil {
			return nil, fmt.Errorf("failed to create project directory: %v", err)
		}

		// Create basic project structure based on language
		language := project["language"].(string)
		switch language {
		case "rust":
			// Create Cargo.toml
			cargoToml := `[package]
name = "` + params.Name + `"
version = "0.1.0"
edition = "2021"

[dependencies]
ftl-sdk = "0.1"
`
			if err := os.WriteFile(filepath.Join(projectPath, "Cargo.toml"), []byte(cargoToml), 0644); err != nil {
				return nil, fmt.Errorf("failed to create Cargo.toml: %v", err)
			}

			// Create src directory
			srcDir := filepath.Join(projectPath, "src")
			if err := os.MkdirAll(srcDir, 0755); err != nil {
				return nil, fmt.Errorf("failed to create src directory: %v", err)
			}

			// Create lib.rs
			libRs := `use ftl_sdk::tool;

#[tool]
fn hello(name: String) -> String {
    format!("Hello, {}!", name)
}
`
			if err := os.WriteFile(filepath.Join(srcDir, "lib.rs"), []byte(libRs), 0644); err != nil {
				return nil, fmt.Errorf("failed to create lib.rs: %v", err)
			}

		case "python":
			// Create pyproject.toml
			pyprojectToml := `[project]
name = "` + params.Name + `"
version = "0.1.0"
dependencies = ["ftl-sdk"]
`
			if err := os.WriteFile(filepath.Join(projectPath, "pyproject.toml"), []byte(pyprojectToml), 0644); err != nil {
				return nil, fmt.Errorf("failed to create pyproject.toml: %v", err)
			}

			// Create main.py
			mainPy := `from ftl_sdk import tool

@tool
def hello(name: str) -> str:
    return f"Hello, {name}!"
`
			if err := os.WriteFile(filepath.Join(projectPath, "main.py"), []byte(mainPy), 0644); err != nil {
				return nil, fmt.Errorf("failed to create main.py: %v", err)
			}

		case "go":
			// Create go.mod
			goMod := `module ` + params.Name + `

go 1.21

require github.com/fastertools/ftl-sdk-go v0.1.0
`
			if err := os.WriteFile(filepath.Join(projectPath, "go.mod"), []byte(goMod), 0644); err != nil {
				return nil, fmt.Errorf("failed to create go.mod: %v", err)
			}

			// Create main.go
			mainGo := `package main

import (
	"github.com/fastertools/ftl-sdk-go/tool"
)

//ftl:tool
func Hello(name string) string {
	return "Hello, " + name + "!"
}

func main() {
	tool.Run()
}
`
			if err := os.WriteFile(filepath.Join(projectPath, "main.go"), []byte(mainGo), 0644); err != nil {
				return nil, fmt.Errorf("failed to create main.go: %v", err)
			}
		}

		project["directory_created"] = true
		project["files_created"] = true
	}

	// Update projects file
	projectsFile := testConfig.GetProjectsFilePath()
	
	// Read existing projects
	var projects []map[string]interface{}
	if data, err := os.ReadFile(projectsFile); err == nil {
		json.Unmarshal(data, &projects)
	}

	// Add new project
	projects = append(projects, project)

	// Write updated projects
	data, err := json.MarshalIndent(projects, "", "  ")
	if err != nil {
		return nil, fmt.Errorf("failed to marshal projects: %v", err)
	}
	if err := os.WriteFile(projectsFile, data, 0644); err != nil {
		return nil, fmt.Errorf("failed to update projects file: %v", err)
	}

	return map[string]interface{}{
		"success": true,
		"message": fmt.Sprintf("Test project '%s' created", params.Name),
		"project": project,
		"projects_file": projectsFile,
	}, nil
}

// GetCreateTestProjectSchema returns the JSON schema for the create_test_project tool
func GetCreateTestProjectSchema() map[string]interface{} {
	return map[string]interface{}{
		"type": "object",
		"properties": map[string]interface{}{
			"name": map[string]interface{}{
				"type":        "string",
				"description": "Name of the test project",
			},
			"language": map[string]interface{}{
				"type":        "string",
				"description": "Programming language (rust, python, go)",
				"enum":        []string{"rust", "python", "go"},
			},
			"type": map[string]interface{}{
				"type":        "string",
				"description": "Project type (tool, app, etc.)",
				"default":     "tool",
			},
			"overrides": map[string]interface{}{
				"type":        "object",
				"description": "Additional project properties to override",
				"additionalProperties": true,
			},
			"create_dir": map[string]interface{}{
				"type":        "boolean",
				"description": "Create project directory and basic files",
				"default":     false,
			},
		},
		"required": []string{"name"},
	}
}