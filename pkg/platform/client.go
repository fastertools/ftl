// Package platform provides the official API for FTL platform integrations.
// This package is designed for cloud platforms that deploy FTL applications
// to WebAssembly runtimes like Fermyon Cloud.
package platform

import (
	"fmt"
	"time"

	"github.com/fastertools/ftl-cli/internal/ftl"
	"github.com/fastertools/ftl-cli/pkg/types"
)

// Client provides platform integration capabilities for FTL deployments.
// Use NewClient to create an instance with your configuration.
type Client struct {
	config Config
	synth  *ftl.Synthesizer
}

// Config defines the platform-specific configuration for processing deployments.
type Config struct {
	// Platform component injection settings
	InjectGateway     bool   // Always inject mcp-gateway (usually true)
	InjectAuthorizer  bool   // Inject mcp-authorizer for non-public apps
	GatewayVersion    string // Version of mcp-gateway to use
	AuthorizerVersion string // Version of mcp-authorizer to use
	
	// Component registry settings
	GatewayRegistry    string // Registry for mcp-gateway (default: ghcr.io)
	AuthorizerRegistry string // Registry for mcp-authorizer (default: ghcr.io)
	
	// Security settings
	RequireRegistryComponents bool // Reject local component sources
	AllowedRegistries         []string // If set, only allow components from these registries
	
	// Deployment settings
	DefaultEnvironment string // Default environment if not specified
	MaxComponents      int    // Maximum number of components allowed (0 = unlimited)
}

// DefaultConfig returns a Config with sensible defaults for production platforms.
func DefaultConfig() Config {
	return Config{
		InjectGateway:     true,
		InjectAuthorizer:  true, // Will be conditional based on access mode
		GatewayVersion:    "0.0.13-alpha.0",
		AuthorizerVersion: "0.0.15-alpha.0",
		GatewayRegistry:   "ghcr.io",
		AuthorizerRegistry: "ghcr.io",
		RequireRegistryComponents: true,
		DefaultEnvironment: "production",
		MaxComponents: 50, // Reasonable limit to prevent abuse
	}
}

// NewClient creates a new platform client with the given configuration.
func NewClient(config Config) *Client {
	return &Client{
		config: config,
		synth:  ftl.NewSynthesizer(),
	}
}

// DeploymentRequest represents a request to deploy an FTL application.
// This is the primary input from your platform's API.
type DeploymentRequest struct {
	// Application configuration
	Application *Application `json:"application"`
	
	// Deployment-specific settings
	Environment string            `json:"environment,omitempty"`
	Variables   map[string]string `json:"variables,omitempty"`
	
	// Optional overrides
	AccessMode   *string  `json:"access_mode,omitempty"`
	AllowedRoles []string `json:"allowed_roles,omitempty"`
	
	// Custom auth configuration (for custom access mode)
	CustomAuth *CustomAuthConfig `json:"custom_auth,omitempty"`
}

// CustomAuthConfig defines custom authentication settings.
type CustomAuthConfig struct {
	Issuer   string   `json:"issuer"`
	Audience []string `json:"audience"`
}

// DeploymentResult contains the processed deployment ready for the platform.
type DeploymentResult struct {
	// Processed application manifest
	Manifest *Manifest `json:"manifest"`
	
	// Generated Spin TOML content
	SpinTOML string `json:"spin_toml"`
	
	// Metadata about the deployment
	Metadata DeploymentMetadata `json:"metadata"`
}

// DeploymentMetadata provides information about the processed deployment.
type DeploymentMetadata struct {
	ProcessedAt       time.Time `json:"processed_at"`
	ComponentCount    int       `json:"component_count"`
	InjectedGateway   bool      `json:"injected_gateway"`
	InjectedAuthorizer bool     `json:"injected_authorizer"`
	AccessMode        string    `json:"access_mode"`
	Environment       string    `json:"environment"`
}

