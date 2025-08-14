package synthesis

import (
	"fmt"
	"strings"
	"testing"
)

func TestApp_BuilderPattern(t *testing.T) {
	app := NewApp("test-app")

	// Test chaining
	app.SetDescription("Test description").
		SetVersion("2.0.0").
		SetAccess(PrivateAccess)

	if app.name != "test-app" {
		t.Errorf("Expected name 'test-app', got %s", app.name)
	}
	if app.description != "Test description" {
		t.Errorf("Expected description 'Test description', got %s", app.description)
	}
	if app.version != "2.0.0" {
		t.Errorf("Expected version '2.0.0', got %s", app.version)
	}
	if app.access != PrivateAccess {
		t.Errorf("Expected PrivateAccess, got %v", app.access)
	}
}

func TestToolBuilder_AllMethods(t *testing.T) {
	app := NewApp("test")

	builder := app.AddTool("comprehensive-tool")

	// Test all builder methods
	builder.
		FromLocal("./tool.wasm").
		WithBuild("make build").
		WithWatch("src/**/*.go", "Makefile").
		WithEnv("ENV1", "value1").
		WithEnv("ENV2", "value2").
		Build()

	if len(app.tools) != 1 {
		t.Fatal("Tool not added")
	}

	tool := app.tools[0]

	if tool.id != "comprehensive-tool" {
		t.Errorf("Expected id 'comprehensive-tool', got %s", tool.id)
	}

	if source, ok := tool.source.(LocalSource); !ok || string(source) != "./tool.wasm" {
		t.Error("Source not set correctly")
	}

	if tool.build == nil || tool.build.Command != "make build" {
		t.Error("Build command not set")
	}

	if len(tool.build.Watch) != 2 {
		t.Error("Watch patterns not set")
	}

	if len(tool.environment) != 2 {
		t.Error("Environment variables not set")
	}

	if tool.environment["ENV1"] != "value1" {
		t.Error("ENV1 not set correctly")
	}
}

func TestToolBuilder_FromRegistry(t *testing.T) {
	app := NewApp("test")

	app.AddTool("registry-tool").
		FromRegistry("ghcr.io", "example/tool", "1.2.3").
		Build()

	tool := app.tools[0]

	source, ok := tool.source.(RegistrySource)
	if !ok {
		t.Fatal("Source should be RegistrySource")
	}

	if source.Registry != "ghcr.io" {
		t.Errorf("Expected registry 'ghcr.io', got %s", source.Registry)
	}
	if source.Package != "example/tool" {
		t.Errorf("Expected package 'example/tool', got %s", source.Package)
	}
	if source.Version != "1.2.3" {
		t.Errorf("Expected version '1.2.3', got %s", source.Version)
	}
}

func TestApp_EnableWorkOSAuth(t *testing.T) {
	app := NewApp("test")

	app.EnableWorkOSAuth("org_abc123")

	if app.auth == nil {
		t.Fatal("Auth not configured")
	}

	if app.auth.Provider != "workos" {
		t.Errorf("Expected provider 'workos', got %s", app.auth.Provider)
	}
	if app.auth.OrgID != "org_abc123" {
		t.Errorf("Expected org_id 'org_abc123', got %s", app.auth.OrgID)
	}
	if app.auth.JWTIssuer != "https://api.workos.com" {
		t.Errorf("Expected JWT issuer 'https://api.workos.com', got %s", app.auth.JWTIssuer)
	}
	if app.auth.JWTAudience != "test" {
		t.Errorf("Expected JWT audience 'test', got %s", app.auth.JWTAudience)
	}
	if app.access != PrivateAccess {
		t.Error("Access should be set to private")
	}
}

func TestApp_EnableCustomAuth(t *testing.T) {
	app := NewApp("test")

	app.EnableCustomAuth("https://auth.example.com", "my-audience")

	if app.auth == nil {
		t.Fatal("Auth not configured")
	}

	if app.auth.Provider != "custom" {
		t.Errorf("Expected provider 'custom', got %s", app.auth.Provider)
	}
	if app.auth.JWTIssuer != "https://auth.example.com" {
		t.Errorf("Expected JWT issuer 'https://auth.example.com', got %s", app.auth.JWTIssuer)
	}
	if app.auth.JWTAudience != "my-audience" {
		t.Errorf("Expected JWT audience 'my-audience', got %s", app.auth.JWTAudience)
	}
	if app.access != PrivateAccess {
		t.Error("Access should be set to private")
	}
}

