// Package ftl provides shared types and utilities for FTL applications
// This package is used by both the CLI and the platform backend to ensure consistency
package ftl

import (
	"encoding/json"
	"fmt"

	"gopkg.in/yaml.v3"
)

// Application represents the FTL application configuration
// This is the canonical schema that both CLI and platform use
type Application struct {
	Name        string      `json:"name" yaml:"name" cue:"name!"`
	Version     string      `json:"version,omitempty" yaml:"version,omitempty" cue:"version"`
	Description string      `json:"description,omitempty" yaml:"description,omitempty" cue:"description"`
	Components  []Component `json:"components,omitempty" yaml:"components,omitempty" cue:"components"`
	Access      AccessMode  `json:"access,omitempty" yaml:"access,omitempty" cue:"access"`
	Auth        AuthConfig  `json:"auth,omitempty" yaml:"auth,omitempty" cue:"auth"`
}

// Component represents a component in the FTL application
type Component struct {
	ID        string            `json:"id" yaml:"id" cue:"id!"`
	Source    ComponentSource   `json:"source" yaml:"source" cue:"source!"`
	Build     *BuildConfig      `json:"build,omitempty" yaml:"build,omitempty" cue:"build"`
	Variables map[string]string `json:"variables,omitempty" yaml:"variables,omitempty" cue:"variables?"`
}

// ComponentSource can be either a local path (string) or a registry reference
type ComponentSource interface {
	IsLocal() bool
	GetPath() string
	GetRegistry() *RegistrySource
}

// LocalSource represents a local file path
type LocalSource string

func (l LocalSource) IsLocal() bool                { return true }
func (l LocalSource) GetPath() string              { return string(l) }
func (l LocalSource) GetRegistry() *RegistrySource { return nil }

// RegistrySource represents a component from a registry
type RegistrySource struct {
	Registry string `json:"registry" yaml:"registry" cue:"registry!"`
	Package  string `json:"package" yaml:"package" cue:"package!"`
	Version  string `json:"version" yaml:"version" cue:"version!"`
}

func (r *RegistrySource) IsLocal() bool                { return false }
func (r *RegistrySource) GetPath() string              { return "" }
func (r *RegistrySource) GetRegistry() *RegistrySource { return r }

// BuildConfig defines build configuration for a component
type BuildConfig struct {
	Command string   `json:"command" yaml:"command" cue:"command!"`
	Workdir string   `json:"workdir,omitempty" yaml:"workdir,omitempty" cue:"workdir?"`
	Watch   []string `json:"watch,omitempty" yaml:"watch,omitempty" cue:"watch?"`
}

// AccessMode defines the access control mode
type AccessMode string

const (
	AccessPublic  AccessMode = "public"
	AccessPrivate AccessMode = "private"
	AccessOrg     AccessMode = "org"
	AccessCustom  AccessMode = "custom"
)

// AuthConfig defines authentication configuration
type AuthConfig struct {
	Provider    AuthProvider `json:"provider" yaml:"provider" cue:"provider!"`
	OrgID       string       `json:"org_id,omitempty" yaml:"org_id,omitempty" cue:"org_id"`
	JWTIssuer   string       `json:"jwt_issuer,omitempty" yaml:"jwt_issuer,omitempty" cue:"jwt_issuer"`
	JWTAudience string       `json:"jwt_audience,omitempty" yaml:"jwt_audience,omitempty" cue:"jwt_audience"`
}

// AuthProvider defines the authentication provider type
type AuthProvider string

const (
	AuthProviderWorkOS AuthProvider = "workos"
	AuthProviderCustom AuthProvider = "custom"
)

// SetDefaults sets default values for the application
func (a *Application) SetDefaults() {
	if a.Version == "" {
		a.Version = "0.1.0"
	}
	if a.Access == "" {
		a.Access = AccessPublic
	}
	if a.Auth.Provider == "" {
		a.Auth.Provider = AuthProviderWorkOS
	}
	if a.Auth.Provider == AuthProviderWorkOS && a.Auth.JWTIssuer == "" {
		a.Auth.JWTIssuer = "https://api.workos.com"
	}
}

