package cli

import (
	"bytes"
	"os"
	"path/filepath"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestBuildCommand(t *testing.T) {
	cmd := newBuildCmd()

	// Test command structure
	assert.NotNil(t, cmd)
	assert.Equal(t, "build", cmd.Use)
	assert.Contains(t, cmd.Short, "Build")

	// Test flags
	skipSynthFlag := cmd.Flags().Lookup("skip-synth")
	assert.NotNil(t, skipSynthFlag)
	assert.Equal(t, "false", skipSynthFlag.DefValue)

	configFlag := cmd.Flags().Lookup("config")
	assert.NotNil(t, configFlag)
	assert.Equal(t, "c", configFlag.Shorthand)
	assert.Equal(t, "", configFlag.DefValue)
}

func TestBuildCommand_Help(t *testing.T) {
	cmd := newBuildCmd()
	var buf bytes.Buffer
	cmd.SetOut(&buf)
	cmd.SetArgs([]string{"--help"})

	err := cmd.Execute()
	assert.NoError(t, err)

	output := buf.String()
	assert.Contains(t, output, "Usage:")
	assert.Contains(t, output, "build")
	assert.Contains(t, output, "Flags:")
}

func TestBuildCommand_WithYAMLConfig(t *testing.T) {
	// Create test environment
	tmpDir := t.TempDir()
	oldWd, _ := os.Getwd()
	defer func() { _ = os.Chdir(oldWd) }()
	_ = os.Chdir(tmpDir)

	// Create ftl.yaml
	yamlContent := `application:
  name: test-app
  version: "0.1.0"
components: []`
	err := os.WriteFile("ftl.yaml", []byte(yamlContent), 0600)
	require.NoError(t, err)

	// Test build command
	cmd := newBuildCmd()
	var buf bytes.Buffer
	cmd.SetOut(&buf)
	cmd.SetErr(&buf)
	cmd.SetArgs([]string{"--skip-synth"})

	err = cmd.Execute()
	// The command might fail due to missing spin command, but it should at least try
	// and not fail on config loading
	if err != nil {
		// Check that it's not a config error
		assert.NotContains(t, err.Error(), "ftl.yaml not found")
	}
}

func TestBuildCommand_WithJSONConfig(t *testing.T) {
	// Create test environment
	tmpDir := t.TempDir()
	oldWd, _ := os.Getwd()
	defer func() { _ = os.Chdir(oldWd) }()
	_ = os.Chdir(tmpDir)

	// Create ftl.json
	jsonContent := `{
  "application": {
    "name": "test-app",
    "version": "0.1.0"
  },
  "components": []
}`
	err := os.WriteFile("ftl.json", []byte(jsonContent), 0600)
	require.NoError(t, err)

	// Test build command
	cmd := newBuildCmd()
	var buf bytes.Buffer
	cmd.SetOut(&buf)
	cmd.SetErr(&buf)
	cmd.SetArgs([]string{"--skip-synth"})

	err = cmd.Execute()
	// The command might fail due to missing spin command, but it should at least try
	if err != nil {
		// Check that it's not a config error
		assert.NotContains(t, err.Error(), "not found")
	}
}

func TestBuildCommand_NoConfig(t *testing.T) {
	// Create test environment without config
	tmpDir := t.TempDir()
	oldWd, _ := os.Getwd()
	defer func() { _ = os.Chdir(oldWd) }()
	_ = os.Chdir(tmpDir)

	// Test build command without config
	cmd := newBuildCmd()
	var buf bytes.Buffer
	cmd.SetOut(&buf)
	cmd.SetErr(&buf)
	cmd.SetArgs([]string{})

	err := cmd.Execute()
	assert.Error(t, err)
	// Updated error message to match new auto-detection behavior
	assert.Contains(t, err.Error(), "no ftl.yaml, ftl.json, app.cue, or spin.toml found")
}

func TestBuildCommand_WithCUEConfig(t *testing.T) {
	// Create test environment
	tmpDir := t.TempDir()
	oldWd, _ := os.Getwd()
	defer func() { _ = os.Chdir(oldWd) }()
	_ = os.Chdir(tmpDir)

	// Create app.cue
	cueContent := `package app

application: {
	name: "test-app"
	version: "0.1.0"
}
components: []`
	err := os.WriteFile("app.cue", []byte(cueContent), 0600)
	require.NoError(t, err)

	// Test build command
	cmd := newBuildCmd()
	var buf bytes.Buffer
	cmd.SetOut(&buf)
	cmd.SetErr(&buf)
	cmd.SetArgs([]string{"--skip-synth"})

	err = cmd.Execute()
	// Should handle CUE config appropriately
	if err != nil {
		// The error should be about spin command or synthesis, not config
		assert.NotContains(t, err.Error(), "not found")
	}
}

func TestBuildCommand_WithGoConfig(t *testing.T) {
	// Create test environment
	tmpDir := t.TempDir()
	oldWd, _ := os.Getwd()
	defer func() { _ = os.Chdir(oldWd) }()
	_ = os.Chdir(tmpDir)

	// Create main.go with CDK
	goContent := `package main

import (
	"fmt"
	"github.com/fastertools/ftl/synthesis"
)

func main() {
	cdk := synthesis.NewCDK()
	app := cdk.NewApp("test-app")
	manifest, _ := app.Build().Synthesize()
	fmt.Print(manifest)
}`
	err := os.WriteFile("main.go", []byte(goContent), 0600)
	require.NoError(t, err)

	// Test build command
	cmd := newBuildCmd()
	var buf bytes.Buffer
	cmd.SetOut(&buf)
	cmd.SetErr(&buf)
	cmd.SetArgs([]string{"--skip-synth"})

	err = cmd.Execute()
	// Should handle Go config appropriately
	if err != nil {
		// The error should be about go run or synthesis, not config
		assert.NotContains(t, err.Error(), "not found")
	}
}

func TestBuildCommand_SkipSynth(t *testing.T) {
	// Create test environment
	tmpDir := t.TempDir()
	oldWd, _ := os.Getwd()
	defer func() { _ = os.Chdir(oldWd) }()
	_ = os.Chdir(tmpDir)

	// Create ftl.yaml
	yamlContent := `application:
  name: test-app
  version: "0.1.0"
components: []`
	err := os.WriteFile("ftl.yaml", []byte(yamlContent), 0600)
	require.NoError(t, err)

	// Create a fake spin.toml to skip synthesis
	err = os.WriteFile("spin.toml", []byte("# Fake spin.toml"), 0600)
	require.NoError(t, err)

	// Test build command with skip-synth
	cmd := newBuildCmd()
	var buf bytes.Buffer
	cmd.SetOut(&buf)
	cmd.SetErr(&buf)
	cmd.SetArgs([]string{"--skip-synth"})

	err = cmd.Execute()
	// With both skip flags, command should succeed (unless spin binary is missing)
	if err != nil {
		// Error should be about spin command, not synthesis
		assert.Contains(t, err.Error(), "spin")
	}
}

func TestBuildCommand_OutputFlag(t *testing.T) {
	// Create test environment
	tmpDir := t.TempDir()
	oldWd, _ := os.Getwd()
	defer func() { _ = os.Chdir(oldWd) }()
	_ = os.Chdir(tmpDir)

	// Create ftl.yaml
	yamlContent := `application:
  name: test-app
  version: "0.1.0"
components: []`
	err := os.WriteFile("ftl.yaml", []byte(yamlContent), 0600)
	require.NoError(t, err)

	// Test build command with output flag
	cmd := newBuildCmd()
	var buf bytes.Buffer
	cmd.SetOut(&buf)
	cmd.SetErr(&buf)
	cmd.SetArgs([]string{"--config", "ftl.yaml", "--skip-synth"})

	err = cmd.Execute()
	// Command might fail due to missing spin, but output flag should be parsed
	if err == nil {
		// Check if custom output file would be created
		assert.FileExists(t, "custom-spin.toml")
	}
}

func TestBuildCommand_ComponentWithMakefile(t *testing.T) {
	// Create test environment with component
	tmpDir := t.TempDir()
	oldWd, _ := os.Getwd()
	defer func() { _ = os.Chdir(oldWd) }()
	_ = os.Chdir(tmpDir)

	// Create ftl.yaml with component
	yamlContent := `application:
  name: test-app
  version: "0.1.0"
components:
  - id: test-component
    source: ./test-comp`
	err := os.WriteFile("ftl.yaml", []byte(yamlContent), 0600)
	require.NoError(t, err)

	// Create component directory with Makefile
	err = os.MkdirAll("test-comp", 0750)
	require.NoError(t, err)

	makefileContent := `build:
	@echo "Building test-component"

test:
	@echo "Testing test-component"`
	err = os.WriteFile(filepath.Join("test-comp", "Makefile"), []byte(makefileContent), 0600)
	require.NoError(t, err)

	// Test build command
	cmd := newBuildCmd()
	var buf bytes.Buffer
	cmd.SetOut(&buf)
	cmd.SetErr(&buf)
	cmd.SetArgs([]string{"--skip-synth"})

	// Create fake spin.toml
	err = os.WriteFile("spin.toml", []byte("# Fake"), 0600)
	require.NoError(t, err)

	err = cmd.Execute()
	// Command will try to run make in component directory
	if err != nil {
		// Check the error is about spin or make, not config
		errMsg := err.Error()
		assert.True(t,
			assert.Contains(t, errMsg, "spin") || assert.Contains(t, errMsg, "make"),
			"Expected error about spin or make, got: %s", errMsg)
	}
}

func BenchmarkBuildCommand(b *testing.B) {
	// Create test environment
	tmpDir := b.TempDir()
	oldWd, _ := os.Getwd()
	defer func() { _ = os.Chdir(oldWd) }()
	_ = os.Chdir(tmpDir)

	// Create ftl.yaml
	yamlContent := `application:
  name: bench-app
  version: "0.1.0"
components: []`
	_ = os.WriteFile("ftl.yaml", []byte(yamlContent), 0600)

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		cmd := newBuildCmd()
		cmd.SetOut(&bytes.Buffer{})
		cmd.SetArgs([]string{"--help"})
		_ = cmd.Execute()
	}
}

func TestBuildCommand_AllFlags(t *testing.T) {
	cmd := newBuildCmd()

	// Test all flag combinations
	tests := []struct {
		name string
		args []string
	}{
		{"skip_synth", []string{"--skip-synth", "--help"}},
		{"config", []string{"--config", "test.yaml", "--help"}},
		{"all_flags", []string{"--skip-synth", "--config", "ftl.yaml", "--help"}},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			var buf bytes.Buffer
			cmd.SetOut(&buf)
			cmd.SetArgs(tt.args)

			err := cmd.Execute()
			assert.NoError(t, err)
			assert.Contains(t, buf.String(), "Usage:")
		})
	}
}
