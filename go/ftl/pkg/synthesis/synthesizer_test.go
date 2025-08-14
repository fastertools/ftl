package synthesis

import (
	"strings"
	"testing"
)

func TestCDK_SimpleApp(t *testing.T) {
	// Create a simple app using the CDK
	cdk := NewCDK()
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

	if !strings.Contains(manifest, "[component.ftl-mcp-gateway]") {
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
	cdk := NewCDK()
	app := cdk.NewApp("secure-app")

	app.AddComponent("tool1").FromLocal("./tool1.wasm").Build()
	app.AddComponent("tool2").FromLocal("./tool2.wasm").Build()

	app.EnableWorkOSAuth("org_123")

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
	cdk := NewCDK()
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
	cdk := NewCDK()
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
	cdk := NewCDK()
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

func TestSynthesizer_DirectYAML(t *testing.T) {
	yamlInput := `
application:
  name: yaml-app
  version: 1.0.0
  description: Test YAML app
components:
  - id: tool1
    source: ./tool1.wasm
    variables:
      LOG_LEVEL: debug
`

	synth := NewSynthesizer()
	manifest, err := synth.SynthesizeYAML([]byte(yamlInput))
	if err != nil {
		t.Fatalf("Failed to synthesize from YAML: %v", err)
	}

	// Debug
	t.Logf("Generated manifest:\n%s", manifest)

	// Verify the manifest contains expected elements
	if !strings.Contains(manifest, "spin_manifest_version = 2") {
		t.Error("Missing spin manifest version")
	}

	if !strings.Contains(manifest, "yaml-app") {
		t.Error("Missing application name from YAML")
	}

	if !strings.Contains(manifest, "[component.tool1]") {
		t.Error("Missing tool1 component")
	}
}

func TestSynthesizer_DirectJSON(t *testing.T) {
	jsonInput := `{
		"application": {
			"name": "json-app",
			"version": "2.0.0",
			"description": "Test JSON app"
		},
		"components": [
			{
				"id": "tool2",
				"source": "./tool2.wasm",
				"variables": {
					"API_KEY": "secret"
				}
			}
		]
	}`

	synth := NewSynthesizer()
	manifest, err := synth.SynthesizeJSON([]byte(jsonInput))
	if err != nil {
		t.Fatalf("Failed to synthesize from JSON: %v", err)
	}

	// Verify the manifest contains expected elements
	if !strings.Contains(manifest, "json-app") {
		t.Error("Missing application name from JSON")
	}

	if !strings.Contains(manifest, "[component.tool2]") {
		t.Error("Missing tool2 component")
	}
}
