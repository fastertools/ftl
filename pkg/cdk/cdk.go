package cdk

import (
	"fmt"

	"cuelang.org/go/cue"
	"cuelang.org/go/cue/cuecontext"
	"github.com/fastertools/ftl-cli/pkg/synthesis"
)

// CDK provides a Go-based Cloud Development Kit for FTL/Spin applications.
// This is the public API for programmatically defining FTL applications.
type CDK struct {
	ctx         *cue.Context
	app         *CDKApp
	synthesizer *synthesis.Synthesizer
}

// New creates a new CDK instance for building FTL applications
func New() *CDK {
	return &CDK{
		ctx:         cuecontext.New(),
		synthesizer: synthesis.NewSynthesizer(),
	}
}

// CDKApp represents an FTL application being built
type CDKApp struct {
	Name        string         `json:"name"`
	Version     string         `json:"version"`
	Description string         `json:"description,omitempty"`
	Components  []CDKComponent `json:"components,omitempty"`
	Access      string         `json:"access,omitempty"`
	Auth        *CDKAuth       `json:"auth,omitempty"`
}

// CDKComponent represents a Wasm component in the application
type CDKComponent struct {
	ID        string            `json:"id"`
	Source    interface{}       `json:"source"` // string for local, map for registry
	Build     *CDKBuildConfig   `json:"build,omitempty"`
	Variables map[string]string `json:"variables,omitempty"`
}

// CDKBuildConfig represents build configuration
type CDKBuildConfig struct {
	Command string   `json:"command"`
	WorkDir string   `json:"workdir,omitempty"`
	Watch   []string `json:"watch,omitempty"`
}

// CDKAuth represents authentication configuration for custom access mode
type CDKAuth struct {
	JWTIssuer         string   `json:"jwt_issuer"`
	JWTAudience       string   `json:"jwt_audience"`
	JWTRequiredScopes []string `json:"jwt_required_scopes,omitempty"`
}

// AppBuilder provides a fluent interface for building applications
type AppBuilder struct {
	cdk *CDK
	app *CDKApp
}

// NewApp creates a new application builder
func (cdk *CDK) NewApp(name string) *AppBuilder {
	app := &CDKApp{
		Name:       name,
		Version:    "0.1.0",
		Access:     "public",
		Components: []CDKComponent{},
	}

	return &AppBuilder{
		cdk: cdk,
		app: app,
	}
}

// SetVersion sets the application version
func (ab *AppBuilder) SetVersion(version string) *AppBuilder {
	ab.app.Version = version
	return ab
}

// SetDescription sets the application description
func (ab *AppBuilder) SetDescription(description string) *AppBuilder {
	ab.app.Description = description
	return ab
}

// SetAccess sets the access level (public or private)
func (ab *AppBuilder) SetAccess(access string) *AppBuilder {
	ab.app.Access = access
	return ab
}

// SetPrivateAccess enables FTL platform authentication (user-only access)
func (ab *AppBuilder) SetPrivateAccess() *AppBuilder {
	ab.app.Access = "private"
	// No auth config needed - FTL handles it
	ab.app.Auth = nil
	return ab
}

// SetOrgAccess enables FTL platform authentication (org-level access)
func (ab *AppBuilder) SetOrgAccess() *AppBuilder {
	ab.app.Access = "org"
	// No auth config needed - FTL handles it
	ab.app.Auth = nil
	return ab
}

// SetCustomAuth enables custom JWT authentication
func (ab *AppBuilder) SetCustomAuth(issuer, audience string) *AppBuilder {
	ab.app.Auth = &CDKAuth{
		JWTIssuer:   issuer,
		JWTAudience: audience,
	}
	ab.app.Access = "custom"
	return ab
}

// AddComponent adds a Wasm component to the application
func (ab *AppBuilder) AddComponent(id string) *ComponentBuilder {
	return &ComponentBuilder{
		app: ab,
		component: CDKComponent{
			ID: id,
			// Variables will be nil unless explicitly set
		},
	}
}

// Build finalizes the application and returns the CDK
func (ab *AppBuilder) Build() *CDK {
	ab.cdk.app = ab.app
	return ab.cdk
}

// ComponentBuilder provides a fluent interface for building components
type ComponentBuilder struct {
	app       *AppBuilder
	component CDKComponent
}

// FromLocal sets the component source as a local path
func (cb *ComponentBuilder) FromLocal(path string) *ComponentBuilder {
	cb.component.Source = path
	return cb
}

// FromRegistry sets the component source from a registry
func (cb *ComponentBuilder) FromRegistry(registry, pkg, version string) *ComponentBuilder {
	cb.component.Source = map[string]string{
		"registry": registry,
		"package":  pkg,
		"version":  version,
	}
	return cb
}

// WithBuild sets the build configuration
func (cb *ComponentBuilder) WithBuild(command string) *ComponentBuilder {
	if cb.component.Build == nil {
		cb.component.Build = &CDKBuildConfig{}
	}
	cb.component.Build.Command = command
	return cb
}

// WithWatch adds watch patterns for development
func (cb *ComponentBuilder) WithWatch(patterns ...string) *ComponentBuilder {
	if cb.component.Build == nil {
		cb.component.Build = &CDKBuildConfig{}
	}
	cb.component.Build.Watch = append(cb.component.Build.Watch, patterns...)
	return cb
}

// WithEnv adds environment variables
func (cb *ComponentBuilder) WithEnv(key, value string) *ComponentBuilder {
	if cb.component.Variables == nil {
		cb.component.Variables = make(map[string]string)
	}
	cb.component.Variables[key] = value
	return cb
}

// Build completes the component and returns to the app builder
func (cb *ComponentBuilder) Build() *AppBuilder {
	cb.app.app.Components = append(cb.app.app.Components, cb.component)
	return cb.app
}

// Synthesize produces a Spin manifest from the CDK application
func (cdk *CDK) Synthesize() (string, error) {
	if cdk.app == nil {
		return "", fmt.Errorf("no application defined - call Build() first")
	}

	// Use the synthesizer to transform the struct to a Spin manifest
	return cdk.synthesizer.SynthesizeFromStruct(cdk.app)
}

// ToCUE exports the current application as CUE source
func (cdk *CDK) ToCUE() (string, error) {
	if cdk.app == nil {
		return "", fmt.Errorf("no application defined - call Build() first")
	}

	// Convert to CUE value
	appValue := cdk.ctx.Encode(cdk.app)
	if appValue.Err() != nil {
		return "", fmt.Errorf("failed to encode app: %w", appValue.Err())
	}

	// Format as CUE source
	return fmt.Sprintf(`package main

app: %v
`, appValue), nil
}

// ValidateWithSchema validates the application against a CUE schema
func (cdk *CDK) ValidateWithSchema(schemaSource string) error {
	if cdk.app == nil {
		return fmt.Errorf("no application defined - call Build() first")
	}

	// Compile the schema
	schema := cdk.ctx.CompileString(schemaSource)
	if schema.Err() != nil {
		return fmt.Errorf("failed to compile schema: %w", schema.Err())
	}

	// Encode the app
	appValue := cdk.ctx.Encode(cdk.app)
	if appValue.Err() != nil {
		return fmt.Errorf("failed to encode app: %w", appValue.Err())
	}

	// Unify with schema and validate
	unified := schema.Unify(appValue)
	if unified.Err() != nil {
		return fmt.Errorf("failed to unify with schema: %w", unified.Err())
	}

	return unified.Validate()
}
