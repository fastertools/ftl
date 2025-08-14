package synthesis

import (
	"fmt"

	"cuelang.org/go/cue"
	"cuelang.org/go/cue/cuecontext"
)

// CDK provides a Go-based Cloud Development Kit for FTL/Spin applications.
// It follows idiomatic CUE patterns: build configuration in Go, validate and transform with CUE.
type CDK struct {
	ctx *cue.Context
	app *CDKApp
}

// NewCDK creates a new CDK instance
func NewCDK() *CDK {
	return &CDK{
		ctx: cuecontext.New(),
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

// CDKAuth represents authentication configuration
type CDKAuth struct {
	Provider    string `json:"provider"`
	OrgID       string `json:"org_id,omitempty"`
	JWTIssuer   string `json:"jwt_issuer,omitempty"`
	JWTAudience string `json:"jwt_audience,omitempty"`
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

// EnableWorkOSAuth enables WorkOS authentication
func (ab *AppBuilder) EnableWorkOSAuth(orgID string) *AppBuilder {
	ab.app.Auth = &CDKAuth{
		Provider:  "workos",
		OrgID:     orgID,
		JWTIssuer: "https://api.workos.com",
	}
	ab.app.Access = "private"
	return ab
}

// EnableCustomAuth enables custom JWT authentication
func (ab *AppBuilder) EnableCustomAuth(issuer, audience string) *AppBuilder {
	ab.app.Auth = &CDKAuth{
		Provider:    "custom",
		JWTIssuer:   issuer,
		JWTAudience: audience,
	}
	ab.app.Access = "private"
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

	// Convert the Go struct to a CUE value using idiomatic patterns
	appValue := cdk.ctx.Encode(cdk.app)
	if appValue.Err() != nil {
		return "", fmt.Errorf("failed to encode app to CUE: %w", appValue.Err())
	}

	// Load our patterns as a CUE schema
	schema := cdk.ctx.CompileString(ftlPatterns)
	if schema.Err() != nil {
		return "", fmt.Errorf("failed to compile patterns: %w", schema.Err())
	}

	// Create the complete CUE program that transforms our app
	program := fmt.Sprintf(`
%s

// Import the app data
_appData: %v

// Wrap it in the FTL application structure
app: #FTLApplication & _appData

// Transform through the pipeline
_transform: #TransformToSpin & {
	input: app
}

// Extract the final manifest
manifest: _transform.output
`, ftlPatterns, appValue)

	// Compile and evaluate
	result := cdk.ctx.CompileString(program)
	if result.Err() != nil {
		return "", fmt.Errorf("failed to compile transformation: %w", result.Err())
	}

	// Extract the manifest
	manifestValue := result.LookupPath(cue.ParsePath("manifest"))
	if manifestValue.Err() != nil {
		return "", fmt.Errorf("failed to extract manifest: %w", manifestValue.Err())
	}

	// Validate the result
	if err := manifestValue.Validate(); err != nil {
		return "", fmt.Errorf("manifest validation failed: %w", err)
	}

	// Encode to TOML
	synth := NewSynthesizer()
	return synth.encodeToTOML(manifestValue)
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
