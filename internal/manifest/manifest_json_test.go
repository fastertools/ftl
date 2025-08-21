package manifest

import (
	"encoding/json"
	"os"
	"path/filepath"
	"testing"

	"gopkg.in/yaml.v3"
)

func TestJSONSupport(t *testing.T) {
	tmpDir := t.TempDir()

	// Test data
	original := &Manifest{
		Name:    "test-app",
		Version: "1.0.0",
		Components: []Component{
			{
				ID:     "local-comp",
				Source: "./local",
			},
			{
				ID: "registry-comp",
				Source: SourceRegistry{
					Registry: "ghcr.io",
					Package:  "user/package",
					Version:  "2.0.0",
				},
			},
		},
	}

	t.Run("SaveAndLoadJSON", func(t *testing.T) {
		jsonPath := filepath.Join(tmpDir, "ftl.json")

		// Save as JSON
		if err := original.Save(jsonPath); err != nil {
			t.Fatalf("Failed to save JSON: %v", err)
		}

		// Load JSON
		loaded, err := Load(jsonPath)
		if err != nil {
			t.Fatalf("Failed to load JSON: %v", err)
		}

		// Verify
		if loaded.Name != original.Name {
			t.Errorf("Name mismatch: got %s, want %s", loaded.Name, original.Name)
		}
		if len(loaded.Components) != 2 {
			t.Fatalf("Expected 2 components, got %d", len(loaded.Components))
		}

		// Check local component
		if loaded.Components[0].ID != "local-comp" {
			t.Errorf("Component 0 ID mismatch: got %s", loaded.Components[0].ID)
		}
		if src, ok := loaded.Components[0].Source.(string); !ok || src != "./local" {
			t.Errorf("Component 0 source mismatch: got %v", loaded.Components[0].Source)
		}

		// Check registry component
		if loaded.Components[1].ID != "registry-comp" {
			t.Errorf("Component 1 ID mismatch: got %s", loaded.Components[1].ID)
		}
		if src, ok := loaded.Components[1].Source.(SourceRegistry); !ok {
			t.Errorf("Component 1 source is not SourceRegistry: %T", loaded.Components[1].Source)
		} else {
			if src.Registry != "ghcr.io" || src.Package != "user/package" || src.Version != "2.0.0" {
				t.Errorf("Registry source mismatch: %+v", src)
			}
		}
	})

	t.Run("SaveAndLoadYAML", func(t *testing.T) {
		yamlPath := filepath.Join(tmpDir, "ftl.yaml")

		// Save as YAML
		if err := original.Save(yamlPath); err != nil {
			t.Fatalf("Failed to save YAML: %v", err)
		}

		// Load YAML
		loaded, err := Load(yamlPath)
		if err != nil {
			t.Fatalf("Failed to load YAML: %v", err)
		}

		// Verify
		if loaded.Name != original.Name {
			t.Errorf("Name mismatch: got %s, want %s", loaded.Name, original.Name)
		}
		if len(loaded.Components) != 2 {
			t.Fatalf("Expected 2 components, got %d", len(loaded.Components))
		}
	})

	t.Run("LoadAuto", func(t *testing.T) {
		testDir := filepath.Join(tmpDir, "auto-test")
		_ = os.MkdirAll(testDir, 0755)

		// Save current dir and change to test dir
		oldDir, _ := os.Getwd()
		_ = os.Chdir(testDir)
		defer func() { _ = os.Chdir(oldDir) }()

		// Test with JSON
		jsonData := `{
			"name": "json-app",
			"version": "1.0.0",
			"components": [
				{
					"id": "comp1",
					"source": {
						"registry": "test.io",
						"package": "pkg",
						"version": "1.0"
					}
				}
			]
		}`
		_ = os.WriteFile("ftl.json", []byte(jsonData), 0644)

		loaded, err := LoadAuto()
		if err != nil {
			t.Fatalf("LoadAuto failed with JSON: %v", err)
		}
		if loaded.Name != "json-app" {
			t.Errorf("LoadAuto JSON: wrong name %s", loaded.Name)
		}

		// Clean up and test with YAML
		os.Remove("ftl.json")
		yamlData := `name: yaml-app
version: 2.0.0
components:
  - id: comp2
    source: ./local`
		_ = os.WriteFile("ftl.yaml", []byte(yamlData), 0644)

		loaded, err = LoadAuto()
		if err != nil {
			t.Fatalf("LoadAuto failed with YAML: %v", err)
		}
		if loaded.Name != "yaml-app" {
			t.Errorf("LoadAuto YAML: wrong name %s", loaded.Name)
		}
	})

	t.Run("JSONMarshalUnmarshal", func(t *testing.T) {
		// Test direct JSON marshaling/unmarshaling
		comp := Component{
			ID: "test",
			Source: SourceRegistry{
				Registry: "ghcr.io",
				Package:  "test/pkg",
				Version:  "1.0.0",
			},
		}

		// Marshal
		data, err := json.Marshal(comp)
		if err != nil {
			t.Fatalf("Failed to marshal component: %v", err)
		}

		// Unmarshal
		var loaded Component
		if err := json.Unmarshal(data, &loaded); err != nil {
			t.Fatalf("Failed to unmarshal component: %v", err)
		}

		// Verify
		if loaded.ID != comp.ID {
			t.Errorf("ID mismatch: got %s", loaded.ID)
		}
		if src, ok := loaded.Source.(SourceRegistry); !ok {
			t.Errorf("Source is not SourceRegistry: %T", loaded.Source)
		} else {
			if src.Registry != "ghcr.io" {
				t.Errorf("Registry mismatch: got %s", src.Registry)
			}
		}
	})
}

