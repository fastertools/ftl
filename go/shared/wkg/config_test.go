package wkg

import (
	"os"
	"path/filepath"
	"strings"
	"testing"

	"gopkg.in/yaml.v3"
)

func TestManager_LoadSave(t *testing.T) {
	// Create temp directory for test
	tmpDir := t.TempDir()
	configPath := filepath.Join(tmpDir, "config.toml")

	manager := NewManagerWithPath(configPath)

	// Test loading non-existent file returns empty config
	config, err := manager.Load()
	if err != nil {
		t.Fatalf("Load() error = %v", err)
	}
	if config == nil || config.Registry == nil {
		t.Error("Load() should return initialized config")
	}

	// Test saving config
	testConfig := &Config{
		DefaultRegistry: "test.registry.com",
		Registry: map[string]*RegistryConfig{
			"example.com": {
				Default: "oci",
				OCI: &OCIConfig{
					Protocol: "https",
					Auth: &OCIAuth{
						Username: "user",
						Password: "pass",
					},
				},
			},
		},
	}

	if err := manager.Save(testConfig); err != nil {
		t.Fatalf("Save() error = %v", err)
	}

	// Test loading saved config
	loadedConfig, err := manager.Load()
	if err != nil {
		t.Fatalf("Load() after save error = %v", err)
	}

	if loadedConfig.DefaultRegistry != testConfig.DefaultRegistry {
		t.Errorf("DefaultRegistry = %v, want %v", 
			loadedConfig.DefaultRegistry, testConfig.DefaultRegistry)
	}

	if loadedConfig.Registry["example.com"] == nil {
		t.Error("Registry entry not loaded")
	}
}

func TestManager_ConfigureECRAuth(t *testing.T) {
	tmpDir := t.TempDir()
	configPath := filepath.Join(tmpDir, "config.toml")
	manager := NewManagerWithPath(configPath)

	// Test data - base64 encoded "AWS:secretpassword"
	authToken := "QVdTOnNlY3JldHBhc3N3b3Jk"
	registryURI := "123456789.dkr.ecr.us-west-2.amazonaws.com"

	// Configure ECR auth
	if err := manager.ConfigureECRAuth(registryURI, authToken); err != nil {
		t.Fatalf("ConfigureECRAuth() error = %v", err)
	}

	// Load and verify
	config, err := manager.Load()
	if err != nil {
		t.Fatalf("Load() error = %v", err)
	}

	regConfig := config.Registry[registryURI]
	if regConfig == nil {
		t.Fatal("Registry config not found")
	}

	if regConfig.Default != "oci" {
		t.Errorf("Default = %v, want oci", regConfig.Default)
	}

	if regConfig.OCI == nil {
		t.Fatal("OCI config not set")
	}

	if regConfig.OCI.Protocol != "https" {
		t.Errorf("Protocol = %v, want https", regConfig.OCI.Protocol)
	}

	if regConfig.OCI.Auth == nil {
		t.Fatal("Auth not set")
	}

	if regConfig.OCI.Auth.Username != "AWS" {
		t.Errorf("Username = %v, want AWS", regConfig.OCI.Auth.Username)
	}

	if regConfig.OCI.Auth.Password != "secretpassword" {
		t.Errorf("Password = %v, want secretpassword", regConfig.OCI.Auth.Password)
	}
}

func TestManager_ConfigureECRAuth_WithHTTPSPrefix(t *testing.T) {
	tmpDir := t.TempDir()
	configPath := filepath.Join(tmpDir, "config.toml")
	manager := NewManagerWithPath(configPath)

	authToken := "QVdTOnNlY3JldHBhc3N3b3Jk"
	registryURI := "https://123456789.dkr.ecr.us-west-2.amazonaws.com"

	if err := manager.ConfigureECRAuth(registryURI, authToken); err != nil {
		t.Fatalf("ConfigureECRAuth() error = %v", err)
	}

	config, err := manager.Load()
	if err != nil {
		t.Fatalf("Load() error = %v", err)
	}

	// Should strip https:// prefix
	expectedRegistry := "123456789.dkr.ecr.us-west-2.amazonaws.com"
	if config.Registry[expectedRegistry] == nil {
		t.Error("Registry config not found (https:// should be stripped)")
	}
}

