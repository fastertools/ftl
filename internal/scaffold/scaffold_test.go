package scaffold

import (
	"os"
	"path/filepath"
	"testing"

	"cuelang.org/go/cue"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"gopkg.in/yaml.v3"

	"github.com/fastertools/ftl-cli/pkg/types"
)

func TestValidateComponentName(t *testing.T) {
	tests := []struct {
		name    string
		input   string
		wantErr bool
		errMsg  string
	}{
		{"valid lowercase", "my-component", false, ""},
		{"valid with numbers", "component-123", false, ""},
		{"valid single letter", "a", false, ""},
		{"empty name", "", true, "cannot be empty"},
		{"starts with number", "123-component", true, "must start with"},
		{"starts with hyphen", "-component", true, "must start with"},
		{"ends with hyphen", "component-", true, "cannot start/end"},
		{"double hyphen", "component--name", true, "double hyphens"},
		{"uppercase letters", "MyComponent", true, "lowercase letters"},
		{"special characters", "comp@nent", true, "lowercase letters"},
		{"spaces", "my component", true, "lowercase letters"},
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
	assert.NotNil(t, scaffolder.templates)
}

func TestListLanguages(t *testing.T) {
	scaffolder, err := NewScaffolder()
	require.NoError(t, err)

	languages := scaffolder.ListLanguages()
	assert.Len(t, languages, 4)
	assert.Contains(t, languages, "rust")
	assert.Contains(t, languages, "typescript")
	assert.Contains(t, languages, "python")
	assert.Contains(t, languages, "go")
}

func TestGetWasmPath(t *testing.T) {
	scaffolder, _ := NewScaffolder()

	tests := []struct {
		name     string
		compName string
		language string
		expected string
	}{
		{"rust normal", "my-component", "rust", "my-component/my_component.wasm"},
		{"rust with hyphens", "test-comp-name", "rust", "test-comp-name/test_comp_name.wasm"},
		{"typescript", "my-component", "typescript", "my-component/dist/my-component.wasm"},
		{"python", "my-component", "python", "my-component/app.wasm"},
		{"go", "my-component", "go", "my-component/main.wasm"},
		{"unknown", "my-component", "unknown", "my-component/my-component.wasm"},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := scaffolder.getWasmPath(tt.compName, tt.language)
			assert.Equal(t, tt.expected, result)
		})
	}
}

func TestDetectConfigFormat(t *testing.T) {
	scaffolder, _ := NewScaffolder()

	tests := []struct {
		name       string
		files      map[string]string
		wantFormat string
		wantPath   string
		wantErr    bool
	}{
		{
			name:       "yaml config",
			files:      map[string]string{"ftl.yaml": "test"},
			wantFormat: "yaml",
			wantPath:   "ftl.yaml",
		},
		{
			name:       "json config",
			files:      map[string]string{"ftl.json": "{}"},
			wantFormat: "json",
			wantPath:   "ftl.json",
		},
		{
			name:       "cue config",
			files:      map[string]string{"app.cue": "package app"},
			wantFormat: "cue",
			wantPath:   "app.cue",
		},
		{
			name: "go config",
			files: map[string]string{
				"main.go": `package main
import "github.com/fastertools/ftl-cli/internal/synthesis"
func main() {
	cdk := synthesis.NewCDK()
}`,
			},
			wantFormat: "go",
			wantPath:   "main.go",
		},
		{
			name:    "no config",
			files:   map[string]string{},
			wantErr: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			tmpDir := t.TempDir()
			oldWd, _ := os.Getwd()
			defer os.Chdir(oldWd)
			os.Chdir(tmpDir)

			// Create test files
			for name, content := range tt.files {
				err := os.WriteFile(name, []byte(content), 0644)
				require.NoError(t, err)
			}

			format, path, err := scaffolder.detectConfigFormat()
			if tt.wantErr {
				assert.Error(t, err)
			} else {
				assert.NoError(t, err)
				assert.Equal(t, tt.wantFormat, format)
				assert.Equal(t, tt.wantPath, path)
			}
		})
	}
}

