package scaffold

import (
	"encoding/json"
	"os"
	"testing"

	"cuelang.org/go/cue"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"gopkg.in/yaml.v3"

	"github.com/fastertools/ftl-cli/go/shared/config"
)

func TestValidateComponentName(t *testing.T) {
	tests := []struct {
		name    string
		input   string
		wantErr bool
		errMsg  string
	}{
		{
			name:    "valid name",
			input:   "my-component",
			wantErr: false,
		},
		{
			name:    "valid single word",
			input:   "component",
			wantErr: false,
		},
		{
			name:    "valid with numbers",
			input:   "component-123",
			wantErr: false,
		},
		{
			name:    "empty name",
			input:   "",
			wantErr: true,
			errMsg:  "empty",
		},
		{
			name:    "starts with number",
			input:   "123-component",
			wantErr: true,
			errMsg:  "must start with",
		},
		{
			name:    "starts with hyphen",
			input:   "-component",
			wantErr: true,
			errMsg:  "must start with",
		},
		{
			name:    "ends with hyphen",
			input:   "component-",
			wantErr: true,
			errMsg:  "cannot start/end",
		},
		{
			name:    "double hyphen",
			input:   "my--component",
			wantErr: true,
			errMsg:  "double hyphens",
		},
		{
			name:    "uppercase letters",
			input:   "MyComponent",
			wantErr: true,
			errMsg:  "lowercase",
		},
		{
			name:    "underscore",
			input:   "my_component",
			wantErr: true,
			errMsg:  "lowercase letters",
		},
		{
			name:    "special characters",
			input:   "my@component",
			wantErr: true,
			errMsg:  "lowercase letters",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := ValidateComponentName(tt.input)
			if tt.wantErr {
				assert.Error(t, err)
				if tt.errMsg != "" {
					assert.Contains(t, err.Error(), tt.errMsg)
				}
			} else {
				assert.NoError(t, err)
			}
		})
	}
}

func TestNewScaffolder(t *testing.T) {
	scaffolder, err := NewScaffolder()
	require.NoError(t, err)
	assert.NotNil(t, scaffolder)
	assert.NotNil(t, scaffolder.ctx)
	assert.True(t, scaffolder.templates.Exists())
}

func TestListLanguages(t *testing.T) {
	scaffolder, err := NewScaffolder()
	require.NoError(t, err)

	languages := scaffolder.ListLanguages()
	assert.Equal(t, []string{"rust", "typescript", "python", "go"}, languages)
}

func TestGetWasmPath(t *testing.T) {
	scaffolder, err := NewScaffolder()
	require.NoError(t, err)

	tests := []struct {
		name     string
		compName string
		language string
		expected string
	}{
		{
			name:     "rust component",
			compName: "my-tool",
			language: "rust",
			expected: "my-tool/my_tool.wasm",
		},
		{
			name:     "typescript component",
			compName: "my-tool",
			language: "typescript",
			expected: "my-tool/dist/my-tool.wasm",
		},
		{
			name:     "python component",
			compName: "my-tool",
			language: "python",
			expected: "my-tool/app.wasm",
		},
		{
			name:     "go component",
			compName: "my-tool",
			language: "go",
			expected: "my-tool/main.wasm",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := scaffolder.getWasmPath(tt.compName, tt.language)
			assert.Equal(t, tt.expected, result)
		})
	}
}

