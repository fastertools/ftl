package cmd

import (
	"os"
	"path/filepath"
	"testing"
)

func TestGenerateMCPConfig(t *testing.T) {
	projectName := "test-project"
	config := generateMCPConfig(projectName)

	// Verify the config contains expected elements
	if !contains(config, projectName) {
		t.Errorf("Config should contain project name %q", projectName)
	}

	if !contains(config, "name: "+projectName) {
		t.Errorf("Config should have name field set to %q", projectName)
	}

	if !contains(config, "auth:") {
		t.Errorf("Config should contain auth section")
	}

	if !contains(config, "mcp:") {
		t.Errorf("Config should contain mcp section")
	}

	if !contains(config, "components:") {
		t.Errorf("Config should contain components section")
	}
}

func TestRunInit(t *testing.T) {
	// Create a temporary directory for testing
	tempDir, err := os.MkdirTemp("", "spinc-test-*")
	if err != nil {
		t.Fatalf("Failed to create temp dir: %v", err)
	}
	defer os.RemoveAll(tempDir)

	// Change to temp directory
	oldWd, err := os.Getwd()
	if err != nil {
		t.Fatalf("Failed to get working directory: %v", err)
	}
	defer os.Chdir(oldWd)

	err = os.Chdir(tempDir)
	if err != nil {
		t.Fatalf("Failed to change to temp directory: %v", err)
	}

	// Test successful init
	projectName := "test-app"
	err = runInit(projectName, "mcp")
	if err != nil {
		t.Errorf("runInit failed: %v", err)
	}

	// Verify files were created
	configPath := filepath.Join(projectName, "spinc.yaml")
	if _, err := os.Stat(configPath); os.IsNotExist(err) {
		t.Errorf("Config file was not created at %s", configPath)
	}

	gitignorePath := filepath.Join(projectName, ".gitignore")
	if _, err := os.Stat(gitignorePath); os.IsNotExist(err) {
		t.Errorf(".gitignore file was not created at %s", gitignorePath)
	}

	// Verify config content
	configData, err := os.ReadFile(configPath)
	if err != nil {
		t.Errorf("Failed to read config file: %v", err)
	}

	configStr := string(configData)
	if !contains(configStr, "name: "+projectName) {
		t.Errorf("Config should contain project name %q", projectName)
	}
}

func TestRunInitInvalidTemplate(t *testing.T) {
	// Create a temporary directory for testing
	tempDir, err := os.MkdirTemp("", "spinc-test-*")
	if err != nil {
		t.Fatalf("Failed to create temp dir: %v", err)
	}
	defer os.RemoveAll(tempDir)

	// Change to temp directory
	oldWd, err := os.Getwd()
	if err != nil {
		t.Fatalf("Failed to get working directory: %v", err)
	}
	defer os.Chdir(oldWd)

	err = os.Chdir(tempDir)
	if err != nil {
		t.Fatalf("Failed to change to temp directory: %v", err)
	}

	// Test with invalid template
	err = runInit("test-app", "invalid-template")
	if err == nil {
		t.Errorf("Expected error for invalid template, but got none")
	}
}

func TestApplySetVariables(t *testing.T) {
	tests := []struct {
		name        string
		config      string
		setVars     []string
		expectError bool
		contains    []string
	}{
		{
			name: "set existing variable",
			config: `name: test-app
version: 1.0.0
debug: false`,
			setVars:     []string{"debug=true"},
			expectError: false,
			contains:    []string{"debug: true"},
		},
		{
			name: "set new variable",
			config: `name: test-app
version: 1.0.0`,
			setVars:     []string{"new_var=value"},
			expectError: false,
			contains:    []string{"new_var: value"},
		},
		{
			name: "multiple set variables",
			config: `name: test-app
version: 1.0.0`,
			setVars:     []string{"var1=value1", "var2=value2"},
			expectError: false,
			contains:    []string{"var1: value1", "var2: value2"},
		},
		{
			name:        "invalid set format",
			config:      `name: test-app`,
			setVars:     []string{"invalid-format"},
			expectError: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result, err := applySetVariables([]byte(tt.config), tt.setVars, "yaml")

			if tt.expectError {
				if err == nil {
					t.Errorf("Expected error but got none")
				}
				return
			}

			if err != nil {
				t.Errorf("Unexpected error: %v", err)
				return
			}

			resultStr := string(result)
			for _, expected := range tt.contains {
				if !contains(resultStr, expected) {
					t.Errorf("Expected result to contain %q, but got:\n%s", expected, resultStr)
				}
			}
		})
	}
}

func TestApplySetVariablesNonYAML(t *testing.T) {
	_, err := applySetVariables([]byte(`{"name": "test"}`), []string{"key=value"}, "json")
	if err == nil {
		t.Errorf("Expected error for non-YAML format, but got none")
	}
}

// Helper function to check if a string contains a substring
func contains(s, substr string) bool {
	return len(s) >= len(substr) && 
		   (s == substr || 
			(len(s) > len(substr) && 
			 (s[:len(substr)] == substr || 
			  s[len(s)-len(substr):] == substr || 
			  containsSubstring(s, substr))))
}

func containsSubstring(s, substr string) bool {
	for i := 0; i <= len(s)-len(substr); i++ {
		if s[i:i+len(substr)] == substr {
			return true
		}
	}
	return false
}