package testing

import (
	"context"
	"encoding/json"
	"fmt"

	"github.com/fastertools/ftl/internal/testing/config"
)

// UpdateTestConfigArgs holds the arguments for the update_test_config tool
type UpdateTestConfigArgs struct {
	Updates map[string]interface{} `json:"updates"` // Key-value pairs to update
	Reset   bool                   `json:"reset,omitempty"` // Reset to defaults before updating
}

// UpdateTestConfig updates the test configuration
func UpdateTestConfig(ctx context.Context, args json.RawMessage) (interface{}, error) {
	var params UpdateTestConfigArgs
	if err := json.Unmarshal(args, &params); err != nil {
		return nil, fmt.Errorf("invalid arguments: %v", err)
	}

	// Get the test configuration
	testConfig := config.GetTestConfig()

	// Reset if requested
	if params.Reset {
		testConfig.Reset()
	}

	// Apply updates if provided
	if len(params.Updates) > 0 {
		if err := testConfig.Update(params.Updates); err != nil {
			return nil, fmt.Errorf("failed to update config: %v", err)
		}
	}

	// Return updated configuration
	jsonData, err := testConfig.ToJSON()
	if err != nil {
		return nil, fmt.Errorf("failed to serialize config: %v", err)
	}

	var result map[string]interface{}
	if err := json.Unmarshal(jsonData, &result); err != nil {
		return nil, fmt.Errorf("failed to parse config: %v", err)
	}

	return map[string]interface{}{
		"success": true,
		"message": "Test configuration updated",
		"config":  result,
	}, nil
}

// GetUpdateTestConfigSchema returns the JSON schema for the update_test_config tool
func GetUpdateTestConfigSchema() map[string]interface{} {
	return map[string]interface{}{
		"type": "object",
		"properties": map[string]interface{}{
			"updates": map[string]interface{}{
				"type":        "object",
				"description": "Key-value pairs to update in the configuration",
				"additionalProperties": true,
			},
			"reset": map[string]interface{}{
				"type":        "boolean",
				"description": "Reset configuration to defaults before applying updates",
				"default":     false,
			},
		},
		"required": []string{"updates"},
	}
}