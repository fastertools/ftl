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

func TestInitWithProjectPath(t *testing.T) {
	tests := []struct {
		name        string
		projectPath string
		projectName string
		wantErr     bool
		check       func(t *testing.T, baseDir, projectPath, projectName string)
	}{
		{
			name:        "absolute project path",
			projectPath: "",  // Will be set to tmpDir in test
			projectName: "absolute-project",
			wantErr:     false,
			check: func(t *testing.T, baseDir, projectPath, projectName string) {
				expectedPath := filepath.Join(projectPath, projectName)
				manifestPath := filepath.Join(expectedPath, "ftl.yaml")
				assert.FileExists(t, manifestPath)
			},
		},
		{
			name:        "nested project path",
			projectPath: "", // Will be set to tmpDir/nested in test
			projectName: "nested-project",
			wantErr:     false,
			check: func(t *testing.T, baseDir, projectPath, projectName string) {
				expectedPath := filepath.Join(projectPath, projectName)
				manifestPath := filepath.Join(expectedPath, "ftl.yaml")
				assert.FileExists(t, manifestPath)
				
				// Verify project is not in working directory
				wrongPath := filepath.Join(baseDir, projectName, "ftl.yaml")
				assert.NoFileExists(t, wrongPath)
			},
		},
		{
			name:        "empty project path uses current directory",
			projectPath: "",
			projectName: "current-dir-project",
			wantErr:     false,
			check: func(t *testing.T, baseDir, projectPath, projectName string) {
				// When ProjectPath is empty, should create in current directory
				expectedPath := filepath.Join(baseDir, projectName)
				manifestPath := filepath.Join(expectedPath, "ftl.yaml")
				assert.FileExists(t, manifestPath)
			},
		},
		{
			name:        "nonexistent project path gets created",
			projectPath: "", // Will be set to tmpDir/nonexistent in test
			projectName: "auto-created-project", 
			wantErr:     false,
			check: func(t *testing.T, baseDir, projectPath, projectName string) {
				expectedPath := filepath.Join(projectPath, projectName)
				manifestPath := filepath.Join(expectedPath, "ftl.yaml")
				assert.FileExists(t, manifestPath)
				
				// Verify the intermediate directories were created
				assert.DirExists(t, projectPath)
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			tmpDir := t.TempDir()
			
			// Set up project path based on test case
			var projectPath string
			switch tt.name {
			case "absolute project path":
				projectPath = tmpDir
			case "nested project path":
				projectPath = filepath.Join(tmpDir, "nested")
				// Create the nested directory
				err := os.MkdirAll(projectPath, 0750)
				require.NoError(t, err)
			case "empty project path uses current directory":
				projectPath = "" // Empty means use current directory
			case "nonexistent project path gets created":
				projectPath = filepath.Join(tmpDir, "nonexistent")
				// Don't create this directory - let the command create it
			}

			// Change to a different working directory to ensure project-path works
			workingDir := filepath.Join(tmpDir, "working")
			err := os.MkdirAll(workingDir, 0750)
			require.NoError(t, err)
			
			oldWd, _ := os.Getwd()
			defer func() { _ = os.Chdir(oldWd) }()
			err = os.Chdir(workingDir)
			require.NoError(t, err)

			opts := &InitOptions{
				Name:          tt.projectName,
				Description:   "Test project with project path",
				Language:      "yaml",
				ProjectPath:   projectPath,
				NoInteractive: true,
			}

			err = runInit(opts)
			if tt.wantErr {
				assert.Error(t, err)
			} else {
				assert.NoError(t, err)
				if tt.check != nil {
					tt.check(t, workingDir, projectPath, tt.projectName)
				}
			}
		})
	}
}

func TestInitProjectPathWithExistingDirectory(t *testing.T) {
	tmpDir := t.TempDir()
	projectsDir := filepath.Join(tmpDir, "projects")
	err := os.MkdirAll(projectsDir, 0750)
	require.NoError(t, err)

	// Create existing project directory
	existingDir := filepath.Join(projectsDir, "existing-project")
	err = os.MkdirAll(existingDir, 0750)
	require.NoError(t, err)

	// Change working directory to somewhere else
	workingDir := filepath.Join(tmpDir, "working")
	err = os.MkdirAll(workingDir, 0750)
	require.NoError(t, err)
	
	oldWd, _ := os.Getwd()
	defer func() { _ = os.Chdir(oldWd) }()
	err = os.Chdir(workingDir)
	require.NoError(t, err)

	// Try to init without force - should fail
	opts := &InitOptions{
		Name:          "existing-project",
		Language:      "yaml",
		ProjectPath:   projectsDir,
		NoInteractive: true,
		Force:         false,
	}
	err = runInit(opts)
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "already exists")

	// Try with force - should succeed
	opts.Force = true
	err = runInit(opts)
	assert.NoError(t, err)
	
	// Verify files were created in the correct location
	manifestPath := filepath.Join(existingDir, "ftl.yaml")
	assert.FileExists(t, manifestPath)
}
