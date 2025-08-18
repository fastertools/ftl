package types

import (
	"testing"

	"github.com/stretchr/testify/assert"
	"gopkg.in/yaml.v3"
)

func TestParseComponentSource(t *testing.T) {
	tests := []struct {
		name             string
		source           interface{}
		expectedLocal    string
		expectedRegistry *RegistrySource
	}{
		{
			name:             "string source",
			source:           "./components/my-comp",
			expectedLocal:    "./components/my-comp",
			expectedRegistry: nil,
		},
		{
			name:             "wasm file source",
			source:           "./build/component.wasm",
			expectedLocal:    "./build/component.wasm",
			expectedRegistry: nil,
		},
		{
			name: "registry source map",
			source: map[string]interface{}{
				"registry": "ghcr.io",
				"package":  "user/component",
				"version":  "1.0.0",
			},
			expectedLocal: "",
			expectedRegistry: &RegistrySource{
				Registry: "ghcr.io",
				Package:  "user/component",
				Version:  "1.0.0",
			},
		},
		{
			name: "registry source map[interface{}]interface{}",
			source: map[interface{}]interface{}{
				"registry": "docker.io",
				"package":  "org/comp",
				"version":  "2.0.0",
			},
			expectedLocal: "",
			expectedRegistry: &RegistrySource{
				Registry: "docker.io",
				Package:  "org/comp",
				Version:  "2.0.0",
			},
		},
		{
			name:             "nil source",
			source:           nil,
			expectedLocal:    "",
			expectedRegistry: nil,
		},
		{
			name:             "invalid type",
			source:           123,
			expectedLocal:    "",
			expectedRegistry: nil,
		},
		{
			name: "incomplete registry source",
			source: map[string]interface{}{
				"registry": "ghcr.io",
			},
			expectedLocal:    "",
			expectedRegistry: nil,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			localPath, registrySource := ParseComponentSource(tt.source)
			assert.Equal(t, tt.expectedLocal, localPath)

			if tt.expectedRegistry == nil {
				assert.Nil(t, registrySource)
			} else {
				assert.NotNil(t, registrySource)
				assert.Equal(t, tt.expectedRegistry.Registry, registrySource.Registry)
				assert.Equal(t, tt.expectedRegistry.Package, registrySource.Package)
				assert.Equal(t, tt.expectedRegistry.Version, registrySource.Version)
			}
		})
	}
}

func TestManifestYAMLMarshaling(t *testing.T) {
	manifest := &Manifest{
		Application: Application{
			Name:        "test-app",
			Version:     "1.0.0",
			Description: "Test application",
		},
		Components: []Component{
			{
				ID:     "comp1",
				Source: "./local/comp1",
				Build: &Build{
					Command: "make build",
					Workdir: "comp1",
					Watch:   []string{"**/*.go"},
				},
				Variables: map[string]string{
					"LOG_LEVEL": "debug",
				},
			},
			{
				ID: "comp2",
				Source: map[string]interface{}{
					"registry": "ghcr.io",
					"package":  "test/comp2",
					"version":  "2.0.0",
				},
			},
		},
		Access: "private",
		Auth: &Auth{
			JWTIssuer:   "https://auth.example.com",
			JWTAudience: "api.example.com",
		},
		Variables: map[string]string{
			"GLOBAL_ENV": "production",
		},
	}

	// Marshal to YAML
	data, err := yaml.Marshal(manifest)
	assert.NoError(t, err)

	// Unmarshal back
	var loaded Manifest
	err = yaml.Unmarshal(data, &loaded)
	assert.NoError(t, err)

	// Verify round-trip
	assert.Equal(t, manifest.Application.Name, loaded.Application.Name)
	assert.Equal(t, manifest.Application.Version, loaded.Application.Version)
	assert.Equal(t, manifest.Application.Description, loaded.Application.Description)
	assert.Len(t, loaded.Components, 2)
	assert.Equal(t, manifest.Access, loaded.Access)
	assert.NotNil(t, loaded.Auth)
	assert.Equal(t, manifest.Auth.JWTIssuer, loaded.Auth.JWTIssuer)
	assert.Equal(t, manifest.Variables["GLOBAL_ENV"], loaded.Variables["GLOBAL_ENV"])
}

