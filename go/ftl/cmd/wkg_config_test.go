package cmd

import (
	"encoding/base64"
	"os"
	"path/filepath"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestWkgConfigManagement(t *testing.T) {
	// Create a temporary directory for test config
	tmpDir := t.TempDir()
	configPath := filepath.Join(tmpDir, ".config", "wasm-pkg", "config.toml")
	
	// Override the config path for testing
	originalPath := wkgConfigPathOverride
	wkgConfigPathOverride = configPath
	defer func() { wkgConfigPathOverride = originalPath }()
	
	t.Run("load_empty_config", func(t *testing.T) {
		config, err := loadWkgConfig()
		require.NoError(t, err)
		assert.Equal(t, "ghcr.io", config.DefaultRegistry)
		assert.NotNil(t, config.Registry)
	})
	
	t.Run("update_ecr_auth", func(t *testing.T) {
		// Create a test ECR token (AWS:testpassword base64 encoded)
		testToken := base64.StdEncoding.EncodeToString([]byte("AWS:testpassword"))
		registryUri := "123456789.dkr.ecr.us-west-2.amazonaws.com"
		
		err := updateWkgAuthForECR(registryUri, testToken)
		require.NoError(t, err)
		
		// Verify the config was written
		assert.FileExists(t, configPath)
		
		// Load and verify the config
		config, err := loadWkgConfig()
		require.NoError(t, err)
		
		regConfig, exists := config.Registry[registryUri]
		require.True(t, exists, "Registry config should exist")
		require.NotNil(t, regConfig.OCI)
		require.NotNil(t, regConfig.OCI.Auth)
		assert.Equal(t, "AWS", regConfig.OCI.Auth.Username)
		assert.Equal(t, "testpassword", regConfig.OCI.Auth.Password)
	})
	
	t.Run("remove_ecr_auth", func(t *testing.T) {
		registryUri := "123456789.dkr.ecr.us-west-2.amazonaws.com"
		
		// First add auth
		testToken := base64.StdEncoding.EncodeToString([]byte("AWS:testpassword"))
		err := updateWkgAuthForECR(registryUri, testToken)
		require.NoError(t, err)
		
		// Then remove it
		err = removeWkgAuthForECR(registryUri)
		require.NoError(t, err)
		
		// Verify it's removed
		config, err := loadWkgConfig()
		require.NoError(t, err)
		
		if regConfig, exists := config.Registry[registryUri]; exists {
			assert.Nil(t, regConfig.OCI.Auth, "Auth should be removed")
		}
	})
	
	t.Run("preserve_existing_registries", func(t *testing.T) {
		// Add a GitHub registry
		config := &WkgConfig{
			DefaultRegistry: "ghcr.io",
			Registry: map[string]*WkgRegistryConfig{
				"ghcr.io": {
					OCI: &WkgOCIConfig{
						Auth: &WkgAuthConfig{
							Username: "testuser",
							Password: "testpass",
						},
					},
				},
			},
		}
		err := saveWkgConfig(config)
		require.NoError(t, err)
		
		// Add ECR auth
		testToken := base64.StdEncoding.EncodeToString([]byte("AWS:ecrpass"))
		registryUri := "999999999.dkr.ecr.us-east-1.amazonaws.com"
		err = updateWkgAuthForECR(registryUri, testToken)
		require.NoError(t, err)
		
		// Verify both registries exist
		config, err = loadWkgConfig()
		require.NoError(t, err)
		
		// Check GitHub registry is preserved
		ghConfig, exists := config.Registry["ghcr.io"]
		require.True(t, exists)
		assert.Equal(t, "testuser", ghConfig.OCI.Auth.Username)
		assert.Equal(t, "testpass", ghConfig.OCI.Auth.Password)
		
		// Check ECR registry was added
		ecrConfig, exists := config.Registry[registryUri]
		require.True(t, exists)
		assert.Equal(t, "AWS", ecrConfig.OCI.Auth.Username)
		assert.Equal(t, "ecrpass", ecrConfig.OCI.Auth.Password)
	})
}

func TestWkgConfigPath(t *testing.T) {
	// Test that we get a reasonable path
	path := getWkgConfigPath()
	assert.Contains(t, path, ".config")
	assert.Contains(t, path, "wasm-pkg")
	assert.Contains(t, path, "config.toml")
	
	// Should use home directory if available
	if homeDir, err := os.UserHomeDir(); err == nil {
		assert.Contains(t, path, homeDir)
	}
}