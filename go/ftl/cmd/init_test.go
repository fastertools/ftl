package cmd

import (
	"os"
	"path/filepath"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
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
			name: "successful init",
			opts: &InitOptions{
				Name:          "test-app",
				Description:   "Test application",
				NoInteractive: true,
			},
			wantErr: false,
			checkFunc: func(t *testing.T, dir string) {
				// Check ftl.yaml exists and is valid
				spincPath := filepath.Join(dir, "test-app", "ftl.yaml")
				assert.FileExists(t, spincPath)

				content, err := os.ReadFile(spincPath)
				require.NoError(t, err)
				assert.Contains(t, string(content), "application:")
				assert.Contains(t, string(content), "name: test-app")

				// Check .gitignore exists
				gitignorePath := filepath.Join(dir, "test-app", ".gitignore")
				assert.FileExists(t, gitignorePath)
			},
		},
		{
			name: "successful init with minimal config",
			opts: &InitOptions{
				Name:          "minimal-app",
				NoInteractive: true,
			},
			wantErr: false,
			checkFunc: func(t *testing.T, dir string) {
				spincPath := filepath.Join(dir, "minimal-app", "ftl.yaml")
				content, err := os.ReadFile(spincPath)
				require.NoError(t, err)
				assert.Contains(t, string(content), "application:")
				assert.Contains(t, string(content), "name: minimal-app")
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
			name: "error - directory exists without force",
			opts: &InitOptions{
				Name:          "existing-app",
				NoInteractive: true,
			},
			wantErr: true,
			errMsg:  "already exists",
			checkFunc: func(t *testing.T, dir string) {
				// Pre-create the directory
				_ = os.MkdirAll(filepath.Join(dir, "existing-app"), 0755)
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
				_ = os.MkdirAll(forceDir, 0755)

				// Create a file that should be overwritten
				testFile := filepath.Join(forceDir, "test.txt")
				os.WriteFile(testFile, []byte("old content"), 0644)

				// After init, check that new files exist
				t.Cleanup(func() {
					spincPath := filepath.Join(forceDir, "ftl.yaml")
					assert.FileExists(t, spincPath)
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
