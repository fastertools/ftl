package platform

// Application represents an FTL application configuration.
// This is the primary structure for defining applications.
type Application struct {
	// Core application metadata
	Name        string `json:"name" yaml:"name"`
	Version     string `json:"version" yaml:"version"`
	Description string `json:"description,omitempty" yaml:"description,omitempty"`
	
	// Access control
	Access string `json:"access,omitempty" yaml:"access,omitempty"` // public, private, org, custom
	Auth   *Auth  `json:"auth,omitempty" yaml:"auth,omitempty"`
	
	// Components that make up the application
	Components []Component `json:"components" yaml:"components"`
	
	// Global variables
	Variables map[string]string `json:"variables,omitempty" yaml:"variables,omitempty"`
}

// Component represents a WebAssembly component in the application.
type Component struct {
	// Unique identifier for the component
	ID string `json:"id" yaml:"id"`
	
	// Source can be either:
	// - A string for local paths: "./my-component"
	// - A map for registry sources: {"registry": "ghcr.io", "package": "org/component", "version": "1.0.0"}
	Source interface{} `json:"source" yaml:"source"`
	
	// Optional build configuration
	Build *BuildConfig `json:"build,omitempty" yaml:"build,omitempty"`
	
	// Component-specific configuration
	Config map[string]interface{} `json:"config,omitempty" yaml:"config,omitempty"`
	
	// Component-specific variables
	Variables map[string]string `json:"variables,omitempty" yaml:"variables,omitempty"`
}

// BuildConfig defines how to build a component.
type BuildConfig struct {
	Command string `json:"command" yaml:"command"`
	Workdir string `json:"workdir,omitempty" yaml:"workdir,omitempty"`
}

// Auth represents authentication configuration.
type Auth struct {
	OrgID       string `json:"org_id,omitempty" yaml:"org_id,omitempty"`
	JWTIssuer   string `json:"jwt_issuer,omitempty" yaml:"jwt_issuer,omitempty"`
	JWTAudience string `json:"jwt_audience,omitempty" yaml:"jwt_audience,omitempty"`
}

// Manifest represents a processed Spin manifest.
// This is what gets deployed to the WebAssembly runtime.
type Manifest struct {
	Application interface{}            `json:"application"`
	Components  interface{}            `json:"components"`
	Triggers    interface{}            `json:"triggers"`
	Variables   map[string]Variable    `json:"variables,omitempty"`
}

// Variable represents a Spin variable configuration.
type Variable struct {
	Default  string `json:"default,omitempty"`
	Required bool   `json:"required,omitempty"`
}

// RegistrySource represents a component from an OCI registry.
// This is returned when parsing component sources.
type RegistrySource struct {
	Registry string `json:"registry"`
	Package  string `json:"package"`
	Version  string `json:"version"`
}

// AccessMode constants for application access control.
const (
	AccessPublic  = "public"  // No authentication required
	AccessPrivate = "private" // Organization authentication required
	AccessOrg     = "org"     // Organization with specific roles
	AccessCustom  = "custom"  // Custom JWT authentication
)

// ComponentLimits defines recommended limits for production.
const (
	MaxComponentsDefault = 50  // Default maximum components per app
	MaxComponentsSoft    = 100 // Soft limit with warning
	MaxComponentsHard    = 200 // Hard limit for enterprise
)

// PlatformComponents defines the standard platform component IDs.
const (
	GatewayComponentID    = "mcp-gateway"
	AuthorizerComponentID = "mcp-authorizer"
)