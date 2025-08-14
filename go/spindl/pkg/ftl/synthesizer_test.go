package ftl

import (
    "strings"
    "testing"
)

func TestSynthesizer_SimpleApp(t *testing.T) {
    // Create a simple app
    app := NewApp("test-app").
        SetDescription("Test application").
        SetVersion("1.0.0")
    
    app.AddTool("calculator").
        FromLocal("./calc.wasm").
        Build()
    
    // Synthesize
    synth := NewSynthesizer()
    manifest, err := synth.SynthesizeApp(app)
    if err != nil {
        t.Fatalf("Failed to synthesize: %v", err)
    }
    
    // Verify key elements are present
    if !strings.Contains(manifest, "spin_manifest_version = 2") {
        t.Error("Missing spin manifest version")
    }
    
    if !strings.Contains(manifest, `name = "test-app"`) {
        t.Error("Missing application name")
    }
    
    if !strings.Contains(manifest, "[component.calculator]") {
        t.Error("Missing calculator component")
    }
    
    if !strings.Contains(manifest, "[component.ftl-mcp-gateway]") {
        t.Error("Missing MCP gateway component")
    }
    
    if !strings.Contains(manifest, `route = "/..."`) {
        t.Error("Missing catch-all route")
    }
    
    if !strings.Contains(manifest, `component_names = "calculator"`) {
        t.Error("Missing component_names variable")
    }
}

func TestSynthesizer_WithAuth(t *testing.T) {
    // Create app with authentication
    app := NewApp("secure-app")
    
    app.AddTool("tool1").FromLocal("./tool1.wasm").Build()
    app.AddTool("tool2").FromLocal("./tool2.wasm").Build()
    
    app.EnableWorkOSAuth("org_123")
    
    // Synthesize
    synth := NewSynthesizer()
    manifest, err := synth.SynthesizeApp(app)
    if err != nil {
        t.Fatalf("Failed to synthesize: %v", err)
    }
    
    // Verify auth components
    if !strings.Contains(manifest, "[component.mcp-authorizer]") {
        t.Error("Missing MCP authorizer component")
    }
    
    if !strings.Contains(manifest, `component = "mcp-authorizer"`) {
        t.Error("Authorizer should be the entry point")
    }
    
    if !strings.Contains(manifest, "route = { private = true }") {
        t.Error("Gateway should have private route in auth mode")
    }
    
    if !strings.Contains(manifest, `component_names = "tool1,tool2"`) {
        t.Error("Incorrect component_names")
    }
}

func TestSynthesizer_RegistrySource(t *testing.T) {
    app := NewApp("registry-app")
    
    app.AddTool("remote-tool").
        FromRegistry("ghcr.io", "example/tool", "1.0.0").
        Build()
    
    synth := NewSynthesizer()
    manifest, err := synth.SynthesizeApp(app)
    if err != nil {
        t.Fatalf("Failed to synthesize: %v", err)
    }
    
    // Verify registry source format
    if !strings.Contains(manifest, `source = { registry = "ghcr.io", package = "example/tool", version = "1.0.0" }`) {
        t.Error("Incorrect registry source format")
    }
}

func TestSynthesizer_BuildConfig(t *testing.T) {
    app := NewApp("build-app")
    
    app.AddTool("built-tool").
        FromLocal("./tool.wasm").
        WithBuild("cargo build --release").
        WithWatch("src/**/*.rs", "Cargo.toml").
        Build()
    
    synth := NewSynthesizer()
    manifest, err := synth.SynthesizeApp(app)
    if err != nil {
        t.Fatalf("Failed to synthesize: %v", err)
    }
    
    // Verify build configuration
    if !strings.Contains(manifest, "[component.built-tool.build]") {
        t.Error("Missing build section")
    }
    
    if !strings.Contains(manifest, `command = "cargo build --release"`) {
        t.Error("Missing build command")
    }
    
    if !strings.Contains(manifest, `watch = ["src/**/*.rs", "Cargo.toml"]`) {
        t.Error("Missing watch patterns")
    }
}

func TestSynthesizer_App_ToCUE(t *testing.T) {
    app := NewApp("cue-test").
        SetDescription("Test CUE generation")
    
    app.AddTool("tool1").FromLocal("./tool1.wasm").Build()
    
    cue, err := app.ToCUE()
    if err != nil {
        t.Fatalf("Failed to generate CUE: %v", err)
    }
    
    // Verify CUE structure
    if !strings.Contains(cue, "app: #FTLApplication") {
        t.Error("Missing FTL application type")
    }
    
    if !strings.Contains(cue, `name: "cue-test"`) {
        t.Error("Missing app name in CUE")
    }
    
    if !strings.Contains(cue, `id: "tool1"`) {
        t.Error("Missing tool in CUE")
    }
}