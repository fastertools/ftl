// Package config provides shared configuration types and utilities for FTL tools
package config

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"github.com/pelletier/go-toml/v2"
	"github.com/pkg/errors"
	"gopkg.in/yaml.v3"
)

// FTLConfig represents the main FTL project configuration
type FTLConfig struct {
	Name        string            `yaml:"name" toml:"name" json:"name"`
	Version     string            `yaml:"version" toml:"version" json:"version"`
	Description string            `yaml:"description,omitempty" toml:"description,omitempty" json:"description,omitempty"`
	Authors     []string          `yaml:"authors,omitempty" toml:"authors,omitempty" json:"authors,omitempty"`
	Deploy      *DeployConfig     `yaml:"deploy,omitempty" toml:"deploy,omitempty" json:"deploy,omitempty"`
	Compose     string            `yaml:"compose,omitempty" toml:"compose,omitempty" json:"compose,omitempty"`
	Variables   map[string]string `yaml:"variables,omitempty" toml:"variables,omitempty" json:"variables,omitempty"`
}

// DeployConfig represents deployment configuration
type DeployConfig struct {
	Environment string            `yaml:"environment" toml:"environment" json:"environment"`
	Region      string            `yaml:"region,omitempty" toml:"region,omitempty" json:"region,omitempty"`
	Endpoint    string            `yaml:"endpoint,omitempty" toml:"endpoint,omitempty" json:"endpoint,omitempty"`
	Auth        *AuthConfig       `yaml:"auth,omitempty" toml:"auth,omitempty" json:"auth,omitempty"`
	Settings    map[string]string `yaml:"settings,omitempty" toml:"settings,omitempty" json:"settings,omitempty"`
}

// AuthConfig represents authentication configuration
type AuthConfig struct {
	Type     string `yaml:"type" toml:"type" json:"type"` // "token", "oauth", "none"
	Token    string `yaml:"token,omitempty" toml:"token,omitempty" json:"token,omitempty"`
	ClientID string `yaml:"client_id,omitempty" toml:"client_id,omitempty" json:"client_id,omitempty"`
	Issuer   string `yaml:"issuer,omitempty" toml:"issuer,omitempty" json:"issuer,omitempty"`
}

// SpinConfig represents a spin.toml configuration (simplified)
type SpinConfig struct {
	Application ApplicationConfig         `toml:"application"`
	Variables   map[string]Variable       `toml:"variables,omitempty"`
	Components  map[string]ComponentConfig `toml:"component,omitempty"`
}

// ApplicationConfig represents Spin application metadata
type ApplicationConfig struct {
	Name        string   `toml:"name"`
	Version     string   `toml:"version,omitempty"`
	Description string   `toml:"description,omitempty"`
	Authors     []string `toml:"authors,omitempty"`
}

// Variable represents a Spin variable
type Variable struct {
	Default  string `toml:"default,omitempty"`
	Required bool   `toml:"required,omitempty"`
}

// ComponentConfig represents a Spin component
type ComponentConfig struct {
	Source               string            `toml:"source"`
	AllowedOutboundHosts []string          `toml:"allowed_outbound_hosts,omitempty"`
	Variables            map[string]string `toml:"variables,omitempty"`
	Environment          map[string]string `toml:"environment,omitempty"`
}

// Load loads an FTL configuration from a file
func Load(path string) (*FTLConfig, error) {
	if path == "" {
		path = findConfigFile()
	}

	data, err := os.ReadFile(path)
	if err != nil {
		return nil, errors.Wrapf(err, "failed to read config file %s", path)
	}

	config := &FTLConfig{}
	ext := strings.ToLower(filepath.Ext(path))

	switch ext {
	case ".yaml", ".yml":
		if err := yaml.Unmarshal(data, config); err != nil {
			return nil, errors.Wrap(err, "failed to parse YAML")
		}
	case ".toml":
		if err := toml.Unmarshal(data, config); err != nil {
			return nil, errors.Wrap(err, "failed to parse TOML")
		}
	default:
		return nil, fmt.Errorf("unsupported config format: %s", ext)
	}

	if err := config.Validate(); err != nil {
		return nil, errors.Wrap(err, "config validation failed")
	}

	return config, nil
}

// Save saves the configuration to a file
func (c *FTLConfig) Save(path string) error {
	if err := c.Validate(); err != nil {
		return errors.Wrap(err, "config validation failed")
	}

	ext := strings.ToLower(filepath.Ext(path))
	var data []byte
	var err error

	switch ext {
	case ".yaml", ".yml":
		data, err = yaml.Marshal(c)
	case ".toml":
		data, err = toml.Marshal(c)
	default:
		return fmt.Errorf("unsupported config format: %s", ext)
	}

	if err != nil {
		return errors.Wrap(err, "failed to marshal config")
	}

	return os.WriteFile(path, data, 0644)
}

// Validate validates the configuration
func (c *FTLConfig) Validate() error {
	if c.Name == "" {
		return errors.New("name is required")
	}

	if !isValidName(c.Name) {
		return fmt.Errorf("invalid name: %s (must be alphanumeric with hyphens)", c.Name)
	}

	if c.Version == "" {
		c.Version = "0.1.0"
	}

	if c.Deploy != nil {
		if err := c.Deploy.Validate(); err != nil {
			return errors.Wrap(err, "deploy config validation failed")
		}
	}

	return nil
}

// Validate validates deployment configuration
func (d *DeployConfig) Validate() error {
	if d.Environment == "" {
		d.Environment = "development"
	}

	validEnvs := []string{"development", "staging", "production"}
	if !contains(validEnvs, d.Environment) {
		return fmt.Errorf("invalid environment: %s (must be one of: %v)", d.Environment, validEnvs)
	}

	if d.Auth != nil {
		if err := d.Auth.Validate(); err != nil {
			return errors.Wrap(err, "auth config validation failed")
		}
	}

	return nil
}

// Validate validates authentication configuration
func (a *AuthConfig) Validate() error {
	validTypes := []string{"token", "oauth", "none"}
	if !contains(validTypes, a.Type) {
		return fmt.Errorf("invalid auth type: %s (must be one of: %v)", a.Type, validTypes)
	}

	switch a.Type {
	case "token":
		if a.Token == "" {
			return errors.New("token is required for token auth")
		}
	case "oauth":
		if a.ClientID == "" {
			return errors.New("client_id is required for oauth")
		}
		if a.Issuer == "" {
			return errors.New("issuer is required for oauth")
		}
	}

	return nil
}

// findConfigFile searches for configuration files in the current directory
func findConfigFile() string {
	candidates := []string{"ftl.yaml", "ftl.toml", "ftl.yml"}
	for _, candidate := range candidates {
		if _, err := os.Stat(candidate); err == nil {
			return candidate
		}
	}
	return "ftl.yaml" // default
}

// isValidName checks if a name is valid (alphanumeric with hyphens)
func isValidName(name string) bool {
	if len(name) == 0 || len(name) > 63 {
		return false
	}
	for i, ch := range name {
		if !((ch >= 'a' && ch <= 'z') || (ch >= '0' && ch <= '9') || ch == '-') {
			return false
		}
		if ch == '-' && (i == 0 || i == len(name)-1) {
			return false
		}
	}
	return true
}

// contains checks if a slice contains a value
func contains(slice []string, value string) bool {
	for _, s := range slice {
		if s == value {
			return true
		}
	}
	return false
}