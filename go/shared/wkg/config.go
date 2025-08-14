package wkg

import (
	"encoding/base64"
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"gopkg.in/yaml.v3"
)

// Config represents wkg configuration
type Config struct {
	DefaultRegistry      string                       `yaml:"default_registry,omitempty"`
	NamespaceRegistries  map[string]string            `yaml:"namespace_registries,omitempty"`
	PackageRegistryOverrides map[string]string       `yaml:"package_registry_overrides,omitempty"`
	Registry             map[string]*RegistryConfig  `yaml:"registry,omitempty"`
}

// RegistryConfig represents configuration for a specific registry
type RegistryConfig struct {
	Default string                 `yaml:"default,omitempty"`
	OCI     *OCIConfig             `yaml:"oci,omitempty"`
	Backend map[string]interface{} `yaml:",inline"`
}

// OCIConfig represents OCI-specific registry configuration
type OCIConfig struct {
	Protocol string    `yaml:"protocol,omitempty"`
	Auth     *OCIAuth  `yaml:"auth,omitempty"`
}

// OCIAuth represents OCI authentication configuration
type OCIAuth struct {
	Username string `yaml:"username,omitempty"`
	Password string `yaml:"password,omitempty"`
}

// Manager manages wkg configuration
type Manager struct {
	configPath string
}

// NewManager creates a new wkg config manager
func NewManager() *Manager {
	return &Manager{
		configPath: getConfigPath(),
	}
}

// NewManagerWithPath creates a new wkg config manager with custom path
func NewManagerWithPath(path string) *Manager {
	return &Manager{
		configPath: path,
	}
}

// getConfigPath returns the default wkg config path
func getConfigPath() string {
	// Check if WKG_CONFIG_FILE is set
	if path := os.Getenv("WKG_CONFIG_FILE"); path != "" {
		return path
	}

	// Default to ~/.config/wasm-pkg/config.toml
	home, err := os.UserHomeDir()
	if err != nil {
		// Fallback to current directory
		return "wkg-config.toml"
	}

	return filepath.Join(home, ".config", "wasm-pkg", "config.toml")
}

// Load reads the wkg configuration from disk
func (m *Manager) Load() (*Config, error) {
	data, err := os.ReadFile(m.configPath)
	if err != nil {
		if os.IsNotExist(err) {
			// Return empty config if file doesn't exist
			return &Config{
				Registry: make(map[string]*RegistryConfig),
			}, nil
		}
		return nil, fmt.Errorf("failed to read config: %w", err)
	}

	var config Config
	if err := yaml.Unmarshal(data, &config); err != nil {
		return nil, fmt.Errorf("failed to parse config: %w", err)
	}

	// Initialize maps if nil
	if config.Registry == nil {
		config.Registry = make(map[string]*RegistryConfig)
	}

	return &config, nil
}

// Save writes the wkg configuration to disk
func (m *Manager) Save(config *Config) error {
	// Ensure directory exists
	dir := filepath.Dir(m.configPath)
	if err := os.MkdirAll(dir, 0755); err != nil {
		return fmt.Errorf("failed to create config directory: %w", err)
	}

	// Marshal to YAML
	data, err := yaml.Marshal(config)
	if err != nil {
		return fmt.Errorf("failed to marshal config: %w", err)
	}

	// Write to file
	if err := os.WriteFile(m.configPath, data, 0644); err != nil {
		return fmt.Errorf("failed to write config: %w", err)
	}

	return nil
}

// ConfigureECRAuth adds ECR authentication to the wkg config
func (m *Manager) ConfigureECRAuth(registryURI, authToken string) error {
	// Load existing config
	config, err := m.Load()
	if err != nil {
		return fmt.Errorf("failed to load config: %w", err)
	}

	// Parse registry hostname from URI
	// Format: 123456789.dkr.ecr.us-west-2.amazonaws.com
	registry := registryURI
	if strings.HasPrefix(registry, "https://") {
		registry = strings.TrimPrefix(registry, "https://")
	}
	if strings.HasPrefix(registry, "http://") {
		registry = strings.TrimPrefix(registry, "http://")
	}
	// Remove any trailing path
	if idx := strings.Index(registry, "/"); idx > 0 {
		registry = registry[:idx]
	}

	// Decode the auth token (base64 encoded "AWS:password")
	decoded, err := base64.StdEncoding.DecodeString(authToken)
	if err != nil {
		return fmt.Errorf("failed to decode auth token: %w", err)
	}

	// Extract password from "AWS:password"
	authString := string(decoded)
	if !strings.HasPrefix(authString, "AWS:") {
		return fmt.Errorf("invalid ECR auth token format")
	}
	password := strings.TrimPrefix(authString, "AWS:")

	// Create or update registry config
	if config.Registry[registry] == nil {
		config.Registry[registry] = &RegistryConfig{}
	}

	config.Registry[registry].Default = "oci"
	config.Registry[registry].OCI = &OCIConfig{
		Protocol: "https",
		Auth: &OCIAuth{
			Username: "AWS",
			Password: password,
		},
	}

	// Save updated config
	if err := m.Save(config); err != nil {
		return fmt.Errorf("failed to save config: %w", err)
	}

	return nil
}

// RemoveRegistry removes a registry from the configuration
func (m *Manager) RemoveRegistry(registry string) error {
	config, err := m.Load()
	if err != nil {
		return fmt.Errorf("failed to load config: %w", err)
	}

	delete(config.Registry, registry)

	if err := m.Save(config); err != nil {
		return fmt.Errorf("failed to save config: %w", err)
	}

	return nil
}

// ClearECRAuth removes all ECR registries from the config
func (m *Manager) ClearECRAuth() error {
	config, err := m.Load()
	if err != nil {
		return fmt.Errorf("failed to load config: %w", err)
	}

	// Remove all ECR registries (they contain "ecr" and "amazonaws.com")
	for registry := range config.Registry {
		if strings.Contains(registry, "ecr") && strings.Contains(registry, "amazonaws.com") {
			delete(config.Registry, registry)
		}
	}

	if err := m.Save(config); err != nil {
		return fmt.Errorf("failed to save config: %w", err)
	}

	return nil
}