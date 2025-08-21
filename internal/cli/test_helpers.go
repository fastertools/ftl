package cli

import (
	"bytes"
	"io"
	"os"
	"testing"

	"github.com/AlecAivazis/survey/v2"
	"github.com/spf13/cobra"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TestCommandExecution helps test cobra command execution
type TestCommandExecution struct {
	Command      *cobra.Command
	Args         []string
	ExpectError  bool
	ExpectOutput []string
	Setup        func(t *testing.T) string
	Cleanup      func(string)
	Validate     func(t *testing.T, output string, err error)
}

// ExecuteCommandTest runs a command test with proper setup/teardown
func ExecuteCommandTest(t *testing.T, test TestCommandExecution) {
	// Setup
	var tmpDir string
	if test.Setup != nil {
		tmpDir = test.Setup(t)
		if test.Cleanup != nil {
			defer test.Cleanup(tmpDir)
		}
	}

	// Capture output
	var stdout, stderr bytes.Buffer
	test.Command.SetOut(&stdout)
	test.Command.SetErr(&stderr)
	test.Command.SetArgs(test.Args)

	// Execute
	err := test.Command.Execute()

	// Check error expectation
	if test.ExpectError {
		assert.Error(t, err)
	} else {
		assert.NoError(t, err)
	}

	// Check output expectations
	output := stdout.String() + stderr.String()
	for _, expected := range test.ExpectOutput {
		assert.Contains(t, output, expected)
	}

	// Custom validation
	if test.Validate != nil {
		test.Validate(t, output, err)
	}
}

// CaptureOutput captures stdout/stderr during function execution
func CaptureOutput(t *testing.T, fn func()) string {
	t.Helper()

	oldStdout := os.Stdout
	oldStderr := os.Stderr
	r, w, err := os.Pipe()
	require.NoError(t, err)

	os.Stdout = w
	os.Stderr = w

	fn()

	_ = w.Close()
	os.Stdout = oldStdout
	os.Stderr = oldStderr

	out, err := io.ReadAll(r)
	require.NoError(t, err)

	return string(out)
}

// MockSurveyAskOne mocks survey.AskOne for testing interactive prompts
func MockSurveyAskOne(response interface{}) func(p survey.Prompt, response interface{}, opts ...survey.AskOpt) error {
	return func(p survey.Prompt, resp interface{}, opts ...survey.AskOpt) error {
		switch v := resp.(type) {
		case *string:
			*v = response.(string)
		case *bool:
			*v = response.(bool)
		case *int:
			*v = response.(int)
		}
		return nil
	}
}

// MockSurveyAsk mocks survey.Ask for testing multiple prompts
func MockSurveyAsk(responses map[string]interface{}) func(qs []*survey.Question, response interface{}, opts ...survey.AskOpt) error {
	return func(qs []*survey.Question, response interface{}, opts ...survey.AskOpt) error {
		// Use reflection to set fields based on question names
		// This is simplified - in real implementation would use reflection
		return nil
	}
}

// CreateTestProject creates a temporary test project structure
func CreateTestProject(t *testing.T, format string) string {
	tmpDir := t.TempDir()

	switch format {
	case "yaml":
		content := `application:
  name: test-app
  version: "0.1.0"
components: []
triggers: []`
		err := os.WriteFile(tmpDir+"/ftl.yaml", []byte(content), 0600)
		require.NoError(t, err)

	case "json":
		content := `{
  "application": {
    "name": "test-app",
    "version": "0.1.0"
  },
  "components": [],
  "triggers": []
}`
		err := os.WriteFile(tmpDir+"/ftl.json", []byte(content), 0600)
		require.NoError(t, err)

	case "cue":
		content := `package app

application: {
	name: "test-app"
	version: "0.1.0"
}
components: []`
		err := os.WriteFile(tmpDir+"/app.cue", []byte(content), 0600)
		require.NoError(t, err)

	case "go":
		mainContent := `package main

import (
	"fmt"
	"github.com/fastertools/ftl-cli/pkg/synthesis"
)

func main() {
	cdk := synthesis.NewCDK()
	app := cdk.NewApp("test-app")
	manifest, _ := app.Build().Synthesize()
	fmt.Print(manifest)
}`
		err := os.WriteFile(tmpDir+"/main.go", []byte(mainContent), 0600)
		require.NoError(t, err)

		modContent := `module test-app

go 1.24

require github.com/fastertools/ftl-cli/go/ftl v0.0.0`
		err = os.WriteFile(tmpDir+"/go.mod", []byte(modContent), 0600)
		require.NoError(t, err)
	}

	// Create .gitignore
	gitignoreContent := `.spin/
spin.toml
*.wasm`
	err := os.WriteFile(tmpDir+"/.gitignore", []byte(gitignoreContent), 0600)
	require.NoError(t, err)

	return tmpDir
}

// CreateTestComponent creates a test component directory structure
func CreateTestComponent(t *testing.T, dir, name, language string) {
	componentDir := dir + "/" + name
	err := os.MkdirAll(componentDir, 0750)
	require.NoError(t, err)

	switch language {
	case "rust":
		err = os.MkdirAll(componentDir+"/src", 0750)
		require.NoError(t, err)
		err = os.WriteFile(componentDir+"/src/lib.rs", []byte("// Rust component"), 0600)
		require.NoError(t, err)
		err = os.WriteFile(componentDir+"/Cargo.toml", []byte("[package]\nname = \""+name+"\""), 0600)
		require.NoError(t, err)

	case "typescript":
		err = os.MkdirAll(componentDir+"/src", 0750)
		require.NoError(t, err)
		err = os.WriteFile(componentDir+"/src/index.ts", []byte("// TypeScript component"), 0600)
		require.NoError(t, err)
		err = os.WriteFile(componentDir+"/package.json", []byte(`{"name": "`+name+`"}`), 0600)
		require.NoError(t, err)

	case "python":
		err = os.MkdirAll(componentDir+"/src", 0750)
		require.NoError(t, err)
		err = os.WriteFile(componentDir+"/src/main.py", []byte("# Python component"), 0600)
		require.NoError(t, err)
		err = os.WriteFile(componentDir+"/pyproject.toml", []byte("[project]\nname = \""+name+"\""), 0600)
		require.NoError(t, err)

	case "go":
		err = os.WriteFile(componentDir+"/main.go", []byte("package main\n// Go component"), 0600)
		require.NoError(t, err)
		err = os.WriteFile(componentDir+"/go.mod", []byte("module "+name), 0600)
		require.NoError(t, err)
	}

	// Create Makefile
	makefileContent := `build:
	@echo "Building ` + name + `"

test:
	@echo "Testing ` + name + `"`
	err = os.WriteFile(componentDir+"/Makefile", []byte(makefileContent), 0600)
	require.NoError(t, err)
}

// TestConfig holds test configuration for commands
type TestConfig struct {
	WorkDir      string
	ConfigFormat string
	Components   []TestComponent
}

type TestComponent struct {
	Name     string
	Language string
	Source   string
}

// SetupTestEnvironment creates a complete test environment
func SetupTestEnvironment(t *testing.T, config TestConfig) string {
	if config.WorkDir == "" {
		config.WorkDir = t.TempDir()
	}

	// Save current directory
	oldWd, err := os.Getwd()
	require.NoError(t, err)

	// Change to test directory
	err = os.Chdir(config.WorkDir)
	require.NoError(t, err)

	// Restore on cleanup
	t.Cleanup(func() {
		_ = os.Chdir(oldWd)
	})

	// Create project structure
	if config.ConfigFormat != "" {
		CreateTestProject(t, config.ConfigFormat)
	}

	// Create components
	for _, comp := range config.Components {
		CreateTestComponent(t, config.WorkDir, comp.Name, comp.Language)
	}

	return config.WorkDir
}

// AssertCommandOutput checks that command output contains expected strings
func AssertCommandOutput(t *testing.T, cmd *cobra.Command, args []string, expected ...string) {
	var buf bytes.Buffer
	cmd.SetOut(&buf)
	cmd.SetErr(&buf)
	cmd.SetArgs(args)

	err := cmd.Execute()
	require.NoError(t, err)

	output := buf.String()
	for _, exp := range expected {
		assert.Contains(t, output, exp)
	}
}

// AssertCommandError checks that command fails with expected error
func AssertCommandError(t *testing.T, cmd *cobra.Command, args []string, expectedErr string) {
	var buf bytes.Buffer
	cmd.SetOut(&buf)
	cmd.SetErr(&buf)
	cmd.SetArgs(args)

	err := cmd.Execute()
	require.Error(t, err)
	assert.Contains(t, err.Error(), expectedErr)
}
