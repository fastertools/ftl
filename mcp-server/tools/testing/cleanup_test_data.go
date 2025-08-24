package testing

import (
	"context"
	"encoding/json"
	"fmt"
	"os"

	"github.com/fastertools/ftl/internal/testing/config"
)

// CleanupTestDataArgs holds the arguments for the cleanup_test_data tool
type CleanupTestDataArgs struct {
	KeepProjectsFile bool `json:"keep_projects_file,omitempty"` // Keep the projects JSON file
	KeepLogs         bool `json:"keep_logs,omitempty"`          // Keep log files
	Force            bool `json:"force,omitempty"`              // Force cleanup without confirmation
}

// CleanupTestDataResult holds the result of the cleanup operation
type CleanupTestDataResult struct {
	Success          bool     `json:"success"`
	Message          string   `json:"message"`
	DeletedDirs      []string `json:"deleted_dirs,omitempty"`
	DeletedFiles     []string `json:"deleted_files,omitempty"`
	SkippedItems     []string `json:"skipped_items,omitempty"`
	Errors           []string `json:"errors,omitempty"`
}

// CleanupTestData removes all test data and resets configuration
func CleanupTestData(ctx context.Context, args json.RawMessage) (interface{}, error) {
	var params CleanupTestDataArgs
	if err := json.Unmarshal(args, &params); err != nil {
		return nil, fmt.Errorf("invalid arguments: %v", err)
	}

	result := CleanupTestDataResult{
		Success:      true,
		DeletedDirs:  []string{},
		DeletedFiles: []string{},
		SkippedItems: []string{},
		Errors:       []string{},
	}

	// Get the test configuration
	testConfig := config.GetTestConfig()

	// Track what we're cleaning
	itemsToClean := []struct {
		path     string
		itemType string
		skip     bool
	}{
		{testConfig.GetTestDataPath(), "directory", false},
		{testConfig.TempDir, "directory", false},
		{testConfig.ScreenshotDir, "directory", false},
		{testConfig.LogDir, "directory", params.KeepLogs},
		{testConfig.GetProjectsFilePath(), "file", params.KeepProjectsFile},
	}

	// Clean each item
	for _, item := range itemsToClean {
		if item.skip {
			result.SkippedItems = append(result.SkippedItems, item.path)
			continue
		}

		if item.itemType == "directory" {
			if err := os.RemoveAll(item.path); err != nil {
				if !os.IsNotExist(err) {
					result.Errors = append(result.Errors, fmt.Sprintf("Failed to remove %s: %v", item.path, err))
					result.Success = false
				}
			} else {
				result.DeletedDirs = append(result.DeletedDirs, item.path)
			}
		} else if item.itemType == "file" {
			if err := os.Remove(item.path); err != nil {
				if !os.IsNotExist(err) {
					result.Errors = append(result.Errors, fmt.Sprintf("Failed to remove %s: %v", item.path, err))
					result.Success = false
				}
			} else {
				result.DeletedFiles = append(result.DeletedFiles, item.path)
			}
		}
	}

	// Reset configuration
	testConfig.Reset()

	// Recreate directories for next test run
	if err := testConfig.EnsureDirectories(); err != nil {
		result.Errors = append(result.Errors, fmt.Sprintf("Failed to recreate directories: %v", err))
		result.Success = false
	}

	// Create empty projects file if it was deleted
	if !params.KeepProjectsFile {
		projectsFile := testConfig.GetProjectsFilePath()
		if err := os.WriteFile(projectsFile, []byte("[]"), 0644); err != nil {
			result.Errors = append(result.Errors, fmt.Sprintf("Failed to create empty projects file: %v", err))
			result.Success = false
		}
	}

	// Set appropriate message
	if result.Success {
		result.Message = fmt.Sprintf("Successfully cleaned up test data (deleted %d directories, %d files)", 
			len(result.DeletedDirs), len(result.DeletedFiles))
	} else {
		result.Message = fmt.Sprintf("Cleanup completed with %d errors", len(result.Errors))
	}

	return result, nil
}

// GetCleanupTestDataSchema returns the JSON schema for the cleanup_test_data tool
func GetCleanupTestDataSchema() map[string]interface{} {
	return map[string]interface{}{
		"type": "object",
		"properties": map[string]interface{}{
			"keep_projects_file": map[string]interface{}{
				"type":        "boolean",
				"description": "Keep the projects JSON file",
				"default":     false,
			},
			"keep_logs": map[string]interface{}{
				"type":        "boolean",
				"description": "Keep log files",
				"default":     false,
			},
			"force": map[string]interface{}{
				"type":        "boolean",
				"description": "Force cleanup without confirmation",
				"default":     false,
			},
		},
	}
}