// ProcessDeployment processes a deployment request according to platform rules.
// This is the main entry point for platform integrations.
func (c *Client) ProcessDeployment(req *DeploymentRequest) (*DeploymentResult, error) {
	// Validate the request
	if err := c.validateRequest(req); err != nil {
		return nil, fmt.Errorf("invalid deployment request: %w", err)
	}
	
	// Apply defaults
	app := c.prepareApplication(req)
	
	// Validate components
	if err := c.validateComponents(app.Components); err != nil {
		return nil, fmt.Errorf("component validation failed: %w", err)
	}
	
	// Inject platform components based on configuration
	c.injectPlatformComponents(app)
	
	// Convert to internal format for synthesis
	internalApp := c.toInternalApplication(app)
	
	// Generate Spin manifest
	manifest, err := c.synth.SynthesizeToSpin(internalApp)
	if err != nil {
		return nil, fmt.Errorf("failed to synthesize manifest: %w", err)
	}
	
	// Generate TOML
	toml, err := c.synth.SynthesizeToTOML(internalApp)
	if err != nil {
		return nil, fmt.Errorf("failed to generate TOML: %w", err)
	}
	
	// Apply deployment variables
	if req.Variables != nil {
		if manifest.Variables == nil {
			manifest.Variables = make(map[string]ftl.SpinVariable)
		}
		for k, v := range req.Variables {
			manifest.Variables[k] = ftl.SpinVariable{
				Default: v,
			}
		}
	}
	
	// Build result
	result := &DeploymentResult{
		Manifest: c.toPublicManifest(manifest),
		SpinTOML: toml,
		Metadata: DeploymentMetadata{
			ProcessedAt:    time.Now().UTC(),
			ComponentCount: len(app.Components),
			InjectedGateway: c.config.InjectGateway,
			InjectedAuthorizer: c.config.InjectAuthorizer && app.Access != "public",
			AccessMode:     app.Access,
			Environment:    c.getEnvironment(req),
		},
	}
	
	return result, nil
}

// ValidateComponents checks that all components meet platform requirements.
// Use this for pre-flight validation before processing.
func (c *Client) ValidateComponents(components []Component) error {
	return c.validateComponents(components)
}

// GenerateTOML generates Spin TOML from an application.
// Use this if you need to regenerate TOML from a modified application.
func (c *Client) GenerateTOML(app *Application) (string, error) {
	internalApp := c.toInternalApplication(app)
	return c.synth.SynthesizeToTOML(internalApp)
}

// validateRequest validates the deployment request.
func (c *Client) validateRequest(req *DeploymentRequest) error {
	if req == nil {
		return fmt.Errorf("request is nil")
	}
	if req.Application == nil {
		return fmt.Errorf("application is required")
	}
	if req.Application.Name == "" {
		return fmt.Errorf("application name is required")
	}
	if req.Application.Version == "" {
		return fmt.Errorf("application version is required")
	}
	
	// Check component limit
	if c.config.MaxComponents > 0 && len(req.Application.Components) > c.config.MaxComponents {
		return fmt.Errorf("too many components: %d (max: %d)", 
			len(req.Application.Components), c.config.MaxComponents)
	}
	
	return nil
}

// validateComponents ensures all components meet platform requirements.
func (c *Client) validateComponents(components []Component) error {
	for _, comp := range components {
		// Parse component source
		localPath, registrySource := types.ParseComponentSource(comp.Source)
		
		// Check if local sources are allowed
		if c.config.RequireRegistryComponents && localPath != "" {
			return fmt.Errorf("component %s: local sources not allowed (source: %s)", 
				comp.ID, localPath)
		}
		
		// Check registry whitelist
		if registrySource != nil && len(c.config.AllowedRegistries) > 0 {
			allowed := false
			for _, reg := range c.config.AllowedRegistries {
				if registrySource.Registry == reg {
					allowed = true
					break
				}
			}
			if !allowed {
				return fmt.Errorf("component %s: registry %s not in allowed list", 
					comp.ID, registrySource.Registry)
			}
		}
		
		// Validate component has an ID
		if comp.ID == "" {
			return fmt.Errorf("component missing ID")
		}
	}
	
	return nil
}

