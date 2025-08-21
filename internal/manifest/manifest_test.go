package manifest

import (
	"os"
	"path/filepath"
	"testing"

	"gopkg.in/yaml.v3"
)

func TestManifestOperations(t *testing.T) {
	// Create a temporary file for testing
	tmpDir := t.TempDir()
	manifestPath := filepath.Join(tmpDir, "ftl.yaml")

	// Test loading non-existent manifest creates default
	m, err := Load(manifestPath)
	if err != nil {
		t.Fatalf("Failed to load non-existent manifest: %v", err)
	}
	if m.Name != "app" {
		t.Errorf("Expected default name 'app', got %s", m.Name)
	}
	if len(m.Components) != 0 {
		t.Errorf("Expected empty components, got %d", len(m.Components))
	}

	// Test adding components
	comp1 := Component{
		ID:     "test-component",
		Source: "local/path",
	}
	if err := m.AddComponent(comp1); err != nil {
		t.Fatalf("Failed to add component: %v", err)
	}

	// Test duplicate component
	if err := m.AddComponent(comp1); err == nil {
		t.Error("Expected error adding duplicate component")
	}

	// Test finding component
	found, idx := m.FindComponent("test-component")
	if found == nil {
		t.Error("Failed to find component")
	}
	if idx != 0 {
		t.Errorf("Expected index 0, got %d", idx)
	}

	// Test registry source
	comp2 := Component{
		ID: "registry-component",
		Source: SourceRegistry{
			Registry: "ghcr.io",
			Package:  "bowlofarugula",
			Version:  "0.0.1",
		},
	}
	if err := m.AddComponent(comp2); err != nil {
		t.Fatalf("Failed to add registry component: %v", err)
	}

	// Test saving and loading
	if err := m.Save(manifestPath); err != nil {
		t.Fatalf("Failed to save manifest: %v", err)
	}

	// Load again and verify
	m2, err := Load(manifestPath)
	if err != nil {
		t.Fatalf("Failed to reload manifest: %v", err)
	}
	if len(m2.Components) != 2 {
		t.Errorf("Expected 2 components, got %d", len(m2.Components))
	}

	// Test removing component
	if err := m2.RemoveComponent("test-component"); err != nil {
		t.Fatalf("Failed to remove component: %v", err)
	}
	if len(m2.Components) != 1 {
		t.Errorf("Expected 1 component after removal, got %d", len(m2.Components))
	}

	// Test removing non-existent component
	if err := m2.RemoveComponent("non-existent"); err == nil {
		t.Error("Expected error removing non-existent component")
	}
}

func TestRegistrySourceYAML(t *testing.T) {
	// Test that registry source serializes correctly
	m := Manifest{
		Name:    "test-app",
		Version: "1.0.0",
		Components: []Component{
			{
				ID: "registry-comp",
				Source: SourceRegistry{
					Registry: "ghcr.io",
					Package:  "user/package",
					Version:  "1.0.0",
				},
			},
			{
				ID:     "local-comp",
				Source: "./local/path",
			},
		},
	}

	// Marshal to YAML
	data, err := yaml.Marshal(&m)
	if err != nil {
		t.Fatalf("Failed to marshal manifest: %v", err)
	}

	// Write to temp file
	tmpFile, err := os.CreateTemp("", "test-manifest-*.yaml")
	if err != nil {
		t.Fatalf("Failed to create temp file: %v", err)
	}
	defer func() { _ = os.Remove(tmpFile.Name()) }()

	if err := os.WriteFile(tmpFile.Name(), data, 0600); err != nil {
		t.Fatalf("Failed to write temp file: %v", err)
	}

	// Load back and verify
	loaded, err := Load(tmpFile.Name())
	if err != nil {
		t.Fatalf("Failed to load manifest: %v", err)
	}

	if len(loaded.Components) != 2 {
		t.Errorf("Expected 2 components, got %d", len(loaded.Components))
	}

	// Check registry component
	if loaded.Components[0].ID != "registry-comp" {
		t.Errorf("Expected registry-comp, got %s", loaded.Components[0].ID)
	}

	// Check local component
	if loaded.Components[1].ID != "local-comp" {
		t.Errorf("Expected local-comp, got %s", loaded.Components[1].ID)
	}
}
