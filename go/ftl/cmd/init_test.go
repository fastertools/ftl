package cmd

import (
	"os"
	"path/filepath"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"gopkg.in/yaml.v3"

	"github.com/fastertools/ftl-cli/go/shared/types"
)

func TestInitCommand(t *testing.T) {
	cmd := newInitCmd()
	assert.NotNil(t, cmd)
	assert.Contains(t, cmd.Use, "init")
	assert.Contains(t, cmd.Short, "Initialize")
}

func TestRunInit(t *testing.T) {
	tests := []struct {
		name    string
		opts    *InitOptions
		wantErr bool
		check   func(t *testing.T, dir string)
	}{
		{
			name: "yaml format",
			opts: &InitOptions{
				Name:          "test-project",
				Description:   "Test project",
				Format:        "yaml",
				NoInteractive: true,
			},
			wantErr: false,
			check: func(t *testing.T, dir string) {
				// Check ftl.yaml exists
				manifestPath := filepath.Join(dir, "test-project", "ftl.yaml")
				assert.FileExists(t, manifestPath)
				
				// Verify content
				data, err := os.ReadFile(manifestPath)
				require.NoError(t, err)
				
				var manifest types.Manifest
				err = yaml.Unmarshal(data, &manifest)
				require.NoError(t, err)
				
				assert.Equal(t, "test-project", manifest.Application.Name)
				assert.Equal(t, "0.1.0", manifest.Application.Version)
				assert.Equal(t, "Test project", manifest.Application.Description)
				assert.Equal(t, "public", manifest.Access)
			},
		},
		{
			name: "json format",
			opts: &InitOptions{
				Name:          "json-project",
				Format:        "json",
				NoInteractive: true,
			},
			wantErr: false,
			check: func(t *testing.T, dir string) {
				jsonPath := filepath.Join(dir, "json-project", "ftl.json")
				assert.FileExists(t, jsonPath)
			},
		},
		{
			name: "go format",
			opts: &InitOptions{
				Name:          "go-project",
				Format:        "go",
				NoInteractive: true,
			},
			wantErr: false,
			check: func(t *testing.T, dir string) {
				mainPath := filepath.Join(dir, "go-project", "main.go")
				assert.FileExists(t, mainPath)
				
				// Check go.mod also created
				goModPath := filepath.Join(dir, "go-project", "go.mod")
				assert.FileExists(t, goModPath)
			},
		},
		{
			name: "cue format",
			opts: &InitOptions{
				Name:          "cue-project",
				Format:        "cue",
				NoInteractive: true,
			},
			wantErr: false,
			check: func(t *testing.T, dir string) {
				cuePath := filepath.Join(dir, "cue-project", "app.cue")
				assert.FileExists(t, cuePath)
			},
		},
		{
			name: "with template",
			opts: &InitOptions{
				Name:          "template-project",
				Template:      "mcp",
				Format:        "yaml",
				NoInteractive: true,
			},
			wantErr: false,
			check: func(t *testing.T, dir string) {
				manifestPath := filepath.Join(dir, "template-project", "ftl.yaml")
				assert.FileExists(t, manifestPath)
				
				// Check .gitignore created
				gitignorePath := filepath.Join(dir, "template-project", ".gitignore")
				assert.FileExists(t, gitignorePath)
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			tmpDir := t.TempDir()
			oldWd, _ := os.Getwd()
			defer os.Chdir(oldWd)
			os.Chdir(tmpDir)

			err := runInit(tt.opts)
			if tt.wantErr {
				assert.Error(t, err)
			} else {
				assert.NoError(t, err)
				if tt.check != nil {
					tt.check(t, tmpDir)
				}
			}
		})
	}
}

func TestInitExistingDirectory(t *testing.T) {
	tmpDir := t.TempDir()
	oldWd, _ := os.Getwd()
	defer os.Chdir(oldWd)
	os.Chdir(tmpDir)

	// Create directory
	os.Mkdir("existing", 0755)

	// Try to init without force
	opts := &InitOptions{
		Name:          "existing",
		Format:        "yaml",
		NoInteractive: true,
		Force:         false,
	}
	err := runInit(opts)
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "already exists")

	// Try with force
	opts.Force = true
	err = runInit(opts)
	assert.NoError(t, err)
}

func TestCreateYAMLConfig(t *testing.T) {
	tmpDir := t.TempDir()
	
	opts := &InitOptions{
		Name:        "test-app",
		Description: "Test application",
	}
	
	err := createYAMLConfig(tmpDir, opts)
	require.NoError(t, err)
	
	// Verify file exists and content is valid
	manifestPath := filepath.Join(tmpDir, "ftl.yaml")
	assert.FileExists(t, manifestPath)
	
	data, err := os.ReadFile(manifestPath)
	require.NoError(t, err)
	
	var manifest types.Manifest
	err = yaml.Unmarshal(data, &manifest)
	require.NoError(t, err)
	
	assert.Equal(t, "test-app", manifest.Application.Name)
	assert.Equal(t, "0.1.0", manifest.Application.Version)
	assert.Equal(t, "Test application", manifest.Application.Description)
}

func TestCreateGitignore(t *testing.T) {
	tmpDir := t.TempDir()
	
	err := createGitignore(tmpDir)
	require.NoError(t, err)
	
	gitignorePath := filepath.Join(tmpDir, ".gitignore")
	assert.FileExists(t, gitignorePath)
	
	content, err := os.ReadFile(gitignorePath)
	require.NoError(t, err)
	
	// Check for essential entries
	contentStr := string(content)
	assert.Contains(t, contentStr, ".spin/")
	assert.Contains(t, contentStr, "spin.toml")
	assert.Contains(t, contentStr, "*.wasm")
	assert.Contains(t, contentStr, ".ftl/")
	assert.Contains(t, contentStr, ".env")
	assert.Contains(t, contentStr, "target/")
	assert.Contains(t, contentStr, "node_modules/")
}

func TestCreateGoConfig(t *testing.T) {
	tmpDir := t.TempDir()
	
	opts := &InitOptions{
		Name:        "go-app",
		Description: "Go application",
	}
	
	err := createGoConfig(tmpDir, opts)
	require.NoError(t, err)
	
	// Check main.go
	mainPath := filepath.Join(tmpDir, "main.go")
	assert.FileExists(t, mainPath)
	
	content, err := os.ReadFile(mainPath)
	require.NoError(t, err)
	contentStr := string(content)
	
	assert.Contains(t, contentStr, "package main")
	assert.Contains(t, contentStr, "synthesis.NewCDK")
	assert.Contains(t, contentStr, "go-app")
	assert.Contains(t, contentStr, "Go application")
	
	// Check go.mod
	goModPath := filepath.Join(tmpDir, "go.mod")
	assert.FileExists(t, goModPath)
}