// prepareApplication applies defaults and overrides to the application.
func (c *Client) prepareApplication(req *DeploymentRequest) *Application {
	app := req.Application
	
	// Apply access mode override
	if req.AccessMode != nil {
		app.Access = *req.AccessMode
	}
	
	// Set default access if not specified
	if app.Access == "" {
		app.Access = "public"
	}
	
	// Apply custom auth if provided
	if req.CustomAuth != nil && app.Access == "custom" {
		if app.Auth == nil {
			app.Auth = &Auth{}
		}
		app.Auth.JWTIssuer = req.CustomAuth.Issuer
		if len(req.CustomAuth.Audience) > 0 {
			app.Auth.JWTAudience = req.CustomAuth.Audience[0]
		}
	}
	
	return app
}

// injectPlatformComponents adds mcp-gateway and mcp-authorizer as configured.
func (c *Client) injectPlatformComponents(app *Application) {
	// Always inject gateway if configured
	if c.config.InjectGateway {
		gateway := Component{
			ID: "mcp-gateway",
			Source: map[string]interface{}{
				"registry": c.config.GatewayRegistry,
				"package":  "fastertools:mcp-gateway",
				"version":  c.config.GatewayVersion,
			},
		}
		app.Components = append([]Component{gateway}, app.Components...)
	}
	
	// Inject authorizer for non-public apps
	if c.config.InjectAuthorizer && app.Access != "public" {
		authorizer := Component{
			ID: "mcp-authorizer",
			Source: map[string]interface{}{
				"registry": c.config.AuthorizerRegistry,
				"package":  "fastertools:mcp-authorizer",
				"version":  c.config.AuthorizerVersion,
			},
		}
		
		// Configure based on access mode
		if app.Auth != nil {
			authorizer.Variables = map[string]string{
				"JWT_ISSUER": app.Auth.JWTIssuer,
			}
			if app.Auth.JWTAudience != "" {
				authorizer.Variables["JWT_AUDIENCE"] = app.Auth.JWTAudience
			}
		}
		
		// Insert after gateway but before user components
		if c.config.InjectGateway {
			components := []Component{app.Components[0], authorizer}
			components = append(components, app.Components[1:]...)
			app.Components = components
		} else {
			app.Components = append([]Component{authorizer}, app.Components...)
		}
	}
}

// getEnvironment determines the environment for the deployment.
func (c *Client) getEnvironment(req *DeploymentRequest) string {
	if req.Environment != "" {
		return req.Environment
	}
	return c.config.DefaultEnvironment
}

// toInternalApplication converts public Application to internal format.
func (c *Client) toInternalApplication(app *Application) *ftl.Application {
	internal := &ftl.Application{
		Name:        app.Name,
		Version:     app.Version,
		Description: app.Description,
		Access:      ftl.AccessMode(app.Access),
		Components:  make([]ftl.Component, len(app.Components)),
		Variables:   app.Variables,
	}
	
	if app.Auth != nil {
		internal.Auth = ftl.AuthConfig{
			Provider:    ftl.AuthProviderCustom,
			JWTIssuer:   app.Auth.JWTIssuer,
			JWTAudience: app.Auth.JWTAudience,
		}
	}
	
	for i, comp := range app.Components {
		// Convert source to internal format
		var source ftl.ComponentSource
		localPath, registrySource := types.ParseComponentSource(comp.Source)
		if localPath != "" {
			source = ftl.LocalSource(localPath)
		} else if registrySource != nil {
			source = &ftl.RegistrySource{
				Registry: registrySource.Registry,
				Package:  registrySource.Package,
				Version:  registrySource.Version,
			}
		}
		
		var build *ftl.BuildConfig
		if comp.Build != nil {
			build = &ftl.BuildConfig{
				Command: comp.Build.Command,
				Workdir: comp.Build.Workdir,
			}
		}
		
		internal.Components[i] = ftl.Component{
			ID:        comp.ID,
			Source:    source,
			Build:     build,
			Variables: comp.Variables,
		}
	}
	
	return internal
}

// toPublicManifest converts internal manifest to public format.
func (c *Client) toPublicManifest(manifest *ftl.SpinManifest) *Manifest {
	pubVars := make(map[string]Variable)
	for k, v := range manifest.Variables {
		pubVars[k] = Variable{
			Default:  v.Default,
			Required: v.Required,
		}
	}
	
	return &Manifest{
		Application: manifest.Application,
		Components:  manifest.Component,
		Triggers:    manifest.Trigger,
		Variables:   pubVars,
	}
}