// Validate validates the application configuration
func (a *Application) Validate() error {
	if a.Name == "" {
		return fmt.Errorf("application name is required")
	}
	
	// Validate name format
	if !isValidName(a.Name) {
		return fmt.Errorf("invalid application name: must be lowercase alphanumeric with hyphens")
	}
	
	// Validate components
	for _, comp := range a.Components {
		if err := comp.Validate(); err != nil {
			return fmt.Errorf("invalid component %s: %w", comp.ID, err)
		}
	}
	
	// Validate auth configuration
	if a.Access == AccessCustom {
		if a.Auth.Provider != AuthProviderCustom {
			return fmt.Errorf("custom access requires custom auth provider")
		}
		if a.Auth.JWTIssuer == "" {
			return fmt.Errorf("JWT issuer is required for custom auth")
		}
		if a.Auth.JWTAudience == "" {
			return fmt.Errorf("JWT audience is required for custom auth")
		}
	}
	
	return nil
}

// Validate validates a component
func (c *Component) Validate() error {
	if c.ID == "" {
		return fmt.Errorf("component ID is required")
	}
	
	if !isValidName(c.ID) {
		return fmt.Errorf("invalid component ID: must be lowercase alphanumeric with hyphens")
	}
	
	if c.Source == nil {
		return fmt.Errorf("component source is required")
	}
	
	return nil
}

// isValidName checks if a name is valid (lowercase alphanumeric with hyphens)
func isValidName(name string) bool {
	if len(name) == 0 {
		return false
	}
	
	for i, ch := range name {
		if i == 0 && (ch < 'a' || ch > 'z') {
			return false // Must start with lowercase letter
		}
		if !((ch >= 'a' && ch <= 'z') || (ch >= '0' && ch <= '9') || ch == '-') {
			return false
		}
	}
	
	return true
}

// UnmarshalJSON implements custom JSON unmarshalling for Component
func (c *Component) UnmarshalJSON(data []byte) error {
	type Alias Component
	aux := &struct {
		Source json.RawMessage `json:"source"`
		*Alias
	}{
		Alias: (*Alias)(c),
	}
	
	if err := json.Unmarshal(data, &aux); err != nil {
		return err
	}
	
	// Try to unmarshal as string first (local source)
	var localPath string
	if err := json.Unmarshal(aux.Source, &localPath); err == nil {
		c.Source = LocalSource(localPath)
		return nil
	}
	
	// Try to unmarshal as registry source
	var regSource RegistrySource
	if err := json.Unmarshal(aux.Source, &regSource); err == nil {
		c.Source = &regSource
		return nil
	}
	
	return fmt.Errorf("invalid component source format")
}

// MarshalJSON implements custom JSON marshalling for Component
func (c Component) MarshalJSON() ([]byte, error) {
	type Alias Component
	
	var source interface{}
	if c.Source != nil {
		if c.Source.IsLocal() {
			source = c.Source.GetPath()
		} else {
			source = c.Source.GetRegistry()
		}
	}
	
	return json.Marshal(&struct {
		Source interface{} `json:"source"`
		*Alias
	}{
		Source: source,
		Alias:  (*Alias)(&c),
	})
}

// UnmarshalYAML implements custom YAML unmarshalling for Component
func (c *Component) UnmarshalYAML(value *yaml.Node) error {
	type Alias Component
	aux := &struct {
		Source yaml.Node `yaml:"source"`
		*Alias
	}{
		Alias: (*Alias)(c),
	}
	
	if err := value.Decode(&aux); err != nil {
		return err
	}
	
	// Try to decode as string first (local source)
	var localPath string
	if err := aux.Source.Decode(&localPath); err == nil {
		c.Source = LocalSource(localPath)
		return nil
	}
	
	// Try to decode as registry source
	var regSource RegistrySource
	if err := aux.Source.Decode(&regSource); err == nil {
		c.Source = &regSource
		return nil
	}
	
	return fmt.Errorf("invalid component source format")
}

// MarshalYAML implements custom YAML marshalling for Component
func (c Component) MarshalYAML() (interface{}, error) {
	type Alias Component
	
	var source interface{}
	if c.Source != nil {
		if c.Source.IsLocal() {
			source = c.Source.GetPath()
		} else {
			source = c.Source.GetRegistry()
		}
	}
	
	// Create a map representation
	result := make(map[string]interface{})
	result["id"] = c.ID
	result["source"] = source
	
	if c.Build != nil {
		result["build"] = c.Build
	}
	
	if len(c.Variables) > 0 {
		result["variables"] = c.Variables
	}
	
	return result, nil
}