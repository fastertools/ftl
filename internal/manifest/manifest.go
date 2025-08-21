// Package manifest provides type-safe FTL manifest operations
package manifest

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"github.com/fastertools/ftl-cli/pkg/validation"
	"gopkg.in/yaml.v3"
)

// Manifest represents an FTL application manifest
type Manifest struct {
	Name        string                 `yaml:"name" json:"name"`
	Version     string                 `yaml:"version,omitempty" json:"version,omitempty"`
	Description string                 `yaml:"description,omitempty" json:"description,omitempty"`
	Access      string                 `yaml:"access,omitempty" json:"access,omitempty"`
	Auth        *validation.AuthConfig `yaml:"auth,omitempty" json:"auth,omitempty"`
	Components  []Component            `yaml:"components,omitempty" json:"components,omitempty"`
	Variables   map[string]string      `yaml:"variables,omitempty" json:"variables,omitempty"`
}

// Component represents a component in the manifest
type Component struct {
	ID        string            `yaml:"id" json:"id"`
	Source    interface{}       `yaml:"source" json:"source"` // Can be string or SourceRegistry
	Build     *BuildConfig      `yaml:"build,omitempty" json:"build,omitempty"`
	Variables map[string]string `yaml:"variables,omitempty" json:"variables,omitempty"`
}

// UnmarshalYAML implements custom YAML unmarshaling for Component
func (c *Component) UnmarshalYAML(unmarshal func(interface{}) error) error {
	// Use an auxiliary type to avoid recursion
	type componentAlias Component
	aux := (*componentAlias)(c)

	// First unmarshal into the auxiliary type
	if err := unmarshal(aux); err != nil {
		return err
	}

	// Now handle the Source field
	if c.Source != nil {
		switch src := c.Source.(type) {
		case string:
			// It's already a string, nothing to do
		case map[string]interface{}:
			// Convert map to SourceRegistry
			registry := SourceRegistry{}
			if v, ok := src["registry"].(string); ok {
				registry.Registry = v
			}
			if v, ok := src["package"].(string); ok {
				registry.Package = v
			}
			if v, ok := src["version"].(string); ok {
				registry.Version = v
			}
			c.Source = registry
		case map[interface{}]interface{}:
			// Handle map[interface{}]interface{} from YAML
			registry := SourceRegistry{}
			if v, ok := src["registry"].(string); ok {
				registry.Registry = v
			}
			if v, ok := src["package"].(string); ok {
				registry.Package = v
			}
			if v, ok := src["version"].(string); ok {
				registry.Version = v
			}
			c.Source = registry
		}
	}

	return nil
}

// MarshalYAML implements custom YAML marshaling for Component
func (c Component) MarshalYAML() (interface{}, error) {
	// Use an auxiliary type to avoid recursion
	type componentAlias Component
	aux := componentAlias(c)

	// Create a map for the result
	result := make(map[string]interface{})

	// Marshal the auxiliary type to get all fields
	data, err := yaml.Marshal(aux)
	if err != nil {
		return nil, err
	}

	// Unmarshal back to a map
	if err := yaml.Unmarshal(data, &result); err != nil {
		return nil, err
	}

	// Handle the Source field specially
	switch src := c.Source.(type) {
	case string:
		result["source"] = src
	case SourceRegistry:
		result["source"] = map[string]interface{}{
			"registry": src.Registry,
			"package":  src.Package,
			"version":  src.Version,
		}
	}

	return result, nil
}

// UnmarshalJSON implements custom JSON unmarshaling for Component
func (c *Component) UnmarshalJSON(data []byte) error {
	// Use an auxiliary type to avoid recursion
	type componentAlias Component
	aux := (*componentAlias)(c)

	// First unmarshal into the auxiliary type
	if err := json.Unmarshal(data, aux); err != nil {
		return err
	}

	// Now handle the Source field
	if c.Source != nil {
		switch src := c.Source.(type) {
		case string:
			// It's already a string, nothing to do
		case map[string]interface{}:
			// Convert map to SourceRegistry
			registry := SourceRegistry{}
			if v, ok := src["registry"].(string); ok {
				registry.Registry = v
			}
			if v, ok := src["package"].(string); ok {
				registry.Package = v
			}
			if v, ok := src["version"].(string); ok {
				registry.Version = v
			}
			c.Source = registry
		}
	}

	return nil
}

// MarshalJSON implements custom JSON marshaling for Component
func (c Component) MarshalJSON() ([]byte, error) {
	// Use an auxiliary type to avoid recursion
	type componentAlias Component

	// Create a new struct with the Source field handled properly
	aux := struct {
		componentAlias
		Source interface{} `json:"source"`
	}{
		componentAlias: componentAlias(c),
	}

	// Handle the Source field specially
	switch src := c.Source.(type) {
	case string:
		aux.Source = src
	case SourceRegistry:
		aux.Source = src
	default:
		aux.Source = c.Source
	}

	return json.Marshal(aux)
}

