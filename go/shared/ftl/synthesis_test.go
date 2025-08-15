package ftl

import (
	"testing"
)

func TestSynthesisWithAllAccessModes(t *testing.T) {
	tests := []struct {
		name    string
		app     Application
		wantErr bool
	}{
		{
			name: "public access",
			app: Application{
				Name:    "test-app",
				Version: "1.0.0",
				Access:  AccessPublic,
			},
			wantErr: false,
		},
		{
			name: "private access with WorkOS",
			app: Application{
				Name:    "test-app",
				Version: "1.0.0",
				Access:  AccessPrivate,
				Auth: AuthConfig{
					Provider: AuthProviderWorkOS,
				},
			},
			wantErr: false,
		},
		{
			name: "org access with WorkOS",
			app: Application{
				Name:    "test-app",
				Version: "1.0.0",
				Access:  AccessOrg,
				Auth: AuthConfig{
					Provider: AuthProviderWorkOS,
					OrgID:    "org_123",
				},
			},
			wantErr: false,
		},
		{
			name: "custom access with JWT",
			app: Application{
				Name:    "test-app",
				Version: "1.0.0",
				Access:  AccessCustom,
				Auth: AuthConfig{
					Provider:    AuthProviderCustom,
					JWTIssuer:   "https://auth.example.com",
					JWTAudience: "api.example.com",
				},
			},
			wantErr: false,
		},
	}

	synth := NewSynthesizer()
	
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			tt.app.SetDefaults()
			
			// Test validation
			if err := tt.app.Validate(); err != nil {
				if !tt.wantErr {
					t.Errorf("Validate() error = %v, wantErr %v", err, tt.wantErr)
				}
				return
			}
			
			// Test synthesis
			manifest, err := synth.SynthesizeToSpin(&tt.app)
			if (err != nil) != tt.wantErr {
				t.Errorf("SynthesizeToSpin() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			
			if err == nil {
				// Verify manifest version is set
				if manifest.SpinManifestVersion != 2 {
					t.Errorf("Expected SpinManifestVersion = 2, got %d", manifest.SpinManifestVersion)
				}
				
				// Verify auth component is added for non-public access
				if tt.app.Access != AccessPublic {
					if _, ok := manifest.Component["mcp-authorizer"]; !ok {
						t.Error("Expected mcp-authorizer component for non-public access")
					}
				}
			}
		})
	}
}

func TestApplicationVariables(t *testing.T) {
	app := Application{
		Name:    "test-app",
		Version: "1.0.0",
		Access:  AccessPublic,
		Variables: map[string]string{
			"API_KEY": "secret123",
			"DEBUG":   "true",
		},
	}
	
	app.SetDefaults()
	if err := app.Validate(); err != nil {
		t.Fatalf("Validation failed: %v", err)
	}
	
	// Variables should be preserved
	if len(app.Variables) != 2 {
		t.Errorf("Expected 2 variables, got %d", len(app.Variables))
	}
}

func TestComponentSourceHelpers(t *testing.T) {
	// Test registry source
	regSource := &RegistrySource{
		Registry: "ghcr.io",
		Package:  "test:component",
		Version:  "1.0.0",
	}
	
	reg, ok := AsRegistry(regSource)
	if !ok || reg == nil {
		t.Error("AsRegistry failed for RegistrySource")
	}
	
	// Test local source
	localSource := LocalSource("./test.wasm")
	
	path, ok := AsLocal(localSource)
	if !ok || path != "./test.wasm" {
		t.Error("AsLocal failed for LocalSource")
	}
	
	// Test wrong casts
	if _, ok := AsRegistry(localSource); ok {
		t.Error("AsRegistry should fail for LocalSource")
	}
	
	if _, ok := AsLocal(regSource); ok {
		t.Error("AsLocal should fail for RegistrySource")
	}
}