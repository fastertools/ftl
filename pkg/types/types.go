// Package types provides minimal types for FTL configuration
// These types are used for parsing user input (YAML/JSON) and API transport only.
// All validation and transformation happens through CUE patterns.
package types

// Manifest represents an FTL application manifest (ftl.yaml)
// This is the minimal structure needed to parse user input.
// Security note: No KV stores, databases, or AI models - these are platform-only.
type Manifest struct {
	Application Application       `yaml:"application" json:"application"`
	Components  []Component       `yaml:"components" json:"components"`
	Access      string            `yaml:"access,omitempty" json:"access,omitempty"`
	Auth        *Auth             `yaml:"auth,omitempty" json:"auth,omitempty"`
	Variables   map[string]string `yaml:"variables,omitempty" json:"variables,omitempty"`
}

// Application represents application metadata
type Application struct {
	Name        string `yaml:"name" json:"name"`
	Version     string `yaml:"version,omitempty" json:"version,omitempty"`
	Description string `yaml:"description,omitempty" json:"description,omitempty"`
}

// Component represents a user-defined component
// Note: No KV stores or other platform resources
type Component struct {
	ID        string            `yaml:"id" json:"id"`
	Source    interface{}       `yaml:"source" json:"source"` // string or map
	Build     *Build            `yaml:"build,omitempty" json:"build,omitempty"`
	Variables map[string]string `yaml:"variables,omitempty" json:"variables,omitempty"`
}

// Build represents build configuration
type Build struct {
	Command string   `yaml:"command" json:"command"`
	Workdir string   `yaml:"workdir,omitempty" json:"workdir,omitempty"`
	Watch   []string `yaml:"watch,omitempty" json:"watch,omitempty"`
}

// Auth represents authentication configuration
type Auth struct {
	JWTIssuer   string `yaml:"jwt_issuer,omitempty" json:"jwt_issuer,omitempty"`
	JWTAudience string `yaml:"jwt_audience,omitempty" json:"jwt_audience,omitempty"`
}

// RegistrySource represents a component from a registry
type RegistrySource struct {
	Registry string `json:"registry" yaml:"registry"`
	Package  string `json:"package" yaml:"package"`
	Version  string `json:"version" yaml:"version"`
}

func ParseComponentSource(source interface{}) (string, *RegistrySource) {
	switch s := source.(type) {
	case string:
		return s, nil
	case map[string]interface{}:
		reg := &RegistrySource{}
		if r, ok := s["registry"].(string); ok {
			reg.Registry = r
		}
		if p, ok := s["package"].(string); ok {
			reg.Package = p
		}
		if v, ok := s["version"].(string); ok {
			reg.Version = v
		}
		if reg.Registry != "" || reg.Package != "" {
			return "", reg
		}
		return "", nil
	case map[interface{}]interface{}:
		reg := &RegistrySource{}
		if r, ok := s["registry"].(string); ok {
			reg.Registry = r
		}
		if p, ok := s["package"].(string); ok {
			reg.Package = p
		}
		if v, ok := s["version"].(string); ok {
			reg.Version = v
		}
		if reg.Registry != "" || reg.Package != "" {
			return "", reg
		}
		return "", nil
	default:
		return "", nil
	}
}
