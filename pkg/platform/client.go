// Package platform provides the API for FTL platform deployments.
// This is used by deployment platforms to process FTL applications consistently.
package platform

import (
	"fmt"

	"cuelang.org/go/cue"
	"github.com/fastertools/ftl-cli/pkg/policy"
	"github.com/fastertools/ftl-cli/pkg/synthesis"
	"github.com/fastertools/ftl-cli/pkg/validation"
)

const (
	// DefaultECRRegistry is the FTL platform's ECR registry
	DefaultECRRegistry = "795394005211.dkr.ecr.us-west-2.amazonaws.com"
)

// Processor handles FTL application processing for platform deployments.
type Processor struct {
	config         Config
	validator      *validation.Validator
	synthesizer    *synthesis.Synthesizer
	policyGenerator *policy.Generator
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
		config:          config,
		validator:       validation.New(),
		synthesizer:     synthesis.NewSynthesizer(),
		policyGenerator: policy.New(),
	}
}

// ProcessRequest represents a deployment request from the platform.
type ProcessRequest struct {
	// The FTL application configuration (YAML or JSON)
	ConfigData []byte

	// Format of the config data
	Format string // "yaml" or "json"

	// Computed allowed user subjects from WorkOS (only used for private/org access modes)
	// For private mode: platform provides the single authenticated user
	// For org mode: platform provides all org members (filtered by allowed_roles if specified)
	// For public/custom modes: this field is ignored
	AllowedSubjects []string

	// Deployment context for M2M authentication and claim forwarding
	DeploymentContext *DeploymentContext
}

// DeploymentContext provides actor and organization context for deployments
type DeploymentContext struct {
	// Actor type performing the deployment
	ActorType string // "user" or "machine"

	// Organization ID for org-scoped deployments
	OrgID string

	// Claims to forward as headers (claim_name -> header_name)
	// Example: {"sub": "X-User-ID", "org_id": "X-Org-ID"}
	ForwardClaims map[string]string
}

// ProcessResult contains the deployment-ready Spin TOML.
type ProcessResult struct {
	// The complete Spin TOML manifest ready for deployment
	SpinTOML string

	// Metadata about what was processed (for platform logging/tracking)
	Metadata ProcessMetadata
}

// ProcessMetadata provides information about the processing.
type ProcessMetadata struct {
	AppName            string
	AppVersion         string
	ComponentCount     int
	AccessMode         string
	InjectedGateway    bool
	InjectedAuthorizer bool
	SubjectsInjected   int // Number of allowed subjects that were injected
}

