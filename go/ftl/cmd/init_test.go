package cmd

import (
	"os"
	"path/filepath"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/fastertools/ftl-cli/go/shared/config"
)

func TestInitCommand(t *testing.T) {
	tests := []struct {
		name      string
		opts      *InitOptions
		wantErr   bool
		errMsg    string
		checkFunc func(t *testing.T, dir string)
	}{
		{
			name: "successful init with MCP template",
			opts: &InitOptions{
				Name:          "test-app",
				Description:   "Test application",
				Template:      "mcp",
				NoInteractive: true,
			},
			wantErr: false,
			checkFunc: func(t *testing.T, dir string) {
				// Check ftl.yaml exists and is valid
				ftlPath := filepath.Join(dir, "test-app", "ftl.yaml")
				assert.FileExists(t, ftlPath)
				
				cfg, err := config.Load(ftlPath)
				require.NoError(t, err)
				assert.Equal(t, "test-app", cfg.Name)
				assert.Equal(t, "Test application", cfg.Description)
				assert.Equal(t, "./spinc.yaml", cfg.Compose)
				
				// Check spinc.yaml exists
				spincPath := filepath.Join(dir, "test-app", "spinc.yaml")
				assert.FileExists(t, spincPath)
				
				content, err := os.ReadFile(spincPath)
				require.NoError(t, err)
				assert.Contains(t, string(content), "MCP Application Configuration")
				assert.Contains(t, string(content), "mcp-gateway")
				
				// Check .gitignore exists
				gitignorePath := filepath.Join(dir, "test-app", ".gitignore")
				assert.FileExists(t, gitignorePath)
			},
		},
		{
			name: "successful init with basic template",
			opts: &InitOptions{
				Name:          "basic-app",
				Template:      "basic",
				NoInteractive: true,
			},
			wantErr: false,
			checkFunc: func(t *testing.T, dir string) {
				spincPath := filepath.Join(dir, "basic-app", "spinc.yaml")
				content, err := os.ReadFile(spincPath)
				require.NoError(t, err)
				assert.Contains(t, string(content), "Basic Application Configuration")
				assert.NotContains(t, string(content), "mcp-gateway")
			},
		},
		{
			name: "successful init with empty template",
			opts: &InitOptions{
				Name:          "empty-app",
				Template:      "empty",
				NoInteractive: true,
			},
			wantErr: false,
			checkFunc: func(t *testing.T, dir string) {
				spincPath := filepath.Join(dir, "empty-app", "spinc.yaml")
				content, err := os.ReadFile(spincPath)
				require.NoError(t, err)
				assert.Contains(t, string(content), "components: {}")
			},
		},
		{
			name: "error - missing name in non-interactive mode",
			opts: &InitOptions{
				NoInteractive: true,
			},
			wantErr: true,
			errMsg:  "project name is required",
		},
		{
			name: "error - invalid template",
			opts: &InitOptions{
				Name:          "test-app",
				Template:      "invalid",
				NoInteractive: true,
			},
			wantErr: true,
			errMsg:  "unknown template",
		},
		{
			name: "error - directory exists without force",
			opts: &InitOptions{
				Name:          "existing-app",
				NoInteractive: true,
			},
			wantErr: true,
			errMsg:  "already exists",
			checkFunc: func(t *testing.T, dir string) {
				// Pre-create the directory
				os.MkdirAll(filepath.Join(dir, "existing-app"), 0755)
			},
		},
		{
			name: "successful overwrite with force",
			opts: &InitOptions{
				Name:          "force-app",
				Force:         true,
				NoInteractive: true,
			},
			wantErr: false,
			checkFunc: func(t *testing.T, dir string) {
				// Pre-create the directory
				forceDir := filepath.Join(dir, "force-app")
				os.MkdirAll(forceDir, 0755)
				
				// Create a file that should be overwritten
				testFile := filepath.Join(forceDir, "test.txt")
				os.WriteFile(testFile, []byte("old content"), 0644)
				
				// After init, check that new files exist
				t.Cleanup(func() {
					ftlPath := filepath.Join(forceDir, "ftl.yaml")
					assert.FileExists(t, ftlPath)
				})
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Create temp directory for test
			tmpDir := t.TempDir()
			
			// Change to temp directory
			origDir, err := os.Getwd()
			require.NoError(t, err)
			defer os.Chdir(origDir)
			
			err = os.Chdir(tmpDir)
			require.NoError(t, err)
			
			// Run pre-check function if provided
			if tt.checkFunc != nil && tt.wantErr {
				tt.checkFunc(t, tmpDir)
			}
			
			// Run init
			err = runInit(tt.opts)
			
			if tt.wantErr {
				assert.Error(t, err)
				if tt.errMsg != "" {
					assert.Contains(t, err.Error(), tt.errMsg)
				}
			} else {
				assert.NoError(t, err)
				
				// Run post-check function if provided
				if tt.checkFunc != nil {
					tt.checkFunc(t, tmpDir)
				}
			}
		})
	}
}

