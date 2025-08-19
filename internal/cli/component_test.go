package cli

import (
	"os"
	"path/filepath"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"gopkg.in/yaml.v3"

	"github.com/fastertools/ftl-cli/pkg/types"
)

func TestComponentCommands(t *testing.T) {
	// Test component command exists and has subcommands
	cmd := newComponentCmd()
	assert.NotNil(t, cmd)
	assert.Equal(t, "component", cmd.Use)

	// Verify subcommands
	subcommands := []string{"add", "list", "remove"}
	for _, name := range subcommands {
		found := false
		for _, sub := range cmd.Commands() {
			if sub.Name() == name {
				found = true
				break
			}
		}
		assert.True(t, found, "Missing subcommand: %s", name)
	}
}

func TestLoadAndSaveComponentManifest(t *testing.T) {
	tmpDir := t.TempDir()
	manifestPath := filepath.Join(tmpDir, "ftl.yaml")

	// Create test manifest
	manifest := &types.Manifest{
		Application: types.Application{
			Name:        "test-app",
			Version:     "0.1.0",
			Description: "Test application",
		},
		Components: []types.Component{
			{
				ID:     "test-component",
				Source: "./test",
				Build: &types.Build{
					Command: "make build",
					Workdir: "test",
					Watch:   []string{"**/*.go"},
				},
			},
		},
		Access: "public",
	}

	// Save manifest
	data, err := yaml.Marshal(manifest)
	require.NoError(t, err)
	err = os.WriteFile(manifestPath, data, 0644)
	require.NoError(t, err)

	// Load and verify
	loaded, err := loadComponentManifest(manifestPath)
	require.NoError(t, err)
	assert.Equal(t, manifest.Application.Name, loaded.Application.Name)
	assert.Equal(t, manifest.Application.Version, loaded.Application.Version)
	assert.Len(t, loaded.Components, 1)
	assert.Equal(t, "test-component", loaded.Components[0].ID)
}

func TestAddComponentValidation(t *testing.T) {
	tests := []struct {
		name    string
		opts    *AddComponentOptions
		wantErr bool
	}{
		{
			name: "valid local source",
			opts: &AddComponentOptions{
				Name:   "my-component",
				Source: "./components/my-component",
			},
			wantErr: false,
		},
		{
			name: "valid registry source",
			opts: &AddComponentOptions{
				Name:     "my-component",
				Registry: "ghcr.io/user/package:1.0.0",
			},
			wantErr: false,
		},
		{
			name: "valid template",
			opts: &AddComponentOptions{
				Name:     "my-component",
				Template: "go-http",
			},
			wantErr: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Create temp directory with manifest
			tmpDir := t.TempDir()
			oldWd, _ := os.Getwd()
			defer func() { _ = os.Chdir(oldWd) }()
			if err := os.Chdir(tmpDir); err != nil {
				t.Fatal(err)
			}

			manifest := &types.Manifest{
				Application: types.Application{
					Name:    "test-app",
					Version: "0.1.0",
				},
				Components: []types.Component{},
				Access:     "public",
			}
			data, _ := yaml.Marshal(manifest)
			_ = os.WriteFile("ftl.yaml", data, 0644)

			// Test add component
			err := runComponentAdd(tt.opts)
			if tt.wantErr {
				assert.Error(t, err)
			} else {
				assert.NoError(t, err)

				// Verify component was added
				updated, err := loadManifest("ftl.yaml")
				require.NoError(t, err)
				assert.Len(t, updated.Components, 1)
				assert.Equal(t, tt.opts.Name, updated.Components[0].ID)
			}
		})
	}
}

func TestRemoveComponentDirect(t *testing.T) {
	tmpDir := t.TempDir()
	manifestPath := filepath.Join(tmpDir, "ftl.yaml")

	// Create manifest with multiple components
	manifest := &types.Manifest{
		Application: types.Application{
			Name:    "test-app",
			Version: "0.1.0",
		},
		Components: []types.Component{
			{ID: "component-1", Source: "./comp1"},
			{ID: "component-2", Source: "./comp2"},
			{ID: "component-3", Source: "./comp3"},
		},
		Access: "public",
	}

	// Test the logic directly without the interactive prompt
	// Find and remove component
	found := false
	newComponents := []types.Component{}
	for _, comp := range manifest.Components {
		if comp.ID == "component-2" {
			found = true
			continue
		}
		newComponents = append(newComponents, comp)
	}

	assert.True(t, found)
	assert.Len(t, newComponents, 2)

	// Update manifest
	manifest.Components = newComponents

	// Save and verify
	data, _ := yaml.Marshal(manifest)
	_ = os.WriteFile(manifestPath, data, 0644)

	// Load and check
	loaded, err := loadComponentManifest(manifestPath)
	require.NoError(t, err)
	assert.Len(t, loaded.Components, 2)

	// Check remaining components
	ids := []string{}
	for _, c := range loaded.Components {
		ids = append(ids, c.ID)
	}
	assert.Contains(t, ids, "component-1")
	assert.Contains(t, ids, "component-3")
	assert.NotContains(t, ids, "component-2")
}

func TestParseComponentSource(t *testing.T) {
	tests := []struct {
		name         string
		source       interface{}
		wantLocal    string
		wantRegistry *types.RegistrySource
	}{
		{
			name:      "local path string",
			source:    "./components/test",
			wantLocal: "./components/test",
		},
		{
			name: "registry source map",
			source: map[string]interface{}{
				"registry": "ghcr.io",
				"package":  "user/component",
				"version":  "1.0.0",
			},
			wantRegistry: &types.RegistrySource{
				Registry: "ghcr.io",
				Package:  "user/component",
				Version:  "1.0.0",
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			local, registry := types.ParseComponentSource(tt.source)
			assert.Equal(t, tt.wantLocal, local)
			if tt.wantRegistry != nil {
				require.NotNil(t, registry)
				assert.Equal(t, tt.wantRegistry.Registry, registry.Registry)
				assert.Equal(t, tt.wantRegistry.Package, registry.Package)
				assert.Equal(t, tt.wantRegistry.Version, registry.Version)
			} else {
				assert.Nil(t, registry)
			}
		})
	}
}
