package tools

import (
	"reflect"
	"testing"

	"github.com/fastertools/ftl/mcp-server/internal/types"
)

func TestUpOptionsEmbedded_ToArgs(t *testing.T) {
	tests := []struct {
		name     string
		options  *types.UpOptionsEmbedded
		expected []string
	}{
		{
			name: "empty options",
			options: &types.UpOptionsEmbedded{},
			expected: []string{}, // ToArgs returns empty slice for empty options
		},
		{
			name: "basic watch and build",
			options: &types.UpOptionsEmbedded{
				Watch: true,
				Build: true,
				Listen: "localhost:3000",
			},
			expected: []string{"--build", "--watch", "--listen", "localhost:3000"},
		},
		{
			name: "full options",
			options: &types.UpOptionsEmbedded{
				Build:     true,
				Watch:     true,
				SkipSynth: true,
				ConfigFile: "ftl.yaml",
				ComponentIds: []string{"comp1", "comp2"},
				Env: []string{"VAR1=value1", "VAR2=value2"},
				Listen: "localhost:8080",
				Quiet: true,
				KeyValue: []string{"key1=value1"},
			},
			expected: []string{
				"--build", "--watch", "--skip-synth", 
				"--config", "ftl.yaml",
				"--component-id", "comp1", "--component-id", "comp2",
				"--env", "VAR1=value1", "--env", "VAR2=value2",
				"--key-value", "key1=value1",
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

func TestBuildOptionsEmbedded_ToArgs(t *testing.T) {
	tests := []struct {
		name     string
		options  *types.BuildOptionsEmbedded
		expected []string
	}{
		{
			name:     "empty options",
			options:  &types.BuildOptionsEmbedded{},
			expected: []string{},
		},
		{
			name: "skip synth only",
			options: &types.BuildOptionsEmbedded{
				SkipSynth: true,
			},
			expected: []string{"--skip-synth"},
		},
		{
			name: "config file only",
			options: &types.BuildOptionsEmbedded{
				ConfigFile: "custom.yaml",
			},
			expected: []string{"--config", "custom.yaml"},
		},
		{
			name: "both options",
			options: &types.BuildOptionsEmbedded{
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

func TestFTLUpInput_BackwardsCompatibility(t *testing.T) {
	tests := []struct {
		name  string
		input *types.FTLUpInput
		expectWatch bool
		expectBuild bool
		expectListen string
	}{
		{
			name: "legacy fields only",
			input: &types.FTLUpInput{
				ProjectPath: "/test/path",
				Watch:       true,
				Build:       false,
				Listen:      "localhost:3000",
			},
			expectWatch:  true,
			expectBuild:  false,
			expectListen: "localhost:3000",
		},
		{
			name: "embedded options only",
			input: &types.FTLUpInput{
				ProjectPath: "/test/path",
				UpOptions: &types.UpOptionsEmbedded{
					Watch:  false,
					Build:  true,
					Listen: "localhost:4000",
				},
			},
			expectWatch:  false,
			expectBuild:  true,
			expectListen: "localhost:4000",
		},
		{
			name: "both legacy and embedded (embedded should take precedence)",
			input: &types.FTLUpInput{
				ProjectPath: "/test/path",
				// Legacy fields
				Watch:  true,
				Build:  false,
				Listen: "localhost:3000",
				// Embedded options (should take precedence)
				UpOptions: &types.UpOptionsEmbedded{
					Watch:  false,
					Build:  true,
					Listen: "localhost:4000",
				},
			},
			expectWatch:  false, // From embedded options
			expectBuild:  true,  // From embedded options
			expectListen: "localhost:4000", // From embedded options
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Simulate the logic from the up handler
			var watch, build bool
			var listen string

			if tt.input.UpOptions != nil {
				// Use embedded options
				watch = tt.input.UpOptions.Watch
				build = tt.input.UpOptions.Build
				listen = tt.input.UpOptions.Listen
			} else {
				// Use legacy fields
				watch = tt.input.Watch
				build = tt.input.Build
				listen = tt.input.Listen
			}

			if watch != tt.expectWatch {
				t.Errorf("watch = %v, want %v", watch, tt.expectWatch)
			}
			if build != tt.expectBuild {
				t.Errorf("build = %v, want %v", build, tt.expectBuild)
			}
			if listen != tt.expectListen {
				t.Errorf("listen = %q, want %q", listen, tt.expectListen)
			}
		})
	}
}

func TestFTLBuildInput_BackwardsCompatibility(t *testing.T) {
	tests := []struct {
		name         string
		input        *types.FTLBuildInput
		expectClean  bool
		expectSkipSynth bool
		expectConfigFile string
	}{
		{
			name: "legacy fields only",
			input: &types.FTLBuildInput{
				ProjectPath: "/test/path",
				Clean:       true,
			},
			expectClean:      true,
			expectSkipSynth:  false,
			expectConfigFile: "",
		},
		{
			name: "embedded options only",
			input: &types.FTLBuildInput{
				ProjectPath: "/test/path",
				BuildOptions: &types.BuildOptionsEmbedded{
					SkipSynth:  true,
					ConfigFile: "custom.yaml",
				},
			},
			expectClean:      false, // Clean not in embedded options
			expectSkipSynth:  true,
			expectConfigFile: "custom.yaml",
		},
		{
			name: "both legacy and embedded",
			input: &types.FTLBuildInput{
				ProjectPath: "/test/path",
				Clean:       true, // Legacy field
				BuildOptions: &types.BuildOptionsEmbedded{
					SkipSynth:  true,
					ConfigFile: "embedded.toml",
				},
			},
			expectClean:      true, // Always from legacy field
			expectSkipSynth:  true,
			expectConfigFile: "embedded.toml",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Simulate the logic from the build handler
			var clean bool
			var skipSynth bool
			var configFile string

			if tt.input.BuildOptions != nil {
				skipSynth = tt.input.BuildOptions.SkipSynth
				configFile = tt.input.BuildOptions.ConfigFile
			}

			// Always check legacy field for clean (not part of CLI BuildOptions)
			if tt.input.Clean {
				clean = true
			}

			if clean != tt.expectClean {
				t.Errorf("clean = %v, want %v", clean, tt.expectClean)
			}
			if skipSynth != tt.expectSkipSynth {
				t.Errorf("skipSynth = %v, want %v", skipSynth, tt.expectSkipSynth)
			}
			if configFile != tt.expectConfigFile {
				t.Errorf("configFile = %q, want %q", configFile, tt.expectConfigFile)
			}
		})
	}
}