func TestApp_ToCUE(t *testing.T) {
	app := NewApp("cue-app").
		SetDescription("CUE test app").
		SetVersion("3.0.0")

	app.AddTool("tool1").
		FromLocal("./tool1.wasm").
		WithBuild("make").
		WithWatch("src/**/*").
		WithEnv("KEY", "value").
		Build()

	app.AddTool("tool2").
		FromRegistry("ghcr.io", "test/tool", "1.0.0").
		Build()

	app.EnableWorkOSAuth("org_123")

	cue, err := app.ToCUE()
	if err != nil {
		t.Fatalf("Failed to generate CUE: %v", err)
	}

	// Verify CUE contains expected elements
	expectations := []string{
		`app: #FTLApplication`,
		`name: "cue-app"`,
		`version: "3.0.0"`,
		`description: "CUE test app"`,
		`id: "tool1"`,
		`source: "./tool1.wasm"`,
		`command: "make"`,
		`watch: ["src/**/*"]`,
		`KEY: "value"`,
		`id: "tool2"`,
		`registry: "ghcr.io"`,
		`package: "test/tool"`,
		`version: "1.0.0"`,
		`access: "private"`,
		`provider: "workos"`,
		`org_id: "org_123"`,
	}

	for _, expected := range expectations {
		if !strings.Contains(cue, expected) {
			t.Errorf("CUE missing expected string: %s", expected)
		}
	}
}

func TestLocalSource_ToCUE(t *testing.T) {
	source := LocalSource("./test.wasm")
	cue := source.toCUE()

	if cue != `"./test.wasm"` {
		t.Errorf("Expected '\"./test.wasm\"', got %s", cue)
	}
}

func TestRegistrySource_ToCUE(t *testing.T) {
	source := RegistrySource{
		Registry: "ghcr.io",
		Package:  "test/package",
		Version:  "1.0.0",
	}

	cue := source.toCUE()

	expected := `{
        registry: "ghcr.io"
        package: "test/package"
        version: "1.0.0"
    }`

	if cue != expected {
		t.Errorf("CUE output mismatch.\nExpected:\n%s\nGot:\n%s", expected, cue)
	}
}

func TestBuildConfig_ToCUE(t *testing.T) {
	tests := []struct {
		name     string
		build    BuildConfig
		expected string
	}{
		{
			name: "command only",
			build: BuildConfig{
				Command: "cargo build",
			},
			expected: `{
        command: "cargo build"
    }`,
		},
		{
			name: "with workdir",
			build: BuildConfig{
				Command: "cargo build",
				Workdir: "./rust",
			},
			expected: `{
        command: "cargo build"
        workdir: "./rust"
    }`,
		},
		{
			name: "with watch",
			build: BuildConfig{
				Command: "cargo build",
				Watch:   []string{"src/**/*.rs", "Cargo.toml"},
			},
			expected: `{
        command: "cargo build"
        watch: ["src/**/*.rs", "Cargo.toml"]
    }`,
		},
		{
			name: "all fields",
			build: BuildConfig{
				Command: "cargo build",
				Workdir: "./rust",
				Watch:   []string{"src/**/*.rs"},
			},
			expected: `{
        command: "cargo build"
        workdir: "./rust"
        watch: ["src/**/*.rs"]
    }`,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := tt.build.toCUE()
			if result != tt.expected {
				t.Errorf("BuildConfig.toCUE() mismatch.\nExpected:\n%s\nGot:\n%s", tt.expected, result)
			}
		})
	}
}

func TestToolBuilder_MultipleEnvironmentVariables(t *testing.T) {
	app := NewApp("test")

	builder := app.AddTool("env-tool")

	// Add multiple environment variables
	for i := 0; i < 10; i++ {
		key := fmt.Sprintf("KEY_%d", i)
		value := fmt.Sprintf("value_%d", i)
		builder.WithEnv(key, value)
	}

	builder.FromLocal("./tool.wasm").Build()

	tool := app.tools[0]

	if len(tool.environment) != 10 {
		t.Errorf("Expected 10 environment variables, got %d", len(tool.environment))
	}

	// Verify all were set correctly
	for i := 0; i < 10; i++ {
		key := fmt.Sprintf("KEY_%d", i)
		expected := fmt.Sprintf("value_%d", i)
		if tool.environment[key] != expected {
			t.Errorf("Environment variable %s: expected %s, got %s", key, expected, tool.environment[key])
		}
	}
}

func TestApp_MultipleTools(t *testing.T) {
	app := NewApp("multi-tool-app")

	// Add 5 different tools
	for i := 1; i <= 5; i++ {
		app.AddTool(fmt.Sprintf("tool-%d", i)).
			FromLocal(fmt.Sprintf("./tool%d.wasm", i)).
			Build()
	}

	if len(app.tools) != 5 {
		t.Errorf("Expected 5 tools, got %d", len(app.tools))
	}

	// Verify each tool
	for i, tool := range app.tools {
		expectedID := fmt.Sprintf("tool-%d", i+1)
		if tool.id != expectedID {
			t.Errorf("Tool %d: expected id %s, got %s", i, expectedID, tool.id)
		}
	}
}
