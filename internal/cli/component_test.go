package cli

import (
	"os"
	"path/filepath"
	"testing"

	"github.com/fastertools/ftl-cli/internal/manifest"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"gopkg.in/yaml.v3"
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

	// Create test manifest with flat structure
	testManifest := map[interface{}]interface{}{
		"name":        "test-app",
		"version":     "0.1.0",
		"description": "Test application",
		"components": []interface{}{
			map[interface{}]interface{}{
				"id":     "test-component",
				"source": "./test",
				"build": map[interface{}]interface{}{
					"command": "make build",
					"workdir": "test",
					"watch":   []interface{}{"**/*.go"},
				},
			},
		},
		"access": "public",
	}

	// Save manifest
	data, err := yaml.Marshal(testManifest)
	require.NoError(t, err)
	err = os.WriteFile(manifestPath, data, 0600)
	require.NoError(t, err)

	// Load and verify using new manifest package
	loaded, err := manifest.Load(manifestPath)
	require.NoError(t, err)
	assert.Equal(t, "test-app", loaded.Name)
	assert.Equal(t, "0.1.0", loaded.Version)
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

			testManifest := map[interface{}]interface{}{
				"name":       "test-app",
				"version":    "0.1.0",
				"components": []interface{}{},
				"access":     "public",
			}
			data, _ := yaml.Marshal(testManifest)
			_ = os.WriteFile("ftl.yaml", data, 0600)

			// Test add component
			err := runComponentAdd(tt.opts)
			if tt.wantErr {
				assert.Error(t, err)
			} else {
				assert.NoError(t, err)

				// Verify component was added
				updated, err := manifest.Load("ftl.yaml")
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
	testManifest := map[interface{}]interface{}{
		"name":    "test-app",
		"version": "0.1.0",
		"components": []interface{}{
			map[interface{}]interface{}{"id": "component-1", "source": "./comp1"},
			map[interface{}]interface{}{"id": "component-2", "source": "./comp2"},
			map[interface{}]interface{}{"id": "component-3", "source": "./comp3"},
		},
		"access": "public",
	}

	// Test the logic directly without the interactive prompt
	// Find and remove component
	found := false
	newComponents := []interface{}{}
	components := testManifest["components"].([]interface{})
	for _, c := range components {
		comp := c.(map[interface{}]interface{})
		if comp["id"] == "component-2" {
			found = true
			continue
		}
		newComponents = append(newComponents, c)
	}

	assert.True(t, found)
	assert.Len(t, newComponents, 2)

	// Update manifest
	testManifest["components"] = newComponents

	// Save and verify
	data, _ := yaml.Marshal(testManifest)
	_ = os.WriteFile(manifestPath, data, 0600)

	// Load and check
	loaded, err := manifest.Load(manifestPath)
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

// TestParseComponentSource is removed as the function is now in platform/client.go
