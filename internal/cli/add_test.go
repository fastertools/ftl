package cli

import (
	"bytes"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	
	"github.com/fastertools/ftl-cli/internal/scaffold"
)

func TestGetMainFileName(t *testing.T) {
	tests := []struct {
		language string
		expected string
	}{
		{"rust", "src/lib.rs"},
		{"typescript", "src/index.ts"},
		{"python", "src/main.py"},
		{"go", "main.go"},
		{"unknown", "main"},
	}

	for _, tt := range tests {
		t.Run(tt.language, func(t *testing.T) {
			result := getMainFileName(tt.language)
			assert.Equal(t, tt.expected, result)
		})
	}
}

func TestGetConfigFileName(t *testing.T) {
	tests := []struct {
		language string
		expected string
	}{
		{"rust", "Cargo.toml"},
		{"typescript", "package.json"},
		{"python", "pyproject.toml"},
		{"go", "go.mod"},
		{"unknown", "config"},
	}

	for _, tt := range tests {
		t.Run(tt.language, func(t *testing.T) {
			result := getConfigFileName(tt.language)
			assert.Equal(t, tt.expected, result)
		})
	}
}

func TestGetSdkSuffix(t *testing.T) {
	tests := []struct {
		language string
		expected string
	}{
		{"rust", "rust"},
		{"typescript", "js"},
		{"python", "python"},
		{"go", "go"},
		{"unknown", "unknown"},
	}

	for _, tt := range tests {
		t.Run(tt.language, func(t *testing.T) {
			result := getSdkSuffix(tt.language)
			assert.Equal(t, tt.expected, result)
		})
	}
}

func TestPrintSuccessMessage(t *testing.T) {
	// Capture stdout
	oldStdout := os.Stdout
	r, w, _ := os.Pipe()
	os.Stdout = w

	// Call the function
	printSuccessMessage("test-component", "rust")

	// Restore stdout and read output
	_ = w.Close()
	os.Stdout = oldStdout
	out, _ := io.ReadAll(r)
	output := string(out)

	// Check for expected content
	assert.Contains(t, output, "Component 'test-component' created successfully!")
	assert.Contains(t, output, "src/lib.rs")
	assert.Contains(t, output, "Cargo.toml")
	assert.Contains(t, output, "test-component/src/lib.rs")
	assert.Contains(t, output, "ftl-sdk-rust")
	assert.Contains(t, output, "cd test-component")
	assert.Contains(t, output, "make build")
	assert.Contains(t, output, "ftl build")
	assert.Contains(t, output, "ftl up")
}

func TestRunAdd_ValidationErrors(t *testing.T) {
	tests := []struct {
		name     string
		opts     *AddOptions
		wantErr  string
		setupDir func(t *testing.T)
	}{
		{
			name: "invalid_component_name",
			opts: &AddOptions{
				Name:     "Invalid-Name",
				Language: "rust",
			},
			wantErr: "component name must",
			setupDir: func(t *testing.T) {
				// Create a basic ftl.yaml
				_ = os.WriteFile("ftl.yaml", []byte("application:\n  name: test\n"), 0600)
			},
		},
		{
			name: "invalid_language",
			opts: &AddOptions{
				Name:     "valid-name",
				Language: "cobol",
			},
			wantErr: "invalid language",
			setupDir: func(t *testing.T) {
				_ = os.WriteFile("ftl.yaml", []byte("application:\n  name: test\n"), 0600)
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Create temp directory
			tmpDir := t.TempDir()
			oldWd, _ := os.Getwd()
			defer func() { _ = os.Chdir(oldWd) }()
			_ = os.Chdir(tmpDir)

			if tt.setupDir != nil {
				tt.setupDir(t)
			}

			err := runAdd(tt.opts)
			assert.Error(t, err)
			assert.Contains(t, err.Error(), tt.wantErr)
		})
	}
}

