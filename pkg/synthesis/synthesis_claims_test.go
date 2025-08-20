package synthesis

import (
	"encoding/json"
	"testing"

	"github.com/BurntSushi/toml"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"gopkg.in/yaml.v3"
)

func TestSynthesizer_RequiredClaims(t *testing.T) {
	tests := []struct {
		name     string
		yaml     string
		override map[string]interface{}
		check    func(t *testing.T, spinTOML string)
	}{
		{
			name: "org mode with user required claims",
			yaml: `
name: test-app
access: org
required_claims:
  role: admin
  department: engineering
components: []
`,
			check: func(t *testing.T, spinTOML string) {
				// Parse the TOML to check the mcp-authorizer configuration
				var manifest map[string]interface{}
				require.NoError(t, toml.Unmarshal([]byte(spinTOML), &manifest))
				
				components := manifest["component"].(map[string]interface{})
				authorizer := components["mcp-authorizer"].(map[string]interface{})
				variables := authorizer["variables"].(map[string]interface{})
				
				// Check that required_claims is set
				requiredClaims, ok := variables["mcp_auth_required_claims"].(string)
				require.True(t, ok, "mcp_auth_required_claims should be set")
				
				// Parse the JSON string
				var claims map[string]interface{}
				require.NoError(t, json.Unmarshal([]byte(requiredClaims), &claims))
				
				assert.Equal(t, "admin", claims["role"])
				assert.Equal(t, "engineering", claims["department"])
			},
		},
		{
			name: "org mode with M2M deployment context",
			yaml: `
name: test-app
access: org
components: []
`,
			override: map[string]interface{}{
				"deployment_context": map[string]interface{}{
					"actor_type": "machine",
					"org_id":     "org_12345",
				},
			},
			check: func(t *testing.T, spinTOML string) {
				var manifest map[string]interface{}
				require.NoError(t, toml.Unmarshal([]byte(spinTOML), &manifest))
				
				components := manifest["component"].(map[string]interface{})
				authorizer := components["mcp-authorizer"].(map[string]interface{})
				variables := authorizer["variables"].(map[string]interface{})
				
				// Check that org_id is required for M2M
				requiredClaims, ok := variables["mcp_auth_required_claims"].(string)
				require.True(t, ok, "mcp_auth_required_claims should be set for M2M")
				
				var claims map[string]interface{}
				require.NoError(t, json.Unmarshal([]byte(requiredClaims), &claims))
				
				assert.Equal(t, "org_12345", claims["org_id"])
			},
		},
		{
			name: "org mode with both user claims and M2M context",
			yaml: `
name: test-app
access: org
required_claims:
  role: admin
  team: platform
components: []
`,
			override: map[string]interface{}{
				"deployment_context": map[string]interface{}{
					"actor_type": "machine",
					"org_id":     "org_67890",
				},
			},
			check: func(t *testing.T, spinTOML string) {
				var manifest map[string]interface{}
				require.NoError(t, toml.Unmarshal([]byte(spinTOML), &manifest))
				
				components := manifest["component"].(map[string]interface{})
				authorizer := components["mcp-authorizer"].(map[string]interface{})
				variables := authorizer["variables"].(map[string]interface{})
				
				requiredClaims, ok := variables["mcp_auth_required_claims"].(string)
				require.True(t, ok)
				
				var claims map[string]interface{}
				require.NoError(t, json.Unmarshal([]byte(requiredClaims), &claims))
				
				// Should have both user-specified and M2M claims
				assert.Equal(t, "admin", claims["role"])
				assert.Equal(t, "platform", claims["team"])
				assert.Equal(t, "org_67890", claims["org_id"])
			},
		},
		{
			name: "private mode with required claims",
			yaml: `
name: test-app
access: private
required_claims:
  verified: true
  beta_access: true
components: []
`,
			check: func(t *testing.T, spinTOML string) {
				var manifest map[string]interface{}
				require.NoError(t, toml.Unmarshal([]byte(spinTOML), &manifest))
				
				components := manifest["component"].(map[string]interface{})
				authorizer := components["mcp-authorizer"].(map[string]interface{})
				variables := authorizer["variables"].(map[string]interface{})
				
				requiredClaims, ok := variables["mcp_auth_required_claims"].(string)
				require.True(t, ok)
				
				var claims map[string]interface{}
				require.NoError(t, json.Unmarshal([]byte(requiredClaims), &claims))
				
				assert.Equal(t, true, claims["verified"])
				assert.Equal(t, true, claims["beta_access"])
			},
		},
		{
			name: "public mode ignores required claims",
			yaml: `
name: test-app
access: public
required_claims:
  role: admin
components: []
`,
			check: func(t *testing.T, spinTOML string) {
				var manifest map[string]interface{}
				require.NoError(t, toml.Unmarshal([]byte(spinTOML), &manifest))
				
				components := manifest["component"].(map[string]interface{})
				
				// Should not have mcp-authorizer for public mode
				_, hasAuthorizer := components["mcp-authorizer"]
				assert.False(t, hasAuthorizer, "public mode should not include mcp-authorizer")
			},
		},
		{
			name: "claim forwarding with deployment context",
			yaml: `
name: test-app
access: org
components: []
`,
			override: map[string]interface{}{
				"deployment_context": map[string]interface{}{
					"actor_type": "user",
					"org_id":     "org_abc",
					"forward_claims": map[string]string{
						"sub":    "X-User-ID",
						"org_id": "X-Org-ID",
						"email":  "X-User-Email",
					},
				},
			},
			check: func(t *testing.T, spinTOML string) {
				var manifest map[string]interface{}
				require.NoError(t, toml.Unmarshal([]byte(spinTOML), &manifest))
				
				components := manifest["component"].(map[string]interface{})
				authorizer := components["mcp-authorizer"].(map[string]interface{})
				variables := authorizer["variables"].(map[string]interface{})
				
				forwardClaims, ok := variables["mcp_auth_forward_claims"].(string)
				require.True(t, ok, "mcp_auth_forward_claims should be set")
				
				var claims map[string]string
				require.NoError(t, json.Unmarshal([]byte(forwardClaims), &claims))
				
				assert.Equal(t, "X-User-ID", claims["sub"])
				assert.Equal(t, "X-Org-ID", claims["org_id"])
				assert.Equal(t, "X-User-Email", claims["email"])
			},
		},
		{
			name: "complex required claims with arrays",
			yaml: `
name: test-app
access: org
required_claims:
  role: admin
  permissions:
    - read
    - write
    - delete
  metadata:
    level: 5
    verified: true
components: []
`,
			check: func(t *testing.T, spinTOML string) {
				var manifest map[string]interface{}
				require.NoError(t, toml.Unmarshal([]byte(spinTOML), &manifest))
				
				components := manifest["component"].(map[string]interface{})
				authorizer := components["mcp-authorizer"].(map[string]interface{})
				variables := authorizer["variables"].(map[string]interface{})
				
				requiredClaims, ok := variables["mcp_auth_required_claims"].(string)
				require.True(t, ok)
				
				var claims map[string]interface{}
				require.NoError(t, json.Unmarshal([]byte(requiredClaims), &claims))
				
				assert.Equal(t, "admin", claims["role"])
				
				perms, ok := claims["permissions"].([]interface{})
				require.True(t, ok)
				assert.Len(t, perms, 3)
				assert.Contains(t, perms, "read")
				assert.Contains(t, perms, "write")
				assert.Contains(t, perms, "delete")
				
				metadata, ok := claims["metadata"].(map[string]interface{})
				require.True(t, ok)
				assert.EqualValues(t, 5, metadata["level"])
				assert.Equal(t, true, metadata["verified"])
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			s := NewSynthesizer()
			
			var spinTOML string
			var err error
			
			if tt.override != nil {
				// Parse YAML first, then synthesize with overrides
				var data interface{}
				if err := yaml.Unmarshal([]byte(tt.yaml), &data); err != nil {
					require.NoError(t, err)
				}
				spinTOML, err = s.SynthesizeWithOverrides(data, tt.override)
			} else {
				spinTOML, err = s.SynthesizeYAML([]byte(tt.yaml))
			}
			
			require.NoError(t, err)
			require.NotEmpty(t, spinTOML)
			
			// Verify it's valid TOML
			var manifest interface{}
			require.NoError(t, toml.Unmarshal([]byte(spinTOML), &manifest))
			
			// Run specific checks
			if tt.check != nil {
				tt.check(t, spinTOML)
			}
		})
	}
}

