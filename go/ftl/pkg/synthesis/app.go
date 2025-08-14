// Package synthesis provides a CDK-style SDK for building FTL applications
package synthesis

import (
	"fmt"
	"strings"

	"cuelang.org/go/cue"
	"cuelang.org/go/cue/cuecontext"
	"cuelang.org/go/cue/format"
)

// App represents an FTL application
type App struct {
	name        string
	version     string
	description string
	tools       []*Tool
	access      AccessMode
	auth        *AuthConfig

	// CUE context for synthesis
	ctx *cue.Context
}

// NewApp creates a new FTL application
func NewApp(name string) *App {
	return &App{
		name:    name,
		version: "0.1.0",
		access:  PublicAccess,
		ctx:     cuecontext.New(),
	}
}

// SetDescription sets the application description
func (a *App) SetDescription(desc string) *App {
	a.description = desc
	return a
}

// SetVersion sets the application version
func (a *App) SetVersion(version string) *App {
	a.version = version
	return a
}

// Tool represents an MCP tool in the application
type Tool struct {
	id          string
	source      ToolSource
	build       *BuildConfig
	environment map[string]string
}

// ToolSource represents where a tool comes from
type ToolSource interface {
	toCUE() string
}

// LocalSource represents a local Wasm file
type LocalSource string

func (s LocalSource) toCUE() string {
	return fmt.Sprintf("%q", string(s))
}

// RegistrySource represents a tool from a registry
type RegistrySource struct {
	Registry string
	Package  string
	Version  string
}

func (s RegistrySource) toCUE() string {
	return fmt.Sprintf(`{
        registry: %q
        package: %q
        version: %q
    }`, s.Registry, s.Package, s.Version)
}

// BuildConfig represents build configuration
type BuildConfig struct {
	Command string
	Workdir string
	Watch   []string
}

func (b *BuildConfig) toCUE() string {
	parts := []string{
		fmt.Sprintf("command: %q", b.Command),
	}
	if b.Workdir != "" {
		parts = append(parts, fmt.Sprintf("workdir: %q", b.Workdir))
	}
	if len(b.Watch) > 0 {
		watches := make([]string, len(b.Watch))
		for i, w := range b.Watch {
			watches[i] = fmt.Sprintf("%q", w)
		}
		parts = append(parts, fmt.Sprintf("watch: [%s]", strings.Join(watches, ", ")))
	}
	return fmt.Sprintf("{\n        %s\n    }", strings.Join(parts, "\n        "))
}

// AddTool adds an MCP tool to the application
func (a *App) AddTool(id string) *ToolBuilder {
	tool := &Tool{id: id}
	a.tools = append(a.tools, tool)
	return &ToolBuilder{tool: tool, app: a}
}

// ToolBuilder provides a fluent interface for configuring tools
type ToolBuilder struct {
	tool *Tool
	app  *App
}

// FromLocal sets the tool source to a local file
func (tb *ToolBuilder) FromLocal(path string) *ToolBuilder {
	tb.tool.source = LocalSource(path)
	return tb
}

// FromRegistry sets the tool source to a registry
func (tb *ToolBuilder) FromRegistry(registry, pkg, version string) *ToolBuilder {
	tb.tool.source = RegistrySource{
		Registry: registry,
		Package:  pkg,
		Version:  version,
	}
	return tb
}

// WithBuild adds build configuration
func (tb *ToolBuilder) WithBuild(cmd string) *ToolBuilder {
	tb.tool.build = &BuildConfig{Command: cmd}
	return tb
}

// WithWatch adds watch patterns for development
func (tb *ToolBuilder) WithWatch(patterns ...string) *ToolBuilder {
	if tb.tool.build == nil {
		tb.tool.build = &BuildConfig{}
	}
	tb.tool.build.Watch = patterns
	return tb
}

// WithEnv adds environment variables
func (tb *ToolBuilder) WithEnv(key, value string) *ToolBuilder {
	if tb.tool.environment == nil {
		tb.tool.environment = make(map[string]string)
	}
	tb.tool.environment[key] = value
	return tb
}

// Build returns the app for chaining
func (tb *ToolBuilder) Build() *App {
	return tb.app
}

// AccessMode represents the access control mode
type AccessMode string

const (
	PublicAccess  AccessMode = "public"
	PrivateAccess AccessMode = "private"
)

// SetAccess sets the access control mode
func (a *App) SetAccess(mode AccessMode) *App {
	a.access = mode
	return a
}

// AuthConfig represents authentication configuration
type AuthConfig struct {
	Provider    string
	OrgID       string
	JWTIssuer   string
	JWTAudience string
}