func TestValidateInputs(t *testing.T) {
	scaffolder, _ := NewScaffolder()

	tests := []struct {
		name     string
		compName string
		language string
		wantErr  bool
	}{
		{"valid", "my-component", "rust", false},
		{"invalid name", "123-comp", "rust", true},
		{"invalid language", "my-component", "java", true},
		{"empty name", "", "rust", true},
		{"empty language", "my-component", "", true},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := scaffolder.validateInputs(tt.compName, tt.language)
			if tt.wantErr {
				assert.Error(t, err)
			} else {
				assert.NoError(t, err)
			}
		})
	}
}

func TestGenerateComponent_BasicFlow(t *testing.T) {
	scaffolder, err := NewScaffolder()
	require.NoError(t, err)

	tmpDir := t.TempDir()
	oldWd, _ := os.Getwd()
	defer os.Chdir(oldWd)
	os.Chdir(tmpDir)

	// Create initial ftl.yaml
	manifest := &types.Manifest{
		Application: types.Application{
			Name:    "test-app",
			Version: "0.1.0",
		},
		Components: []types.Component{},
		Access:     "public",
	}
	data, _ := yaml.Marshal(manifest)
	os.WriteFile("ftl.yaml", data, 0644)

	// Generate a Rust component
	err = scaffolder.GenerateComponent("test-tool", "rust")
	assert.NoError(t, err)

	// Check files were created
	assert.DirExists(t, "test-tool")
	assert.FileExists(t, "test-tool/Cargo.toml")
	assert.FileExists(t, "test-tool/src/lib.rs")
	assert.FileExists(t, "test-tool/Makefile")

	// Check Cargo.toml has correct package name (underscores)
	cargoContent, _ := os.ReadFile("test-tool/Cargo.toml")
	assert.Contains(t, string(cargoContent), `name = "test_tool"`)

	// Check ftl.yaml was updated
	updatedData, _ := os.ReadFile("ftl.yaml")
	var updatedManifest types.Manifest
	yaml.Unmarshal(updatedData, &updatedManifest)
	assert.Len(t, updatedManifest.Components, 1)
	assert.Equal(t, "test-tool", updatedManifest.Components[0].ID)
	assert.Equal(t, "test-tool/test_tool.wasm", updatedManifest.Components[0].Source)
}

func TestGenerateComponent_TypeScript(t *testing.T) {
	scaffolder, _ := NewScaffolder()

	tmpDir := t.TempDir()
	oldWd, _ := os.Getwd()
	defer os.Chdir(oldWd)
	os.Chdir(tmpDir)

	// Create initial ftl.yaml
	manifest := &types.Manifest{
		Application: types.Application{
			Name:    "test-app",
			Version: "0.1.0",
		},
		Components: []types.Component{},
		Access:     "public",
	}
	data, _ := yaml.Marshal(manifest)
	os.WriteFile("ftl.yaml", data, 0644)

	// Generate TypeScript component
	err := scaffolder.GenerateComponent("ts-tool", "typescript")
	assert.NoError(t, err)

	// Check TypeScript files
	assert.FileExists(t, "ts-tool/package.json")
	assert.FileExists(t, "ts-tool/tsconfig.json")
	assert.FileExists(t, "ts-tool/src/index.ts")
	assert.FileExists(t, "ts-tool/Makefile")

	// Check package.json
	pkgContent, _ := os.ReadFile("ts-tool/package.json")
	assert.Contains(t, string(pkgContent), `"name": "ts-tool"`)
}

func TestGenerateComponent_Python(t *testing.T) {
	scaffolder, _ := NewScaffolder()

	tmpDir := t.TempDir()
	oldWd, _ := os.Getwd()
	defer os.Chdir(oldWd)
	os.Chdir(tmpDir)

	// Create initial ftl.yaml
	manifest := &types.Manifest{
		Application: types.Application{
			Name:    "test-app",
			Version: "0.1.0",
		},
		Components: []types.Component{},
		Access:     "public",
	}
	data, _ := yaml.Marshal(manifest)
	os.WriteFile("ftl.yaml", data, 0644)

	// Generate Python component
	err := scaffolder.GenerateComponent("py-tool", "python")
	assert.NoError(t, err)

	// Check Python files
	assert.FileExists(t, "py-tool/src/main.py")
	assert.FileExists(t, "py-tool/pyproject.toml")
	assert.FileExists(t, "py-tool/Makefile")
}