func TestJSONRoundTrip(t *testing.T) {
	// Create a complex manifest
	m := &Manifest{
		Name:        "complex-app",
		Version:     "3.0.0",
		Description: "A complex test app",
		Components: []Component{
			{
				ID:     "string-source",
				Source: "./local/path",
				Build: &BuildConfig{
					Command: "make build",
					Watch:   []string{"**/*.go"},
				},
			},
			{
				ID: "registry-source",
				Source: SourceRegistry{
					Registry: "docker.io",
					Package:  "library/nginx",
					Version:  "latest",
				},
			},
		},
		Variables: map[string]string{
			"ENV": "production",
		},
	}

	// Marshal to JSON
	jsonData, err := json.MarshalIndent(m, "", "  ")
	if err != nil {
		t.Fatalf("Failed to marshal: %v", err)
	}

	// Unmarshal back
	var loaded Manifest
	if err := json.Unmarshal(jsonData, &loaded); err != nil {
		t.Fatalf("Failed to unmarshal: %v", err)
	}

	// Verify everything survived the round trip
	if loaded.Name != m.Name || loaded.Version != m.Version {
		t.Errorf("Basic fields mismatch")
	}

	if len(loaded.Components) != 2 {
		t.Fatalf("Component count mismatch: got %d", len(loaded.Components))
	}

	// Check string source
	if src, ok := loaded.Components[0].Source.(string); !ok || src != "./local/path" {
		t.Errorf("String source failed roundtrip: %v", loaded.Components[0].Source)
	}

	// Check registry source
	if _, ok := loaded.Components[1].Source.(SourceRegistry); !ok {
		t.Errorf("Registry source failed roundtrip: %T", loaded.Components[1].Source)
	}
}

func TestYAMLRoundTrip(t *testing.T) {
	// Create a complex manifest
	m := &Manifest{
		Name:    "yaml-app",
		Version: "1.0.0",
		Components: []Component{
			{
				ID: "registry",
				Source: SourceRegistry{
					Registry: "ghcr.io",
					Package:  "test",
					Version:  "1.0.0",
				},
			},
		},
	}

	// Marshal to YAML
	yamlData, err := yaml.Marshal(m)
	if err != nil {
		t.Fatalf("Failed to marshal: %v", err)
	}

	// Unmarshal back
	var loaded Manifest
	if err := yaml.Unmarshal(yamlData, &loaded); err != nil {
		t.Fatalf("Failed to unmarshal: %v", err)
	}

	// Check registry source survived
	if src, ok := loaded.Components[0].Source.(SourceRegistry); !ok {
		t.Errorf("Registry source failed YAML roundtrip: got type %T", loaded.Components[0].Source)
	} else {
		if src.Registry != "ghcr.io" {
			t.Errorf("Registry mismatch after YAML roundtrip: %s", src.Registry)
		}
	}
}