func TestRunAdd_Success(t *testing.T) {
	// Skip this test if running in CI without interactive capability
	if os.Getenv("CI") == "true" {
		t.Skip("Skipping interactive test in CI")
	}

	// Create temp directory with ftl.yaml
	tmpDir := t.TempDir()
	oldWd, _ := os.Getwd()
	defer func() { _ = os.Chdir(oldWd) }()
	_ = os.Chdir(tmpDir)

	// Create ftl.yaml with flat structure
	ftlConfig := `name: test-app
version: "0.1.0"
components: []
access: public
`
	require.NoError(t, os.WriteFile("ftl.yaml", []byte(ftlConfig), 0600))

	// Test with all parameters provided
	opts := &AddOptions{
		Name:     "my-tool",
		Language: "rust",
	}

	// Capture output
	oldStdout := os.Stdout
	r, w, _ := os.Pipe()
	os.Stdout = w

	err := runAdd(opts)

	// Restore stdout
	_ = w.Close()
	os.Stdout = oldStdout
	out, _ := io.ReadAll(r)
	output := string(out)

	// Check success
	assert.NoError(t, err)

	// Verify component directory was created
	assert.DirExists(t, "my-tool")
	assert.FileExists(t, filepath.Join("my-tool", "Cargo.toml"))
	assert.FileExists(t, filepath.Join("my-tool", "src", "lib.rs"))
	assert.FileExists(t, filepath.Join("my-tool", "Makefile"))

	// Check output messages
	assert.Contains(t, output, "Creating rust component 'my-tool'")
	assert.Contains(t, output, "Component 'my-tool' created successfully!")
}

func TestNewAddCmd(t *testing.T) {
	cmd := newAddCmd()

	assert.NotNil(t, cmd)
	assert.Equal(t, "add [name]", cmd.Use)
	assert.Contains(t, cmd.Short, "Add a new component")
	assert.NotNil(t, cmd.RunE)

	// Check flags
	languageFlag := cmd.Flags().Lookup("language")
	assert.NotNil(t, languageFlag)
	assert.Equal(t, "l", languageFlag.Shorthand)

	// Test args handling
	cmd.SetArgs([]string{"test-component"})
	err := cmd.ParseFlags([]string{})
	assert.NoError(t, err)
}

// MockScaffolder for testing runAdd without actual file operations
type MockScaffolder struct {
	GenerateComponentFunc func(name, language string) error
	ValidateNameFunc      func(name string) error
	ListLanguagesFunc     func() []string
}

func (m *MockScaffolder) GenerateComponent(name, language string) error {
	if m.GenerateComponentFunc != nil {
		return m.GenerateComponentFunc(name, language)
	}
	return nil
}

func (m *MockScaffolder) ValidateComponentName(name string) error {
	if m.ValidateNameFunc != nil {
		return m.ValidateNameFunc(name)
	}
	return nil
}

func (m *MockScaffolder) ListLanguages() []string {
	if m.ListLanguagesFunc != nil {
		return m.ListLanguagesFunc()
	}
	return []string{"rust", "typescript", "python", "go"}
}

func TestRunAdd_WithMockScaffolder(t *testing.T) {
	tests := []struct {
		name      string
		opts      *AddOptions
		mockSetup func(*MockScaffolder)
		wantErr   bool
		errMsg    string
	}{
		{
			name: "successful_generation",
			opts: &AddOptions{
				Name:     "test-component",
				Language: "rust",
			},
			mockSetup: func(m *MockScaffolder) {
				m.GenerateComponentFunc = func(name, language string) error {
					assert.Equal(t, "test-component", name)
					assert.Equal(t, "rust", language)
					return nil
				}
			},
			wantErr: false,
		},
		{
			name: "scaffolder_error",
			opts: &AddOptions{
				Name:     "test-component",
				Language: "rust",
			},
			mockSetup: func(m *MockScaffolder) {
				m.GenerateComponentFunc = func(name, language string) error {
					return fmt.Errorf("scaffolding failed")
				}
			},
			wantErr: true,
			errMsg:  "scaffolding failed",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			mock := &MockScaffolder{}
			if tt.mockSetup != nil {
				tt.mockSetup(mock)
			}

			// We can't easily inject the mock into runAdd without refactoring,
			// so this test serves as documentation of the testing approach
			// In production code, we'd refactor to allow dependency injection
		})
	}
}