func TestGenerateComponent_Rust(t *testing.T) {
	// Create temp directory
	tmpDir := t.TempDir()
	oldWd, _ := os.Getwd()
	defer func() { _ = os.Chdir(oldWd) }()
	_ = os.Chdir(tmpDir)

	// Create minimal ftl.yaml
	ftlConfig := config.FTLConfig{
		Application: config.ApplicationConfig{
			Name:    "test-app",
			Version: "0.1.0",
		},
	}
	data, _ := yaml.Marshal(&ftlConfig)
	_ = os.WriteFile("ftl.yaml", data, 0644)

	// Generate component
	scaffolder, err := NewScaffolder()
	require.NoError(t, err)

	err = scaffolder.GenerateComponent("my-rust-tool", "rust")
	require.NoError(t, err)

	// Check generated files
	expectedFiles := []string{
		"my-rust-tool/Cargo.toml",
		"my-rust-tool/Makefile",
		"my-rust-tool/src/lib.rs",
		"my-rust-tool/.gitignore",
	}

	for _, file := range expectedFiles {
		assert.FileExists(t, file)
	}

	// Check Cargo.toml content
	cargoContent, err := os.ReadFile("my-rust-tool/Cargo.toml")
	require.NoError(t, err)
	assert.Contains(t, string(cargoContent), `name = "my_rust_tool"`)
	assert.Contains(t, string(cargoContent), "ftl-sdk")

	// Check lib.rs content
	libContent, err := os.ReadFile("my-rust-tool/src/lib.rs")
	require.NoError(t, err)
	assert.Contains(t, string(libContent), "use ftl_sdk")
	assert.Contains(t, string(libContent), "tools!")

	// Check ftl.yaml was updated
	updatedData, err := os.ReadFile("ftl.yaml")
	require.NoError(t, err)

	var updatedConfig config.FTLConfig
	err = yaml.Unmarshal(updatedData, &updatedConfig)
	require.NoError(t, err)

	assert.Len(t, updatedConfig.Components, 1)
	assert.Equal(t, "my-rust-tool", updatedConfig.Components[0].ID)
	assert.Equal(t, "my-rust-tool/my_rust_tool.wasm", updatedConfig.Components[0].Source)
	assert.NotNil(t, updatedConfig.Components[0].Build)
	assert.Equal(t, "make build", updatedConfig.Components[0].Build.Command)
}

func TestGenerateComponent_TypeScript(t *testing.T) {
	// Create temp directory
	tmpDir := t.TempDir()
	oldWd, _ := os.Getwd()
	defer func() { _ = os.Chdir(oldWd) }()
	_ = os.Chdir(tmpDir)

	// Create minimal ftl.yaml
	ftlConfig := config.FTLConfig{
		Application: config.ApplicationConfig{
			Name:    "test-app",
			Version: "0.1.0",
		},
	}
	data, _ := yaml.Marshal(&ftlConfig)
	_ = os.WriteFile("ftl.yaml", data, 0644)

	// Generate component
	scaffolder, err := NewScaffolder()
	require.NoError(t, err)

	err = scaffolder.GenerateComponent("my-ts-tool", "typescript")
	require.NoError(t, err)

	// Check generated files
	expectedFiles := []string{
		"my-ts-tool/package.json",
		"my-ts-tool/tsconfig.json",
		"my-ts-tool/Makefile",
		"my-ts-tool/src/index.ts",
		"my-ts-tool/.gitignore",
	}

	for _, file := range expectedFiles {
		assert.FileExists(t, file)
	}

	// Check package.json content
	packageContent, err := os.ReadFile("my-ts-tool/package.json")
	require.NoError(t, err)
	assert.Contains(t, string(packageContent), `"name": "my-ts-tool"`)
	assert.Contains(t, string(packageContent), "ftl-sdk")

	// Check index.ts content
	indexContent, err := os.ReadFile("my-ts-tool/src/index.ts")
	require.NoError(t, err)
	assert.Contains(t, string(indexContent), "import { createTools")
	assert.Contains(t, string(indexContent), "exampleTool")
}

func TestGenerateComponent_Python(t *testing.T) {
	// Create temp directory
	tmpDir := t.TempDir()
	oldWd, _ := os.Getwd()
	defer func() { _ = os.Chdir(oldWd) }()
	_ = os.Chdir(tmpDir)

	// Create minimal ftl.yaml
	ftlConfig := config.FTLConfig{
		Application: config.ApplicationConfig{
			Name:    "test-app",
			Version: "0.1.0",
		},
	}
	data, _ := yaml.Marshal(&ftlConfig)
	_ = os.WriteFile("ftl.yaml", data, 0644)

	// Generate component
	scaffolder, err := NewScaffolder()
	require.NoError(t, err)

	err = scaffolder.GenerateComponent("my-py-tool", "python")
	require.NoError(t, err)

	// Check generated files
	expectedFiles := []string{
		"my-py-tool/pyproject.toml",
		"my-py-tool/Makefile",
		"my-py-tool/src/__init__.py",
		"my-py-tool/src/main.py",
		"my-py-tool/tests/__init__.py",
		"my-py-tool/tests/test_main.py",
		"my-py-tool/.gitignore",
	}

	for _, file := range expectedFiles {
		assert.FileExists(t, file)
	}

	// Check pyproject.toml content
	pyprojectContent, err := os.ReadFile("my-py-tool/pyproject.toml")
	require.NoError(t, err)
	assert.Contains(t, string(pyprojectContent), `name = "my-py-tool"`)
	assert.Contains(t, string(pyprojectContent), "ftl-sdk")
}

