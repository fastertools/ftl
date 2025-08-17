package cmd

import (
	"bytes"
	"os"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestDeployCommand(t *testing.T) {
	cmd := newDeployCmd()

	// Test command structure
	assert.NotNil(t, cmd)
	assert.Equal(t, "deploy [flags]", cmd.Use)
	assert.Contains(t, cmd.Short, "Deploy")

	// Test flags
	envFlag := cmd.Flags().Lookup("environment")
	assert.NotNil(t, envFlag)
	assert.Equal(t, "e", envFlag.Shorthand)
	assert.Equal(t, "production", envFlag.DefValue)

	fileFlag := cmd.Flags().Lookup("file")
	assert.NotNil(t, fileFlag)
	assert.Equal(t, "f", fileFlag.Shorthand)
	assert.Equal(t, "", fileFlag.DefValue)

	dryRunFlag := cmd.Flags().Lookup("dry-run")
	assert.NotNil(t, dryRunFlag)
	assert.Equal(t, "false", dryRunFlag.DefValue)
}

func TestDeployCommand_Help(t *testing.T) {
	cmd := newDeployCmd()
	var buf bytes.Buffer
	cmd.SetOut(&buf)
	cmd.SetArgs([]string{"--help"})

	err := cmd.Execute()
	assert.NoError(t, err)

	output := buf.String()
	assert.Contains(t, output, "Usage:")
	assert.Contains(t, output, "deploy")
	assert.Contains(t, output, "Flags:")
	assert.Contains(t, output, "--environment")
	assert.Contains(t, output, "--file")
	assert.Contains(t, output, "--dry-run")
}

func TestDeployCommand_NoConfig(t *testing.T) {
	// Create test environment without config
	tmpDir := t.TempDir()
	oldWd, _ := os.Getwd()
	defer func() { _ = os.Chdir(oldWd) }()
	_ = os.Chdir(tmpDir)

	cmd := newDeployCmd()
	var buf bytes.Buffer
	cmd.SetOut(&buf)
	cmd.SetErr(&buf)
	cmd.SetArgs([]string{})

	// This should fail because no config exists
	err := cmd.Execute()
	assert.Error(t, err)
}

func TestDeployCommand_WithEnvironment(t *testing.T) {
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
name = "test-app"
version = "0.1.0"`
	err = os.WriteFile("spin.toml", []byte(spinContent), 0644)
	require.NoError(t, err)

	cmd := newDeployCmd()
	var buf bytes.Buffer
	cmd.SetOut(&buf)
	cmd.SetErr(&buf)
	cmd.SetArgs([]string{"--environment", "staging"})

	// Mock the exec.Command to avoid calling real spin
	oldExecCommand := ExecCommand
	ExecCommand = MockExecCommandHelper
	defer func() { ExecCommand = oldExecCommand }()

	// This will still fail but for different reasons (spin command mock)
	err = cmd.Execute()
	// We expect an error but not because of missing config
	if err != nil {
		assert.NotContains(t, err.Error(), "not found")
	}
}

func TestDeployCommand_AllFlags(t *testing.T) {
	cmd := newDeployCmd()

	// Test all flag combinations
	tests := []struct {
		name string
		args []string
	}{
		{"environment_flag", []string{"--environment", "prod", "--help"}},
		{"file_flag", []string{"--file", "custom.yaml", "--help"}},
		{"dry_run_flag", []string{"--dry-run", "--help"}},
		{"all_flags", []string{"--environment", "production", "--file", "custom.yaml", "--dry-run", "--help"}},
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

func BenchmarkDeployCommand(b *testing.B) {
	for i := 0; i < b.N; i++ {
		cmd := newDeployCmd()
		cmd.SetOut(&bytes.Buffer{})
		cmd.SetArgs([]string{"--help"})
		_ = cmd.Execute()
	}
}
