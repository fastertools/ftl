package ftl

import (
	"encoding/base64"
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"github.com/pelletier/go-toml/v2"
)

// ECRAuth represents parsed ECR authentication credentials
type ECRAuth struct {
	Registry string
	Username string
	Password string
}

// ParseECRToken decodes an ECR authorization token into usable credentials
func ParseECRToken(registryURI string, authToken string) (*ECRAuth, error) {
	// Decode the base64 authorization token
	decoded, err := base64.StdEncoding.DecodeString(authToken)
	if err != nil {
		return nil, fmt.Errorf("failed to decode ECR token: %w", err)
	}

	// Extract username and password (format is "AWS:password")
	parts := strings.SplitN(string(decoded), ":", 2)
	if len(parts) != 2 || parts[0] != "AWS" {
		return nil, fmt.Errorf("invalid ECR token format")
	}

	// Clean up registry URI (remove protocol if present)
	registry := strings.TrimPrefix(registryURI, "https://")
	registry = strings.TrimPrefix(registry, "http://")

	return &ECRAuth{
		Registry: registry,
		Username: parts[0],
		Password: parts[1],
	}, nil
}

// WkgConfig represents the wasm-pkg config structure
type WkgConfig struct {
	DefaultRegistry          string                        `toml:"default_registry,omitempty"`
	PackageRegistryOverrides map[string]string            `toml:"package_registry_overrides,omitempty"`
	Registry                 map[string]*WkgRegistryConfig `toml:"registry,omitempty"`
}

// WkgRegistryConfig represents a registry configuration
type WkgRegistryConfig struct {
	OCI *WkgOCIConfig `toml:"oci,omitempty"`
}

// WkgOCIConfig represents OCI registry configuration
type WkgOCIConfig struct {
	Auth *WkgAuthConfig `toml:"auth,omitempty"`
}

// WkgAuthConfig represents authentication configuration
type WkgAuthConfig struct {
	Username string `toml:"username"`
	Password string `toml:"password"`
}

// WkgConfigManager manages wkg configuration for ECR authentication
type WkgConfigManager struct {
	configPath string
}

// NewWkgConfigManager creates a new wkg config manager
// If configPath is empty, it uses the default path ~/.config/wasm-pkg/config.toml
func NewWkgConfigManager(configPath string) *WkgConfigManager {
	if configPath == "" {
		homeDir, err := os.UserHomeDir()
		if err != nil {
			// Fallback to current directory
			configPath = ".config/wasm-pkg/config.toml"
		} else {
			configPath = filepath.Join(homeDir, ".config", "wasm-pkg", "config.toml")
		}
	}
	return &WkgConfigManager{
		configPath: configPath,
	}
}

// LoadConfig loads the wkg configuration file
func (m *WkgConfigManager) LoadConfig() (*WkgConfig, error) {
	// If config doesn't exist, return empty config
	if _, err := os.Stat(m.configPath); os.IsNotExist(err) {
		return &WkgConfig{
			DefaultRegistry: "ghcr.io",
			Registry:        make(map[string]*WkgRegistryConfig),
		}, nil
	}

	data, err := os.ReadFile(m.configPath)
	if err != nil {
		return nil, fmt.Errorf("failed to read wkg config: %w", err)
	}

	var config WkgConfig
	if err := toml.Unmarshal(data, &config); err != nil {
		return nil, fmt.Errorf("failed to parse wkg config: %w", err)
	}

	// Initialize maps if nil
	if config.Registry == nil {
		config.Registry = make(map[string]*WkgRegistryConfig)
	}

	return &config, nil
}

// SaveConfig saves the wkg configuration file
func (m *WkgConfigManager) SaveConfig(config *WkgConfig) error {
	// Ensure directory exists
	configDir := filepath.Dir(m.configPath)
	if err := os.MkdirAll(configDir, 0755); err != nil {
		return fmt.Errorf("failed to create config directory: %w", err)
	}

	data, err := toml.Marshal(config)
	if err != nil {
		return fmt.Errorf("failed to marshal wkg config: %w", err)
	}

	if err := os.WriteFile(m.configPath, data, 0644); err != nil {
		return fmt.Errorf("failed to write wkg config: %w", err)
	}

	return nil
}

// SetECRAuth updates the wkg config with ECR authentication
func (m *WkgConfigManager) SetECRAuth(ecrAuth *ECRAuth) error {
	// Load existing config
	config, err := m.LoadConfig()
	if err != nil {
		return fmt.Errorf("failed to load wkg config: %w", err)
	}

	// Update or create registry config
	if config.Registry[ecrAuth.Registry] == nil {
		config.Registry[ecrAuth.Registry] = &WkgRegistryConfig{}
	}
	if config.Registry[ecrAuth.Registry].OCI == nil {
		config.Registry[ecrAuth.Registry].OCI = &WkgOCIConfig{}
	}

	config.Registry[ecrAuth.Registry].OCI.Auth = &WkgAuthConfig{
		Username: ecrAuth.Username,
		Password: ecrAuth.Password,
	}

	// Save updated config
	if err := m.SaveConfig(config); err != nil {
		return fmt.Errorf("failed to save wkg config: %w", err)
	}

	return nil
}

// RemoveECRAuth removes ECR authentication from wkg config
func (m *WkgConfigManager) RemoveECRAuth(registryURI string) error {
	// Load existing config
	config, err := m.LoadConfig()
	if err != nil {
		return fmt.Errorf("failed to load wkg config: %w", err)
	}

	// Clean up registry URI
	registry := strings.TrimPrefix(registryURI, "https://")
	registry = strings.TrimPrefix(registry, "http://")

	// Remove auth for this registry
	if regConfig, exists := config.Registry[registry]; exists {
		if regConfig.OCI != nil {
			regConfig.OCI.Auth = nil
		}
	}

	// Save updated config
	if err := m.SaveConfig(config); err != nil {
		return fmt.Errorf("failed to save wkg config: %w", err)
	}

	return nil
}

// UpdateWkgAuthForECR is a convenience function that parses the ECR token and updates wkg config
// This is what both CLI and backend can use
func UpdateWkgAuthForECR(configPath string, registryURI string, authToken string) error {
	// Parse the ECR token
	ecrAuth, err := ParseECRToken(registryURI, authToken)
	if err != nil {
		return err
	}

	// Create config manager
	manager := NewWkgConfigManager(configPath)

	// Set the ECR auth
	return manager.SetECRAuth(ecrAuth)
}

