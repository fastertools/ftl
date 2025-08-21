package cli

import (
	"os"
	"path/filepath"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"gopkg.in/yaml.v3"
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
			name: "yaml config",
			opts: &InitOptions{
				Name:          "test-project",
				Description:   "Test project",
				Language:      "yaml",
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

				var manifest map[interface{}]interface{}
				err = yaml.Unmarshal(data, &manifest)
				require.NoError(t, err)

				assert.Equal(t, "test-project", manifest["name"])
				assert.Equal(t, "0.1.0", manifest["version"])
				assert.Equal(t, "Test project", manifest["description"])
				assert.Equal(t, "public", manifest["access"])
			},
		},
		{
			name: "json config",
			opts: &InitOptions{
				Name:          "json-project",
				Language:      "json",
				NoInteractive: true,
			},
			wantErr: false,
			check: func(t *testing.T, dir string) {
				jsonPath := filepath.Join(dir, "json-project", "ftl.json")
				assert.FileExists(t, jsonPath)
			},
		},
		{
			name: "go config",
			opts: &InitOptions{
				Name:          "go-project",
				Language:      "go",
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
			name: "cue config",
			opts: &InitOptions{
				Name:          "cue-project",
				Language:      "cue",
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
				Language:      "yaml",
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
			defer func() { _ = os.Chdir(oldWd) }()
			if err := os.Chdir(tmpDir); err != nil {
				t.Fatal(err)
			}

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
	defer func() { _ = os.Chdir(oldWd) }()
	if err := os.Chdir(tmpDir); err != nil {
		t.Fatal(err)
	}

	// Create directory
	if err := os.Mkdir("existing", 0750); err != nil {
		t.Fatal(err)
	}

	// Try to init without force
	opts := &InitOptions{
		Name:          "existing",
		Language:      "yaml",
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

func TestYAMLProjectGeneration(t *testing.T) {
	tmpDir := t.TempDir()

	opts := &InitOptions{
		Name:          "test-app",
		Description:   "Test application",
		Language:      "yaml",
		NoInteractive: true,
	}

	oldWd, _ := os.Getwd()
	defer func() { _ = os.Chdir(oldWd) }()
	if err := os.Chdir(tmpDir); err != nil {
		t.Fatal(err)
	}

	err := runInit(opts)
	require.NoError(t, err)

	// Verify file exists and content is valid
	manifestPath := filepath.Join(tmpDir, "test-app", "ftl.yaml")
	assert.FileExists(t, manifestPath)

	data, err := os.ReadFile(manifestPath)
	require.NoError(t, err)

	var manifest map[interface{}]interface{}
	err = yaml.Unmarshal(data, &manifest)
	require.NoError(t, err)

	assert.Equal(t, "test-app", manifest["name"])
	assert.Equal(t, "0.1.0", manifest["version"])
	assert.Equal(t, "Test application", manifest["description"])
}

func TestGitignoreGeneration(t *testing.T) {
	tmpDir := t.TempDir()

	opts := &InitOptions{
		Name:          "gitignore-test",
		Language:      "yaml",
		NoInteractive: true,
	}

	oldWd, _ := os.Getwd()
	defer func() { _ = os.Chdir(oldWd) }()
	if err := os.Chdir(tmpDir); err != nil {
		t.Fatal(err)
	}

	err := runInit(opts)
	require.NoError(t, err)

	gitignorePath := filepath.Join(tmpDir, "gitignore-test", ".gitignore")
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

func TestGoProjectGeneration(t *testing.T) {
	tmpDir := t.TempDir()

	opts := &InitOptions{
		Name:          "go-app",
		Description:   "Go application",
		Language:      "go",
		NoInteractive: true,
	}

	oldWd, _ := os.Getwd()
	defer func() { _ = os.Chdir(oldWd) }()
	if err := os.Chdir(tmpDir); err != nil {
		t.Fatal(err)
	}

	err := runInit(opts)
	require.NoError(t, err)

	// Check main.go
	mainPath := filepath.Join(tmpDir, "go-app", "main.go")
	assert.FileExists(t, mainPath)

	content, err := os.ReadFile(mainPath)
	require.NoError(t, err)
	contentStr := string(content)

	assert.Contains(t, contentStr, "package main")
	assert.Contains(t, contentStr, "cdk.New")
	assert.Contains(t, contentStr, "go-app")
	assert.Contains(t, contentStr, "Go application")

	// Check go.mod
	goModPath := filepath.Join(tmpDir, "go-app", "go.mod")
	assert.FileExists(t, goModPath)
}
