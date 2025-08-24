package testing

import (
	"context"
	"encoding/json"
	"fmt"

	"github.com/fastertools/ftl/internal/testing/config"
)

// GetTestConfigArgs holds the arguments for the get_test_config tool
type GetTestConfigArgs struct {
	Format string `json:"format,omitempty"` // Output format: "json" (default) or "summary"
}

// GetTestConfig returns the current test configuration
func GetTestConfig(ctx context.Context, args json.RawMessage) (interface{}, error) {
	var params GetTestConfigArgs
	if err := json.Unmarshal(args, &params); err != nil {
		return nil, fmt.Errorf("invalid arguments: %v", err)
	}

	// Default format
	if params.Format == "" {
		params.Format = "json"
	}

	// Get the test configuration
	testConfig := config.GetTestConfig()

	// Ensure directories exist
	if err := testConfig.EnsureDirectories(); err != nil {
		return nil, fmt.Errorf("failed to ensure directories: %v", err)
	}

	if params.Format == "summary" {
		// Return a summary view
		return map[string]interface{}{
			"test_mode":      testConfig.TestMode,
			"test_run_id":    testConfig.TestRunID,
			"environment":    testConfig.Environment,
			"base_url":       testConfig.BaseURL,
			"projects_file":  testConfig.GetProjectsFilePath(),
			"test_data_dir":  testConfig.GetTestDataPath(),
			"feature_flags": map[string]bool{
				"watch_mode":      testConfig.EnableWatchMode,
				"debug_logging":   testConfig.EnableDebugLogging,
				"screenshots":     testConfig.EnableScreenshots,
				"performance_log": testConfig.EnablePerformanceLog,
			},
			"test_metadata": map[string]interface{}{
				"suite":    testConfig.TestSuite,
				"file":     testConfig.TestFile,
				"name":     testConfig.TestName,
				"started":  testConfig.StartedAt,
				"updated":  testConfig.LastUpdated,
			},
		}, nil
	}

	// Return full JSON configuration
	jsonData, err := testConfig.ToJSON()
	if err != nil {
		return nil, fmt.Errorf("failed to serialize config: %v", err)
	}

	var result map[string]interface{}
	if err := json.Unmarshal(jsonData, &result); err != nil {
		return nil, fmt.Errorf("failed to parse config: %v", err)
	}

	return result, nil
}

// GetTestConfigSchema returns the JSON schema for the get_test_config tool
func GetTestConfigSchema() map[string]interface{} {
	return map[string]interface{}{
		"type": "object",
		"properties": map[string]interface{}{
			"format": map[string]interface{}{
				"type":        "string",
				"description": "Output format: 'json' (full config) or 'summary' (key fields only)",
				"enum":        []string{"json", "summary"},
				"default":     "json",
			},
		},
	}
}