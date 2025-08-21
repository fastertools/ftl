// +build ignore

package main

import (
    "encoding/json"
    "fmt"
    "os"
    
    "github.com/fastertools/ftl/go/spindl/pkg/ftl"
)

// ToolConfig represents tool configuration loaded from JSON
type ToolConfig struct {
    ID       string            `json:"id"`
    Type     string            `json:"type"`
    Source   string            `json:"source"`
    Registry *RegistryInfo     `json:"registry,omitempty"`
    Build    *BuildInfo        `json:"build,omitempty"`
    Env      map[string]string `json:"env,omitempty"`
}

type RegistryInfo struct {
    Registry string `json:"registry"`
    Package  string `json:"package"`
    Version  string `json:"version"`
}

type BuildInfo struct {
    Command string   `json:"command"`
    Watch   []string `json:"watch"`
}

func main() {
    // Load tool configuration from JSON file
    configFile := os.Getenv("TOOLS_CONFIG")
    if configFile == "" {
        configFile = "tools.json"
    }
    
    data, err := os.ReadFile(configFile)
    if err != nil {
        fmt.Fprintf(os.Stderr, "Error reading config: %v\n", err)
        os.Exit(1)
    }
    
    var tools []ToolConfig
    if err := json.Unmarshal(data, &tools); err != nil {
        fmt.Fprintf(os.Stderr, "Error parsing config: %v\n", err)
        os.Exit(1)
    }
    
    // Create FTL application dynamically
    app := ftl.NewApp("dynamic-platform").
        SetDescription("Dynamically generated MCP platform").
        SetVersion(os.Getenv("VERSION"))
    
    // Add tools based on configuration
    for _, tool := range tools {
        builder := app.AddTool(tool.ID)
        
        // Configure source based on type
        switch tool.Type {
        case "local":
            builder.FromLocal(tool.Source)
        case "registry":
            if tool.Registry != nil {
                builder.FromRegistry(
                    tool.Registry.Registry,
                    tool.Registry.Package,
                    tool.Registry.Version,
                )
            }
        }
        
        // Add build configuration if present
        if tool.Build != nil {
            builder.WithBuild(tool.Build.Command)
            if len(tool.Build.Watch) > 0 {
                builder.WithWatch(tool.Build.Watch...)
            }
        }
        
        // Add environment variables
        for k, v := range tool.Env {
            builder.WithEnv(k, v)
        }
        
        builder.Build()
    }
    
    // Configure authentication based on environment
    authMode := os.Getenv("AUTH_MODE")
    switch authMode {
    case "workos":
        app.EnableWorkOSAuth(os.Getenv("WORKOS_ORG_ID"))
    case "custom":
        app.EnableCustomAuth(
            os.Getenv("JWT_ISSUER"),
            os.Getenv("JWT_AUDIENCE"),
        )
    default:
        app.SetAccess(ftl.PublicAccess)
    }
    
    // Generate based on output format
    outputFormat := os.Getenv("OUTPUT_FORMAT")
    switch outputFormat {
    case "cue":
        // Output CUE for debugging
        cue, _ := app.ToCUE()
        fmt.Println(cue)
    default:
        // Default to spin.toml
        synthesizer := ftl.NewSynthesizer()
        manifest, err := synthesizer.SynthesizeApp(app)
        if err != nil {
            fmt.Fprintf(os.Stderr, "Error: %v\n", err)
            os.Exit(1)
        }
        fmt.Println(manifest)
    }
}