func TestGenerateComponent_Go(t *testing.T) {
	// Create temp directory
	tmpDir := t.TempDir()
	oldWd, _ := os.Getwd()
	defer func() { _ = os.Chdir(oldWd) }()
	_ = os.Chdir(tmpDir)

	// Create minimal ftl.yaml
	ftlConfig := config.FTLConfig{
		Application: config.ApplicationConfig{
			Name:    "test-app",
			Version: "0.1.0",
		},
	}
	data, _ := yaml.Marshal(&ftlConfig)
	_ = os.WriteFile("ftl.yaml", data, 0644)

	// Generate component
	scaffolder, err := NewScaffolder()
	require.NoError(t, err)

	err = scaffolder.GenerateComponent("my-go-tool", "go")
	require.NoError(t, err)

	// Check generated files
	expectedFiles := []string{
		"my-go-tool/go.mod",
		"my-go-tool/Makefile",
		"my-go-tool/main.go",
		"my-go-tool/main_test.go",
		"my-go-tool/.gitignore",
	}

	for _, file := range expectedFiles {
		assert.FileExists(t, file)
	}

	// Check go.mod content
	gomodContent, err := os.ReadFile("my-go-tool/go.mod")
	require.NoError(t, err)
	assert.Contains(t, string(gomodContent), "module github.com/example/my-go-tool")
	assert.Contains(t, string(gomodContent), "github.com/fastertools/ftl-cli/sdk/go")
}

func TestGenerateComponent_InvalidInputs(t *testing.T) {
	scaffolder, err := NewScaffolder()
	require.NoError(t, err)

	// Test invalid component name
	err = scaffolder.validateInputs("Invalid-Name", "rust")
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "invalid component name")

	// Test invalid language
	err = scaffolder.validateInputs("valid-name", "invalid")
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "invalid language")
}

func TestGenerateComponent_DuplicateComponent(t *testing.T) {
	// Create temp directory
	tmpDir := t.TempDir()
	oldWd, _ := os.Getwd()
	defer func() { _ = os.Chdir(oldWd) }()
	_ = os.Chdir(tmpDir)

	// Create ftl.yaml with existing component
	ftlConfig := config.FTLConfig{
		Application: config.ApplicationConfig{
			Name:    "test-app",
			Version: "0.1.0",
		},
		Components: []config.ComponentConfig{
			{
				ID:     "existing-tool",
				Source: "existing-tool/main.wasm",
			},
		},
	}
	data, _ := yaml.Marshal(&ftlConfig)
	_ = os.WriteFile("ftl.yaml", data, 0644)

	// Try to generate component with same name
	scaffolder, err := NewScaffolder()
	require.NoError(t, err)

	err = scaffolder.GenerateComponent("existing-tool", "rust")
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "already exists")
}

func TestGenerateComponent_NoFTLConfig(t *testing.T) {
	// Create temp directory without ftl.yaml
	tmpDir := t.TempDir()
	oldWd, _ := os.Getwd()
	defer func() { _ = os.Chdir(oldWd) }()
	_ = os.Chdir(tmpDir)

	// Try to generate component
	scaffolder, err := NewScaffolder()
	require.NoError(t, err)

	err = scaffolder.GenerateComponent("my-tool", "rust")
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "no FTL configuration found")
}

func TestDetectConfigFormat(t *testing.T) {
	scaffolder, err := NewScaffolder()
	require.NoError(t, err)

	tests := []struct {
		name       string
		setupFiles func()
		wantFormat string
		wantPath   string
		wantError  bool
	}{
		{
			name: "yaml_config",
			setupFiles: func() {
				_ = os.WriteFile("ftl.yaml", []byte("application:\n  name: test\n"), 0644)
			},
			wantFormat: "yaml",
			wantPath:   "ftl.yaml",
		},
		{
			name: "json_config",
			setupFiles: func() {
				_ = os.WriteFile("ftl.json", []byte(`{"application":{"name":"test"}}`), 0644)
			},
			wantFormat: "json",
			wantPath:   "ftl.json",
		},
		{
			name: "cue_config",
			setupFiles: func() {
				_ = os.WriteFile("app.cue", []byte(`app: {name: "test"}`), 0644)
			},
			wantFormat: "cue",
			wantPath:   "app.cue",
		},
		{
			name: "go_config",
			setupFiles: func() {
				goCode := `package main
import "github.com/fastertools/ftl-cli/go/ftl/pkg/synthesis"
func main() {
	cdk := synthesis.NewCDK()
}`
				_ = os.WriteFile("main.go", []byte(goCode), 0644)
			},
			wantFormat: "go",
			wantPath:   "main.go",
		},
		{
			name: "go_file_not_ftl",
			setupFiles: func() {
				_ = os.WriteFile("main.go", []byte(`package main
func main() { println("hello") }`), 0644)
			},
			wantError: true,
		},
		{
			name:       "no_config",
			setupFiles: func() {},
			wantError:  true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			tmpDir := t.TempDir()
			oldWd, _ := os.Getwd()
			defer func() { _ = os.Chdir(oldWd) }()
			_ = os.Chdir(tmpDir)

			tt.setupFiles()

			format, path, err := scaffolder.detectConfigFormat()
			if tt.wantError {
				assert.Error(t, err)
			} else {
				assert.NoError(t, err)
				assert.Equal(t, tt.wantFormat, format)
				assert.Equal(t, tt.wantPath, path)
			}
		})
	}
}

