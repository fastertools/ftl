package ftl

import (
	"fmt"
	"strings"
	"testing"
)

// Integration test for the complete CDK workflow
func TestIntegration_CompleteWorkflow(t *testing.T) {
	// Build a complex application using the CDK
	app := NewApp("integration-platform").
		SetDescription("Integration test platform").
		SetVersion("2.0.0")
	
	// Add multiple tools
	app.AddTool("api-gateway").
		FromLocal("./api.wasm").
		WithBuild("cargo build --release").
		WithWatch("src/**/*.rs", "Cargo.toml").
		WithEnv("LOG_LEVEL", "info").
		WithEnv("PORT", "8080").
		Build()
	
	app.AddTool("database-connector").
		FromRegistry("ghcr.io", "tools/db-connector", "3.1.0").
		WithEnv("DB_HOST", "localhost").
		WithEnv("DB_PORT", "5432").
		Build()
	
	app.AddTool("auth-service").
		FromLocal("./auth.wasm").
		WithBuild("npm run build").
		WithWatch("src/**/*.ts", "package.json").
		Build()
	
	// Enable authentication
	app.EnableWorkOSAuth("org_integration_test")
	
	// Synthesize
	synth := NewSynthesizer()
	manifest, err := synth.SynthesizeApp(app)
	if err != nil {
		t.Fatalf("Synthesis failed: %v", err)
	}
	
	// Verify the output contains all expected elements
	verifyManifest(t, manifest, []string{
		// Application metadata
		"spin_manifest_version = 2",
		`name = "integration-platform"`,
		`version = "2.0.0"`,
		`description = "Integration test platform"`,
		
		// Components
		"[component.api-gateway]",
		"[component.database-connector]",
		"[component.auth-service]",
		"[component.ftl-mcp-gateway]",
		"[component.mcp-authorizer]",
		
		// Build configurations
		"[component.api-gateway.build]",
		`command = "cargo build --release"`,
		`watch = ["src/**/*.rs", "Cargo.toml"]`,
		
		// Environment variables
		"[component.api-gateway.variables]",
		`LOG_LEVEL = "info"`,
		`PORT = "8080"`,
		
		// Registry source
		`source = { registry = "ghcr.io", package = "tools/db-connector", version = "3.1.0" }`,
		
		// Auth configuration
		`mcp_jwt_issuer = "https://api.workos.com"`,
		`mcp_jwt_audience = "integration-platform"`,
		
		// Component names
		`component_names = "api-gateway,auth-service,database-connector"`,
		
		// Triggers
		"[[trigger.http]]",
		`component = "mcp-authorizer"`,
		`route = "/..."`,
	})
}

func TestIntegration_PublicMode(t *testing.T) {
	// Test public mode configuration
	app := NewApp("public-app").
		SetDescription("Public application")
	
	app.AddTool("public-tool").
		FromLocal("./tool.wasm").
		Build()
	
	// Default is public, so no need to set explicitly
	
	synth := NewSynthesizer()
	manifest, err := synth.SynthesizeApp(app)
	if err != nil {
		t.Fatalf("Failed to synthesize: %v", err)
	}
	
	// In public mode, gateway should be the entry point
	verifyManifest(t, manifest, []string{
		"[[trigger.http]]",
		`route = "/..."`,
		`component = "ftl-mcp-gateway"`,
	})
	
	// Should NOT have authorizer
	if strings.Contains(manifest, "mcp-authorizer") {
		t.Error("Public mode should not have authorizer")
	}
}

func TestIntegration_MultipleToolTypes(t *testing.T) {
	// Test different tool source types in one app
	app := NewApp("multi-source")
	
	// Local tool
	app.AddTool("local").
		FromLocal("./local.wasm").
		WithBuild("make").
		Build()
	
	// Registry tool from ghcr.io
	app.AddTool("ghcr").
		FromRegistry("ghcr.io", "org/tool", "1.0.0").
		Build()
	
	// Another registry with different version
	app.AddTool("versioned").
		FromRegistry("docker.io", "library/tool", "latest").
		Build()
	
	synth := NewSynthesizer()
	manifest, err := synth.SynthesizeApp(app)
	if err != nil {
		t.Fatalf("Failed to synthesize: %v", err)
	}
	
	// Verify all three tools with their sources
	verifyManifest(t, manifest, []string{
		"[component.local]",
		`source = "./local.wasm"`,
		"[component.ghcr]",
		`source = { registry = "ghcr.io", package = "org/tool", version = "1.0.0" }`,
		"[component.versioned]",
		`source = { registry = "docker.io", package = "library/tool", version = "latest" }`,
	})
}