// TestSynthesizer_RequiredClaimsJSON tests JSON input format
func TestSynthesizer_RequiredClaimsJSON(t *testing.T) {
	jsonConfig := `{
		"name": "test-app",
		"access": "org",
		"required_claims": {
			"role": "admin",
			"permissions": ["read", "write"]
		},
		"components": []
	}`

	s := NewSynthesizer()
	spinTOML, err := s.SynthesizeJSON([]byte(jsonConfig))
	require.NoError(t, err)
	require.NotEmpty(t, spinTOML)
	
	// Check that required claims are properly set
	assert.Contains(t, spinTOML, "mcp_auth_required_claims")
	
	// Parse and verify
	var manifest map[string]interface{}
	require.NoError(t, toml.Unmarshal([]byte(spinTOML), &manifest))
	
	components := manifest["component"].(map[string]interface{})
	authorizer := components["mcp-authorizer"].(map[string]interface{})
	variables := authorizer["variables"].(map[string]interface{})
	
	requiredClaims := variables["mcp_auth_required_claims"].(string)
	var claims map[string]interface{}
	require.NoError(t, json.Unmarshal([]byte(requiredClaims), &claims))
	
	assert.Equal(t, "admin", claims["role"])
	perms := claims["permissions"].([]interface{})
	assert.Len(t, perms, 2)
	assert.Contains(t, perms, "read")
	assert.Contains(t, perms, "write")
}

// TestSynthesizer_NoRequiredClaims ensures no claims are set when not specified
func TestSynthesizer_NoRequiredClaims(t *testing.T) {
	yaml := `
name: test-app
access: org
components: []
`

	s := NewSynthesizer()
	spinTOML, err := s.SynthesizeYAML([]byte(yaml))
	require.NoError(t, err)
	
	// Should have authorizer but no required_claims
	assert.Contains(t, spinTOML, "mcp-authorizer")
	assert.NotContains(t, spinTOML, "mcp_auth_required_claims")
}