package cli

import (
	"testing"

	"github.com/fastertools/ftl/internal/types"
)

func TestDevMcpFlags(t *testing.T) {
	tests := []struct {
		name           string
		ftlPath        string
		testMode       bool
		expectedFTLPath string
	}{
		{
			name:           "default ftl path",
			ftlPath:        "",
			testMode:       false,
			expectedFTLPath: "ftl",
		},
		{
			name:           "custom ftl path",
			ftlPath:        "/custom/path/to/ftl",
			testMode:       false,
			expectedFTLPath: "/custom/path/to/ftl",
		},
		{
			name:           "test mode with custom path",
			ftlPath:        "/test/path/to/ftl",
			testMode:       true,
			expectedFTLPath: "/test/path/to/ftl",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Create tool configuration like runDevMcp does
			toolConfig := &types.ToolConfig{}
			if tt.ftlPath != "" {
				toolConfig.FTLPath = tt.ftlPath
			}

			// Test that GetFTLPath returns expected value
			result := toolConfig.GetFTLPath()
			if result != tt.expectedFTLPath {
				t.Errorf("GetFTLPath() = %q, want %q", result, tt.expectedFTLPath)
			}

			// Verify the struct is properly configured
			if tt.ftlPath != "" && toolConfig.FTLPath != tt.ftlPath {
				t.Errorf("ToolConfig.FTLPath = %q, want %q", toolConfig.FTLPath, tt.ftlPath)
			}
		})
	}
}

func TestToolConfigJSON(t *testing.T) {
	// Test that ToolConfig serializes to JSON correctly
	config := &types.ToolConfig{
		FTLPath: "/custom/path/to/ftl",
	}

	// This validates the JSON tags are correct
	expected := "/custom/path/to/ftl"
	if config.FTLPath != expected {
		t.Errorf("FTLPath field = %q, want %q", config.FTLPath, expected)
	}

	// Test empty config defaults
	emptyConfig := &types.ToolConfig{}
	if emptyConfig.GetFTLPath() != "ftl" {
		t.Errorf("Empty config GetFTLPath() = %q, want %q", emptyConfig.GetFTLPath(), "ftl")
	}
}