// EnableWorkOSAuth enables WorkOS authentication
func (a *App) EnableWorkOSAuth(orgID string) *App {
	a.auth = &AuthConfig{
		Provider:    "workos",
		OrgID:       orgID,
		JWTIssuer:   "https://api.workos.com",
		JWTAudience: a.name,
	}
	a.access = PrivateAccess
	return a
}

// EnableCustomAuth enables custom authentication
func (a *App) EnableCustomAuth(issuer, audience string) *App {
	a.auth = &AuthConfig{
		Provider:    "custom",
		JWTIssuer:   issuer,
		JWTAudience: audience,
	}
	a.access = PrivateAccess
	return a
}

// ToCUE generates the CUE representation of the application
func (a *App) ToCUE() (string, error) {
	var cueBuilder strings.Builder

	// Generate app definition using the FTL patterns
	cueBuilder.WriteString(`app: #FTLApplication & {
    name: "`)
	cueBuilder.WriteString(a.name)
	cueBuilder.WriteString(`"
    version: "`)
	cueBuilder.WriteString(a.version)
	cueBuilder.WriteString(`"`)

	if a.description != "" {
		cueBuilder.WriteString(`
    description: "`)
		cueBuilder.WriteString(a.description)
		cueBuilder.WriteString(`"`)
	}

	cueBuilder.WriteString(`
    
    tools: [`)

	// Add tools
	for i, tool := range a.tools {
		if i > 0 {
			cueBuilder.WriteString(",")
		}
		cueBuilder.WriteString(`
        {
            id: "`)
		cueBuilder.WriteString(tool.id)
		cueBuilder.WriteString(`"
            source: `)
		cueBuilder.WriteString(tool.source.toCUE())

		if tool.build != nil {
			cueBuilder.WriteString(`
            build: `)
			cueBuilder.WriteString(tool.build.toCUE())
		}

		if len(tool.environment) > 0 {
			cueBuilder.WriteString(`
            environment: {`)
			for k, v := range tool.environment {
				cueBuilder.WriteString(fmt.Sprintf(`
                %s: %q`, k, v))
			}
			cueBuilder.WriteString(`
            }`)
		}

		cueBuilder.WriteString(`
        }`)
	}
	cueBuilder.WriteString(`
    ]
    
    access: "`)
	cueBuilder.WriteString(string(a.access))
	cueBuilder.WriteString(`"`)

	if a.auth != nil {
		cueBuilder.WriteString(`
    auth: {
        provider: "`)
		cueBuilder.WriteString(a.auth.Provider)
		cueBuilder.WriteString(`"`)

		if a.auth.OrgID != "" {
			cueBuilder.WriteString(`
        org_id: "`)
			cueBuilder.WriteString(a.auth.OrgID)
			cueBuilder.WriteString(`"`)
		}

		if a.auth.JWTIssuer != "" {
			cueBuilder.WriteString(`
        jwt_issuer: "`)
			cueBuilder.WriteString(a.auth.JWTIssuer)
			cueBuilder.WriteString(`"`)
		}

		if a.auth.JWTAudience != "" {
			cueBuilder.WriteString(`
        jwt_audience: "`)
			cueBuilder.WriteString(a.auth.JWTAudience)
			cueBuilder.WriteString(`"`)
		}

		cueBuilder.WriteString(`
    }`)
	}

	cueBuilder.WriteString(`
}
`)

	return cueBuilder.String(), nil
}

// Synthesize generates the final spin.toml via CUE evaluation
func (a *App) Synthesize() (string, error) {
	// Generate CUE
	cueStr, err := a.ToCUE()
	if err != nil {
		return "", fmt.Errorf("failed to generate CUE: %w", err)
	}

	// Parse and evaluate the CUE
	v := a.ctx.CompileString(cueStr)
	if v.Err() != nil {
		return "", fmt.Errorf("failed to compile CUE: %w", v.Err())
	}

	// Extract the spin manifest
	spin := v.LookupPath(cue.ParsePath("spin"))
	if spin.Err() != nil {
		return "", fmt.Errorf("failed to extract spin manifest: %w", spin.Err())
	}

	// Convert to TOML
	// This would use the existing synthesis logic
	return synthToTOML(spin)
}

// synthToTOML converts CUE value to TOML (stub)
func synthToTOML(v cue.Value) (string, error) {
	// This would integrate with existing SpinDL synthesis
	// For now, just format as CUE to show the structure
	b, err := format.Node(v.Syntax())
	return string(b), err
}
