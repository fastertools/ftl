// Package constructs provides CDK-style constructs for Spin applications
package constructs

import (
    "fmt"
)

// Example of what a Go CDK-style API could look like for FTL

// Stack represents a synthesizable application stack
type Stack struct {
    Name        string
    Version     string
    Description string
    components  []Component
    triggers    []Trigger
}

// NewFTLStack creates a new FTL application stack
func NewFTLStack(name string) *Stack {
    return &Stack{
        Name:    name,
        Version: "0.1.0",
    }
}

// AddMCPTool adds an MCP tool to the stack with automatic routing
func (s *Stack) AddMCPTool(id string, props ToolProps) *MCPTool {
    tool := &MCPTool{
        ID:     id,
        Source: props.Source,
        Build:  props.Build,
    }
    
    // Automatically create private trigger
    s.components = append(s.components, tool)
    s.triggers = append(s.triggers, Trigger{
        Type:      "http",
        Route:     PrivateRoute{},
        Component: id,
    })
    
    return tool
}

// EnableAuth configures authentication for the stack
func (s *Stack) EnableAuth(provider AuthProvider, config AuthConfig) {
    // Add MCP authorizer component
    authorizer := &MCPAuthorizer{
        Provider: provider,
        Config:   config,
    }
    
    s.components = append(s.components, authorizer)
    
    // Authorizer becomes the public entry point
    s.triggers = append(s.triggers, Trigger{
        Type:      "http",
        Route:     "/...",
        Component: "mcp-authorizer",
    })
    
    // Gateway becomes private
    s.updateGatewayToPrivate()
}

// Synthesize generates the final spin.toml
func (s *Stack) Synthesize() (string, error) {
    // Convert the high-level constructs to spin.toml format
    manifest := map[string]interface{}{
        "spin_manifest_version": 2,
        "application": map[string]interface{}{
            "name":        s.Name,
            "version":     s.Version,
            "description": s.Description,
        },
    }
    
    // Add components
    components := make(map[string]interface{})
    for _, comp := range s.components {
        components[comp.GetID()] = comp.ToManifest()
    }
    manifest["component"] = components
    
    // Add triggers
    var httpTriggers []interface{}
    for _, trig := range s.triggers {
        httpTriggers = append(httpTriggers, trig.ToManifest())
    }
    manifest["trigger"] = map[string]interface{}{
        "http": httpTriggers,
    }
    
    // Convert to TOML
    return toTOML(manifest)
}

// Component interface for all components
type Component interface {
    GetID() string
    ToManifest() map[string]interface{}
}

// MCPTool represents an MCP tool component
type MCPTool struct {
    ID     string
    Source Source
    Build  *BuildConfig
}

func (t *MCPTool) GetID() string { return t.ID }
func (t *MCPTool) ToManifest() map[string]interface{} {
    m := map[string]interface{}{
        "source": t.Source.ToManifest(),
    }
    if t.Build != nil {
        m["build"] = t.Build.ToManifest()
    }
    return m
}

// Source types
type Source interface {
    ToManifest() interface{}
}

type LocalSource string
func (s LocalSource) ToManifest() interface{} { return string(s) }

type RegistrySource struct {
    Registry string
    Package  string
    Version  string
}
func (s RegistrySource) ToManifest() interface{} {
    return map[string]string{
        "registry": s.Registry,
        "package":  s.Package,
        "version":  s.Version,
    }
}

// Example usage:
func Example() {
    // Create a new FTL application
    app := NewFTLStack("my-mcp-app")
    app.Description = "My MCP application"
    
    // Add MCP tools
    app.AddMCPTool("calculator", ToolProps{
        Source: LocalSource("./calc.wasm"),
        Build: &BuildConfig{
            Command: "cargo build --release",
        },
    })
    
    app.AddMCPTool("weather", ToolProps{
        Source: RegistrySource{
            Registry: "ghcr.io",
            Package:  "example/weather",
            Version:  "1.0.0",
        },
    })
    
    // Enable authentication
    app.EnableAuth(AuthWorkOS, AuthConfig{
        OrgID:       "org_123",
        JWTIssuer:   "https://api.workos.com",
        JWTAudience: "my-app",
    })
    
    // Synthesize to spin.toml
    manifest, _ := app.Synthesize()
    fmt.Println(manifest)
}

// Additional types would be defined...
type ToolProps struct {
    Source Source
    Build  *BuildConfig
}

type BuildConfig struct {
    Command string
    Workdir string
    Watch   []string
}

type AuthProvider int
const (
    AuthWorkOS AuthProvider = iota
    AuthAuth0
    AuthCustom
)

type AuthConfig struct {
    OrgID       string
    JWTIssuer   string
    JWTAudience string
}

type Trigger struct {
    Type      string
    Route     interface{}
    Component string
}

type PrivateRoute struct{}

type MCPAuthorizer struct {
    Provider AuthProvider
    Config   AuthConfig
}

func (a *MCPAuthorizer) GetID() string { return "mcp-authorizer" }
func (a *MCPAuthorizer) ToManifest() map[string]interface{} {
    // Implementation...
    return nil
}

func (t Trigger) ToManifest() map[string]interface{} {
    // Implementation...
    return nil
}

func (s *Stack) updateGatewayToPrivate() {
    // Implementation...
}

func toTOML(v interface{}) (string, error) {
    // Implementation...
    return "", nil
}

func (b *BuildConfig) ToManifest() map[string]interface{} {
    // Implementation...  
    return nil
}