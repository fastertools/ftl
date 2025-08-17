package cmd

import (
	"encoding/json"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/fastertools/ftl-cli/go/shared/config"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"gopkg.in/yaml.v3"
)

func TestCreateYAMLConfig(t *testing.T) {
	tests := []struct {
		name        string
		opts        *InitOptions
		wantName    string
		wantVersion string
		wantDesc    string
	}{
		{
			name: "with_description",
			opts: &InitOptions{
				Name:        "test-app",
				Description: "My test application",
			},
			wantName:    "test-app",
			wantVersion: "0.1.0",
			wantDesc:    "My test application",
		},
		{
			name: "without_description",
			opts: &InitOptions{
				Name:        "test-app",
				Description: "",
			},
			wantName:    "test-app",
			wantVersion: "0.1.0",
			wantDesc:    "test-app - An FTL application",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			tmpDir := t.TempDir()

			err := createYAMLConfig(tmpDir, tt.opts)
			assert.NoError(t, err)

			// Read and verify the config
			configPath := filepath.Join(tmpDir, "ftl.yaml")
			assert.FileExists(t, configPath)

			data, err := os.ReadFile(configPath)
			require.NoError(t, err)

			var cfg config.FTLConfig
			err = yaml.Unmarshal(data, &cfg)
			require.NoError(t, err)

			assert.Equal(t, tt.wantName, cfg.Application.Name)
			assert.Equal(t, tt.wantVersion, cfg.Application.Version)
			assert.Equal(t, tt.wantDesc, cfg.Application.Description)
		})
	}
}

func TestCreateJSONConfig(t *testing.T) {
	tests := []struct {
		name        string
		opts        *InitOptions
		wantName    string
		wantVersion string
		wantDesc    string
	}{
		{
			name: "with_description",
			opts: &InitOptions{
				Name:        "json-app",
				Description: "JSON test application",
			},
			wantName:    "json-app",
			wantVersion: "0.1.0",
			wantDesc:    "JSON test application",
		},
		{
			name: "without_description",
			opts: &InitOptions{
				Name:        "json-app",
				Description: "",
			},
			wantName:    "json-app",
			wantVersion: "0.1.0",
			wantDesc:    "json-app - An FTL application",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			tmpDir := t.TempDir()

			err := createJSONConfig(tmpDir, tt.opts)
			assert.NoError(t, err)

			// Read and verify the config
			configPath := filepath.Join(tmpDir, "ftl.json")
			assert.FileExists(t, configPath)

			data, err := os.ReadFile(configPath)
			require.NoError(t, err)

			// Check it's valid JSON
			var jsonData map[string]interface{}
			err = json.Unmarshal(data, &jsonData)
			require.NoError(t, err)

			// Verify structure
			assert.Contains(t, jsonData, "application")
			app := jsonData["application"].(map[string]interface{})
			assert.Equal(t, tt.wantName, app["name"])
			assert.Equal(t, tt.wantVersion, app["version"])
			assert.Equal(t, tt.wantDesc, app["description"])

			// Check components array exists
			assert.Contains(t, jsonData, "components")
			components := jsonData["components"].([]interface{})
			assert.Empty(t, components)

			// Check triggers array exists
			assert.Contains(t, jsonData, "triggers")
			triggers := jsonData["triggers"].([]interface{})
			assert.Empty(t, triggers)
		})
	}
}

func TestCreateGoConfig(t *testing.T) {
	tests := []struct {
		name     string
		opts     *InitOptions
		checkFor []string
	}{
		{
			name: "with_description",
			opts: &InitOptions{
				Name:        "go-app",
				Description: "Go test application",
			},
			checkFor: []string{
				"package main",
				"import",
				"synthesis.NewCDK",
				`NewApp("go-app")`,
				`SetDescription("Go test application")`,
				"SetVersion(\"0.1.0\")",
			},
		},
		{
			name: "without_description",
			opts: &InitOptions{
				Name:        "go-app",
				Description: "",
			},
			checkFor: []string{
				"package main",
				`NewApp("go-app")`,
				`SetDescription("go-app - An FTL application")`,
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			tmpDir := t.TempDir()

			err := createGoConfig(tmpDir, tt.opts)
			assert.NoError(t, err)

			// Check main.go
			mainPath := filepath.Join(tmpDir, "main.go")
			assert.FileExists(t, mainPath)

			mainContent, err := os.ReadFile(mainPath)
			require.NoError(t, err)

			for _, check := range tt.checkFor {
				assert.Contains(t, string(mainContent), check, "main.go should contain: %s", check)
			}

			// Check go.mod
			goModPath := filepath.Join(tmpDir, "go.mod")
			assert.FileExists(t, goModPath)

			goModContent, err := os.ReadFile(goModPath)
			require.NoError(t, err)

			assert.Contains(t, string(goModContent), "module "+tt.opts.Name)
			assert.Contains(t, string(goModContent), "go 1.21")
			assert.Contains(t, string(goModContent), "github.com/fastertools/ftl-cli/go/ftl")
		})
	}
}

