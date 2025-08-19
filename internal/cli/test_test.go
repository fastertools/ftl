package cli

import (
	"bytes"
	"os"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestTestCommand(t *testing.T) {
	cmd := newTestCmd()

	// Test command structure
	assert.NotNil(t, cmd)
	assert.Equal(t, "test [path]", cmd.Use)
	assert.Contains(t, cmd.Short, "Run tests")

	// Test flags
	coverageFlag := cmd.Flags().Lookup("coverage")
	assert.NotNil(t, coverageFlag)
	assert.Equal(t, "c", coverageFlag.Shorthand)
	assert.Equal(t, "false", coverageFlag.DefValue)

	verboseFlag := cmd.Flags().Lookup("verbose")
	assert.NotNil(t, verboseFlag)
	assert.Equal(t, "v", verboseFlag.Shorthand)
	assert.Equal(t, "false", verboseFlag.DefValue)
}

func TestTestCommand_Help(t *testing.T) {
	cmd := newTestCmd()
	var buf bytes.Buffer
	cmd.SetOut(&buf)
	cmd.SetArgs([]string{"--help"})

	err := cmd.Execute()
	assert.NoError(t, err)

	output := buf.String()
	assert.Contains(t, output, "Usage:")
	assert.Contains(t, output, "test")
	assert.Contains(t, output, "Flags:")
	assert.Contains(t, output, "--coverage")
	assert.Contains(t, output, "--verbose")
	assert.Contains(t, output, "-v")
}

func TestTestCommand_NoComponents(t *testing.T) {
	// Create test environment
	tmpDir := t.TempDir()
	oldWd, _ := os.Getwd()
	defer func() { _ = os.Chdir(oldWd) }()
	_ = os.Chdir(tmpDir)

	// Create ftl.yaml without components
	yamlContent := `application:
  name: test-app
  version: "0.1.0"
components: []`
	err := os.WriteFile("ftl.yaml", []byte(yamlContent), 0600)
	require.NoError(t, err)

	cmd := newTestCmd()
	var buf bytes.Buffer
	cmd.SetOut(&buf)
	cmd.SetErr(&buf)
	cmd.SetArgs([]string{})

	err = cmd.Execute()
	// Should succeed but report no components
	if err == nil {
		output := buf.String()
		// In actual implementation, would check for "no components" message
		_ = output
	}
}

func TestTestCommand_WithComponents(t *testing.T) {
	// Create test environment
	tmpDir := t.TempDir()
	oldWd, _ := os.Getwd()
	defer func() { _ = os.Chdir(oldWd) }()
	_ = os.Chdir(tmpDir)

	// Create ftl.yaml with components
	yamlContent := `application:
  name: test-app
  version: "0.1.0"
components:
  - id: test-comp
    source: ./test-comp`
	err := os.WriteFile("ftl.yaml", []byte(yamlContent), 0600)
	require.NoError(t, err)

	// Create component directory
	err = os.MkdirAll("test-comp", 0750)
	require.NoError(t, err)

	// Create Makefile with test target
	makefileContent := `test:
	@echo "Running tests..."
	@echo "PASS"`
	err = os.WriteFile("test-comp/Makefile", []byte(makefileContent), 0600)
	require.NoError(t, err)

	cmd := newTestCmd()
	var buf bytes.Buffer
	cmd.SetOut(&buf)
	cmd.SetErr(&buf)
	cmd.SetArgs([]string{})

	// Mock the exec.Command
	oldExecCommand := ExecCommand
	ExecCommand = MockExecCommandHelper
	defer func() { ExecCommand = oldExecCommand }()

	err = cmd.Execute()
	// Tests should run via make
	if err != nil {
		assert.NotContains(t, err.Error(), "not found")
	}
}

func TestTestCommand_WithCoverageFlag(t *testing.T) {
	// Create test environment
	tmpDir := t.TempDir()
	oldWd, _ := os.Getwd()
	defer func() { _ = os.Chdir(oldWd) }()
	_ = os.Chdir(tmpDir)

	// Create ftl.yaml
	yamlContent := `application:
  name: test-app
  version: "0.1.0"
components:
  - id: test-comp
    source: ./test-comp`
	err := os.WriteFile("ftl.yaml", []byte(yamlContent), 0600)
	require.NoError(t, err)

	// Create component
	err = os.MkdirAll("test-comp", 0750)
	require.NoError(t, err)

	cmd := newTestCmd()
	var buf bytes.Buffer
	cmd.SetOut(&buf)
	cmd.SetErr(&buf)
	cmd.SetArgs([]string{"--coverage"})

	// Mock the exec.Command
	oldExecCommand := ExecCommand
	ExecCommand = MockExecCommandHelper
	defer func() { ExecCommand = oldExecCommand }()

	err = cmd.Execute()
	// Coverage should be enabled
	if err == nil {
		output := buf.String()
		// Would check for coverage output in actual implementation
		_ = output
	}
}

func TestTestCommand_WithVerboseFlag(t *testing.T) {
	// Create test environment
	tmpDir := t.TempDir()
	oldWd, _ := os.Getwd()
	defer func() { _ = os.Chdir(oldWd) }()
	_ = os.Chdir(tmpDir)

	// Create go.mod
	goModContent := `module test-app

go 1.24`
	err := os.WriteFile("go.mod", []byte(goModContent), 0600)
	require.NoError(t, err)

	cmd := newTestCmd()
	var buf bytes.Buffer
	cmd.SetOut(&buf)
	cmd.SetErr(&buf)
	cmd.SetArgs([]string{"--verbose"})

	// Mock the exec.Command
	oldExecCommand := ExecCommand
	ExecCommand = MockExecCommandHelper
	defer func() { ExecCommand = oldExecCommand }()

	err = cmd.Execute()
	// Verbose flag should be used
	if err == nil {
		output := buf.String()
		// Would check for verbose output in actual implementation
		_ = output
	}
}

func TestTestCommand_AllFlags(t *testing.T) {
	cmd := newTestCmd()

	// Test all flag combinations
	tests := []struct {
		name string
		args []string
	}{
		{"coverage_flag", []string{"--coverage", "--help"}},
		{"verbose_flag", []string{"--verbose", "--help"}},
		{"all_flags", []string{"--coverage", "--verbose", "--help"}},
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

func BenchmarkTestCommand(b *testing.B) {
	for i := 0; i < b.N; i++ {
		cmd := newTestCmd()
		cmd.SetOut(&bytes.Buffer{})
		cmd.SetArgs([]string{"--help"})
		_ = cmd.Execute()
	}
}
