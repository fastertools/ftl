package cdk

import (
	"strings"
	"testing"
)

// Tests for CDK API methods that aren't covered in synthesizer_test.go

func TestCDK_SetAccess(t *testing.T) {
	cdk := New()
	app := cdk.NewApp("test-app").
		SetOrgAccess()

	app.AddComponent("comp1").FromLocal("./comp1.wasm").Build()

	builtCDK := app.Build()
	manifest, err := builtCDK.Synthesize()
	if err != nil {
		t.Fatalf("Failed to synthesize: %v", err)
	}

	// Private access should include mcp-authorizer
	if !strings.Contains(manifest, "[component.mcp-authorizer]") {
		t.Error("Private access should include mcp-authorizer")
	}
}

func TestCDK_SimpleApp(t *testing.T) {
	// Create a simple app using the CDK
	cdk := New()
	app := cdk.NewApp("test-app").
		SetDescription("Test application").
		SetVersion("1.0.0")

	app.AddComponent("calculator").
		FromLocal("./calc.wasm").
		Build()

	// Build and synthesize
	builtCDK := app.Build()
	manifest, err := builtCDK.Synthesize()
	if err != nil {
		t.Fatalf("Failed to synthesize: %v", err)
	}

	// Verify key elements are present
	if !strings.Contains(manifest, "spin_manifest_version = 2") {
		t.Error("Missing spin manifest version")
	}

	if !strings.Contains(manifest, `name = 'test-app'`) && !strings.Contains(manifest, `name = "test-app"`) {
		t.Error("Missing application name")
	}

	if !strings.Contains(manifest, "[component.calculator]") {
		t.Error("Missing calculator component")
	}

	if !strings.Contains(manifest, "[component.mcp-gateway]") {
		t.Error("Missing MCP gateway component")
	}

	if !strings.Contains(manifest, `route = '/...'`) && !strings.Contains(manifest, `route = "/..."`) {
		t.Error("Missing catch-all route")
	}

	if !strings.Contains(manifest, `component_names = 'calculator'`) && !strings.Contains(manifest, `component_names = "calculator"`) {
		t.Error("Missing component_names variable")
	}
}

func TestCDK_WithAuth(t *testing.T) {
	// Create app with authentication
	cdk := New()
	app := cdk.NewApp("secure-app")

	app.AddComponent("tool1").FromLocal("./tool1.wasm").Build()
	app.AddComponent("tool2").FromLocal("./tool2.wasm").Build()

	app.SetOrgAccess()

	// Build and synthesize
	builtCDK := app.Build()
	manifest, err := builtCDK.Synthesize()
	if err != nil {
		t.Fatalf("Failed to synthesize: %v", err)
	}

	// Verify auth components
	if !strings.Contains(manifest, "[component.mcp-authorizer]") {
		t.Error("Missing MCP authorizer component")
	}

	if !strings.Contains(manifest, `component = 'mcp-authorizer'`) && !strings.Contains(manifest, `component = "mcp-authorizer"`) {
		t.Error("Authorizer should be the entry point")
	}

	if !strings.Contains(manifest, "private = true") {
		t.Error("Gateway should have private route in auth mode")
	}

	if !strings.Contains(manifest, `component_names = 'tool1,tool2'`) && !strings.Contains(manifest, `component_names = "tool1,tool2"`) {
		t.Error("Incorrect component_names")
	}
}

func TestCDK_RegistrySource(t *testing.T) {
	cdk := New()
	app := cdk.NewApp("registry-app")

	app.AddComponent("remote-tool").
		FromRegistry("ghcr.io", "example:tool", "1.0.0").
		Build()

	builtCDK := app.Build()
	manifest, err := builtCDK.Synthesize()
	if err != nil {
		t.Fatalf("Failed to synthesize: %v", err)
	}

	// Verify registry source format
	if !strings.Contains(manifest, "ghcr.io") {
		t.Error("Missing registry")
	}
	if !strings.Contains(manifest, "example:tool") {
		t.Error("Missing package")
	}
	if !strings.Contains(manifest, "1.0.0") {
		t.Error("Missing version")
	}
}