func TestComponentBuildConfig(t *testing.T) {
	build := &Build{
		Command: "cargo build --release",
		Workdir: "./rust-comp",
		Watch:   []string{"src/**/*.rs", "Cargo.toml"},
	}

	assert.Equal(t, "cargo build --release", build.Command)
	assert.Equal(t, "./rust-comp", build.Workdir)
	assert.Len(t, build.Watch, 2)
	assert.Contains(t, build.Watch, "src/**/*.rs")
	assert.Contains(t, build.Watch, "Cargo.toml")
}

func TestRegistrySource(t *testing.T) {
	source := &RegistrySource{
		Registry: "ghcr.io",
		Package:  "fermyon/spin-hello-world",
		Version:  "0.2.0",
	}

	assert.Equal(t, "ghcr.io", source.Registry)
	assert.Equal(t, "fermyon/spin-hello-world", source.Package)
	assert.Equal(t, "0.2.0", source.Version)
}

func TestAuthConfig(t *testing.T) {
	auth := &Auth{
		JWTIssuer:   "https://auth.mycompany.com",
		JWTAudience: "api.mycompany.com",
	}

	assert.Equal(t, "https://auth.mycompany.com", auth.JWTIssuer)
	assert.Equal(t, "api.mycompany.com", auth.JWTAudience)
}

func TestApplicationConfig(t *testing.T) {
	app := Application{
		Name:        "my-mcp-tools",
		Version:     "2.1.0",
		Description: "My collection of MCP tools",
	}

	assert.Equal(t, "my-mcp-tools", app.Name)
	assert.Equal(t, "2.1.0", app.Version)
	assert.Equal(t, "My collection of MCP tools", app.Description)
}

func TestComponentWithVariables(t *testing.T) {
	comp := Component{
		ID:     "database-tool",
		Source: "./db-tool",
		Variables: map[string]string{
			"DB_HOST":   "localhost",
			"DB_PORT":   "5432",
			"DB_NAME":   "mydb",
			"LOG_LEVEL": "info",
		},
	}

	assert.Equal(t, "database-tool", comp.ID)
	assert.Equal(t, "./db-tool", comp.Source)
	assert.Len(t, comp.Variables, 4)
	assert.Equal(t, "localhost", comp.Variables["DB_HOST"])
	assert.Equal(t, "5432", comp.Variables["DB_PORT"])
}

func TestManifestDefaults(t *testing.T) {
	// Test minimal manifest
	manifest := &Manifest{
		Application: Application{
			Name:    "minimal-app",
			Version: "0.1.0",
		},
	}

	// Access should default to empty string (will be set by synthesis)
	assert.Equal(t, "", manifest.Access)

	// Auth should be nil by default
	assert.Nil(t, manifest.Auth)

	// Components should be nil by default (YAML omitempty will handle it)
	assert.Nil(t, manifest.Components)

	// Variables should be nil
	assert.Nil(t, manifest.Variables)
}

func TestComplexComponentSource(t *testing.T) {
	// Test that complex source structures are preserved
	yamlData := `
application:
  name: test-app
  version: 1.0.0
components:
  - id: complex-comp
    source:
      registry: my.registry.com
      package: org/component
      version: 3.0.0-beta.1
      extra: ignored
`
	var manifest Manifest
	err := yaml.Unmarshal([]byte(yamlData), &manifest)
	assert.NoError(t, err)

	assert.Len(t, manifest.Components, 1)
	comp := manifest.Components[0]
	assert.Equal(t, "complex-comp", comp.ID)

	// ParseComponentSource should extract the registry info
	local, registry := ParseComponentSource(comp.Source)
	assert.Empty(t, local)
	assert.NotNil(t, registry)
	assert.Equal(t, "my.registry.com", registry.Registry)
	assert.Equal(t, "org/component", registry.Package)
	assert.Equal(t, "3.0.0-beta.1", registry.Version)
}