func TestCreateCUEConfig(t *testing.T) {
	tests := []struct {
		name     string
		opts     *InitOptions
		checkFor []string
	}{
		{
			name: "with_description",
			opts: &InitOptions{
				Name:        "cue-app",
				Description: "CUE test application",
			},
			checkFor: []string{
				"package app",
				"import \"github.com/fastertools/ftl-cli/patterns\"",
				`name:        "cue-app"`,
				`version:     "0.1.0"`,
				`description: "CUE test application"`,
				"components: [",
				`access: "public"`,
			},
		},
		{
			name: "without_description",
			opts: &InitOptions{
				Name:        "cue-app",
				Description: "",
			},
			checkFor: []string{
				`name:        "cue-app"`,
				`description: "cue-app - An FTL application"`,
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			tmpDir := t.TempDir()

			err := createCUEConfig(tmpDir, tt.opts)
			assert.NoError(t, err)

			// Check app.cue
			cuePath := filepath.Join(tmpDir, "app.cue")
			assert.FileExists(t, cuePath)

			cueContent, err := os.ReadFile(cuePath)
			require.NoError(t, err)

			for _, check := range tt.checkFor {
				assert.Contains(t, string(cueContent), check, "app.cue should contain: %s", check)
			}
		})
	}
}

func TestCreateGitignoreContent(t *testing.T) {
	tmpDir := t.TempDir()

	err := createGitignore(tmpDir)
	assert.NoError(t, err)

	gitignorePath := filepath.Join(tmpDir, ".gitignore")
	assert.FileExists(t, gitignorePath)

	content, err := os.ReadFile(gitignorePath)
	require.NoError(t, err)

	// Check for important entries
	expectedEntries := []string{
		".spin/",
		"spin.toml",
		"*.wasm",
		".ftl/",
		".env",
		"target/",
		"node_modules/",
		"__pycache__/",
		".DS_Store",
	}

	for _, entry := range expectedEntries {
		assert.Contains(t, string(content), entry)
	}
}

func TestRunInit(t *testing.T) {
	tests := []struct {
		name      string
		opts      *InitOptions
		wantFiles []string
		wantErr   bool
	}{
		{
			name: "yaml_format",
			opts: &InitOptions{
				Name:          "test-project",
				Description:   "Test project",
				Format:        "yaml",
				NoInteractive: true,
			},
			wantFiles: []string{"ftl.yaml", ".gitignore"},
			wantErr:   false,
		},
		{
			name: "json_format",
			opts: &InitOptions{
				Name:          "json-project",
				Format:        "json",
				NoInteractive: true,
			},
			wantFiles: []string{"ftl.json", ".gitignore"},
			wantErr:   false,
		},
		{
			name: "go_format",
			opts: &InitOptions{
				Name:          "go-project",
				Format:        "go",
				NoInteractive: true,
			},
			wantFiles: []string{"main.go", "go.mod", ".gitignore"},
			wantErr:   false,
		},
		{
			name: "cue_format",
			opts: &InitOptions{
				Name:          "cue-project",
				Format:        "cue",
				NoInteractive: true,
			},
			wantFiles: []string{"app.cue", ".gitignore"},
			wantErr:   false,
		},
		{
			name: "no_name_no_interactive",
			opts: &InitOptions{
				NoInteractive: true,
			},
			wantErr: true,
		},
		{
			name: "unsupported_format",
			opts: &InitOptions{
				Name:          "bad-project",
				Format:        "xml",
				NoInteractive: true,
			},
			wantErr: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Create temp directory for test
			tmpDir := t.TempDir()
			oldWd, _ := os.Getwd()
			defer func() { _ = os.Chdir(oldWd) }()
			_ = os.Chdir(tmpDir)

			err := runInit(tt.opts)

			if tt.wantErr {
				assert.Error(t, err)
				return
			}

			assert.NoError(t, err)

			// Check that expected directory was created
			if tt.opts.Name != "" {
				assert.DirExists(t, tt.opts.Name)

				// Check for expected files
				for _, file := range tt.wantFiles {
					filePath := filepath.Join(tt.opts.Name, file)
					assert.FileExists(t, filePath, "Expected file %s to exist", file)
				}
			}
		})
	}
}