func TestGenerateTemplates(t *testing.T) {
	t.Run("MCP template", func(t *testing.T) {
		content := generateMCPTemplate("test-app")
		assert.Contains(t, content, "name: test-app")
		assert.Contains(t, content, "MCP Application Configuration")
		assert.Contains(t, content, "mcp-gateway")
		assert.Contains(t, content, "mcp-authorizer")
		assert.Contains(t, content, "auth:")
	})

	t.Run("Basic template", func(t *testing.T) {
		content := generateBasicTemplate("test-app")
		assert.Contains(t, content, "name: test-app")
		assert.Contains(t, content, "Basic Application Configuration")
		assert.NotContains(t, content, "mcp-gateway")
		assert.NotContains(t, content, "auth:")
	})

	t.Run("Empty template", func(t *testing.T) {
		content := generateEmptyTemplate("test-app")
		assert.Contains(t, content, "name: test-app")
		assert.Contains(t, content, "components: {}")
		assert.NotContains(t, content, "mcp-gateway")
	})
}

func TestCreateGitignore(t *testing.T) {
	tmpDir := t.TempDir()
	
	err := createGitignore(tmpDir)
	assert.NoError(t, err)
	
	gitignorePath := filepath.Join(tmpDir, ".gitignore")
	assert.FileExists(t, gitignorePath)
	
	content, err := os.ReadFile(gitignorePath)
	require.NoError(t, err)
	
	// Check for important entries
	assert.Contains(t, string(content), ".spin/")
	assert.Contains(t, string(content), "spin.toml")
	assert.Contains(t, string(content), "*.wasm")
	assert.Contains(t, string(content), ".ftl/")
	assert.Contains(t, string(content), ".env")
	assert.Contains(t, string(content), "node_modules/")
	assert.Contains(t, string(content), "__pycache__/")
}

func TestCreateFTLConfig(t *testing.T) {
	tmpDir := t.TempDir()
	
	tests := []struct {
		name string
		opts *InitOptions
		want config.FTLConfig
	}{
		{
			name: "with description",
			opts: &InitOptions{
				Name:        "test-app",
				Description: "Custom description",
			},
			want: config.FTLConfig{
				Name:        "test-app",
				Version:     "0.1.0",
				Description: "Custom description",
				Compose:     "./spinc.yaml",
			},
		},
		{
			name: "without description",
			opts: &InitOptions{
				Name: "test-app",
			},
			want: config.FTLConfig{
				Name:        "test-app",
				Version:     "0.1.0",
				Description: "test-app - An FTL application",
				Compose:     "./spinc.yaml",
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := createFTLConfig(tmpDir, tt.opts)
			assert.NoError(t, err)
			
			configPath := filepath.Join(tmpDir, "ftl.yaml")
			cfg, err := config.Load(configPath)
			require.NoError(t, err)
			
			assert.Equal(t, tt.want.Name, cfg.Name)
			assert.Equal(t, tt.want.Version, cfg.Version)
			assert.Equal(t, tt.want.Description, cfg.Description)
			assert.Equal(t, tt.want.Compose, cfg.Compose)
		})
	}
}

func TestCreateExampleComponent(t *testing.T) {
	tmpDir := t.TempDir()
	
	err := createExampleComponent(tmpDir, "mcp")
	assert.NoError(t, err)
	
	componentDir := filepath.Join(tmpDir, "components", "example")
	assert.DirExists(t, componentDir)
}