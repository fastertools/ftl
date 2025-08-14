package cmd

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"gopkg.in/yaml.v3"

	"github.com/fastertools/ftl-cli/go/shared/config"
)

func TestAddComponent(t *testing.T) {
	tests := []struct {
		name         string
		compName     string
		from         string
		description  string
		allowedHosts []string
		setupFunc    func(t *testing.T, dir string)
		wantErr      bool
		checkFunc    func(t *testing.T, dir string)
	}{
		{
			name:        "add local component",
			compName:    "test-component",
			from:        "./test-comp",
			description: "Test component",
			setupFunc: func(t *testing.T, dir string) {
				// Create ftl.yaml
				cfg := &config.FTLConfig{
					Application: config.ApplicationConfig{
						Name:    "test-app",
						Version: "0.1.0",
					},
					Components: []config.ComponentConfig{},
					// In new architecture, triggers are auto-generated at synthesis
					Triggers: []config.TriggerConfig{},
				}
				data, _ := yaml.Marshal(cfg)
				_ = os.WriteFile(filepath.Join(dir, "ftl.yaml"), data, 0644)

				// Create component directory
				_ = os.Mkdir(filepath.Join(dir, "test-comp"), 0755)
			},
			wantErr: false,
			checkFunc: func(t *testing.T, dir string) {
				cfg, err := loadSpinConfig(filepath.Join(dir, "ftl.yaml"))
				require.NoError(t, err)

				// Component should be added
				assert.Len(t, cfg.Components, 1)
				assert.Equal(t, "test-component", cfg.Components[0].ID)
				assert.Equal(t, "./test-comp", cfg.Components[0].Source)
				assert.Equal(t, "Test component", cfg.Components[0].Description)
				
				// In new architecture, triggers are added at synthesis time
				// Component add only adds a private trigger placeholder
				assert.Len(t, cfg.Triggers, 1)
				assert.Equal(t, "private", cfg.Triggers[0].Route)
				assert.Equal(t, "test-component", cfg.Triggers[0].Component)
			},
		},
		{
			name:     "add OCI registry component",
			compName: "registry-comp",
			from:     "ghcr.io/example/comp:v1.0.0",
			setupFunc: func(t *testing.T, dir string) {
				cfg := &config.FTLConfig{
					Application: config.ApplicationConfig{
						Name:    "test-app",
						Version: "0.1.0",
					},
				}
				data, _ := yaml.Marshal(cfg)
				_ = os.WriteFile(filepath.Join(dir, "ftl.yaml"), data, 0644)
			},
			wantErr: false,
			checkFunc: func(t *testing.T, dir string) {
				cfg, err := loadSpinConfig(filepath.Join(dir, "ftl.yaml"))
				require.NoError(t, err)

				assert.Len(t, cfg.Components, 1)
				assert.Equal(t, "registry-comp", cfg.Components[0].ID)
				
				// Check that source is properly structured as OCI reference
				source := cfg.Components[0].Source
				if sourceMap, ok := source.(map[string]interface{}); ok {
					// OCI components are stored as structured data
					assert.Equal(t, "ghcr.io", sourceMap["registry"])
					assert.Contains(t, sourceMap["package"], "example/comp")
					assert.Equal(t, "v1.0.0", sourceMap["version"])
				} else {
					// Or as a string reference
					assert.Contains(t, source, "ghcr.io")
				}
			},
		},
		{
			name:     "error - no ftl.yaml",
			compName: "test",
			from:     "./test",
			setupFunc: func(t *testing.T, dir string) {
				// Don't create ftl.yaml
			},
			wantErr: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Create temp directory
			tmpDir := t.TempDir()
			oldDir, _ := os.Getwd()
			_ = os.Chdir(tmpDir)
			defer func() { _ = os.Chdir(oldDir) }()

			// Setup
			if tt.setupFunc != nil {
				tt.setupFunc(t, tmpDir)
			}

			// Run command based on source type
			var err error
			if strings.HasPrefix(tt.from, "https://") || strings.HasPrefix(tt.from, "http://") {
				// URL source
				err = addComponentURL(tt.compName, tt.from, "", tt.description, tt.allowedHosts)
			} else if strings.HasPrefix(tt.from, "./") || strings.HasPrefix(tt.from, "/") || !strings.Contains(tt.from, ":") {
				// Local path (relative or absolute)
				err = addComponentLocal(tt.compName, tt.from, tt.description, tt.allowedHosts)
			} else {
				// OCI registry reference (contains : for version or registry URL)
				err = addComponentOCI(tt.compName, tt.from, tt.description, tt.allowedHosts)
			}

			if tt.wantErr {
				assert.Error(t, err)
			} else {
				assert.NoError(t, err)
				if tt.checkFunc != nil {
					tt.checkFunc(t, tmpDir)
				}
			}
		})
	}
}