func BenchmarkGetMainFileName(b *testing.B) {
	languages := []string{"rust", "typescript", "python", "go", "unknown"}

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_ = getMainFileName(languages[i%len(languages)])
	}
}

func BenchmarkGetConfigFileName(b *testing.B) {
	languages := []string{"rust", "typescript", "python", "go", "unknown"}

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_ = getConfigFileName(languages[i%len(languages)])
	}
}

func TestRunAdd_InteractiveMode(t *testing.T) {
	// This test documents the interactive behavior but is skipped
	// because it requires user input
	t.Skip("Interactive test - requires manual input")

	// When running manually:
	// 1. Don't provide Name in options
	// 2. Survey will prompt for component name
	// 3. Don't provide Language in options
	// 4. Survey will prompt with language selection

	opts := &AddOptions{}
	_ = runAdd(opts)
	// Would prompt for:
	// - Component name
	// - Language selection
}

func TestDetectGoProject(t *testing.T) {
	tests := []struct {
		name        string
		setupFunc   func(dir string) error
		expected    bool
		description string
	}{
		{
			name: "go_cdk_project",
			setupFunc: func(dir string) error {
				return os.WriteFile(filepath.Join(dir, "main.go"), []byte("package main"), 0644)
			},
			expected:    true,
			description: "Should detect Go CDK project with main.go",
		},
		{
			name: "yaml_project",
			setupFunc: func(dir string) error {
				return os.WriteFile(filepath.Join(dir, "ftl.yaml"), []byte("name: test\nversion: 1.0.0"), 0644)
			},
			expected:    false,
			description: "Should not detect YAML project as Go project",
		},
		{
			name: "json_project",
			setupFunc: func(dir string) error {
				return os.WriteFile(filepath.Join(dir, "ftl.json"), []byte(`{"name": "test"}`), 0644)
			},
			expected:    false,
			description: "Should not detect JSON project as Go project",
		},
		{
			name: "cue_project",
			setupFunc: func(dir string) error {
				return os.WriteFile(filepath.Join(dir, "app.cue"), []byte("package app"), 0644)
			},
			expected:    false,
			description: "Should not detect CUE project as Go project",
		},
		{
			name: "no_project_files",
			setupFunc: func(dir string) error {
				return os.WriteFile(filepath.Join(dir, "other.go"), []byte("package main"), 0644)
			},
			expected:    false,
			description: "Should not detect project without FTL project files",
		},
		{
			name: "empty_directory",
			setupFunc: func(dir string) error {
				return nil // No files created
			},
			expected:    false,
			description: "Should not detect project in empty directory",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Create temporary directory
			tempDir := t.TempDir()

			// Change to temp directory
			originalDir, err := os.Getwd()
			require.NoError(t, err)
			defer func() { _ = os.Chdir(originalDir) }()

			err = os.Chdir(tempDir)
			require.NoError(t, err)

			// Setup test files
			err = tt.setupFunc(tempDir)
			require.NoError(t, err)

			// Test using scaffold detection
			projectConfig := scaffold.DetectProject()
			result := projectConfig.Type == scaffold.ProjectTypeGo
			assert.Equal(t, tt.expected, result, tt.description)
		})
	}
}


