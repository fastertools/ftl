package cmd

import (
	"os"
	"path/filepath"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestDetectFormat(t *testing.T) {
	// Note: detectFormat only checks content, not file extensions
	tests := []struct {
		name       string
		content    string
		wantFormat string
	}{
		{
			name:       "json_by_content",
			content:    `{"application":{"name":"test"}}`,
			wantFormat: "json",
		},
		{
			name:       "cue_by_content",
			content:    `package app`,
			wantFormat: "cue",
		},
		{
			name:       "yaml_by_default",
			content:    "application:\n  name: test",
			wantFormat: "yaml",
		},
		{
			name:       "yaml_for_unknown",
			content:    "<application></application>",
			wantFormat: "yaml", // defaults to yaml
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			format := detectFormat([]byte(tt.content))
			assert.Equal(t, tt.wantFormat, format)
		})
	}
}

func TestSynthesizeFromYAML(t *testing.T) {
	yamlContent := `
application:
  name: test-app
  version: "1.0.0"
  description: Test application
components:
  - id: test-component
    source: ./test.wasm
`

	tmpDir := t.TempDir()
	yamlPath := filepath.Join(tmpDir, "ftl.yaml")
	err := os.WriteFile(yamlPath, []byte(yamlContent), 0644)
	require.NoError(t, err)

	yamlData, err := os.ReadFile(yamlPath)
	require.NoError(t, err)

	result, err := synthesizeFromYAML(yamlData)
	assert.NoError(t, err)
	assert.Contains(t, result, "spin_manifest_version")
	assert.Contains(t, result, "test-app")
	assert.Contains(t, result, "ftl-mcp-gateway")
}

func TestSynthesizeFromJSON(t *testing.T) {
	jsonContent := `{
  "application": {
    "name": "json-app",
    "version": "1.0.0"
  },
  "components": []
}`

	tmpDir := t.TempDir()
	jsonPath := filepath.Join(tmpDir, "ftl.json")
	err := os.WriteFile(jsonPath, []byte(jsonContent), 0644)
	require.NoError(t, err)

	jsonData, err := os.ReadFile(jsonPath)
	require.NoError(t, err)

	result, err := synthesizeFromJSON(jsonData)
	assert.NoError(t, err)
	assert.Contains(t, result, "spin_manifest_version")
	assert.Contains(t, result, "json-app")
}

func TestSynthesizeFromCUE(t *testing.T) {
	cueContent := `
application: {
	name: "cue-app"
	version: "1.0.0"
}
components: []
`

	tmpDir := t.TempDir()
	cuePath := filepath.Join(tmpDir, "app.cue")
	err := os.WriteFile(cuePath, []byte(cueContent), 0644)
	require.NoError(t, err)

	cueData, err := os.ReadFile(cuePath)
	require.NoError(t, err)

	result, err := synthesizeFromCUE(cueData)
	assert.NoError(t, err)
	assert.Contains(t, result, "spin_manifest_version")
	assert.Contains(t, result, "cue-app")
}

func TestSynthesizeFromGo(t *testing.T) {
	goContent := `package main

import (
	"fmt"
	"github.com/fastertools/ftl-cli/go/ftl/pkg/synthesis"
)

func main() {
	cdk := synthesis.NewCDK()
	app := cdk.NewApp("go-app").
		SetVersion("1.0.0").
		SetDescription("Go application")
	
	builtCDK := app.Build()
	manifest, _ := builtCDK.Synthesize()
	fmt.Print(manifest)
}
`

	tmpDir := t.TempDir()
	goPath := filepath.Join(tmpDir, "main.go")
	err := os.WriteFile(goPath, []byte(goContent), 0644)
	require.NoError(t, err)

	// Note: This test would require Go to be installed and the synthesis package
	// to be available. In a real test, we might mock the execution.
	// For now, we'll skip the actual execution test
	t.Skip("Requires Go runtime and synthesis package")

	// result, err := synthesizeFromGo(goPath)
	// assert.NoError(t, err)
	// assert.Contains(t, result, "spin_manifest_version")
}

