package cmd

import (
	"os"
	"path/filepath"
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
		route        string
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
			route:       "/test",
			description: "Test component",
			setupFunc: func(t *testing.T, dir string) {
				// Create spinc.yaml
				cfg := &config.FTLConfig{
					Application: config.ApplicationConfig{
						Name:    "test-app",
						Version: "0.1.0",
					},
					Components: []config.ComponentConfig{},
					Triggers:   []config.TriggerConfig{},
				}
				data, _ := yaml.Marshal(cfg)
				os.WriteFile(filepath.Join(dir, "spinc.yaml"), data, 0644)

				// Create component directory
				os.Mkdir(filepath.Join(dir, "test-comp"), 0755)
			},
			wantErr: false,
			checkFunc: func(t *testing.T, dir string) {
				// Load and check config
				cfg, err := loadSpinConfig(filepath.Join(dir, "spinc.yaml"))
				require.NoError(t, err)

				assert.Len(t, cfg.Components, 1)
				assert.Equal(t, "test-component", cfg.Components[0].ID)
				assert.Equal(t, "./test-comp", cfg.Components[0].Source)
				assert.Equal(t, "Test component", cfg.Components[0].Description)

				assert.Len(t, cfg.Triggers, 1)
				assert.Equal(t, "/test", cfg.Triggers[0].Route)
			},
		},
		{
			name:     "add registry component",
			compName: "registry-comp",
			from:     "registry://example.com/comp:v1.0.0",
			setupFunc: func(t *testing.T, dir string) {
				cfg := &config.FTLConfig{
					Application: config.ApplicationConfig{
						Name:    "test-app",
						Version: "0.1.0",
					},
				}
				data, _ := yaml.Marshal(cfg)
				os.WriteFile(filepath.Join(dir, "spinc.yaml"), data, 0644)
			},
			wantErr: false,
			checkFunc: func(t *testing.T, dir string) {
				cfg, err := loadSpinConfig(filepath.Join(dir, "spinc.yaml"))
				require.NoError(t, err)

				assert.Len(t, cfg.Components, 1)
				assert.Equal(t, "registry-comp", cfg.Components[0].ID)
				assert.Contains(t, cfg.Components[0].Source, "registry://")
			},
		},
		{
			name:     "error - duplicate component",
			compName: "existing",
			from:     "./existing",
			setupFunc: func(t *testing.T, dir string) {
				cfg := &config.FTLConfig{
					Application: config.ApplicationConfig{
						Name:    "test-app",
						Version: "0.1.0",
					},
					Components: []config.ComponentConfig{
						{ID: "existing", Source: "./old"},
					},
				}
				data, _ := yaml.Marshal(cfg)
				os.WriteFile(filepath.Join(dir, "spinc.yaml"), data, 0644)
				os.Mkdir(filepath.Join(dir, "existing"), 0755)
			},
			wantErr: true,
		},
		{
			name:     "error - invalid component name",
			compName: "Invalid-Name",
			from:     "./comp",
			setupFunc: func(t *testing.T, dir string) {
				cfg := &config.FTLConfig{
					Application: config.ApplicationConfig{
						Name:    "test-app",
						Version: "0.1.0",
					},
				}
				data, _ := yaml.Marshal(cfg)
				os.WriteFile(filepath.Join(dir, "spinc.yaml"), data, 0644)
			},
			wantErr: true,
		},
		{
			name:     "error - no spinc.yaml",
			compName: "test",
			from:     "./test",
			setupFunc: func(t *testing.T, dir string) {
				// Don't create spinc.yaml
			},
			wantErr: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Create temp directory
			tmpDir := t.TempDir()

			// Change to temp directory
			oldDir, _ := os.Getwd()
			os.Chdir(tmpDir)
			defer os.Chdir(oldDir)

			// Setup
			if tt.setupFunc != nil {
				tt.setupFunc(t, tmpDir)
			}

			// Run command
			err := addComponent(tt.compName, tt.from, tt.route, tt.description, tt.allowedHosts)

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
		wantErr   bool
	}{
		{
			name: "list multiple components",
			setupFunc: func(t *testing.T, dir string) {
				cfg := &config.FTLConfig{
					Application: config.ApplicationConfig{
						Name:    "test-app",
						Version: "0.1.0",
					},
					Components: []config.ComponentConfig{
						{ID: "comp1", Source: "./comp1", Description: "Component 1"},
						{ID: "comp2", Source: "registry://example.com/comp2:v1.0.0"},
					},
					Triggers: []config.TriggerConfig{
						{Type: config.TriggerTypeHTTP, Component: "comp1", Route: "/comp1"},
					},
				}
				data, _ := yaml.Marshal(cfg)
				os.WriteFile(filepath.Join(dir, "spinc.yaml"), data, 0644)
			},
			wantErr: false,
		},
		{
			name: "empty components",
			setupFunc: func(t *testing.T, dir string) {
				cfg := &config.FTLConfig{
					Application: config.ApplicationConfig{
						Name:    "test-app",
						Version: "0.1.0",
					},
				}
				data, _ := yaml.Marshal(cfg)
				os.WriteFile(filepath.Join(dir, "spinc.yaml"), data, 0644)
			},
			wantErr: false,
		},
		{
			name:      "no spinc.yaml",
			setupFunc: func(t *testing.T, dir string) {},
			wantErr:   true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			tmpDir := t.TempDir()
			oldDir, _ := os.Getwd()
			os.Chdir(tmpDir)
			defer os.Chdir(oldDir)

			if tt.setupFunc != nil {
				tt.setupFunc(t, tmpDir)
			}

			err := listComponents()

			if tt.wantErr {
				assert.Error(t, err)
			} else {
				assert.NoError(t, err)
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
						Name:    "test-app",
						Version: "0.1.0",
					},
					Components: []config.ComponentConfig{
						{ID: "comp1", Source: "./comp1"},
						{ID: "comp2", Source: "./comp2"},
					},
					Triggers: []config.TriggerConfig{
						{Type: config.TriggerTypeHTTP, Component: "comp1", Route: "/comp1"},
						{Type: config.TriggerTypeHTTP, Component: "comp2", Route: "/comp2"},
					},
				}
				data, _ := yaml.Marshal(cfg)
				os.WriteFile(filepath.Join(dir, "spinc.yaml"), data, 0644)
			},
			wantErr: false,
			checkFunc: func(t *testing.T, dir string) {
				cfg, err := loadSpinConfig(filepath.Join(dir, "spinc.yaml"))
				require.NoError(t, err)

				// Should only have comp2 left
				assert.Len(t, cfg.Components, 1)
				assert.Equal(t, "comp2", cfg.Components[0].ID)

				// Should only have comp2 trigger
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
						Name:    "test-app",
						Version: "0.1.0",
					},
					Components: []config.ComponentConfig{
						{ID: "comp1", Source: "./comp1"},
					},
				}
				data, _ := yaml.Marshal(cfg)
				os.WriteFile(filepath.Join(dir, "spinc.yaml"), data, 0644)
			},
			wantErr: true,
		},
		{
			name:      "error - no spinc.yaml",
			compName:  "test",
			setupFunc: func(t *testing.T, dir string) {},
			wantErr:   true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			tmpDir := t.TempDir()
			oldDir, _ := os.Getwd()
			os.Chdir(tmpDir)
			defer os.Chdir(oldDir)

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

func TestValidateComponentName(t *testing.T) {
	tests := []struct {
		name    string
		input   string
		wantErr bool
	}{
		{name: "valid lowercase", input: "mycomponent", wantErr: false},
		{name: "valid with hyphen", input: "my-component", wantErr: false},
		{name: "valid with underscore", input: "my_component", wantErr: false},
		{name: "valid with numbers", input: "component123", wantErr: false},
		{name: "invalid uppercase", input: "MyComponent", wantErr: true},
		{name: "invalid leading hyphen", input: "-component", wantErr: true},
		{name: "invalid trailing hyphen", input: "component-", wantErr: true},
		{name: "invalid double hyphen", input: "my--component", wantErr: true},
		{name: "invalid empty", input: "", wantErr: true},
		{name: "invalid special chars", input: "my@component", wantErr: true},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := validateComponentName(tt.input)
			if tt.wantErr {
				assert.Error(t, err)
			} else {
				assert.NoError(t, err)
			}
		})
	}
}

func TestDetermineSourceType(t *testing.T) {
	tests := []struct {
		name   string
		source string
		want   string
	}{
		{name: "registry with protocol", source: "registry://example.com/comp:v1", want: "registry"},
		{name: "ECR registry", source: "123456.dkr.ecr.us-west-2.amazonaws.com/app/comp:latest", want: "registry"},
		{name: "Docker Hub", source: "docker.io/library/comp:latest", want: "registry"},
		{name: "local relative path", source: "./my-component", want: "local"},
		{name: "local absolute path", source: "/usr/local/components/my-comp", want: "local"},
		{name: "local wasm file", source: "component.wasm", want: "local"},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := determineSourceType(tt.source)
			assert.Equal(t, tt.want, got)
		})
	}
}