// SourceRegistry represents a registry source
type SourceRegistry struct {
	Registry string `yaml:"registry" json:"registry"`
	Package  string `yaml:"package" json:"package"`
	Version  string `yaml:"version" json:"version"`
}

// BuildConfig represents build configuration
type BuildConfig struct {
	Command string   `yaml:"command" json:"command"`
	Workdir string   `yaml:"workdir,omitempty" json:"workdir,omitempty"`
	Watch   []string `yaml:"watch,omitempty" json:"watch,omitempty"`
}

// Load reads and parses an FTL manifest file (supports both YAML and JSON)
func Load(path string) (*Manifest, error) {
	// Clean the path to prevent directory traversal
	path = filepath.Clean(path)

	// Check if file exists
	if _, err := os.Stat(path); os.IsNotExist(err) {
		// Return a default manifest if file doesn't exist
		return &Manifest{
			Name:       "app",
			Version:    "0.1.0",
			Components: []Component{},
			Access:     "public",
		}, nil
	}

	data, err := os.ReadFile(path)
	if err != nil {
		return nil, fmt.Errorf("failed to read manifest: %w", err)
	}

	var manifest Manifest

	// Determine format based on file extension
	if strings.HasSuffix(path, ".json") {
		if err := json.Unmarshal(data, &manifest); err != nil {
			return nil, fmt.Errorf("failed to parse JSON manifest: %w", err)
		}
	} else {
		// Default to YAML for .yaml, .yml, or no extension
		if err := yaml.Unmarshal(data, &manifest); err != nil {
			return nil, fmt.Errorf("failed to parse YAML manifest: %w", err)
		}
	}

	// Ensure components is initialized
	if manifest.Components == nil {
		manifest.Components = []Component{}
	}

	return &manifest, nil
}

// LoadAuto tries to load ftl.yaml or ftl.json automatically
func LoadAuto() (*Manifest, error) {
	// Try ftl.yaml first (preferred)
	if _, err := os.Stat("ftl.yaml"); err == nil {
		return Load("ftl.yaml")
	}

	// Try ftl.yml
	if _, err := os.Stat("ftl.yml"); err == nil {
		return Load("ftl.yml")
	}

	// Try ftl.json
	if _, err := os.Stat("ftl.json"); err == nil {
		return Load("ftl.json")
	}

	// Return default manifest if no files exist
	return &Manifest{
		Name:       "app",
		Version:    "0.1.0",
		Components: []Component{},
		Access:     "public",
	}, nil
}

// Save writes the manifest to a file (format determined by extension)
func (m *Manifest) Save(path string) error {
	var data []byte
	var err error

	// Determine format based on file extension
	if strings.HasSuffix(path, ".json") {
		data, err = json.MarshalIndent(m, "", "  ")
		if err != nil {
			return fmt.Errorf("failed to marshal manifest as JSON: %w", err)
		}
	} else {
		// Default to YAML for .yaml, .yml, or no extension
		data, err = yaml.Marshal(m)
		if err != nil {
			return fmt.Errorf("failed to marshal manifest as YAML: %w", err)
		}
	}

	return os.WriteFile(path, data, 0600)
}

// SaveAuto saves to the appropriate file (ftl.yaml or ftl.json)
func (m *Manifest) SaveAuto() error {
	// Check which file exists
	if _, err := os.Stat("ftl.yaml"); err == nil {
		return m.Save("ftl.yaml")
	}

	if _, err := os.Stat("ftl.yml"); err == nil {
		return m.Save("ftl.yml")
	}

	if _, err := os.Stat("ftl.json"); err == nil {
		return m.Save("ftl.json")
	}

	// Default to ftl.yaml if no file exists
	return m.Save("ftl.yaml")
}

// FindComponent finds a component by ID
func (m *Manifest) FindComponent(id string) (*Component, int) {
	for i, comp := range m.Components {
		if comp.ID == id {
			return &comp, i
		}
	}
	return nil, -1
}

// AddComponent adds a new component to the manifest
func (m *Manifest) AddComponent(comp Component) error {
	// Check if component already exists
	if existing, _ := m.FindComponent(comp.ID); existing != nil {
		return fmt.Errorf("component '%s' already exists", comp.ID)
	}

	m.Components = append(m.Components, comp)
	return nil
}

// RemoveComponent removes a component by ID
func (m *Manifest) RemoveComponent(id string) error {
	_, index := m.FindComponent(id)
	if index == -1 {
		return fmt.Errorf("component '%s' not found", id)
	}

	// Remove the component
	m.Components = append(m.Components[:index], m.Components[index+1:]...)
	return nil
}

// ParseRegistrySource parses a registry string into a SourceRegistry
// Format: registry/namespace:package@version
func ParseRegistrySource(registry string) (*SourceRegistry, error) {
	// Simple parsing for now - can be enhanced
	// Expected format: "ghcr.io/user:package:version" or similar
	return &SourceRegistry{
		Registry: registry,
		Package:  registry, // Will be properly parsed in component_add.go
		Version:  "latest",
	}, nil
}