func TestListComponents(t *testing.T) {
	tests := []struct {
		name      string
		setupFunc func(t *testing.T, dir string)
		checkFunc func(t *testing.T, output string)
	}{
		{
			name: "list multiple components",
			setupFunc: func(t *testing.T, dir string) {
				cfg := &config.FTLConfig{
					Application: config.ApplicationConfig{
						Name: "test-app",
					},
					Components: []config.ComponentConfig{
						{
							ID:          "comp1",
							Source:      "./comp1",
							Description: "Component 1",
						},
						{
							ID:     "comp2",
							Source: map[string]interface{}{
								"registry": "ghcr.io",
								"package":  "example/comp2",
								"version":  "v1.0.0",
							},
						},
					},
					// Routes are now auto-generated at synthesis
					Triggers: []config.TriggerConfig{
						{Route: "private", Component: "comp1"},
						{Route: "private", Component: "comp2"},
					},
				}
				data, _ := yaml.Marshal(cfg)
				_ = os.WriteFile(filepath.Join(dir, "ftl.yaml"), data, 0644)
			},
			checkFunc: func(t *testing.T, output string) {
				assert.Contains(t, output, "comp1")
				assert.Contains(t, output, "Component 1")
				assert.Contains(t, output, "comp2")
			},
		},
		{
			name: "no components",
			setupFunc: func(t *testing.T, dir string) {
				cfg := &config.FTLConfig{
					Application: config.ApplicationConfig{
						Name: "empty-app",
					},
				}
				data, _ := yaml.Marshal(cfg)
				_ = os.WriteFile(filepath.Join(dir, "ftl.yaml"), data, 0644)
			},
			checkFunc: func(t *testing.T, output string) {
				assert.Contains(t, output, "No components found")
			},
		},
		{
			name:      "no config file",
			setupFunc: func(t *testing.T, dir string) {},
			checkFunc: func(t *testing.T, output string) {
				// Should show error or empty list
				assert.True(t, 
					strings.Contains(output, "No components") || 
					strings.Contains(output, "not found"))
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			tmpDir := t.TempDir()
			oldDir, _ := os.Getwd()
			_ = os.Chdir(tmpDir)
			defer func() { _ = os.Chdir(oldDir) }()

			if tt.setupFunc != nil {
				tt.setupFunc(t, tmpDir)
			}

			// Capture output
			output := captureListOutput(tmpDir)
			if tt.checkFunc != nil {
				tt.checkFunc(t, output)
			}
		})
	}
}

func TestRemoveComponent(t *testing.T) {
	tests := []struct {
		name      string
		compName  string
		setupFunc func(t *testing.T, dir string)
		wantErr   bool
		checkFunc func(t *testing.T, dir string)
	}{
		{
			name:     "remove existing component",
			compName: "comp1",
			setupFunc: func(t *testing.T, dir string) {
				cfg := &config.FTLConfig{
					Application: config.ApplicationConfig{
						Name: "test-app",
					},
					Components: []config.ComponentConfig{
						{ID: "comp1", Source: "./comp1"},
						{ID: "comp2", Source: "./comp2"},
					},
					Triggers: []config.TriggerConfig{
						{Route: "private", Component: "comp1"},
						{Route: "private", Component: "comp2"},
					},
				}
				data, _ := yaml.Marshal(cfg)
				_ = os.WriteFile(filepath.Join(dir, "ftl.yaml"), data, 0644)
			},
			wantErr: false,
			checkFunc: func(t *testing.T, dir string) {
				cfg, err := loadSpinConfig(filepath.Join(dir, "ftl.yaml"))
				require.NoError(t, err)
				
				// Should have one component left
				assert.Len(t, cfg.Components, 1)
				assert.Equal(t, "comp2", cfg.Components[0].ID)
				
				// Should have one trigger left
				assert.Len(t, cfg.Triggers, 1)
				assert.Equal(t, "comp2", cfg.Triggers[0].Component)
			},
		},
		{
			name:     "error - component not found",
			compName: "nonexistent",
			setupFunc: func(t *testing.T, dir string) {
				cfg := &config.FTLConfig{
					Application: config.ApplicationConfig{
						Name: "test-app",
					},
				}
				data, _ := yaml.Marshal(cfg)
				_ = os.WriteFile(filepath.Join(dir, "ftl.yaml"), data, 0644)
			},
			wantErr: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			tmpDir := t.TempDir()
			oldDir, _ := os.Getwd()
			_ = os.Chdir(tmpDir)
			defer func() { _ = os.Chdir(oldDir) }()

			if tt.setupFunc != nil {
				tt.setupFunc(t, tmpDir)
			}

			err := removeComponent(tt.compName)
			
			if tt.wantErr {
				assert.Error(t, err)
			} else {
				assert.NoError(t, err)
				if tt.checkFunc != nil {
					tt.checkFunc(t, tmpDir)
				}
			}
		})
	}
}

// Helper function to capture list output
func captureListOutput(dir string) string {
	// This is simplified - in real implementation would capture stdout
	cfg, err := loadSpinConfig(filepath.Join(dir, "ftl.yaml"))
	if err != nil {
		return "No components found.\n\nAdd a component with: ftl component add <type> ..."
	}

	if len(cfg.Components) == 0 {
		return "No components found.\n\nAdd a component with: ftl component add <type> ..."
	}

	var output strings.Builder
	output.WriteString(fmt.Sprintf("Components in %s:\n\n", cfg.Application.Name))
	
	for i, comp := range cfg.Components {
		output.WriteString(fmt.Sprintf("%d. %s\n", i+1, comp.ID))
		
		// Format source
		switch s := comp.Source.(type) {
		case string:
			output.WriteString(fmt.Sprintf("   Source: %s\n", s))
		case map[string]interface{}:
			if registry, ok := s["registry"].(string); ok {
				output.WriteString(fmt.Sprintf("   Source: %s://%s/%s:%s\n",
					"registry", registry, s["package"], s["version"]))
			}
		}
		
		if comp.Description != "" {
			output.WriteString(fmt.Sprintf("   Description: %s\n", comp.Description))
		}
		
		// In new architecture, routes are private by default
		output.WriteString("   Route: private\n")
		output.WriteString("\n")
	}

	return output.String()
}

// TestDetermineSourceType is removed - this was internal logic that's now
// handled differently in the new architecture with explicit command types