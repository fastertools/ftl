// Package platform provides the API for FTL platform deployments.
// This is used by deployment platforms to process FTL applications consistently.
package platform

import (
	"encoding/json"
	"fmt"

	"github.com/fastertools/ftl-cli/pkg/synthesis"
	"github.com/fastertools/ftl-cli/pkg/validation"
	"gopkg.in/yaml.v3"
)

const (
	// DefaultECRRegistry is the FTL platform's ECR registry
	DefaultECRRegistry = "795394005211.dkr.ecr.us-west-2.amazonaws.com"
)

// Processor handles FTL application processing for platform deployments.
type Processor struct {
	config      Config
	validator   *validation.Validator
	synthesizer *synthesis.Synthesizer
}

// Config defines platform-specific settings.
type Config struct {
	// Gateway component settings
	GatewayRegistry string // Default: ghcr.io
	GatewayPackage  string // Default: fastertools:mcp-gateway
	GatewayVersion  string // Default: latest stable version

	// Authorizer component settings
	AuthorizerRegistry string // Default: ghcr.io
	AuthorizerPackage  string // Default: fastertools:mcp-authorizer
	AuthorizerVersion  string // Default: latest stable version

	// Security settings
	RequireRegistryComponents bool     // If true, reject local file sources
	AllowedRegistries         []string // Whitelist of allowed registries (empty = allow all)
}

// DefaultConfig returns production-ready default configuration.
func DefaultConfig() Config {
	return Config{
		GatewayRegistry:           "ghcr.io",
		GatewayPackage:            "fastertools:mcp-gateway",
		GatewayVersion:            "0.0.13-alpha.0",
		AuthorizerRegistry:        "ghcr.io",
		AuthorizerPackage:         "fastertools:mcp-authorizer",
		AuthorizerVersion:         "0.0.15-alpha.0",
		RequireRegistryComponents: true,
		AllowedRegistries: []string{
			"ghcr.io",          // For gateway and authorizer
			DefaultECRRegistry, // For user components
		},
	}
}

// NewProcessor creates a new platform processor.
func NewProcessor(config Config) *Processor {
	return &Processor{
		config:      config,
		validator:   validation.New(),
		synthesizer: synthesis.NewSynthesizer(),
	}
}

// ProcessRequest represents a deployment request from the platform.
type ProcessRequest struct {
	// The FTL application configuration (YAML or JSON)
	ConfigData []byte

	// Format of the config data
	Format string // "yaml" or "json"

	// Deployment-specific variables to inject
	Variables map[string]string

	// For org access mode - computed allowed user subjects from WorkOS
	// The platform should:
	// 1. Call WorkOS to get org members
	// 2. Filter by allowed_roles if specified in the app config
	// 3. Pass the resulting subject list here
	AllowedSubjects []string
}

// ProcessResult contains the deployment-ready Spin TOML.
type ProcessResult struct {
	// The complete Spin TOML manifest
	SpinTOML string

	// Metadata about what was processed
	Metadata ProcessMetadata
}

// ProcessMetadata provides information about the processing.
type ProcessMetadata struct {
	AppName            string
	AppVersion         string
	ComponentCount     int
	AccessMode         string
	AllowedRoles       []string // For org mode - roles to filter by
	InjectedGateway    bool
	InjectedAuthorizer bool
}

// Process handles an FTL deployment request.
func (p *Processor) Process(req ProcessRequest) (*ProcessResult, error) {
	// 1. Parse and validate the configuration
	var app map[string]interface{}

	switch req.Format {
	case "yaml":
		if err := yaml.Unmarshal(req.ConfigData, &app); err != nil {
			return nil, fmt.Errorf("invalid YAML: %w", err)
		}
	case "json":
		if err := json.Unmarshal(req.ConfigData, &app); err != nil {
			return nil, fmt.Errorf("invalid JSON: %w", err)
		}
	default:
		return nil, fmt.Errorf("unsupported format: %s", req.Format)
	}

	// 2. Validate components if strict mode
	if p.config.RequireRegistryComponents {
		if err := p.validateComponents(app); err != nil {
			return nil, err
		}
	}

	// 3. Determine access mode and extract allowed_roles
	accessMode := "public"
	if access, ok := app["access"].(string); ok {
		accessMode = access
	}

	var allowedRoles []string
	if accessMode == "org" {
		if roles, ok := app["allowed_roles"].([]interface{}); ok {
			for _, role := range roles {
				if r, ok := role.(string); ok {
					allowedRoles = append(allowedRoles, r)
				}
			}
		}
	}

	// 4. Handle org access allowed subjects
	if accessMode == "org" && len(req.AllowedSubjects) > 0 {
		app["allowed_subjects"] = req.AllowedSubjects
	}

	// 5. Prepare platform overrides
	overrides := map[string]interface{}{
		"gateway_version":    p.config.GatewayVersion,
		"authorizer_version": p.config.AuthorizerVersion,
	}

	// 6. Synthesize to Spin TOML with platform overrides
	spinTOML, err := p.synthesizer.SynthesizeWithOverrides(app, overrides)
	if err != nil {
		return nil, fmt.Errorf("synthesis failed: %w", err)
	}

	// 7. Build result
	result := &ProcessResult{
		SpinTOML: spinTOML,
		Metadata: ProcessMetadata{
			AppName:            app["name"].(string),
			AppVersion:         getStringOrDefault(app["version"], "0.1.0"),
			ComponentCount:     countComponents(app),
			AccessMode:         accessMode,
			AllowedRoles:       allowedRoles,
			InjectedGateway:    true,
			InjectedAuthorizer: accessMode != "public",
		},
	}

	return result, nil
}

// validateComponents ensures all components meet platform requirements.
func (p *Processor) validateComponents(app map[string]interface{}) error {
	components, ok := app["components"].([]interface{})
	if !ok {
		return nil // No components is valid
	}

	for _, comp := range components {
		component := comp.(map[string]interface{})
		source := component["source"]

		// Check if source is a string (local path)
		if _, isString := source.(string); isString {
			return fmt.Errorf("local component sources not allowed in production")
		}

		// Check registry whitelist
		if sourceMap, ok := source.(map[string]interface{}); ok {
			registry := sourceMap["registry"].(string)
			if !p.isAllowedRegistry(registry) {
				return fmt.Errorf("registry not allowed: %s", registry)
			}
		}
	}

	return nil
}

// isAllowedRegistry checks if a registry is in the whitelist.
func (p *Processor) isAllowedRegistry(registry string) bool {
	if len(p.config.AllowedRegistries) == 0 {
		return true // No whitelist means all allowed
	}

	for _, allowed := range p.config.AllowedRegistries {
		if registry == allowed {
			return true
		}
	}
	return false
}

// Helper functions

func getStringOrDefault(val interface{}, defaultVal string) string {
	if s, ok := val.(string); ok {
		return s
	}
	return defaultVal
}

func countComponents(app map[string]interface{}) int {
	if components, ok := app["components"].([]interface{}); ok {
		return len(components)
	}
	return 0
}
