package project

import (
	"fmt"
	"os"
	"path/filepath"
)

// PathOptions holds options for project path resolution
type PathOptions struct {
	// ProjectPath is the base path specified by --project-path flag
	ProjectPath string
	// CreateNew indicates whether we're creating a new project (true for init, false for add/up)
	CreateNew bool
	// ProjectName is used when CreateNew is true to create subdirectory
	ProjectName string
}

// ResolveWorkingDirectory resolves the working directory for a command based on options
// For init: returns ProjectPath/ProjectName (creates new subdirectory)
// For add/up: returns ProjectPath or current directory if empty
func ResolveWorkingDirectory(opts PathOptions) (string, error) {
	if opts.CreateNew {
		// For init command - create new project directory
		if opts.ProjectPath != "" {
			return filepath.Join(opts.ProjectPath, opts.ProjectName), nil
		}
		return opts.ProjectName, nil
	}

	// For existing project commands (add, up) - work in project directory
	if opts.ProjectPath != "" {
		return opts.ProjectPath, nil
	}
	return ".", nil
}

// ValidateProjectDirectory checks if the directory is a valid FTL project
// (contains ftl.yaml, ftl.json, app.cue, or spin.toml)
func ValidateProjectDirectory(dir string) error {
	configFiles := []string{"ftl.yaml", "ftl.json", "app.cue", "spin.toml"}
	
	for _, configFile := range configFiles {
		if _, err := os.Stat(filepath.Join(dir, configFile)); err == nil {
			return nil // Found a valid config file
		}
	}
	
	return fmt.Errorf("not in an FTL project directory (no ftl.yaml, ftl.json, app.cue, or spin.toml found): %s", dir)
}

// EnsureDirectoryExists creates the directory if it doesn't exist
func EnsureDirectoryExists(dir string) error {
	if err := os.MkdirAll(dir, 0750); err != nil {
		return fmt.Errorf("failed to create directory %s: %w", dir, err)
	}
	return nil
}

// CheckDirectoryExists checks if directory already exists (for init command validation)
func CheckDirectoryExists(dir string) (bool, error) {
	_, err := os.Stat(dir)
	if os.IsNotExist(err) {
		return false, nil
	}
	if err != nil {
		return false, err
	}
	return true, nil
}

// StandardProjectPathFlag returns consistent flag description for project-path
func StandardProjectPathFlag(commandType string) string {
	switch commandType {
	case "init":
		return "base path where project should be created (defaults to current directory)"
	case "existing":
		return "path to FTL project directory (defaults to current directory)"
	default:
		return "path to project directory (defaults to current directory)"
	}
}