func TestManager_ClearECRAuth(t *testing.T) {
	tmpDir := t.TempDir()
	configPath := filepath.Join(tmpDir, "config.toml")
	manager := NewManagerWithPath(configPath)

	// Set up config with ECR and non-ECR registries
	config := &Config{
		Registry: map[string]*RegistryConfig{
			"123456789.dkr.ecr.us-west-2.amazonaws.com": {
				Default: "oci",
			},
			"987654321.dkr.ecr.eu-west-1.amazonaws.com": {
				Default: "oci",
			},
			"regular.registry.com": {
				Default: "oci",
			},
		},
	}

	if err := manager.Save(config); err != nil {
		t.Fatalf("Save() error = %v", err)
	}

	// Clear ECR auth
	if err := manager.ClearECRAuth(); err != nil {
		t.Fatalf("ClearECRAuth() error = %v", err)
	}

	// Load and verify
	loadedConfig, err := manager.Load()
	if err != nil {
		t.Fatalf("Load() error = %v", err)
	}

	// ECR registries should be removed
	for registry := range loadedConfig.Registry {
		if strings.Contains(registry, "ecr") && strings.Contains(registry, "amazonaws.com") {
			t.Errorf("ECR registry %s should have been removed", registry)
		}
	}

	// Non-ECR registry should remain
	if loadedConfig.Registry["regular.registry.com"] == nil {
		t.Error("Non-ECR registry should not be removed")
	}
}

func TestGetConfigPath(t *testing.T) {
	// Test with env var
	testPath := "/custom/path/config.toml"
	os.Setenv("WKG_CONFIG_FILE", testPath)
	defer os.Unsetenv("WKG_CONFIG_FILE")

	path := getConfigPath()
	if path != testPath {
		t.Errorf("getConfigPath() = %v, want %v", path, testPath)
	}

	// Test without env var (should use home dir)
	os.Unsetenv("WKG_CONFIG_FILE")
	path = getConfigPath()
	
	home, _ := os.UserHomeDir()
	expected := filepath.Join(home, ".config", "wasm-pkg", "config.toml")
	if path != expected {
		t.Errorf("getConfigPath() = %v, want %v", path, expected)
	}
}

func TestConfig_YAMLFormat(t *testing.T) {
	// Test that our config produces correct YAML format
	config := &Config{
		DefaultRegistry: "test.registry.com",
		Registry: map[string]*RegistryConfig{
			"ecr.amazonaws.com": {
				Default: "oci",
				OCI: &OCIConfig{
					Protocol: "https",
					Auth: &OCIAuth{
						Username: "AWS",
						Password: "secret",
					},
				},
			},
		},
	}

	data, err := yaml.Marshal(config)
	if err != nil {
		t.Fatalf("yaml.Marshal() error = %v", err)
	}

	yamlStr := string(data)

	// Check key sections exist
	if !strings.Contains(yamlStr, "default_registry: test.registry.com") {
		t.Error("YAML missing default_registry")
	}
	if !strings.Contains(yamlStr, "registry:") {
		t.Error("YAML missing registry section")
	}
	if !strings.Contains(yamlStr, "ecr.amazonaws.com:") {
		t.Error("YAML missing registry entry")
	}
	if !strings.Contains(yamlStr, "default: oci") {
		t.Error("YAML missing default backend")
	}
	if !strings.Contains(yamlStr, "oci:") {
		t.Error("YAML missing oci section")
	}
	if !strings.Contains(yamlStr, "protocol: https") {
		t.Error("YAML missing protocol")
	}
	if !strings.Contains(yamlStr, "auth:") {
		t.Error("YAML missing auth section")
	}
	if !strings.Contains(yamlStr, "username: AWS") {
		t.Error("YAML missing username")
	}
	if !strings.Contains(yamlStr, "password: secret") {
		t.Error("YAML missing password")
	}
}