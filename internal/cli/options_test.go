package cli

import (
	"reflect"
	"testing"
)

func TestUpOptions_ToArgs(t *testing.T) {
	tests := []struct {
		name     string
		options  *UpOptions
		expected []string
	}{
		{
			name:     "empty options",
			options:  &UpOptions{},
			expected: []string{},
		},
		{
			name: "basic options",
			options: &UpOptions{
				Build:  true,
				Watch:  true,
				Listen: "localhost:3000",
			},
			expected: []string{"--build", "--watch", "--listen", "localhost:3000"},
		},
		{
			name: "full options",
			options: &UpOptions{
				Build:      true,
				Watch:      true,
				SkipSynth:  true,
				ConfigFile: "ftl.yaml",
				ComponentIds: []string{"comp1", "comp2"},
				Env:        []string{"VAR1=value1"},
				Listen:     "localhost:8080",
				Quiet:      true,
			},
			expected: []string{
				"--build", "--watch", "--skip-synth",
				"--config", "ftl.yaml",
				"--component-id", "comp1", "--component-id", "comp2",
				"--env", "VAR1=value1",
				"--quiet",
				"--listen", "localhost:8080",
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := tt.options.ToArgs()

			// Handle nil vs empty slice comparison
			if len(result) == 0 && len(tt.expected) == 0 {
				return // Both are effectively empty
			}

			if !reflect.DeepEqual(result, tt.expected) {
				t.Errorf("ToArgs() = %v, want %v", result, tt.expected)
			}
		})
	}
}

func TestBuildOptions_ToArgs(t *testing.T) {
	tests := []struct {
		name     string
		options  *BuildOptions
		expected []string
	}{
		{
			name:     "empty options",
			options:  &BuildOptions{},
			expected: []string{},
		},
		{
			name: "skip synth",
			options: &BuildOptions{
				SkipSynth: true,
			},
			expected: []string{"--skip-synth"},
		},
		{
			name: "config file",
			options: &BuildOptions{
				ConfigFile: "custom.yaml",
			},
			expected: []string{"--config", "custom.yaml"},
		},
		{
			name: "both options",
			options: &BuildOptions{
				SkipSynth:  true,
				ConfigFile: "ftl.toml",
			},
			expected: []string{"--skip-synth", "--config", "ftl.toml"},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := tt.options.ToArgs()

			// Handle nil vs empty slice comparison
			if len(result) == 0 && len(tt.expected) == 0 {
				return // Both are effectively empty
			}

			if !reflect.DeepEqual(result, tt.expected) {
				t.Errorf("ToArgs() = %v, want %v", result, tt.expected)
			}
		})
	}
}

func TestDeployOptions_ToArgs(t *testing.T) {
	tests := []struct {
		name     string
		options  *DeployOptions
		expected []string
	}{
		{
			name:     "empty options",
			options:  &DeployOptions{},
			expected: []string{},
		},
		{
			name: "basic options",
			options: &DeployOptions{
				Environment: "production",
				ConfigFile:  "deploy.yaml",
				DryRun:      true,
			},
			expected: []string{
				"--environment", "production",
				"--config", "deploy.yaml",
				"--dry-run",
			},
		},
		{
			name: "with variables",
			options: &DeployOptions{
				Environment: "staging",
				Variables:   map[string]string{"KEY1": "value1", "KEY2": "value2"},
			},
			// Note: map iteration order is not guaranteed, so we'll test length and presence
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := tt.options.ToArgs()

			// Handle nil vs empty slice comparison
			if len(result) == 0 && len(tt.expected) == 0 {
				return // Both are effectively empty
			}

			// Special handling for test with variables (map iteration order)
			if tt.name == "with variables" {
				// Should have: --environment staging --variable KEY1=value1 --variable KEY2=value2
				expectedLength := 6 // --environment staging --variable KEY1=value1 --variable KEY2=value2
				if len(result) != expectedLength {
					t.Errorf("ToArgs() length = %d, want %d", len(result), expectedLength)
					return
				}
				
				// Check that it contains expected elements
				args := make(map[string]bool)
				for _, arg := range result {
					args[arg] = true
				}
				
				expected := []string{"--environment", "staging", "--variable", "KEY1=value1", "--variable", "KEY2=value2"}
				for _, exp := range expected {
					if !args[exp] {
						t.Errorf("ToArgs() missing expected arg: %s", exp)
					}
				}
				return
			}

			if !reflect.DeepEqual(result, tt.expected) {
				t.Errorf("ToArgs() = %v, want %v", result, tt.expected)
			}
		})
	}
}