func TestIntegration_ComplexEnvironment(t *testing.T) {
	// Test complex environment variable configurations
	app := NewApp("env-test")
	
	tool := app.AddTool("complex-env").FromLocal("./tool.wasm")
	
	// Add many environment variables
	envVars := map[string]string{
		"DATABASE_URL":     "postgresql://user:pass@localhost/db",
		"REDIS_URL":        "redis://localhost:6379",
		"API_KEY":          "sk-1234567890abcdef",
		"SECRET_KEY":       "super-secret-key-123",
		"LOG_LEVEL":        "debug",
		"MAX_CONNECTIONS":  "100",
		"TIMEOUT_SECONDS":  "30",
		"ENABLE_METRICS":   "true",
		"METRICS_PORT":     "9090",
		"TRACE_ENABLED":    "false",
	}
	
	for k, v := range envVars {
		tool.WithEnv(k, v)
	}
	
	tool.Build()
	
	synth := NewSynthesizer()
	manifest, err := synth.SynthesizeApp(app)
	if err != nil {
		t.Fatalf("Failed to synthesize: %v", err)
	}
	
	// Verify all environment variables are present
	for k, v := range envVars {
		expected := k + " = \"" + v + "\""
		if !strings.Contains(manifest, expected) {
			t.Errorf("Missing environment variable: %s", expected)
		}
	}
}

func TestIntegration_LargeScale(t *testing.T) {
	// Test with many tools to ensure scalability
	app := NewApp("large-scale")
	
	// Add 50 tools
	for i := 0; i < 50; i++ {
		name := string('a'+rune(i%26)) + "-tool-" + string('0'+rune(i/10)) + string('0'+rune(i%10))
		app.AddTool(name).
			FromLocal("./tool.wasm").
			WithEnv("INDEX", fmt.Sprintf("%d", i)).
			Build()
	}
	
	synth := NewSynthesizer()
	manifest, err := synth.SynthesizeApp(app)
	if err != nil {
		t.Fatalf("Failed to synthesize large app: %v", err)
	}
	
	// Verify basic structure is intact
	if !strings.Contains(manifest, "spin_manifest_version = 2") {
		t.Error("Missing manifest version")
	}
	
	// Count components (50 tools + gateway + maybe authorizer)
	componentCount := strings.Count(manifest, "[component.")
	if componentCount < 51 {
		t.Errorf("Expected at least 51 components, got %d", componentCount)
	}
	
	// Count triggers (should match component count)
	triggerCount := strings.Count(manifest, "[[trigger.http]]")
	if triggerCount < 51 {
		t.Errorf("Expected at least 51 triggers, got %d", triggerCount)
	}
}

func TestIntegration_EdgeCases(t *testing.T) {
	// Test various edge cases
	tests := []struct {
		name  string
		setup func() *App
		check func(t *testing.T, manifest string)
	}{
		{
			name: "no tools",
			setup: func() *App {
				return NewApp("empty")
			},
			check: func(t *testing.T, manifest string) {
				if strings.Contains(manifest, "component_names") {
					t.Error("Empty app should not have component_names")
				}
			},
		},
		{
			name: "single tool",
			setup: func() *App {
				app := NewApp("single")
				app.AddTool("only").FromLocal("./only.wasm").Build()
				return app
			},
			check: func(t *testing.T, manifest string) {
				if !strings.Contains(manifest, `component_names = "only"`) {
					t.Error("Single tool should be in component_names")
				}
			},
		},
		{
			name: "tool with no env",
			setup: func() *App {
				app := NewApp("no-env")
				app.AddTool("plain").FromLocal("./plain.wasm").Build()
				return app
			},
			check: func(t *testing.T, manifest string) {
				if strings.Contains(manifest, "[component.plain.variables]") {
					t.Error("Tool without env should not have variables section")
				}
			},
		},
		{
			name: "tool with no build",
			setup: func() *App {
				app := NewApp("no-build")
				app.AddTool("prebuild").FromLocal("./prebuild.wasm").Build()
				return app
			},
			check: func(t *testing.T, manifest string) {
				if strings.Contains(manifest, "[component.prebuild.build]") {
					t.Error("Tool without build should not have build section")
				}
			},
		},
	}
	
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			app := tt.setup()
			synth := NewSynthesizer()
			manifest, err := synth.SynthesizeApp(app)
			if err != nil {
				t.Fatalf("Failed to synthesize: %v", err)
			}
			tt.check(t, manifest)
		})
	}
}

// Helper function to verify manifest contains expected strings
func verifyManifest(t *testing.T, manifest string, expected []string) {
	t.Helper()
	for _, exp := range expected {
		if !strings.Contains(manifest, exp) {
			t.Errorf("Manifest missing expected content: %s", exp)
		}
	}
}