func TestRunInit_ExistingDirectory(t *testing.T) {
	tmpDir := t.TempDir()
	oldWd, _ := os.Getwd()
	defer func() { _ = os.Chdir(oldWd) }()
	_ = os.Chdir(tmpDir)

	// Create existing directory
	_ = os.Mkdir("existing", 0755)

	// Try without force flag
	opts := &InitOptions{
		Name:          "existing",
		Format:        "yaml",
		NoInteractive: true,
		Force:         false,
	}

	err := runInit(opts)
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "already exists")

	// Try with force flag
	opts.Force = true
	err = runInit(opts)
	assert.NoError(t, err)
}

func TestPromptForName(t *testing.T) {
	// This test is skipped as it requires interactive input
	t.Skip("Interactive test - requires user input")

	opts := &InitOptions{}
	err := promptForName(opts)
	// Would prompt user for project name
	_ = err
}

func TestPromptForFormat(t *testing.T) {
	// This test is skipped as it requires interactive input
	t.Skip("Interactive test - requires user input")

	opts := &InitOptions{}
	err := promptForFormat(opts)
	// Would prompt user to select format
	_ = err
}

// Benchmark tests
func BenchmarkCreateYAMLConfig(b *testing.B) {
	tmpDir := b.TempDir()
	opts := &InitOptions{
		Name:        "bench-app",
		Description: "Benchmark app",
	}

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_ = createYAMLConfig(tmpDir, opts)
	}
}

func BenchmarkCreateJSONConfig(b *testing.B) {
	tmpDir := b.TempDir()
	opts := &InitOptions{
		Name:        "bench-app",
		Description: "Benchmark app",
	}

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_ = createJSONConfig(tmpDir, opts)
	}
}

// Test the format-specific output messages
func TestInitOutputMessages(t *testing.T) {
	// This test verifies the console output for different formats
	// In a real implementation, we'd capture stdout

	formats := []string{"yaml", "json", "go", "cue"}

	for _, format := range formats {
		t.Run(format+"_output", func(t *testing.T) {
			// The actual runInit prints different instructions based on format
			// We're documenting the expected behavior here

			switch format {
			case "go":
				// Should print:
				// 2. Edit main.go to add your components
				// 3. go run main.go > spin.toml
				// 4. spin up
			case "cue":
				// Should print:
				// 2. Edit app.cue to add your components
				// 3. ftl synth app.cue
				// 4. spin up
			default:
				// Should print:
				// 2. Edit ftl.yaml/ftl.json to add your components
				// 3. ftl build
				// 4. ftl up
			}
		})
	}
}

// Test helper function to verify config content structure
func TestConfigContentStructure(t *testing.T) {
	t.Run("yaml_has_proper_structure", func(t *testing.T) {
		tmpDir := t.TempDir()
		opts := &InitOptions{Name: "test", Description: "Test"}

		err := createYAMLConfig(tmpDir, opts)
		require.NoError(t, err)

		content, _ := os.ReadFile(filepath.Join(tmpDir, "ftl.yaml"))

		// Should have proper YAML structure
		assert.True(t, strings.HasPrefix(string(content), "application:"))
		assert.Contains(t, string(content), "name:")
		assert.Contains(t, string(content), "version:")
	})

	t.Run("json_is_valid", func(t *testing.T) {
		tmpDir := t.TempDir()
		opts := &InitOptions{Name: "test", Description: "Test"}

		err := createJSONConfig(tmpDir, opts)
		require.NoError(t, err)

		content, _ := os.ReadFile(filepath.Join(tmpDir, "ftl.json"))

		// Should be valid JSON
		var js interface{}
		err = json.Unmarshal(content, &js)
		assert.NoError(t, err)
	})
}
