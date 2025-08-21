package platform

import (
	"strings"
	"testing"

	"github.com/BurntSushi/toml"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestProcessorAuthModes(t *testing.T) {
	processor := NewProcessor(DefaultConfig())

	t.Run("Public Mode", func(t *testing.T) {
		req := ProcessRequest{
			Format: "yaml",
			ConfigData: []byte(`
name: public-app
access: public
components:
  - id: tool1
    source:
      registry: ghcr.io
      package: test/tool
      version: v1.0.0
`),
		}

		result, err := processor.Process(req)
		require.NoError(t, err)
		require.NotNil(t, result)

		// Verify no authorizer is injected for public mode
		assert.Equal(t, "public", result.Metadata.AccessMode)
		assert.False(t, result.Metadata.InjectedAuthorizer)
		assert.NotContains(t, result.SpinTOML, "mcp-authorizer")
		assert.Contains(t, result.SpinTOML, "mcp-gateway")
	})

	t.Run("Private Mode", func(t *testing.T) {
		req := ProcessRequest{
			Format: "yaml",
			ConfigData: []byte(`
name: private-app
access: private
components:
  - id: secure-tool
    source:
      registry: ghcr.io
      package: test/secure
      version: v1.0.0
`),
			AllowedSubjects: []string{"user_owner_123"},
		}

		result, err := processor.Process(req)
		require.NoError(t, err)
		require.NotNil(t, result)

		// Verify authorizer is injected with policy
		assert.Equal(t, "private", result.Metadata.AccessMode)
		assert.True(t, result.Metadata.InjectedAuthorizer)
		assert.Contains(t, result.SpinTOML, "mcp-authorizer")
		
		// Parse and verify policy variables
		var manifest map[string]interface{}
		require.NoError(t, toml.Unmarshal([]byte(result.SpinTOML), &manifest))
		
		components := manifest["component"].(map[string]interface{})
		authorizer := components["mcp-authorizer"].(map[string]interface{})
		variables := authorizer["variables"].(map[string]interface{})
		
		// Check that policy is injected
		policy, ok := variables["mcp_policy"].(string)
		assert.True(t, ok, "mcp_policy should be set")
		assert.Contains(t, policy, "package mcp.authorization")
		assert.Contains(t, policy, "input.token.sub == data.owner")
		
		// Check that policy data is injected
		policyData, ok := variables["mcp_policy_data"].(string)
		assert.True(t, ok, "mcp_policy_data should be set")
		assert.Contains(t, policyData, "user_owner_123")
	})

	t.Run("Organization Mode - User Deployment", func(t *testing.T) {
		req := ProcessRequest{
			Format: "yaml",
			ConfigData: []byte(`
name: org-app
access: org
components:
  - id: org-tool
    source:
      registry: ghcr.io
      package: test/org-tool
      version: v1.0.0
`),
			AllowedSubjects: []string{"user_alice", "user_bob", "user_charlie"},
			DeploymentContext: &DeploymentContext{
				ActorType: "user",
				OrgID:     "org_test123",
			},
		}

		result, err := processor.Process(req)
		require.NoError(t, err)
		require.NotNil(t, result)

		assert.Equal(t, "org", result.Metadata.AccessMode)
		assert.True(t, result.Metadata.InjectedAuthorizer)
		
		// Parse and verify
		var manifest map[string]interface{}
		require.NoError(t, toml.Unmarshal([]byte(result.SpinTOML), &manifest))
		
		components := manifest["component"].(map[string]interface{})
		authorizer := components["mcp-authorizer"].(map[string]interface{})
		variables := authorizer["variables"].(map[string]interface{})
		
		// Check policy handles both users and machines
		policy, ok := variables["mcp_policy"].(string)
		assert.True(t, ok)
		assert.Contains(t, policy, "not input.token.claims.org_id") // User check
		assert.Contains(t, policy, "input.token.claims.org_id == data.org_id") // Machine check
		
		// Check policy data has both members and org_id
		policyData, ok := variables["mcp_policy_data"].(string)
		assert.True(t, ok)
		assert.Contains(t, policyData, "org_test123")
		assert.Contains(t, policyData, "user_alice")
		assert.Contains(t, policyData, "user_bob")
		assert.Contains(t, policyData, "user_charlie")
	})

	t.Run("Organization Mode - Machine Deployment", func(t *testing.T) {
		req := ProcessRequest{
			Format: "yaml",
			ConfigData: []byte(`
name: machine-deployed-app
access: org
components:
  - id: ci-tool
    source:
      registry: ghcr.io
      package: test/ci-tool
      version: v1.0.0
`),
			// Machine deployments might not have user members yet
			AllowedSubjects: []string{},
			DeploymentContext: &DeploymentContext{
				ActorType: "machine",
				OrgID:     "org_machine123",
			},
		}

		result, err := processor.Process(req)
		require.NoError(t, err)
		require.NotNil(t, result)

		assert.Equal(t, "org", result.Metadata.AccessMode)
		assert.True(t, result.Metadata.InjectedAuthorizer)
		
		// Verify policy data includes org_id even without members
		var manifest map[string]interface{}
		require.NoError(t, toml.Unmarshal([]byte(result.SpinTOML), &manifest))
		
		components := manifest["component"].(map[string]interface{})
		authorizer := components["mcp-authorizer"].(map[string]interface{})
		variables := authorizer["variables"].(map[string]interface{})
		
		policyData, ok := variables["mcp_policy_data"].(string)
		assert.True(t, ok)
		assert.Contains(t, policyData, "org_machine123")
		assert.Contains(t, policyData, "members") // Should have empty members array
	})

	t.Run("Custom Mode", func(t *testing.T) {
		req := ProcessRequest{
			Format: "yaml",
			ConfigData: []byte(`
name: custom-app
access: custom
auth:
  jwt_issuer: "https://custom-auth.example.com"
  jwt_audience: "custom-api"
  jwt_jwks_uri: "https://custom-auth.example.com/.well-known/jwks.json"
  policy: |
    package mcp.authorization
    default allow = false
    allow {
      input.token.claims.role == "admin"
    }
  policy_data: |
    {"admin_roles": ["super_admin", "org_admin"]}
components:
  - id: custom-tool
    source:
      registry: ghcr.io
      package: test/custom
      version: v1.0.0
`),
		}

		result, err := processor.Process(req)
		require.NoError(t, err)
		require.NotNil(t, result)

		assert.Equal(t, "custom", result.Metadata.AccessMode)
		assert.True(t, result.Metadata.InjectedAuthorizer)
		
		// Verify custom auth config is passed through
		assert.Contains(t, result.SpinTOML, "custom-auth.example.com")
		assert.Contains(t, result.SpinTOML, "custom-api")
		
		// Parse and verify custom policy
		var manifest map[string]interface{}
		require.NoError(t, toml.Unmarshal([]byte(result.SpinTOML), &manifest))
		
		components := manifest["component"].(map[string]interface{})
		authorizer := components["mcp-authorizer"].(map[string]interface{})
		variables := authorizer["variables"].(map[string]interface{})
		
		// Check custom policy is passed through
		policy, ok := variables["mcp_policy"].(string)
		assert.True(t, ok)
		assert.Contains(t, policy, "input.token.claims.role == \"admin\"")
		
		// Check custom policy data
		policyData, ok := variables["mcp_policy_data"].(string)
		assert.True(t, ok)
		assert.Contains(t, policyData, "admin_roles")
	})
}

func TestProcessorEdgeCases(t *testing.T) {
	processor := NewProcessor(DefaultConfig())

	t.Run("Private Mode Without Subjects", func(t *testing.T) {
		req := ProcessRequest{
			Format: "yaml",
			ConfigData: []byte(`
name: private-no-owner
access: private
components:
  - id: tool
    source:
      registry: ghcr.io
      package: test/tool
      version: v1.0.0
`),
			AllowedSubjects: []string{}, // No owner provided
		}

		result, err := processor.Process(req)
		require.NoError(t, err)
		
		// Should still inject authorizer but no policy
		assert.True(t, result.Metadata.InjectedAuthorizer)
		
		// Policy shouldn't be generated without an owner
		var manifest map[string]interface{}
		require.NoError(t, toml.Unmarshal([]byte(result.SpinTOML), &manifest))
		
		components := manifest["component"].(map[string]interface{})
		authorizer := components["mcp-authorizer"].(map[string]interface{})
		variables := authorizer["variables"].(map[string]interface{})
		
		// Should not have policy without owner
		_, hasPolicy := variables["mcp_policy"]
		assert.False(t, hasPolicy, "Should not generate policy without owner")
	})

	t.Run("Org Mode Without Context", func(t *testing.T) {
		req := ProcessRequest{
			Format: "yaml",
			ConfigData: []byte(`
name: org-no-context
access: org
components:
  - id: tool
    source:
      registry: ghcr.io
      package: test/tool
      version: v1.0.0
`),
			AllowedSubjects: []string{"user_1"},
			// No deployment context
		}

		result, err := processor.Process(req)
		require.NoError(t, err)
		
		// Should still work but without org_id in policy data
		assert.True(t, result.Metadata.InjectedAuthorizer)
	})

	t.Run("Invalid Access Mode Falls Back", func(t *testing.T) {
		req := ProcessRequest{
			Format: "yaml",
			ConfigData: []byte(`
name: default-app
components:
  - id: tool
    source:
      registry: ghcr.io
      package: test/tool
      version: v1.0.0
`),
		}

		result, err := processor.Process(req)
		require.NoError(t, err)
		
		// Should default to public
		assert.Equal(t, "public", result.Metadata.AccessMode)
		assert.False(t, result.Metadata.InjectedAuthorizer)
	})

	t.Run("Component Registry Validation", func(t *testing.T) {
		processor := NewProcessor(Config{
			GatewayVersion:            "0.0.13-alpha.0",
			AuthorizerVersion:         "0.0.15-alpha.0",
			RequireRegistryComponents: true,
			AllowedRegistries:         []string{"ghcr.io"},
		})

		t.Run("Reject Local Sources", func(t *testing.T) {
			req := ProcessRequest{
				Format: "yaml",
				ConfigData: []byte(`
name: local-app
components:
  - id: local-tool
    source: ./local.wasm
`),
			}

			_, err := processor.Process(req)
			assert.Error(t, err)
			assert.Contains(t, err.Error(), "local component sources not allowed")
		})

		t.Run("Reject Disallowed Registry", func(t *testing.T) {
			req := ProcessRequest{
				Format: "yaml",
				ConfigData: []byte(`
name: bad-registry-app
components:
  - id: tool
    source:
      registry: evil.registry.com
      package: bad/tool
      version: v1.0.0
`),
			}

			_, err := processor.Process(req)
			assert.Error(t, err)
			assert.Contains(t, err.Error(), "registry not allowed")
		})

		t.Run("Allow Whitelisted Registry", func(t *testing.T) {
			req := ProcessRequest{
				Format: "yaml",
				ConfigData: []byte(`
name: good-registry-app
components:
  - id: tool
    source:
      registry: ghcr.io
      package: good/tool
      version: v1.0.0
`),
			}

			result, err := processor.Process(req)
			assert.NoError(t, err)
			assert.NotNil(t, result)
		})
	})
}

func TestProcessorMetadata(t *testing.T) {
	processor := NewProcessor(DefaultConfig())

	req := ProcessRequest{
		Format: "yaml",
		ConfigData: []byte(`
name: metadata-test
version: "2.0.0"
description: "Test application"
access: org
components:
  - id: tool1
    source:
      registry: ghcr.io
      package: test/tool1
      version: v1.0.0
  - id: tool2
    source:
      registry: ghcr.io
      package: test/tool2
      version: v1.0.0
`),
		AllowedSubjects: []string{"user_1", "user_2", "user_3"},
		DeploymentContext: &DeploymentContext{
			ActorType: "user",
			OrgID:     "org_meta",
		},
	}

	result, err := processor.Process(req)
	require.NoError(t, err)

	// Verify metadata
	assert.Equal(t, "metadata-test", result.Metadata.AppName)
	assert.Equal(t, "2.0.0", result.Metadata.AppVersion)
	assert.Equal(t, 2, result.Metadata.ComponentCount)
	assert.Equal(t, "org", result.Metadata.AccessMode)
	assert.True(t, result.Metadata.InjectedGateway)
	assert.True(t, result.Metadata.InjectedAuthorizer)
	assert.Equal(t, 3, result.Metadata.SubjectsInjected)
}

func TestProcessorPolicyGeneration(t *testing.T) {
	processor := NewProcessor(DefaultConfig())

	t.Run("Verify Policy Content", func(t *testing.T) {
		testCases := []struct {
			name           string
			accessMode     string
			allowedSubjects []string
			deploymentCtx  *DeploymentContext
			validatePolicy func(t *testing.T, policy string, policyData string)
		}{
			{
				name:           "Private Policy Structure",
				accessMode:     "private",
				allowedSubjects: []string{"owner_123"},
				validatePolicy: func(t *testing.T, policy string, policyData string) {
					// Check policy structure
					assert.Contains(t, policy, "package mcp.authorization")
					assert.Contains(t, policy, "default allow = false")
					assert.Contains(t, policy, "data.owner")
					
					// Check data
					assert.Contains(t, policyData, "owner_123")
				},
			},
			{
				name:           "Org Policy Dual Mode",
				accessMode:     "org",
				allowedSubjects: []string{"user_a", "user_b"},
				deploymentCtx: &DeploymentContext{
					ActorType: "user",
					OrgID:     "org_dual",
				},
				validatePolicy: func(t *testing.T, policy string, policyData string) {
					// Check both user and machine paths
					assert.Contains(t, policy, "not input.token.claims.org_id")
					assert.Contains(t, policy, "data.members[_]")
					assert.Contains(t, policy, "input.token.claims.org_id == data.org_id")
					
					// Check data has both
					assert.Contains(t, policyData, "org_dual")
					assert.Contains(t, policyData, "user_a")
					assert.Contains(t, policyData, "user_b")
				},
			},
		}

		for _, tc := range testCases {
			t.Run(tc.name, func(t *testing.T) {
				yamlTemplate := `
name: test-app
access: %s
components:
  - id: tool
    source:
      registry: ghcr.io
      package: test/tool
      version: v1.0.0
`
				req := ProcessRequest{
					Format:            "yaml",
					ConfigData:        []byte(strings.TrimSpace(strings.Replace(yamlTemplate, "%s", tc.accessMode, 1))),
					AllowedSubjects:   tc.allowedSubjects,
					DeploymentContext: tc.deploymentCtx,
				}

				result, err := processor.Process(req)
				require.NoError(t, err)

				// Extract policy from result
				var manifest map[string]interface{}
				require.NoError(t, toml.Unmarshal([]byte(result.SpinTOML), &manifest))
				
				components := manifest["component"].(map[string]interface{})
				authorizer := components["mcp-authorizer"].(map[string]interface{})
				variables := authorizer["variables"].(map[string]interface{})
				
				policy, _ := variables["mcp_policy"].(string)
				policyData, _ := variables["mcp_policy_data"].(string)
				
				tc.validatePolicy(t, policy, policyData)
			})
		}
	})
}

func TestProcessorValidation(t *testing.T) {
	processor := NewProcessor(DefaultConfig())

	t.Run("Invalid YAML", func(t *testing.T) {
		req := ProcessRequest{
			Format:     "yaml",
			ConfigData: []byte(`invalid: yaml: structure:`),
		}

		_, err := processor.Process(req)
		assert.Error(t, err)
		assert.Contains(t, err.Error(), "validation failed")
	})

	t.Run("Invalid JSON", func(t *testing.T) {
		req := ProcessRequest{
			Format:     "json",
			ConfigData: []byte(`{"invalid": json}`),
		}

		_, err := processor.Process(req)
		assert.Error(t, err)
	})

	t.Run("Unsupported Format", func(t *testing.T) {
		req := ProcessRequest{
			Format:     "xml",
			ConfigData: []byte(`<xml/>`),
		}

		_, err := processor.Process(req)
		assert.Error(t, err)
		assert.Contains(t, err.Error(), "unsupported format")
	})

	t.Run("Missing Required Fields", func(t *testing.T) {
		req := ProcessRequest{
			Format: "yaml",
			ConfigData: []byte(`
components:
  - id: tool
    source: ./tool.wasm
`),
		}

		_, err := processor.Process(req)
		assert.Error(t, err)
		// Name is required
	})
}