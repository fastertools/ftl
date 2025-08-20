package platform

import (
	"encoding/json"
	"testing"

	"github.com/BurntSushi/toml"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestProcessor_RequiredClaimsWithDeploymentContext(t *testing.T) {
	processor := NewProcessor(DefaultConfig())

	tests := []struct {
		name    string
		request ProcessRequest
		check   func(t *testing.T, result *ProcessResult)
	}{
		{
			name: "M2M deployment to org-scoped app",
			request: ProcessRequest{
				ConfigData: []byte(`
name: test-app
access: org
components: []
`),
				Format: "yaml",
				DeploymentContext: &DeploymentContext{
					ActorType: "machine",
					OrgID:     "org_test123",
				},
			},
			check: func(t *testing.T, result *ProcessResult) {
				require.NotNil(t, result)
				require.NotEmpty(t, result.SpinTOML)

				// Parse and check for org_id claim requirement
				var manifest map[string]interface{}
				require.NoError(t, toml.Unmarshal([]byte(result.SpinTOML), &manifest))

				components := manifest["component"].(map[string]interface{})
				authorizer, ok := components["mcp-authorizer"].(map[string]interface{})
				require.True(t, ok, "mcp-authorizer should exist for org mode")

				variables := authorizer["variables"].(map[string]interface{})
				requiredClaims, ok := variables["mcp_auth_required_claims"].(string)
				require.True(t, ok, "required_claims should be set for M2M to org app")

				var claims map[string]interface{}
				require.NoError(t, json.Unmarshal([]byte(requiredClaims), &claims))
				assert.Equal(t, "org_test123", claims["org_id"])
			},
		},
		{
			name: "User deployment with claim forwarding",
			request: ProcessRequest{
				ConfigData: []byte(`
name: test-app
access: private
components: []
`),
				Format: "yaml",
				DeploymentContext: &DeploymentContext{
					ActorType: "user",
					ForwardClaims: map[string]string{
						"sub":   "X-User-ID",
						"email": "X-User-Email",
						"name":  "X-User-Name",
					},
				},
			},
			check: func(t *testing.T, result *ProcessResult) {
				require.NotNil(t, result)
				require.NotEmpty(t, result.SpinTOML)

				var manifest map[string]interface{}
				require.NoError(t, toml.Unmarshal([]byte(result.SpinTOML), &manifest))

				components := manifest["component"].(map[string]interface{})
				authorizer := components["mcp-authorizer"].(map[string]interface{})
				variables := authorizer["variables"].(map[string]interface{})

				forwardClaims, ok := variables["mcp_auth_forward_claims"].(string)
				require.True(t, ok, "forward_claims should be set")

				var claims map[string]string
				require.NoError(t, json.Unmarshal([]byte(forwardClaims), &claims))
				assert.Equal(t, "X-User-ID", claims["sub"])
				assert.Equal(t, "X-User-Email", claims["email"])
				assert.Equal(t, "X-User-Name", claims["name"])
			},
		},
		{
			name: "M2M with both org_id and claim forwarding",
			request: ProcessRequest{
				ConfigData: []byte(`
name: test-app
access: org
components: []
`),
				Format: "yaml",
				DeploymentContext: &DeploymentContext{
					ActorType: "machine",
					OrgID:     "org_abc789",
					ForwardClaims: map[string]string{
						"org_id": "X-Org-ID",
						"sub":    "X-Machine-ID",
					},
				},
			},
			check: func(t *testing.T, result *ProcessResult) {
				require.NotNil(t, result)

				var manifest map[string]interface{}
				require.NoError(t, toml.Unmarshal([]byte(result.SpinTOML), &manifest))

				components := manifest["component"].(map[string]interface{})
				authorizer := components["mcp-authorizer"].(map[string]interface{})
				variables := authorizer["variables"].(map[string]interface{})

				// Check required claims
				requiredClaims, ok := variables["mcp_auth_required_claims"].(string)
				require.True(t, ok)
				var reqClaims map[string]interface{}
				require.NoError(t, json.Unmarshal([]byte(requiredClaims), &reqClaims))
				assert.Equal(t, "org_abc789", reqClaims["org_id"])

				// Check forward claims
				forwardClaims, ok := variables["mcp_auth_forward_claims"].(string)
				require.True(t, ok)
				var fwdClaims map[string]string
				require.NoError(t, json.Unmarshal([]byte(forwardClaims), &fwdClaims))
				assert.Equal(t, "X-Org-ID", fwdClaims["org_id"])
				assert.Equal(t, "X-Machine-ID", fwdClaims["sub"])
			},
		},
		{
			name: "User deployment to org app without org_id requirement",
			request: ProcessRequest{
				ConfigData: []byte(`
name: test-app
access: org
components: []
`),
				Format: "yaml",
				DeploymentContext: &DeploymentContext{
					ActorType: "user",
					OrgID:     "org_user123",
				},
			},
			check: func(t *testing.T, result *ProcessResult) {
				require.NotNil(t, result)

				var manifest map[string]interface{}
				require.NoError(t, toml.Unmarshal([]byte(result.SpinTOML), &manifest))

				components := manifest["component"].(map[string]interface{})
				authorizer := components["mcp-authorizer"].(map[string]interface{})
				variables := authorizer["variables"].(map[string]interface{})

				// Users should NOT have org_id requirement automatically added
				_, hasRequiredClaims := variables["mcp_auth_required_claims"]
				assert.False(t, hasRequiredClaims, "user deployments should not auto-inject org_id requirement")
			},
		},
		{
			name: "Public app ignores deployment context",
			request: ProcessRequest{
				ConfigData: []byte(`
name: test-app
access: public
components: []
`),
				Format: "yaml",
				DeploymentContext: &DeploymentContext{
					ActorType: "machine",
					OrgID:     "org_ignored",
					ForwardClaims: map[string]string{
						"sub": "X-User-ID",
					},
				},
			},
			check: func(t *testing.T, result *ProcessResult) {
				require.NotNil(t, result)

				var manifest map[string]interface{}
				require.NoError(t, toml.Unmarshal([]byte(result.SpinTOML), &manifest))

				components := manifest["component"].(map[string]interface{})
				_, hasAuthorizer := components["mcp-authorizer"]
				assert.False(t, hasAuthorizer, "public apps should not have authorizer")
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result, err := processor.Process(tt.request)
			require.NoError(t, err)
			tt.check(t, result)
		})
	}
}

func TestProcessor_UserRequiredClaimsWithPlatformContext(t *testing.T) {
	processor := NewProcessor(DefaultConfig())

	// Test that user-specified required_claims are preserved and merged with platform context
	request := ProcessRequest{
		ConfigData: []byte(`
name: test-app
access: org
required_claims:
  role: admin
  department: engineering
components: []
`),
		Format: "yaml",
		DeploymentContext: &DeploymentContext{
			ActorType: "machine",
			OrgID:     "org_platform",
		},
	}

	result, err := processor.Process(request)
	require.NoError(t, err)
	require.NotNil(t, result)

	var manifest map[string]interface{}
	require.NoError(t, toml.Unmarshal([]byte(result.SpinTOML), &manifest))

	components := manifest["component"].(map[string]interface{})
	authorizer := components["mcp-authorizer"].(map[string]interface{})
	variables := authorizer["variables"].(map[string]interface{})

	requiredClaims, ok := variables["mcp_auth_required_claims"].(string)
	require.True(t, ok)

	var claims map[string]interface{}
	require.NoError(t, json.Unmarshal([]byte(requiredClaims), &claims))

	// Should have both user claims AND platform-injected org_id
	assert.Equal(t, "admin", claims["role"])
	assert.Equal(t, "engineering", claims["department"])
	assert.Equal(t, "org_platform", claims["org_id"])
}

func TestProcessor_ComplexRequiredClaims(t *testing.T) {
	processor := NewProcessor(DefaultConfig())

	request := ProcessRequest{
		ConfigData: []byte(`
name: test-app
access: private
required_claims:
  verified: true
  clearance_level: 5
  permissions:
    - read
    - write
    - admin
  metadata:
    team: platform
    location: sf
components: []
`),
		Format: "yaml",
	}

	result, err := processor.Process(request)
	require.NoError(t, err)
	require.NotNil(t, result)

	// Verify the complex claims are properly encoded
	assert.Contains(t, result.SpinTOML, "mcp_auth_required_claims")

	var manifest map[string]interface{}
	require.NoError(t, toml.Unmarshal([]byte(result.SpinTOML), &manifest))

	components := manifest["component"].(map[string]interface{})
	authorizer := components["mcp-authorizer"].(map[string]interface{})
	variables := authorizer["variables"].(map[string]interface{})

	requiredClaims := variables["mcp_auth_required_claims"].(string)
	var claims map[string]interface{}
	require.NoError(t, json.Unmarshal([]byte(requiredClaims), &claims))

	assert.Equal(t, true, claims["verified"])
	assert.EqualValues(t, 5, claims["clearance_level"])

	perms := claims["permissions"].([]interface{})
	assert.Len(t, perms, 3)
	assert.Contains(t, perms, "read")
	assert.Contains(t, perms, "write")
	assert.Contains(t, perms, "admin")

	metadata := claims["metadata"].(map[string]interface{})
	assert.Equal(t, "platform", metadata["team"])
	assert.Equal(t, "sf", metadata["location"])
}