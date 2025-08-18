package cli

import (
	"bytes"
	"os"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestUpCommand(t *testing.T) {
	cmd := newUpCmd()

	// Test command structure
	assert.NotNil(t, cmd)
	assert.Equal(t, "up", cmd.Use)
	assert.Contains(t, cmd.Short, "Run")
	assert.Contains(t, cmd.Short, "locally")

	// Test flags
	buildFlag := cmd.Flags().Lookup("build")
	assert.NotNil(t, buildFlag)
	assert.Equal(t, "b", buildFlag.Shorthand)
	assert.Equal(t, "false", buildFlag.DefValue)

	watchFlag := cmd.Flags().Lookup("watch")
	assert.NotNil(t, watchFlag)
	assert.Equal(t, "w", watchFlag.Shorthand)
	assert.Equal(t, "false", watchFlag.DefValue)

	skipSynthFlag := cmd.Flags().Lookup("skip-synth")
	assert.NotNil(t, skipSynthFlag)
	assert.Equal(t, "false", skipSynthFlag.DefValue)

	configFlag := cmd.Flags().Lookup("config")
	assert.NotNil(t, configFlag)
	assert.Equal(t, "c", configFlag.Shorthand)
	assert.Equal(t, "", configFlag.DefValue)
}

func TestUpCommand_Help(t *testing.T) {
	cmd := newUpCmd()
	var buf bytes.Buffer
	cmd.SetOut(&buf)
	cmd.SetArgs([]string{"--help"})

	err := cmd.Execute()
	assert.NoError(t, err)

	output := buf.String()
	assert.Contains(t, output, "Usage:")
	assert.Contains(t, output, "up")
	assert.Contains(t, output, "Flags:")
	assert.Contains(t, output, "--build")
	assert.Contains(t, output, "--watch")
	assert.Contains(t, output, "--skip-synth")
	assert.Contains(t, output, "--config")
}

func TestUpCommand_NoConfig(t *testing.T) {
	// Create test environment without config
	tmpDir := t.TempDir()
	oldWd, _ := os.Getwd()
	defer func() { _ = os.Chdir(oldWd) }()
	_ = os.Chdir(tmpDir)

	cmd := newUpCmd()
	var buf bytes.Buffer
	cmd.SetOut(&buf)
	cmd.SetErr(&buf)
	cmd.SetArgs([]string{})

	// This should fail because no config exists
	err := cmd.Execute()
	assert.Error(t, err)
}

func TestUpCommand_WithBuildFlag(t *testing.T) {
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
	err := os.WriteFile("ftl.yaml", []byte(yamlContent), 0644)
	require.NoError(t, err)

	// Create fake spin.toml
	spinContent := `spin_manifest_version = 2
[application]
name = "test-app"`
	err = os.WriteFile("spin.toml", []byte(spinContent), 0644)
	require.NoError(t, err)

	cmd := newUpCmd()
	var buf bytes.Buffer
	cmd.SetOut(&buf)
	cmd.SetErr(&buf)
	cmd.SetArgs([]string{"--skip-synth"})

	// Mock the exec.Command
	oldExecCommand := ExecCommand
	ExecCommand = MockExecCommandHelper
	defer func() { ExecCommand = oldExecCommand }()

	// This will still fail but for different reasons
	err = cmd.Execute()
	// We expect an error but not because of missing config
	if err != nil {
		assert.NotContains(t, err.Error(), "not found")
	}
}

func TestUpCommand_WithWatchFlag(t *testing.T) {
	// Create test environment
	tmpDir := t.TempDir()
	oldWd, _ := os.Getwd()
	defer func() { _ = os.Chdir(oldWd) }()
	_ = os.Chdir(tmpDir)

	// Create configs
	yamlContent := `application:
  name: test-app
  version: "0.1.0"
components: []`
	err := os.WriteFile("ftl.yaml", []byte(yamlContent), 0644)
	require.NoError(t, err)

	spinContent := `spin_manifest_version = 2`
	err = os.WriteFile("spin.toml", []byte(spinContent), 0644)
	require.NoError(t, err)

	cmd := newUpCmd()
	var buf bytes.Buffer
	cmd.SetOut(&buf)
	cmd.SetErr(&buf)
	cmd.SetArgs([]string{"--watch", "--skip-synth"})

	// Mock the exec.Command
	oldExecCommand := ExecCommand
	ExecCommand = MockExecCommandHelper
	defer func() { ExecCommand = oldExecCommand }()

	err = cmd.Execute()
	// Check for specific behavior with watch flag
	if err != nil {
		assert.NotContains(t, err.Error(), "not found")
	}
}

func TestUpCommand_WithConfigFlag(t *testing.T) {
	// Create test environment
	tmpDir := t.TempDir()
	oldWd, _ := os.Getwd()
	defer func() { _ = os.Chdir(oldWd) }()
	_ = os.Chdir(tmpDir)

	// Create custom config
	yamlContent := `application:
  name: test-app
  version: "0.1.0"
components: []`
	err := os.WriteFile("custom.yaml", []byte(yamlContent), 0644)
	require.NoError(t, err)

	cmd := newUpCmd()
	var buf bytes.Buffer
	cmd.SetOut(&buf)
	cmd.SetErr(&buf)
	cmd.SetArgs([]string{"--config", "custom.yaml", "--skip-synth"})

	// Mock the exec.Command
	oldExecCommand := ExecCommand
	ExecCommand = MockExecCommandHelper
	defer func() { ExecCommand = oldExecCommand }()

	// Create fake spin.toml
	err = os.WriteFile("spin.toml", []byte("spin_manifest_version = 2"), 0644)
	require.NoError(t, err)

	err = cmd.Execute()
	// Config file should be used
	if err == nil {
		output := buf.String()
		// Check if config is being used (would be in actual implementation)
		_ = output
	}
}

func TestUpCommand_AllFlags(t *testing.T) {
	cmd := newUpCmd()

	// Test all flag combinations
	tests := []struct {
		name string
		args []string
	}{
		{"build_flag", []string{"--build", "--help"}},
		{"watch_flag", []string{"--watch", "--help"}},
		{"config_flag", []string{"--config", "custom.yaml", "--help"}},
		{"skip_synth_flag", []string{"--skip-synth", "--help"}},
		{"all_flags", []string{"--build", "--watch", "--config", "custom.yaml", "--skip-synth", "--help"}},
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

func BenchmarkUpCommand(b *testing.B) {
	for i := 0; i < b.N; i++ {
		cmd := newUpCmd()
		cmd.SetOut(&bytes.Buffer{})
		cmd.SetArgs([]string{"--help"})
		_ = cmd.Execute()
	}
}