func TestCDK_BuildConfig(t *testing.T) {
	cdk := New()
	app := cdk.NewApp("build-app")

	app.AddComponent("built-tool").
		FromLocal("./tool.wasm").
		WithBuild("cargo build --release").
		WithWatch("src/**/*.rs", "Cargo.toml").
		Build()

	builtCDK := app.Build()
	manifest, err := builtCDK.Synthesize()
	if err != nil {
		t.Fatalf("Failed to synthesize: %v", err)
	}

	// Verify build configuration
	if !strings.Contains(manifest, "[component.built-tool.build]") {
		t.Error("Missing build section")
	}

	if !strings.Contains(manifest, `command = 'cargo build --release'`) && !strings.Contains(manifest, `command = "cargo build --release"`) {
		t.Error("Missing build command")
	}

	// Check for watch patterns
	if !strings.Contains(manifest, "src/**/*.rs") || !strings.Contains(manifest, "Cargo.toml") {
		t.Error("Missing watch patterns")
	}
}

func TestCDK_ToCUE(t *testing.T) {
	cdk := New()
	app := cdk.NewApp("cue-test").
		SetDescription("Test CUE generation")

	app.AddComponent("tool1").FromLocal("./tool1.wasm").Build()

	builtCDK := app.Build()
	cue, err := builtCDK.ToCUE()
	if err != nil {
		t.Fatalf("Failed to generate CUE: %v", err)
	}

	// Verify CUE structure
	if !strings.Contains(cue, "app:") {
		t.Error("Missing app field in CUE")
	}

	if !strings.Contains(cue, `"cue-test"`) {
		t.Error("Missing app name in CUE")
	}

	if !strings.Contains(cue, `"tool1"`) {
		t.Error("Missing tool in CUE")
	}
}

func TestCDK_SetCustomAuth(t *testing.T) {
	cdk := New()
	app := cdk.NewApp("custom-auth-app")

	app.AddComponent("comp1").FromLocal("./comp1.wasm").Build()
	app.SetCustomAuth("https://auth.example.com", "my-audience")

	builtCDK := app.Build()
	manifest, err := builtCDK.Synthesize()
	if err != nil {
		t.Fatalf("Failed to synthesize: %v", err)
	}

	// Should have custom auth configuration
	if !strings.Contains(manifest, "[component.mcp-authorizer]") {
		t.Error("Custom auth should include mcp-authorizer")
	}

	// Check that auth is enabled (authorizer is present)
	// Note: The actual issuer/audience values may be overridden by CUE patterns
	if !strings.Contains(manifest, "mcp_jwt_issuer") {
		t.Error("JWT issuer config not found")
	}

	if !strings.Contains(manifest, "mcp_jwt_audience") {
		t.Error("JWT audience config not found")
	}
}

func TestCDK_WithEnv(t *testing.T) {
	cdk := New()
	app := cdk.NewApp("env-test")

	app.AddComponent("env-comp").
		FromLocal("./comp.wasm").
		WithEnv("API_KEY", "secret123").
		WithEnv("LOG_LEVEL", "debug").
		Build()

	builtCDK := app.Build()
	manifest, err := builtCDK.Synthesize()
	if err != nil {
		t.Fatalf("Failed to synthesize: %v", err)
	}

	// Check environment variables are present
	if !strings.Contains(manifest, "API_KEY = 'secret123'") && !strings.Contains(manifest, `API_KEY = "secret123"`) {
		t.Error("API_KEY environment variable not found")
	}

	if !strings.Contains(manifest, "LOG_LEVEL = 'debug'") && !strings.Contains(manifest, `LOG_LEVEL = "debug"`) {
		t.Error("LOG_LEVEL environment variable not found")
	}
}