func TestGenerateComponent_Go(t *testing.T) {
	scaffolder, _ := NewScaffolder()

	tmpDir := t.TempDir()
	oldWd, _ := os.Getwd()
	defer os.Chdir(oldWd)
	os.Chdir(tmpDir)

	// Create initial ftl.yaml
	manifest := &types.Manifest{
		Application: types.Application{
			Name:    "test-app",
			Version: "0.1.0",
		},
		Components: []types.Component{},
		Access:     "public",
	}
	data, _ := yaml.Marshal(manifest)
	os.WriteFile("ftl.yaml", data, 0644)

	// Generate Go component
	err := scaffolder.GenerateComponent("go-tool", "go")
	assert.NoError(t, err)

	// Check Go files
	assert.FileExists(t, "go-tool/main.go")
	assert.FileExists(t, "go-tool/go.mod")
	assert.FileExists(t, "go-tool/Makefile")

	// Check go.mod
	goModContent, _ := os.ReadFile("go-tool/go.mod")
	assert.Contains(t, string(goModContent), "module github.com/example/go-tool")
}

func TestGenerateComponent_DuplicateName(t *testing.T) {
	scaffolder, _ := NewScaffolder()

	tmpDir := t.TempDir()
	oldWd, _ := os.Getwd()
	defer os.Chdir(oldWd)
	os.Chdir(tmpDir)

	// Create ftl.yaml with existing component
	manifest := &types.Manifest{
		Application: types.Application{
			Name:    "test-app",
			Version: "0.1.0",
		},
		Components: []types.Component{
			{ID: "existing", Source: "./existing"},
		},
		Access: "public",
	}
	data, _ := yaml.Marshal(manifest)
	os.WriteFile("ftl.yaml", data, 0644)

	// Try to generate component with duplicate name
	err := scaffolder.GenerateComponent("existing", "rust")
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "already exists")
}

func TestGenerateComponent_InvalidInputs(t *testing.T) {
	scaffolder, _ := NewScaffolder()

	tmpDir := t.TempDir()
	oldWd, _ := os.Getwd()
	defer os.Chdir(oldWd)
	os.Chdir(tmpDir)

	// Create ftl.yaml
	manifest := &types.Manifest{
		Application: types.Application{
			Name:    "test-app",
			Version: "0.1.0",
		},
	}
	data, _ := yaml.Marshal(manifest)
	os.WriteFile("ftl.yaml", data, 0644)

	// Test invalid component name
	err := scaffolder.GenerateComponent("123-invalid", "rust")
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "invalid component name")

	// Test invalid language
	err = scaffolder.GenerateComponent("valid-name", "java")
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "invalid language")
}

func TestGenerateComponent_JSONConfig(t *testing.T) {
	scaffolder, _ := NewScaffolder()

	tmpDir := t.TempDir()
	oldWd, _ := os.Getwd()
	defer os.Chdir(oldWd)
	os.Chdir(tmpDir)

	// Create ftl.json instead of yaml
	jsonContent := `{
		"application": {
			"name": "test-app",
			"version": "0.1.0"
		},
		"components": [],
		"access": "public"
	}`
	os.WriteFile("ftl.json", []byte(jsonContent), 0644)

	// Generate component
	err := scaffolder.GenerateComponent("json-comp", "rust")
	assert.NoError(t, err)

	// Verify JSON was updated
	updatedJSON, _ := os.ReadFile("ftl.json")
	assert.Contains(t, string(updatedJSON), "json-comp")
}

func TestUpdateFTLConfig_UnsupportedFormats(t *testing.T) {
	scaffolder, _ := NewScaffolder()

	tests := []struct {
		name      string
		format    string
		setupFile string
		content   string
		errMsg    string
	}{
		{
			name:      "go config",
			format:    "go",
			setupFile: "main.go",
			content:   `package main; import "github.com/fastertools/ftl-cli/internal/synthesis"; func main() { synthesis.NewCDK() }`,
			errMsg:    "Go-based configurations require manual",
		},
		{
			name:      "cue config",
			format:    "cue",
			setupFile: "app.cue",
			content:   `package app`,
			errMsg:    "CUE configurations require manual",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			tmpDir := t.TempDir()
			oldWd, _ := os.Getwd()
			defer os.Chdir(oldWd)
			os.Chdir(tmpDir)

			// Create config file
			os.WriteFile(tt.setupFile, []byte(tt.content), 0644)

			// Try to generate component
			err := scaffolder.GenerateComponent("test-comp", "rust")
			assert.Error(t, err)
			assert.Contains(t, err.Error(), tt.errMsg)
		})
	}
}

