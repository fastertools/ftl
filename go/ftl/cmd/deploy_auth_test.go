package cmd

import (
	"os"
	"path/filepath"
	"testing"

	"github.com/pelletier/go-toml/v2"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/fastertools/ftl-cli/go/shared/ftl"
)

func TestConfigureWkgAuth(t *testing.T) {
	// Create a temporary directory for the test
	tmpDir := t.TempDir()

	// Override the config home for testing
	oldXDG := os.Getenv("XDG_CONFIG_HOME")
	defer func() {
		if oldXDG != "" {
			os.Setenv("XDG_CONFIG_HOME", oldXDG)
		} else {
			os.Unsetenv("XDG_CONFIG_HOME")
		}
	}()
	os.Setenv("XDG_CONFIG_HOME", tmpDir)

	// Test ECR auth
	ecrAuth := &ftl.ECRAuth{
		Registry: "123456789.dkr.ecr.us-west-2.amazonaws.com",
		Username: "AWS",
		Password: "test-password-123",
	}

	// Configure wkg auth
	err := configureWkgAuth(ecrAuth)
	require.NoError(t, err)

	// Verify the config file was created
	configPath := filepath.Join(tmpDir, "wasm-pkg", "config.toml")
	assert.FileExists(t, configPath)

	// Read and verify the config content
	data, err := os.ReadFile(configPath)
	require.NoError(t, err)

	var config map[string]interface{}
	err = toml.Unmarshal(data, &config)
	require.NoError(t, err)

	// Check registry configuration (we should NOT have set default_registry)
	registry, ok := config["registry"].(map[string]interface{})
	require.True(t, ok, "registry should be a map")

	ecrConfig, ok := registry[ecrAuth.Registry].(map[string]interface{})
	require.True(t, ok, "ECR registry config should exist")

	ociConfig, ok := ecrConfig["oci"].(map[string]interface{})
	require.True(t, ok, "oci config should exist")

	authConfig, ok := ociConfig["auth"].(map[string]interface{})
	require.True(t, ok, "auth config should exist")

	assert.Equal(t, "AWS", authConfig["username"])
	assert.Equal(t, "test-password-123", authConfig["password"])

	// Test updating existing config
	ecrAuth2 := &ftl.ECRAuth{
		Registry: "987654321.dkr.ecr.eu-west-1.amazonaws.com",
		Username: "AWS",
		Password: "another-password-456",
	}

	err = configureWkgAuth(ecrAuth2)
	require.NoError(t, err)

	// Read updated config
	data, err = os.ReadFile(configPath)
	require.NoError(t, err)

	err = toml.Unmarshal(data, &config)
	require.NoError(t, err)

	registry = config["registry"].(map[string]interface{})

	// Check both registries exist
	assert.Contains(t, registry, ecrAuth.Registry)
	assert.Contains(t, registry, ecrAuth2.Registry)

	// Verify the first registry auth is still there
	ecrConfig1 := registry[ecrAuth.Registry].(map[string]interface{})
	auth1 := ecrConfig1["oci"].(map[string]interface{})["auth"].(map[string]interface{})
	assert.Equal(t, "test-password-123", auth1["password"])

	// Verify the second registry auth
	ecrConfig2 := registry[ecrAuth2.Registry].(map[string]interface{})
	auth2 := ecrConfig2["oci"].(map[string]interface{})["auth"].(map[string]interface{})
	assert.Equal(t, "another-password-456", auth2["password"])
}