func TestGenerateGoSnippet(t *testing.T) {
	tests := []struct {
		name     string
		toolName string
		expected []string // Expected strings that should be present in output
	}{
		{
			name:     "simple_tool_name",
			toolName: "calculator",
			expected: []string{
				`app.AddComponent("calculator")`,
				`FromLocal("./calculator/calculator.wasm")`,
				`WithBuild("cd calculator && make build")`,
				"Build()",
			},
		},
		{
			name:     "hyphenated_tool_name",
			toolName: "json-formatter",
			expected: []string{
				`app.AddComponent("json-formatter")`,
				`FromLocal("./json-formatter/json-formatter.wasm")`,
				`WithBuild("cd json-formatter && make build")`,
				"Build()",
			},
		},
		{
			name:     "underscore_tool_name",
			toolName: "base64_encoder",
			expected: []string{
				`app.AddComponent("base64_encoder")`,
				`FromLocal("./base64_encoder/base64_encoder.wasm")`,
				`WithBuild("cd base64_encoder && make build")`,
				"Build()",
			},
		},
		{
			name:     "single_character_name",
			toolName: "a",
			expected: []string{
				`app.AddComponent("a")`,
				`FromLocal("./a/a.wasm")`,
				`WithBuild("cd a && make build")`,
				"Build()",
			},
		},
		{
			name:     "empty_tool_name",
			toolName: "",
			expected: []string{
				`app.AddComponent("")`,
				`FromLocal(".//.wasm")`,
				`WithBuild("cd  && make build")`,
				"Build()",
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := generateGoSnippet(tt.toolName)

			// Check that all expected strings are present
			for _, expected := range tt.expected {
				assert.Contains(t, result, expected,
					"generateGoSnippet(%q) missing expected string: %q", tt.toolName, expected)
			}

			// Ensure the snippet contains proper Go syntax elements
			assert.Contains(t, result, "// Add this to your main.go file",
				"generateGoSnippet(%q) missing comment header", tt.toolName)

			// Count method calls to ensure proper chaining
			methodCalls := []string{"AddComponent", "FromLocal", "WithBuild", "Build"}
			for _, method := range methodCalls {
				assert.Contains(t, result, method,
					"generateGoSnippet(%q) missing method call: %s", tt.toolName, method)
			}
		})
	}
}

func TestGenerateGoSnippetFormat(t *testing.T) {
	toolName := "test-tool"
	result := generateGoSnippet(toolName)

	// Test that the result is properly formatted Go code
	lines := strings.Split(result, "\n")
	
	// Should have comment header
	require.True(t, len(lines) >= 3, "Snippet should have at least 3 lines")
	assert.Contains(t, lines[0], "// Add this to your main.go", "Should start with comment header")

	// Should have proper indentation for chained method calls
	var codeLines []string
	for _, line := range lines {
		trimmed := strings.TrimSpace(line)
		if trimmed != "" && !strings.HasPrefix(trimmed, "//") {
			codeLines = append(codeLines, line)
		}
	}

	require.True(t, len(codeLines) >= 4, "Should have at least 4 lines of actual code")

	// First line should start with app.AddComponent
	assert.Contains(t, codeLines[0], "app.AddComponent", 
		"First code line should contain app.AddComponent, got: %s", codeLines[0])

	// Subsequent lines should be properly indented method calls
	expectedMethods := []string{"FromLocal", "WithBuild", "Build"}
	for i, method := range expectedMethods {
		if i+1 < len(codeLines) {
			assert.Contains(t, codeLines[i+1], method,
				"Line %d should contain %s method, got: %s", i+2, method, codeLines[i+1])
		}
	}

	// Check indentation - method calls should be indented
	for i := 1; i < len(codeLines); i++ {
		assert.True(t, strings.HasPrefix(codeLines[i], "    "),
			"Code line %d should be indented with 4 spaces: %s", i+1, codeLines[i])
	}
}

