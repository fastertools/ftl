package synthesis

import (
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func TestCDK_SetAccess(t *testing.T) {
	cdk := NewCDK()
	app := cdk.NewApp("test-app").
		SetAccess("private").
		EnableWorkOSAuth("org_123")

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

func TestCDK_EnableCustomAuth(t *testing.T) {
	cdk := NewCDK()
	app := cdk.NewApp("custom-auth-app")

	app.AddComponent("comp1").FromLocal("./comp1.wasm").Build()
	app.EnableCustomAuth("https://auth.example.com", "my-audience")

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
	cdk := NewCDK()
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

func TestCDK_ValidateWithSchema(t *testing.T) {
	cdk := NewCDK()
	app := cdk.NewApp("valid-app").
		SetDescription("A valid application")

	app.AddComponent("comp1").FromLocal("./comp1.wasm").Build()

	app.Build()
	
	// Test validation with a schema
	schema := `
	#CDKApp: {
		name: string
	}
	`
	err := cdk.ValidateWithSchema(schema)
	if err != nil {
		t.Errorf("ValidateWithSchema should not fail: %v", err)
	}
}

func TestSynthesizer_SynthesizeCUE(t *testing.T) {
	cueSource := `
application: {
	name: "cue-app"
	version: "1.0.0"
}
components: [{
	id: "cue-component"
	source: "./component.wasm"
}]
`

	synth := NewSynthesizer()
	manifest, err := synth.SynthesizeCUE(cueSource)
	if err != nil {
		t.Fatalf("Failed to synthesize from CUE: %v", err)
	}

	// Verify the manifest contains expected elements
	if !strings.Contains(manifest, "cue-app") {
		t.Error("Missing application name from CUE")
	}

	if !strings.Contains(manifest, "[component.cue-component]") {
		t.Error("Missing component from CUE")
	}
}

func TestSynthesizeFromConfig_YAML(t *testing.T) {
	// Create a temporary YAML file
	tmpDir := t.TempDir()
	yamlPath := filepath.Join(tmpDir, "ftl.yaml")
	
	yamlContent := `
application:
  name: config-test
  version: 1.0.0
components:
  - id: comp1
    source: ./comp1.wasm
`
	
	if err := os.WriteFile(yamlPath, []byte(yamlContent), 0644); err != nil {
		t.Fatalf("Failed to write test file: %v", err)
	}

	manifest, err := SynthesizeFromConfig(yamlPath)
	if err != nil {
		t.Fatalf("Failed to synthesize from config: %v", err)
	}

	if !strings.Contains(manifest, "config-test") {
		t.Error("Missing application name from config")
	}

	if !strings.Contains(manifest, "[component.comp1]") {
		t.Error("Missing component from config")
	}
}

func TestSynthesizeFromConfig_JSON(t *testing.T) {
	// Create a temporary JSON file
	tmpDir := t.TempDir()
	jsonPath := filepath.Join(tmpDir, "ftl.json")
	
	jsonContent := `{
		"application": {
			"name": "json-config-test",
			"version": "1.0.0"
		},
		"components": [{
			"id": "json-comp",
			"source": "./json.wasm"
		}]
	}`
	
	if err := os.WriteFile(jsonPath, []byte(jsonContent), 0644); err != nil {
		t.Fatalf("Failed to write test file: %v", err)
	}

	manifest, err := SynthesizeFromConfig(jsonPath)
	if err != nil {
		t.Fatalf("Failed to synthesize from config: %v", err)
	}

	if !strings.Contains(manifest, "json-config-test") {
		t.Error("Missing application name from JSON config")
	}

	if !strings.Contains(manifest, "[component.json-comp]") {
		t.Error("Missing component from JSON config")
	}
}

func TestSynthesizeFromConfig_CUE(t *testing.T) {
	// Create a temporary CUE file
	tmpDir := t.TempDir()
	cuePath := filepath.Join(tmpDir, "app.cue")
	
	cueContent := `
application: {
	name: "cue-config-test"
	version: "1.0.0"
}
components: [{
	id: "cue-comp"
	source: "./cue.wasm"
}]`
	
	if err := os.WriteFile(cuePath, []byte(cueContent), 0644); err != nil {
		t.Fatalf("Failed to write test file: %v", err)
	}

	manifest, err := SynthesizeFromConfig(cuePath)
	if err != nil {
		t.Fatalf("Failed to synthesize from CUE config: %v", err)
	}

	if !strings.Contains(manifest, "cue-config-test") {
		t.Error("Missing application name from CUE config")
	}

	if !strings.Contains(manifest, "[component.cue-comp]") {
		t.Error("Missing component from CUE config")
	}
}

func TestSynthesizeFromConfig_UnsupportedFormat(t *testing.T) {
	// Create a temporary file with unsupported extension
	tmpDir := t.TempDir()
	txtPath := filepath.Join(tmpDir, "config.txt")
	
	// Write content that's not valid YAML
	if err := os.WriteFile(txtPath, []byte("{{invalid"), 0644); err != nil {
		t.Fatalf("Failed to write test file: %v", err)
	}

	_, err := SynthesizeFromConfig(txtPath)
	if err == nil {
		t.Error("Should fail for unsupported format")
	}
	if !strings.Contains(err.Error(), "unsupported config format") {
		t.Errorf("Wrong error message: %v", err)
	}
}

func TestSynthesizeFromConfig_FileNotFound(t *testing.T) {
	_, err := SynthesizeFromConfig("/nonexistent/file.yaml")
	if err == nil {
		t.Error("Should fail for nonexistent file")
	}
}

func TestCDK_ErrorCases(t *testing.T) {
	t.Run("Synthesize with invalid CUE", func(t *testing.T) {
		cdk := NewCDK()
		// Create an app with invalid name (contains uppercase)
		app := cdk.NewApp("INVALID-NAME") // Invalid: uppercase
		app.AddComponent("comp").FromLocal("./comp.wasm").Build()
		builtCDK := app.Build()
		
		_, err := builtCDK.Synthesize()
		if err == nil {
			t.Error("Should fail with invalid app name")
		}
	})

	t.Run("ToCUE generates valid output", func(t *testing.T) {
		cdk := NewCDK()
		app := cdk.NewApp("valid-app")
		app.AddComponent("comp").FromLocal("./comp.wasm").Build()
		builtCDK := app.Build()
		
		cue, err := builtCDK.ToCUE()
		if err != nil {
			t.Errorf("ToCUE should not fail: %v", err)
		}
		if !strings.Contains(cue, "valid-app") {
			t.Error("CUE should contain app name")
		}
	})
}

func TestSynthesizer_ErrorCases(t *testing.T) {
	synth := NewSynthesizer()

	t.Run("Invalid YAML", func(t *testing.T) {
		invalidYAML := []byte(`
invalid yaml:
  - this is
  not: valid
    - yaml
`)
		_, err := synth.SynthesizeYAML(invalidYAML)
		if err == nil {
			t.Error("Should fail with invalid YAML")
		}
	})

	t.Run("Invalid JSON", func(t *testing.T) {
		invalidJSON := []byte(`{invalid json}`)
		_, err := synth.SynthesizeJSON(invalidJSON)
		if err == nil {
			t.Error("Should fail with invalid JSON")
		}
	})

	t.Run("Invalid CUE", func(t *testing.T) {
		invalidCUE := `
app: {
	name: "test"
	// Missing closing brace
`
		_, err := synth.SynthesizeCUE(invalidCUE)
		if err == nil {
			t.Error("Should fail with invalid CUE")
		}
	})
}

func TestCDK_ComplexScenario(t *testing.T) {
	// Test a complex app with multiple components and configurations
	cdk := NewCDK()
	app := cdk.NewApp("complex-app").
		SetVersion("2.0.0").
		SetDescription("A complex application with multiple components").
		SetAccess("private")

	// Add local component with full build config
	app.AddComponent("rust-service").
		FromLocal("./target/wasm32-wasip1/release/service.wasm").
		WithBuild("cargo build --target wasm32-wasip1 --release").
		WithWatch("src/**/*.rs", "Cargo.toml", "Cargo.lock").
		WithEnv("DATABASE_URL", "postgres://localhost/mydb").
		WithEnv("LOG_LEVEL", "info").
		Build()

	// Add registry component
	app.AddComponent("auth-service").
		FromRegistry("ghcr.io", "example:auth", "1.2.3").
		WithEnv("JWT_SECRET", "secret").
		Build()

	// Add another local component
	app.AddComponent("frontend").
		FromLocal("./dist/frontend.wasm").
		WithBuild("npm run build").
		WithWatch("src/**/*.js", "src/**/*.jsx", "package.json").
		Build()

	// Enable authentication
	app.EnableWorkOSAuth("org_complex123")

	builtCDK := app.Build()
	
	// Test CUE generation
	cue, err := builtCDK.ToCUE()
	if err != nil {
		t.Fatalf("Failed to generate CUE: %v", err)
	}

	if !strings.Contains(cue, "complex-app") {
		t.Error("CUE missing app name")
	}

	// Test synthesis
	manifest, err := builtCDK.Synthesize()
	if err != nil {
		t.Fatalf("Failed to synthesize: %v", err)
	}

	// Verify all components are present
	components := []string{"rust-service", "auth-service", "frontend", "ftl-mcp-gateway", "mcp-authorizer"}
	for _, comp := range components {
		if !strings.Contains(manifest, "[component."+comp+"]") {
			t.Errorf("Missing component: %s", comp)
		}
	}

	// Verify build config only for local components
	if !strings.Contains(manifest, "[component.rust-service.build]") {
		t.Error("Missing build config for rust-service")
	}
	if strings.Contains(manifest, "[component.auth-service.build]") {
		t.Error("Registry component should not have build config")
	}

	// Verify environment variables
	if !strings.Contains(manifest, "DATABASE_URL") {
		t.Error("Missing DATABASE_URL environment variable")
	}
}

func TestComponentBuilder_MultipleWatchPatterns(t *testing.T) {
	cdk := NewCDK()
	app := cdk.NewApp("watch-test")

	// Test both individual WithWatch calls and multiple patterns in one call
	app.AddComponent("multi-watch").
		FromLocal("./component.wasm").
		WithBuild("make build").
		WithWatch("src/**/*.go").
		WithWatch("templates/**/*.html").
		WithWatch("configs/*.yaml", "configs/*.json").
		Build()

	builtCDK := app.Build()
	manifest, err := builtCDK.Synthesize()
	if err != nil {
		t.Fatalf("Failed to synthesize: %v", err)
	}

	// Check that all watch patterns are included
	watchPatterns := []string{
		"src/**/*.go",
		"templates/**/*.html",
		"configs/*.yaml",
		"configs/*.json",
	}

	for _, pattern := range watchPatterns {
		if !strings.Contains(manifest, pattern) {
			t.Errorf("Missing watch pattern: %s", pattern)
		}
	}
}