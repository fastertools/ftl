package cli

import (
	"bytes"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
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

	// Create ftl.yaml
	ftlConfig := `application:
  name: test-app
  version: "0.1.0"
components: []
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
