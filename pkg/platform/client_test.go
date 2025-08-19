package platform

import (
	"strings"
	"testing"
	"time"
)

func TestNewClient(t *testing.T) {
	config := DefaultConfig()
	client := NewClient(config)
	
	if client == nil {
		t.Fatal("NewClient returned nil")
	}
	
	if client.config.GatewayVersion != "0.0.13-alpha.0" {
		t.Errorf("unexpected gateway version: %s", client.config.GatewayVersion)
	}
}

func TestProcessDeployment(t *testing.T) {
	tests := []struct {
		name    string
		config  Config
		request *DeploymentRequest
		wantErr bool
		checks  func(t *testing.T, result *DeploymentResult)
	}{
		{
			name:   "public app with gateway injection",
			config: DefaultConfig(),
			request: &DeploymentRequest{
				Application: &Application{
					Name:    "test-app",
					Version: "1.0.0",
					Access:  "public",
					Components: []Component{
						{
							ID: "test-component",
							Source: map[string]interface{}{
								"registry": "ghcr.io",
								"package":  "test/component",
								"version":  "1.0.0",
							},
						},
					},
				},
				Environment: "production",
			},
			wantErr: false,
			checks: func(t *testing.T, result *DeploymentResult) {
				// Should inject gateway but not authorizer for public apps
				if !result.Metadata.InjectedGateway {
					t.Error("gateway should be injected")
				}
				if result.Metadata.InjectedAuthorizer {
					t.Error("authorizer should not be injected for public apps")
				}
				if result.Metadata.ComponentCount != 2 { // gateway + test component
					t.Errorf("expected 2 components, got %d", result.Metadata.ComponentCount)
				}
				if result.Metadata.AccessMode != "public" {
					t.Errorf("expected access mode 'public', got %s", result.Metadata.AccessMode)
				}
			},
		},
		{
			name:   "private app with auth injection",
			config: DefaultConfig(),
			request: &DeploymentRequest{
				Application: &Application{
					Name:    "secure-app",
					Version: "2.0.0",
					Access:  "private",
					Auth: &Auth{
						JWTIssuer:   "https://auth.example.com",
						JWTAudience: "api.example.com",
					},
					Components: []Component{
						{
							ID: "secure-component",
							Source: map[string]interface{}{
								"registry": "ghcr.io",
								"package":  "test/secure",
								"version":  "2.0.0",
							},
						},
					},
				},
			},
			wantErr: false,
			checks: func(t *testing.T, result *DeploymentResult) {
				if !result.Metadata.InjectedGateway {
					t.Error("gateway should be injected")
				}
				if !result.Metadata.InjectedAuthorizer {
					t.Error("authorizer should be injected for private apps")
				}
				if result.Metadata.ComponentCount != 3 { // gateway + authorizer + component
					t.Errorf("expected 3 components, got %d", result.Metadata.ComponentCount)
				}
			},
		},
		{
			name: "reject local sources when configured",
			config: Config{
				RequireRegistryComponents: true,
				MaxComponents:            50,
			},
			request: &DeploymentRequest{
				Application: &Application{
					Name:    "local-app",
					Version: "1.0.0",
					Components: []Component{
						{
							ID:     "local-component",
							Source: "./local/path",
						},
					},
				},
			},
			wantErr: true,
		},
		{
			name: "enforce registry whitelist",
			config: Config{
				RequireRegistryComponents: true,
				AllowedRegistries:        []string{"ghcr.io"},
				MaxComponents:            50,
			},
			request: &DeploymentRequest{
				Application: &Application{
					Name:    "restricted-app",
					Version: "1.0.0",
					Components: []Component{
						{
							ID: "bad-registry",
							Source: map[string]interface{}{
								"registry": "unauthorized.com",
								"package":  "bad/component",
								"version":  "1.0.0",
							},
						},
					},
				},
			},
			wantErr: true,
		},
		{
			name: "enforce component limit",
			config: Config{
				MaxComponents: 2,
			},
			request: &DeploymentRequest{
				Application: &Application{
					Name:    "too-many-components",
					Version: "1.0.0",
					Components: []Component{
						{ID: "comp1", Source: map[string]interface{}{"registry": "ghcr.io", "package": "test/c1", "version": "1.0.0"}},
						{ID: "comp2", Source: map[string]interface{}{"registry": "ghcr.io", "package": "test/c2", "version": "1.0.0"}},
						{ID: "comp3", Source: map[string]interface{}{"registry": "ghcr.io", "package": "test/c3", "version": "1.0.0"}},
					},
				},
			},
			wantErr: true,
		},
		{
			name:   "custom auth configuration",
			config: DefaultConfig(),
			request: &DeploymentRequest{
				Application: &Application{
					Name:    "custom-auth-app",
					Version: "1.0.0",
					Access:  "custom",
					Components: []Component{
						{
							ID: "auth-component",
							Source: map[string]interface{}{
								"registry": "ghcr.io",
								"package":  "test/auth",
								"version":  "1.0.0",
							},
						},
					},
				},
				CustomAuth: &CustomAuthConfig{
					Issuer:   "https://custom-auth.example.com",
					Audience: []string{"custom-api.example.com"},
				},
			},
			wantErr: false,
			checks: func(t *testing.T, result *DeploymentResult) {
				if !result.Metadata.InjectedAuthorizer {
					t.Error("authorizer should be injected for custom auth")
				}
				if result.Metadata.AccessMode != "custom" {
					t.Errorf("expected access mode 'custom', got %s", result.Metadata.AccessMode)
				}
			},
		},
		{
			name:   "org access with allowed subjects",
			config: DefaultConfig(),
			request: &DeploymentRequest{
				Application: &Application{
					Name:    "org-app",
					Version: "1.0.0",
					Access:  "org",
					Auth: &Auth{
						OrgID:     "org_123456",
						JWTIssuer: "https://api.workos.com",
					},
					Components: []Component{
						{
							ID: "org-component",
							Source: map[string]interface{}{
								"registry": "ghcr.io",
								"package":  "test/org",
								"version":  "1.0.0",
							},
						},
					},
				},
				AllowedSubjects: []string{"user_01234", "user_56789", "user_abcde"},
				AllowedRoles:    []string{"admin", "developer"},
			},
			wantErr: false,
			checks: func(t *testing.T, result *DeploymentResult) {
				if !result.Metadata.InjectedAuthorizer {
					t.Error("authorizer should be injected for org access")
				}
				if result.Metadata.AccessMode != "org" {
					t.Errorf("expected access mode 'org', got %s", result.Metadata.AccessMode)
				}
				
				// Verify the allowed subjects were injected into the authorizer variables
				if result.SpinTOML == "" {
					t.Error("SpinTOML should not be empty")
				}
				// Check that the TOML contains the allowed subjects variable
				expectedSubjects := "mcp_auth_allowed_subjects = 'user_01234,user_56789,user_abcde'"
				if !strings.Contains(result.SpinTOML, expectedSubjects) {
					t.Errorf("SpinTOML should contain allowed subjects variable: %s", expectedSubjects)
				}
			},
		},
		{
			name:   "deployment variables applied",
			config: DefaultConfig(),
			request: &DeploymentRequest{
				Application: &Application{
					Name:    "var-app",
					Version: "1.0.0",
					Components: []Component{
						{
							ID: "var-component",
							Source: map[string]interface{}{
								"registry": "ghcr.io",
								"package":  "test/vars",
								"version":  "1.0.0",
							},
						},
					},
				},
				Variables: map[string]string{
					"API_KEY":    "secret123",
					"ENVIRONMENT": "production",
				},
			},
			wantErr: false,
			checks: func(t *testing.T, result *DeploymentResult) {
				if result.Manifest.Variables == nil {
					t.Fatal("variables should be set")
				}
				if result.Manifest.Variables["API_KEY"].Default != "secret123" {
					t.Error("API_KEY variable not set correctly")
				}
				if result.Manifest.Variables["ENVIRONMENT"].Default != "production" {
					t.Error("ENVIRONMENT variable not set correctly")
				}
			},
		},
	}
	
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			client := NewClient(tt.config)
			result, err := client.ProcessDeployment(tt.request)
			
			if (err != nil) != tt.wantErr {
				t.Errorf("ProcessDeployment() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			
			if !tt.wantErr && tt.checks != nil {
				tt.checks(t, result)
				
				// Common checks
				if result.SpinTOML == "" {
					t.Error("SpinTOML should not be empty")
				}
				if result.Manifest == nil {
					t.Error("Manifest should not be nil")
				}
				if result.Metadata.ProcessedAt.IsZero() {
					t.Error("ProcessedAt should be set")
				}
				if result.Metadata.ProcessedAt.After(time.Now()) {
					t.Error("ProcessedAt should not be in the future")
				}
			}
		})
	}
}