func TestGenerateComponent_JSONConfig(t *testing.T) {
	// Create temp directory with ftl.json
	tmpDir := t.TempDir()
	oldWd, _ := os.Getwd()
	defer func() { _ = os.Chdir(oldWd) }()
	_ = os.Chdir(tmpDir)

	// Create ftl.json
	jsonConfig := `{
  "application": {
    "name": "test-app",
    "version": "0.1.0"
  },
  "components": []
}`
	_ = os.WriteFile("ftl.json", []byte(jsonConfig), 0644)

	// Generate component
	scaffolder, err := NewScaffolder()
	require.NoError(t, err)

	err = scaffolder.GenerateComponent("my-tool", "rust")
	require.NoError(t, err)

	// Check component was added to ftl.json
	data, err := os.ReadFile("ftl.json")
	require.NoError(t, err)

	var cfg config.FTLConfig
	err = json.Unmarshal(data, &cfg)
	require.NoError(t, err)

	assert.Len(t, cfg.Components, 1)
	assert.Equal(t, "my-tool", cfg.Components[0].ID)
	assert.Contains(t, cfg.Components[0].Source, "my_tool.wasm") // Rust uses underscores
}

func TestGenerateComponent_GoConfig(t *testing.T) {
	// Create temp directory with main.go
	tmpDir := t.TempDir()
	oldWd, _ := os.Getwd()
	defer func() { _ = os.Chdir(oldWd) }()
	_ = os.Chdir(tmpDir)

	// Create main.go with CDK
	goCode := `package main
import "github.com/fastertools/ftl-cli/go/ftl/pkg/synthesis"
func main() {
	cdk := synthesis.NewCDK()
	app := cdk.NewApp("test")
}`
	_ = os.WriteFile("main.go", []byte(goCode), 0644)

	// Try to generate component
	scaffolder, err := NewScaffolder()
	require.NoError(t, err)

	err = scaffolder.GenerateComponent("my-tool", "rust")
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "Go-based configurations require manual")
	assert.Contains(t, err.Error(), "app.AddComponent")
}

func TestGenerateComponent_CUEConfig(t *testing.T) {
	// Create temp directory with app.cue
	tmpDir := t.TempDir()
	oldWd, _ := os.Getwd()
	defer func() { _ = os.Chdir(oldWd) }()
	_ = os.Chdir(tmpDir)

	// Create app.cue
	_ = os.WriteFile("app.cue", []byte(`app: {name: "test"}`), 0644)

	// Try to generate component
	scaffolder, err := NewScaffolder()
	require.NoError(t, err)

	err = scaffolder.GenerateComponent("my-tool", "python")
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "CUE configurations require manual")
	assert.Contains(t, err.Error(), "app.cue components array")
}

func TestCreateComponentInstance(t *testing.T) {
	scaffolder, err := NewScaffolder()
	require.NoError(t, err)

	// Test creating Rust component instance
	component, err := scaffolder.createComponentInstance("test-component", "rust")
	require.NoError(t, err)

	// Check name is set correctly
	name, err := component.LookupPath(cue.ParsePath("name")).String()
	require.NoError(t, err)
	assert.Equal(t, "test-component", name)

	// Check language is set correctly
	language, err := component.LookupPath(cue.ParsePath("language")).String()
	require.NoError(t, err)
	assert.Equal(t, "rust", language)

	// Check build config exists
	build := component.LookupPath(cue.ParsePath("build"))
	assert.True(t, build.Exists())

	// Check files exist
	files := component.LookupPath(cue.ParsePath("files"))
	assert.True(t, files.Exists())
}