// Process handles an FTL deployment request.
//
// The platform only needs to provide:
//   - The raw FTL configuration (YAML/JSON)
//   - Computed allowed subjects for private/org modes (from WorkOS)
//
// The processor handles all FTL-specific logic internally:
//   - Validation against CUE schema
//   - Component registry validation
//   - Synthesis to Spin TOML
//   - Gateway/authorizer injection
//
// For access modes:
//   - public: No allowed subjects needed
//   - private: Platform provides single authenticated user
//   - org: Platform provides org members (filtered by allowed_roles if specified in config)
//   - custom: No allowed subjects needed (app handles its own auth)
func (p *Processor) Process(req ProcessRequest) (*ProcessResult, error) {
	// 1. Validate and parse the configuration to typed structure
	var cueValue interface{}
	var err error

	switch req.Format {
	case "yaml":
		cueValue, err = p.validator.ValidateYAML(req.ConfigData)
		if err != nil {
			return nil, fmt.Errorf("validation failed: %w", err)
		}
	case "json":
		cueValue, err = p.validator.ValidateJSON(req.ConfigData)
		if err != nil {
			return nil, fmt.Errorf("validation failed: %w", err)
		}
	default:
		return nil, fmt.Errorf("unsupported format: %s", req.Format)
	}

	// Extract typed Application from validated CUE value
	validatedApp, err := validation.ExtractApplication(cueValue.(cue.Value))
	if err != nil {
		return nil, fmt.Errorf("failed to extract application: %w", err)
	}

	// 2. Validate components if strict mode
	if p.config.RequireRegistryComponents {
		if err := p.validateComponents(validatedApp); err != nil {
			return nil, err
		}
	}

	// 3. Handle access mode
	accessMode := validatedApp.Access
	if accessMode == "" {
		accessMode = "public"
	}

	// Track subjects injected for metadata
	subjectsInjected := len(req.AllowedSubjects)

	// 4. Generate Rego policy based on access mode
	var authPolicy *policy.Policy

	switch accessMode {
	case "private":
		// Private mode: Only allow the app owner
		if len(req.AllowedSubjects) > 0 {
			ctx := &policy.Context{
				OwnerSubject: req.AllowedSubjects[0], // Platform provides single owner
			}
			authPolicy, err = p.policyGenerator.Generate(policy.ModePrivate, ctx)
			if err != nil {
				return nil, fmt.Errorf("failed to generate private policy: %w", err)
			}
		}

	case "org":
		// Org mode: Organization members and machines
		if req.DeploymentContext != nil && req.DeploymentContext.OrgID != "" {
			ctx := &policy.Context{
				OrgMembers: req.AllowedSubjects,
				OrgID:      req.DeploymentContext.OrgID,
				ActorType:  req.DeploymentContext.ActorType,
			}
			authPolicy, err = p.policyGenerator.Generate(policy.ModeOrg, ctx)
			if err != nil {
				return nil, fmt.Errorf("failed to generate org policy: %w", err)
			}
		}

	case "custom":
		// Custom mode: User provides policy via auth config
		// No platform policy generation needed

	case "public":
		// Public mode: No auth, no policy needed
	}

	// 5. Prepare platform overrides
	overrides := map[string]interface{}{
		"gateway_version":    p.config.GatewayVersion,
		"authorizer_version": p.config.AuthorizerVersion,
	}

	// Add authorization policy if generated
	if authPolicy != nil {
		overrides["authorization_policy"] = authPolicy.Source
		
		policyDataJSON, err := authPolicy.ToJSON()
		if err != nil {
			return nil, fmt.Errorf("failed to serialize policy data: %w", err)
		}
		if policyDataJSON != "" {
			overrides["authorization_policy_data"] = policyDataJSON
		}
	}

	// Add deployment context if provided
	if req.DeploymentContext != nil {
		deploymentCtx := map[string]interface{}{
			"actor_type": req.DeploymentContext.ActorType,
		}
		if req.DeploymentContext.OrgID != "" {
			deploymentCtx["org_id"] = req.DeploymentContext.OrgID
		}
		if len(req.DeploymentContext.ForwardClaims) > 0 {
			deploymentCtx["forward_claims"] = req.DeploymentContext.ForwardClaims
		}
		overrides["deployment_context"] = deploymentCtx
	}

	// 6. Synthesize to Spin TOML with platform overrides
	// The synthesizer accepts interface{} so it can work with both maps and structs
	spinTOML, err := p.synthesizer.SynthesizeWithOverrides(validatedApp, overrides)
	if err != nil {
		return nil, fmt.Errorf("synthesis failed: %w", err)
	}

	// 7. Build result with SpinTOML and metadata
	result := &ProcessResult{
		SpinTOML: spinTOML,
		Metadata: ProcessMetadata{
			AppName:            validatedApp.Name,
			AppVersion:         getStringOrDefault(validatedApp.Version, "0.1.0"),
			ComponentCount:     len(validatedApp.Components),
			AccessMode:         accessMode,
			InjectedGateway:    true,
			InjectedAuthorizer: accessMode != "public",
			SubjectsInjected:   subjectsInjected,
		},
	}

	return result, nil
}

// validateComponents ensures all components meet platform requirements.
func (p *Processor) validateComponents(app *validation.Application) error {
	for _, component := range app.Components {
		// Check if source is local (not allowed in production)
		if _, isLocal := component.Source.(*validation.LocalSource); isLocal {
			return fmt.Errorf("local component sources not allowed in production")
		}

		// Check registry whitelist
		if regSource, ok := component.Source.(*validation.RegistrySource); ok {
			if !p.isAllowedRegistry(regSource.Registry) {
				return fmt.Errorf("registry not allowed: %s", regSource.Registry)
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