func TestValidateComponents(t *testing.T) {
	tests := []struct {
		name       string
		config     Config
		components []Component
		wantErr    bool
	}{
		{
			name:   "valid registry components",
			config: DefaultConfig(),
			components: []Component{
				{
					ID: "test",
					Source: map[string]interface{}{
						"registry": "ghcr.io",
						"package":  "test/component",
						"version":  "1.0.0",
					},
				},
			},
			wantErr: false,
		},
		{
			name: "reject local when required",
			config: Config{
				RequireRegistryComponents: true,
			},
			components: []Component{
				{
					ID:     "local",
					Source: "./local/path",
				},
			},
			wantErr: true,
		},
		{
			name: "component without ID",
			config: DefaultConfig(),
			components: []Component{
				{
					Source: map[string]interface{}{
						"registry": "ghcr.io",
						"package":  "test/component",
						"version":  "1.0.0",
					},
				},
			},
			wantErr: true,
		},
	}
	
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			client := NewClient(tt.config)
			err := client.ValidateComponents(tt.components)
			if (err != nil) != tt.wantErr {
				t.Errorf("ValidateComponents() error = %v, wantErr %v", err, tt.wantErr)
			}
		})
	}
}

func TestPlatformComponentInjection(t *testing.T) {
	config := DefaultConfig()
	client := NewClient(config)
	
	// Test app that should get both gateway and authorizer
	app := &Application{
		Name:    "test-app",
		Version: "1.0.0",
		Access:  "private",
		Components: []Component{
			{
				ID:     "user-component",
				Source: map[string]interface{}{"registry": "ghcr.io", "package": "test/comp", "version": "1.0.0"},
			},
		},
	}
	
	// Create a dummy request for testing
	req := &DeploymentRequest{
		Application: app,
	}
	client.injectPlatformComponents(app, req)
	
	// Should have 3 components: gateway, authorizer, user component
	if len(app.Components) != 3 {
		t.Fatalf("expected 3 components, got %d", len(app.Components))
	}
	
	// First should be gateway
	if app.Components[0].ID != "mcp-gateway" {
		t.Errorf("first component should be mcp-gateway, got %s", app.Components[0].ID)
	}
	
	// Second should be authorizer
	if app.Components[1].ID != "mcp-authorizer" {
		t.Errorf("second component should be mcp-authorizer, got %s", app.Components[1].ID)
	}
	
	// Third should be user component
	if app.Components[2].ID != "user-component" {
		t.Errorf("third component should be user-component, got %s", app.Components[2].ID)
	}
	
	// Test public app (should only get gateway)
	publicApp := &Application{
		Name:    "public-app",
		Version: "1.0.0",
		Access:  "public",
		Components: []Component{
			{
				ID:     "public-component",
				Source: map[string]interface{}{"registry": "ghcr.io", "package": "test/pub", "version": "1.0.0"},
			},
		},
	}
	
	// Create a dummy request for testing public app
	publicReq := &DeploymentRequest{
		Application: publicApp,
	}
	client.injectPlatformComponents(publicApp, publicReq)
	
	// Should have 2 components: gateway, user component
	if len(publicApp.Components) != 2 {
		t.Fatalf("expected 2 components for public app, got %d", len(publicApp.Components))
	}
	
	if publicApp.Components[0].ID != "mcp-gateway" {
		t.Errorf("first component should be mcp-gateway, got %s", publicApp.Components[0].ID)
	}
	
	if publicApp.Components[1].ID != "public-component" {
		t.Errorf("second component should be public-component, got %s", publicApp.Components[1].ID)
	}
}