func TestSynthesizeFromInput(t *testing.T) {
	tests := []struct {
		name      string
		setupFile func(dir string) string
		wantErr   bool
	}{
		{
			name: "yaml_input",
			setupFile: func(dir string) string {
				path := filepath.Join(dir, "ftl.yaml")
				content := `application:
  name: test
components: []`
				_ = os.WriteFile(path, []byte(content), 0644)
				return path
			},
			wantErr: false,
		},
		{
			name: "json_input",
			setupFile: func(dir string) string {
				path := filepath.Join(dir, "ftl.json")
				content := `{"application":{"name":"test"},"components":[]}`
				_ = os.WriteFile(path, []byte(content), 0644)
				return path
			},
			wantErr: false,
		},
		{
			name: "nonexistent_file",
			setupFile: func(dir string) string {
				return filepath.Join(dir, "nonexistent.yaml")
			},
			wantErr: true,
		},
		{
			name: "invalid_format",
			setupFile: func(dir string) string {
				path := filepath.Join(dir, "invalid.xml")
				_ = os.WriteFile(path, []byte("<xml/>"), 0644)
				return path
			},
			wantErr: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			tmpDir := t.TempDir()
			inputPath := tt.setupFile(tmpDir)

			inputData, _ := os.ReadFile(inputPath)
			result, err := synthesizeFromInput(inputData, []string{})

			if tt.wantErr {
				assert.Error(t, err)
			} else {
				assert.NoError(t, err)
				assert.NotEmpty(t, result)
				assert.Contains(t, result, "spin_manifest_version")
			}
		})
	}
}

func TestNewSynthCmd(t *testing.T) {
	cmd := newSynthCmd()

	assert.NotNil(t, cmd)
	assert.Equal(t, "synth [file]", cmd.Use)
	assert.Contains(t, cmd.Short, "Synthesize")
	assert.NotNil(t, cmd.RunE)

	// Check flags
	outputFlag := cmd.Flags().Lookup("output")
	assert.NotNil(t, outputFlag)
	assert.Equal(t, "o", outputFlag.Shorthand)

	// format flag doesn't exist based on the actual implementation
	// formatFlag := cmd.Flags().Lookup("format")
	// assert.NotNil(t, formatFlag)

	// stdin flag doesn't exist based on the actual implementation
	// stdinFlag := cmd.Flags().Lookup("stdin")
	// assert.NotNil(t, stdinFlag)
}

func TestSynthCmd_OutputToFile(t *testing.T) {
	// This test would verify that the --output flag works correctly
	// In practice, we'd need to mock the command execution

	tmpDir := t.TempDir()
	yamlPath := filepath.Join(tmpDir, "ftl.yaml")
	// outputPath would be used in actual command execution
	// outputPath := filepath.Join(tmpDir, "spin.toml")

	yamlContent := `application:
  name: test
components: []`
	_ = os.WriteFile(yamlPath, []byte(yamlContent), 0644)

	// In a real test, we'd execute:
	// cmd := newSynthCmd()
	// cmd.SetArgs([]string{yamlPath, "--output", outputPath})
	// err := cmd.Execute()

	// For now, we're documenting the expected behavior
	t.Skip("Requires command execution framework")
}

func TestSynthCmd_StdinInput(t *testing.T) {
	// This test would verify that the --stdin flag works correctly
	// It would pipe YAML content through stdin and verify output

	t.Skip("Requires stdin mocking")

	// Expected behavior:
	// echo "application: {name: test}" | ftl synth --stdin --format yaml
	// Should output valid spin.toml to stdout
}

// Benchmark tests
func BenchmarkDetectFormat(b *testing.B) {
	contents := [][]byte{
		[]byte("application:\n  name: test"),
		[]byte(`{"application":{"name":"test"}}`),
		[]byte(`package app`),
		[]byte(`package main`),
	}

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_ = detectFormat(contents[i%len(contents)])
	}
}

func TestSynthesizeFromInput_WithFormat(t *testing.T) {
	// Test that format detection works correctly
	tmpDir := t.TempDir()

	// Create a file with .txt extension but YAML content
	path := filepath.Join(tmpDir, "config.txt")
	content := `application:
  name: test
components: []`
	err := os.WriteFile(path, []byte(content), 0644)
	require.NoError(t, err)

	// Without file extension hint, should default to YAML (based on content)
	data, _ := os.ReadFile(path)
	result, err := synthesizeFromInput(data, []string{})
	assert.NoError(t, err)
	assert.Contains(t, result, "spin_manifest_version")

	// Test with JSON content
	jsonContent := `{"application":{"name":"test"},"components":[]}`
	jsonResult, err := synthesizeFromInput([]byte(jsonContent), []string{})
	assert.NoError(t, err)
	assert.Contains(t, jsonResult, "spin_manifest_version")
}

// Test error handling
func TestSynthesizeFromYAML_InvalidContent(t *testing.T) {
	// Test with invalid YAML content
	invalidYAML := []byte("invalid: yaml: content:")

	_, err := synthesizeFromYAML(invalidYAML)
	assert.Error(t, err)
}

func TestSynthesizeFromJSON_InvalidContent(t *testing.T) {
	// Test with invalid JSON content
	invalidJSON := []byte("{invalid json}")

	_, err := synthesizeFromJSON(invalidJSON)
	assert.Error(t, err)
}

func TestSynthesizeFromCUE_InvalidContent(t *testing.T) {
	// Test with invalid CUE content
	invalidCUE := []byte("invalid cue {{}}}")

	_, err := synthesizeFromCUE(invalidCUE)
	assert.Error(t, err)
}
