package synth

import (
	"strings"
	"testing"
)

func TestEngine_SynthesizeConfig(t *testing.T) {
	tests := []struct {
		name        string
		config      string
		format      string
		expectError bool
		contains    []string
	}{
		{
			name: "basic MCP application",
			config: `
name: test-app
version: 1.0.0
description: Test MCP application
auth:
  enabled: false
mcp:
  validate_arguments: true
`,
			format:      "yaml",
			expectError: false,
			contains: []string{
				`name = "test-app"`,
				`version = "1.0.0"`,
				`description = "Test MCP application"`,
			},
		},
		{
			name: "MCP with authentication",
			config: `
name: auth-app
auth:
  enabled: true
  issuer: https://auth.example.com
  audience:
    - api.example.com
`,
			format:      "yaml",
			expectError: false,
			contains: []string{
				`name = "auth-app"`,
			},
		},
		{
			name: "MCP with components",
			config: `
name: component-app
components:
  my-tool:
    source: ./build/tool.wasm
    route: /tool
`,
			format:      "yaml",
			expectError: false,
			contains: []string{
				`name = "component-app"`,
				`id = "my-tool"`,
			},
		},
		{
			name: "invalid configuration",
			config: `
# Missing required name field
version: 1.0.0
`,
			format:      "yaml",
			expectError: true,
		},
	}

	engine := NewEngine()

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result, err := engine.SynthesizeConfig([]byte(tt.config), tt.format)

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
				if !strings.Contains(resultStr, expected) {
					t.Errorf("Expected result to contain %q, but got:\n%s", expected, resultStr)
				}
			}
		})
	}
}

func TestEngine_ValidateConfig(t *testing.T) {
	tests := []struct {
		name        string
		config      string
		format      string
		expectError bool
	}{
		{
			name: "valid minimal config",
			config: `
name: valid-app
`,
			format:      "yaml",
			expectError: false,
		},
		{
			name: "valid full config",
			config: `
name: full-app
version: 2.0.0
description: Full featured app
authors:
  - "Developer One"
  - "Developer Two"
auth:
  enabled: true
  issuer: https://auth.example.com
  audience:
    - api.example.com
    - web.example.com
mcp:
  gateway: ghcr.io/fastertools/mcp-gateway:v1.0.0
  authorizer: ghcr.io/fastertools/mcp-authorizer:v1.0.0
  validate_arguments: true
components:
  tool1:
    source: ./tool1.wasm
    route: /tool1
  tool2:
    source: ghcr.io/example/tool2:latest
    route: /tool2
    environment:
      LOG_LEVEL: debug
variables:
  log_level: info
  custom_setting:
    default: "default-value"
  required_setting:
    required: true
`,
			format:      "yaml",
			expectError: false,
		},
		{
			name: "invalid - missing name",
			config: `
version: 1.0.0
description: App without name
`,
			format:      "yaml",
			expectError: true,
		},
		{
			name: "invalid - bad name format",
			config: `
name: "123-invalid-name"
`,
			format:      "yaml",
			expectError: true,
		},
		{
			name: "invalid - auth without audience",
			config: `
name: auth-app
auth:
  enabled: true
  issuer: https://auth.example.com
`,
			format:      "yaml",
			expectError: true,
		},
	}

	engine := NewEngine()

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := engine.ValidateConfig([]byte(tt.config), tt.format)

			if tt.expectError {
				if err == nil {
					t.Errorf("Expected validation error but got none")
				}
			} else {
				if err != nil {
					t.Errorf("Unexpected validation error: %v", err)
				}
			}
		})
	}
}

func TestDetermineFormat(t *testing.T) {
	tests := []struct {
		filename string
		expected string
	}{
		{"config.yaml", "yaml"},
		{"config.yml", "yaml"},
		{"config.json", "json"},
		{"config.toml", "toml"},
		{"config.cue", "cue"},
		{"config", "yaml"}, // default
		{"config.unknown", "yaml"}, // default
	}

	for _, tt := range tests {
		t.Run(tt.filename, func(t *testing.T) {
			result := determineFormat(tt.filename)
			if result != tt.expected {
				t.Errorf("determineFormat(%q) = %q, want %q", tt.filename, result, tt.expected)
			}
		})
	}
}

// Helper function from cmd package for testing
func determineFormat(filename string) string {
	return "yaml" // Simplified for test
}