func TestConfigureWkgAuth_PreservesExistingConfig(t *testing.T) {
	// Create a temporary directory for the test
	tmpDir := t.TempDir()

	// Override the config home for testing
	oldXDG := os.Getenv("XDG_CONFIG_HOME")
	defer func() {
		if oldXDG != "" {
			os.Setenv("XDG_CONFIG_HOME", oldXDG)
		} else {
			os.Unsetenv("XDG_CONFIG_HOME")
		}
	}()
	os.Setenv("XDG_CONFIG_HOME", tmpDir)

	// Create existing config with user's settings
	configDir := filepath.Join(tmpDir, "wasm-pkg")
	configPath := filepath.Join(configDir, "config.toml")
	require.NoError(t, os.MkdirAll(configDir, 0755))

	existingConfig := `default_registry = "my-custom-registry.io"
namespace_prefix = "my-org"

[registry."my-custom-registry.io"]
url = "https://my-custom-registry.io"

[registry."my-custom-registry.io".oci]
protocol = "oci"

[registry."my-custom-registry.io".oci.auth]
username = "myuser"
password = "mypass"
`
	require.NoError(t, os.WriteFile(configPath, []byte(existingConfig), 0600))

	// Configure ECR auth
	ecrAuth := &ftl.ECRAuth{
		Registry: "123456789.dkr.ecr.us-west-2.amazonaws.com",
		Username: "AWS",
		Password: "test-password-123",
	}

	err := configureWkgAuth(ecrAuth)
	require.NoError(t, err)

	// Read and verify the config
	data, err := os.ReadFile(configPath)
	require.NoError(t, err)

	var config map[string]interface{}
	err = toml.Unmarshal(data, &config)
	require.NoError(t, err)

	// Verify existing settings are preserved
	assert.Equal(t, "my-custom-registry.io", config["default_registry"], "default_registry should be unchanged")
	assert.Equal(t, "my-org", config["namespace_prefix"], "namespace_prefix should be unchanged")

	// Verify existing registry is preserved
	registry := config["registry"].(map[string]interface{})
	assert.Contains(t, registry, "my-custom-registry.io", "existing registry should be preserved")

	existingReg := registry["my-custom-registry.io"].(map[string]interface{})
	existingAuth := existingReg["oci"].(map[string]interface{})["auth"].(map[string]interface{})
	assert.Equal(t, "myuser", existingAuth["username"], "existing registry auth should be unchanged")
	assert.Equal(t, "mypass", existingAuth["password"], "existing registry auth should be unchanged")

	// Verify ECR registry was added
	assert.Contains(t, registry, ecrAuth.Registry, "ECR registry should be added")
	ecrConfig := registry[ecrAuth.Registry].(map[string]interface{})
	ecrAuthConfig := ecrConfig["oci"].(map[string]interface{})["auth"].(map[string]interface{})
	assert.Equal(t, "AWS", ecrAuthConfig["username"])
	assert.Equal(t, "test-password-123", ecrAuthConfig["password"])
}

func TestConfigureMultiPlatformAuth(t *testing.T) {
	// Skip this test as it requires docker to be available and would make actual docker login calls
	t.Skip("Skipping integration test that requires docker")
}

func TestGetConfigHome(t *testing.T) {
	tests := []struct {
		name        string
		xdgHome     string
		home        string
		userProfile string
		expected    string
	}{
		{
			name:     "XDG_CONFIG_HOME set",
			xdgHome:  "/custom/config",
			home:     "/home/user",
			expected: "/custom/config",
		},
		{
			name:     "HOME set",
			xdgHome:  "",
			home:     "/home/user",
			expected: "/home/user/.config",
		},
		{
			name:        "Windows USERPROFILE",
			xdgHome:     "",
			home:        "",
			userProfile: "C:\\Users\\user",
			expected:    filepath.Join("C:\\Users\\user", ".config"),
		},
		{
			name:        "Fallback",
			xdgHome:     "",
			home:        "",
			userProfile: "",
			expected:    ".config",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Save and restore environment
			oldXDG := os.Getenv("XDG_CONFIG_HOME")
			oldHome := os.Getenv("HOME")
			oldUserProfile := os.Getenv("USERPROFILE")
			defer func() {
				os.Setenv("XDG_CONFIG_HOME", oldXDG)
				os.Setenv("HOME", oldHome)
				os.Setenv("USERPROFILE", oldUserProfile)
			}()

			// Set test environment
			if tt.xdgHome != "" {
				os.Setenv("XDG_CONFIG_HOME", tt.xdgHome)
			} else {
				os.Unsetenv("XDG_CONFIG_HOME")
			}
			if tt.home != "" {
				os.Setenv("HOME", tt.home)
			} else {
				os.Unsetenv("HOME")
			}
			if tt.userProfile != "" {
				os.Setenv("USERPROFILE", tt.userProfile)
			} else {
				os.Unsetenv("USERPROFILE")
			}

			result := getConfigHome()
			assert.Equal(t, tt.expected, result)
		})
	}
}