func TestPrintSuccessMessage_GoProject(t *testing.T) {
	// Create temporary directory with Go CDK project
	tempDir := t.TempDir()
	originalDir, err := os.Getwd()
	require.NoError(t, err)
	defer func() { _ = os.Chdir(originalDir) }()

	err = os.Chdir(tempDir)
	require.NoError(t, err)

	// Create main.go to indicate Go project
	err = os.WriteFile("main.go", []byte("package main"), 0644)
	require.NoError(t, err)

	// Capture both stdout and stderr
	oldStdout := os.Stdout
	oldStderr := os.Stderr
	rOut, wOut, _ := os.Pipe()
	rErr, wErr, _ := os.Pipe()
	os.Stdout = wOut
	os.Stderr = wErr

	// Call the function
	printSuccessMessage("test-component", "rust")

	// Restore stdout/stderr and read output
	_ = wOut.Close()
	_ = wErr.Close()
	os.Stdout = oldStdout
	os.Stderr = oldStderr
	
	stdoutOut, _ := io.ReadAll(rOut)
	stderrOut, _ := io.ReadAll(rErr)
	output := string(stdoutOut) + string(stderrOut)

	// Check Go CDK specific content
	assert.Contains(t, output, "🔧 Go CDK Project Detected!")
	assert.Contains(t, output, "⚠ Manual component registration required for Go-based configurations.")
	assert.Contains(t, output, "Add the following to your main.go file:")
	assert.Contains(t, output, `app.AddComponent("test-component")`)
	assert.Contains(t, output, "Add the component registration code to your main.go")
	assert.Contains(t, output, "ftl build' to generate the updated configuration")

	// Should still contain common elements
	assert.Contains(t, output, "Component 'test-component' created successfully!")
	assert.Contains(t, output, "📁 Component structure:")
	assert.Contains(t, output, "💡 Edit")
}

func TestPrintSuccessMessage_NonGoProject(t *testing.T) {
	// Create temporary directory without Go CDK project
	tempDir := t.TempDir()
	originalDir, err := os.Getwd()
	require.NoError(t, err)
	defer func() { _ = os.Chdir(originalDir) }()

	err = os.Chdir(tempDir)
	require.NoError(t, err)

	// Create ftl.yaml instead
	err = os.WriteFile("ftl.yaml", []byte("name: test\nversion: 1.0.0"), 0644)
	require.NoError(t, err)

	// Capture stdout
	oldStdout := os.Stdout
	r, w, _ := os.Pipe()
	os.Stdout = w

	// Call the function
	printSuccessMessage("test-component", "rust")

	// Restore stdout and read output
	_ = w.Close()
	os.Stdout = oldStdout
	out, _ := io.ReadAll(r)
	output := string(out)

	// Should NOT contain Go CDK specific content
	assert.NotContains(t, output, "🔧 Go CDK Project Detected!")
	assert.NotContains(t, output, "Manual component registration required")
	assert.NotContains(t, output, "Add the following to your main.go file:")

	// Should contain standard next steps
	assert.Contains(t, output, "🔨 Next steps:")
	assert.Contains(t, output, "Return to project root and run 'ftl build'")

	// Should still contain common elements
	assert.Contains(t, output, "Component 'test-component' created successfully!")
	assert.Contains(t, output, "📁 Component structure:")
}

// Test helper function to verify the integration points
func TestAddCommandIntegration(t *testing.T) {
	// Test that our new functions integrate properly with the existing add command logic
	
	// Test that generateGoSnippet produces valid-looking Go code
	snippet := generateGoSnippet("example-tool")
	
	// Should contain proper Go method chaining syntax
	assert.Contains(t, snippet, ".", "Snippet should contain method chaining with dots")
	
	// Should contain proper string quoting
	assert.Contains(t, snippet, `"example-tool"`, "Snippet should contain properly quoted tool name")
	
	// Should be multi-line for readability
	lines := strings.Split(snippet, "\n")
	assert.True(t, len(lines) >= 5, "Snippet should be multi-line for better readability")
	
	// Should have consistent indentation
	var codeLines []string
	for _, line := range lines {
		if strings.TrimSpace(line) != "" && !strings.HasPrefix(strings.TrimSpace(line), "//") {
			codeLines = append(codeLines, line)
		}
	}
	
	// Verify method chaining structure
	if len(codeLines) >= 4 {
		assert.Contains(t, codeLines[0], "app.AddComponent")
		assert.Contains(t, codeLines[1], "FromLocal")
		assert.Contains(t, codeLines[2], "WithBuild") 
		assert.Contains(t, codeLines[3], "Build()")
	}
}