func TestGenerateFiles_RustNameConversion(t *testing.T) {
	scaffolder, _ := NewScaffolder()

	tmpDir := t.TempDir()
	oldWd, _ := os.Getwd()
	defer os.Chdir(oldWd)
	os.Chdir(tmpDir)

	// Create ftl.yaml
	manifest := &types.Manifest{
		Application: types.Application{
			Name:    "test-app",
			Version: "0.1.0",
		},
	}
	data, _ := yaml.Marshal(manifest)
	os.WriteFile("ftl.yaml", data, 0644)

	// Generate Rust component with hyphens
	err := scaffolder.GenerateComponent("my-cool-tool", "rust")
	assert.NoError(t, err)

	// Check Cargo.toml has underscores for package name
	cargoContent, _ := os.ReadFile("my-cool-tool/Cargo.toml")
	cargoStr := string(cargoContent)
	assert.Contains(t, cargoStr, `name = "my_cool_tool"`)

	// Check Makefile references underscore version
	makefileContent, _ := os.ReadFile("my-cool-tool/Makefile")
	makefileStr := string(makefileContent)
	assert.Contains(t, makefileStr, "my_cool_tool.wasm")

	// Check ftl.yaml has correct WASM path
	updatedData, _ := os.ReadFile("ftl.yaml")
	var updatedManifest types.Manifest
	yaml.Unmarshal(updatedData, &updatedManifest)
	assert.Equal(t, "my-cool-tool/my_cool_tool.wasm", updatedManifest.Components[0].Source)
}

func TestCreateComponentInstance(t *testing.T) {
	scaffolder, err := NewScaffolder()
	require.NoError(t, err)

	// Test creating instances for each language
	languages := []string{"rust", "typescript", "python", "go"}

	for _, lang := range languages {
		t.Run(lang, func(t *testing.T) {
			component, err := scaffolder.createComponentInstance("test-comp", lang)
			assert.NoError(t, err)
			assert.True(t, component.Exists())

			// Check that files are defined
			files := component.LookupPath(cue.ParsePath("files"))
			assert.True(t, files.Exists())

			// Check language field
			langField := component.LookupPath(cue.ParsePath("language"))
			langValue, _ := langField.String()
			assert.Equal(t, lang, langValue)
		})
	}
}

func TestGenerateComponent_CreatesCorrectStructure(t *testing.T) {
	scaffolder, _ := NewScaffolder()

	tests := []struct {
		language      string
		expectedFiles []string
	}{
		{
			"rust",
			[]string{"Cargo.toml", "src/lib.rs", "Makefile"},
		},
		{
			"typescript",
			[]string{"package.json", "tsconfig.json", "src/index.ts"},
		},
		{
			"python",
			[]string{"src/main.py", "pyproject.toml", "Makefile"},
		},
		{
			"go",
			[]string{"main.go", "go.mod", "Makefile"},
		},
	}

	for _, tt := range tests {
		t.Run(tt.language, func(t *testing.T) {
			tmpDir := t.TempDir()
			oldWd, _ := os.Getwd()
			defer os.Chdir(oldWd)
			os.Chdir(tmpDir)

			// Create ftl.yaml
			manifest := &types.Manifest{
				Application: types.Application{
					Name:    "test-app",
					Version: "0.1.0",
				},
			}
			data, _ := yaml.Marshal(manifest)
			os.WriteFile("ftl.yaml", data, 0644)

			// Generate component
			compName := tt.language + "-comp"
			err := scaffolder.GenerateComponent(compName, tt.language)
			assert.NoError(t, err)

			// Check all expected files exist
			for _, file := range tt.expectedFiles {
				path := filepath.Join(compName, file)
				assert.FileExists(t, path, "Missing file: %s", file)
			}
		})
	}
}
