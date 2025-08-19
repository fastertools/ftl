package platform

import (
	"strings"
	"testing"
)

func TestProcessor_Process(t *testing.T) {
	tests := []struct {
		name    string
		config  Config
		request ProcessRequest
		wantErr bool
		checks  func(t *testing.T, result *ProcessResult)
	}{
		{
			name:   "simple public app",
			config: DefaultConfig(),
			request: ProcessRequest{
				Format: "yaml",
				ConfigData: []byte(`
name: test-app
version: "1.0.0"
components:
  - id: api
    source:
      registry: ghcr.io
      package: test/api
      version: v1.0.0
`),
			},
			wantErr: false,
			checks: func(t *testing.T, result *ProcessResult) {
				if result.SpinTOML == "" {
					t.Error("SpinTOML should not be empty")
				}
				if !strings.Contains(result.SpinTOML, "mcp-gateway") {
					t.Error("SpinTOML should contain mcp-gateway")
				}
				if strings.Contains(result.SpinTOML, "mcp-authorizer") {
					t.Error("SpinTOML should not contain mcp-authorizer for public app")
				}
				if result.Metadata.AppName != "test-app" {
					t.Errorf("expected app name 'test-app', got %s", result.Metadata.AppName)
				}
			},
		},
		{
			name:   "private app with authorizer",
			config: DefaultConfig(),
			request: ProcessRequest{
				Format: "yaml",
				ConfigData: []byte(`
name: private-app
version: "2.0.0"
access: private
components:
  - id: backend
    source:
      registry: ghcr.io
      package: test/backend
      version: v2.0.0
`),
			},
			wantErr: false,
			checks: func(t *testing.T, result *ProcessResult) {
				if !strings.Contains(result.SpinTOML, "mcp-gateway") {
					t.Error("SpinTOML should contain mcp-gateway")
				}
				if !strings.Contains(result.SpinTOML, "mcp-authorizer") {
					t.Error("SpinTOML should contain mcp-authorizer for private app")
				}
				if result.Metadata.AccessMode != "private" {
					t.Errorf("expected access mode 'private', got %s", result.Metadata.AccessMode)
				}
				if !result.Metadata.InjectedAuthorizer {
					t.Error("authorizer should be injected for private app")
				}
			},
		},
		{
			name: "reject local sources in production",
			config: Config{
				RequireRegistryComponents: true,
				AllowedRegistries:         []string{"ghcr.io"},
			},
			request: ProcessRequest{
				Format: "yaml",
				ConfigData: []byte(`
name: local-app
components:
  - id: local-component
    source: ./build/component.wasm
`),
			},
			wantErr: true,
		},
		{
			name: "enforce registry whitelist",
			config: Config{
				RequireRegistryComponents: true,
				AllowedRegistries:         []string{"ghcr.io"},
				GatewayRegistry:           "ghcr.io",
				GatewayPackage:            "fastertools:mcp-gateway",
				GatewayVersion:            "0.0.13-alpha.0",
			},
			request: ProcessRequest{
				Format: "yaml",
				ConfigData: []byte(`
name: external-registry
components:
  - id: external
    source:
      registry: docker.io
      package: untrusted/component
      version: latest
`),
			},
			wantErr: true,
		},
		{
			name:   "org access with allowed subjects",
			config: DefaultConfig(),
			request: ProcessRequest{
				Format: "yaml",
				ConfigData: []byte(`
name: org-app
access: org
components:
  - id: org-service
    source:
      registry: ghcr.io
      package: test/service
      version: v1.0.0
`),
				AllowedSubjects: []string{"user_123", "user_456"},
			},
			wantErr: false,
			checks: func(t *testing.T, result *ProcessResult) {
				if !strings.Contains(result.SpinTOML, "mcp-authorizer") {
					t.Error("SpinTOML should contain mcp-authorizer for org app")
				}
				// The allowed_subjects should be passed through to CUE
				if result.Metadata.AccessMode != "org" {
					t.Errorf("expected access mode 'org', got %s", result.Metadata.AccessMode)
				}
			},
		},
		{
			name:   "org access with allowed roles",
			config: DefaultConfig(),
			request: ProcessRequest{
				Format: "yaml",
				ConfigData: []byte(`
name: org-roles-app
access: org
allowed_roles: ["admin", "developer"]
components:
  - id: admin-service
    source:
      registry: ghcr.io
      package: test/admin
      version: v1.0.0
`),
				// Platform would compute this by calling WorkOS and filtering by roles
				AllowedSubjects: []string{"admin_user_001", "dev_user_002"},
			},
			wantErr: false,
			checks: func(t *testing.T, result *ProcessResult) {
				if !strings.Contains(result.SpinTOML, "mcp-authorizer") {
					t.Error("SpinTOML should contain mcp-authorizer for org app")
				}
				if result.Metadata.AccessMode != "org" {
					t.Errorf("expected access mode 'org', got %s", result.Metadata.AccessMode)
				}
				if len(result.Metadata.AllowedRoles) != 2 {
					t.Errorf("expected 2 allowed roles, got %d", len(result.Metadata.AllowedRoles))
				}
				if result.Metadata.AllowedRoles[0] != "admin" || result.Metadata.AllowedRoles[1] != "developer" {
					t.Errorf("expected roles [admin, developer], got %v", result.Metadata.AllowedRoles)
				}
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			processor := NewProcessor(tt.config)
			result, err := processor.Process(tt.request)

			if (err != nil) != tt.wantErr {
				t.Errorf("Process() error = %v, wantErr %v", err, tt.wantErr)
				return
			}

			if !tt.wantErr && tt.checks != nil {
				tt.checks(t, result)
			}
		})
	}
}

func TestDefaultConfig(t *testing.T) {
	config := DefaultConfig()

	if config.GatewayRegistry != "ghcr.io" {
		t.Errorf("expected gateway registry 'ghcr.io', got %s", config.GatewayRegistry)
	}

	if !config.RequireRegistryComponents {
		t.Error("RequireRegistryComponents should be true by default")
	}

	if len(config.AllowedRegistries) != 1 || config.AllowedRegistries[0] != "ghcr.io" {
		t.Error("default allowed registries should be [ghcr.io]")
	}
}