func TestGenerateGoSnippet_SpecialCharacters(t *testing.T) {
	// Test with various special characters that might appear in tool names
	tests := []struct {
		name     string
		toolName string
	}{
		{"with_periods", "tool.name"},
		{"with_spaces", "tool name"}, // Should be avoided but test robustness
		{"with_numbers", "tool123"},
		{"mixed_case", "MyTool"},
		{"unicode", "tööl"}, // Edge case - should handle gracefully
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := generateGoSnippet(tt.toolName)
			
			// Should always produce a result
			assert.NotEmpty(t, result, "Should always produce output")
			
			// Should contain the tool name as provided
			assert.Contains(t, result, tt.toolName, "Should contain the tool name")
			
			// Should maintain structure
			assert.Contains(t, result, "app.AddComponent", "Should contain AddComponent call")
			assert.Contains(t, result, "Build()", "Should contain Build call")
		})
	}
}

// Test for the component validation in the actual add flow
func TestAddCommandValidation(t *testing.T) {
	// Test that the add command properly validates inputs
	cmd := newAddCmd()

	// Test with too many args
	cmd.SetArgs([]string{"arg1", "arg2"})
	err := cmd.Execute()
	assert.Error(t, err) // Should fail with MaximumNArgs(1)

	// Reset for next test
	cmd = newAddCmd()

	// Test with valid single arg
	cmd.SetArgs([]string{"my-component", "--language", "rust"})
	// This would normally execute runAdd, but we can't mock it easily
	// This test documents the expected behavior
}

// Helper function tests for better coverage
func TestAddCommandHelpers(t *testing.T) {
	t.Run("all_languages_have_main_files", func(t *testing.T) {
		languages := []string{"rust", "typescript", "python", "go"}
		for _, lang := range languages {
			mainFile := getMainFileName(lang)
			assert.NotEmpty(t, mainFile)
			assert.NotEqual(t, "main", mainFile) // Should have specific extension
		}
	})

	t.Run("all_languages_have_config_files", func(t *testing.T) {
		languages := []string{"rust", "typescript", "python", "go"}
		for _, lang := range languages {
			configFile := getConfigFileName(lang)
			assert.NotEmpty(t, configFile)
			assert.NotEqual(t, "config", configFile) // Should have specific name
		}
	})

	t.Run("all_languages_have_sdk_suffix", func(t *testing.T) {
		languages := []string{"rust", "typescript", "python", "go"}
		for _, lang := range languages {
			suffix := getSdkSuffix(lang)
			assert.NotEmpty(t, suffix)
			// TypeScript is special case - uses "js"
			if lang == "typescript" {
				assert.Equal(t, "js", suffix)
			}
		}
	})
}

// Output formatting test
func TestPrintSuccessMessageFormatting(t *testing.T) {
	// Create a buffer to capture output
	var buf bytes.Buffer

	// Temporarily replace fmt.Printf and fmt.Println
	// In real code, we'd refactor to use an io.Writer

	languages := []string{"rust", "typescript", "python", "go"}

	for _, lang := range languages {
		t.Run(lang, func(t *testing.T) {
			// Reset buffer
			buf.Reset()

			// Capture stdout
			oldStdout := os.Stdout
			r, w, _ := os.Pipe()
			os.Stdout = w

			printSuccessMessage("test-comp", lang)

			_ = w.Close()
			os.Stdout = oldStdout
			out, _ := io.ReadAll(r)

			// Verify structure markers are present
			assert.Contains(t, string(out), "📁 Component structure:")
			assert.Contains(t, string(out), "💡 Edit")
			assert.Contains(t, string(out), "🔨 Next steps:")
			assert.Contains(t, string(out), "📚 Learn more")
		})
	}
}
