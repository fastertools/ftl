// +build integration

package policy

import (
	"context"
	"fmt"
	"testing"

	"github.com/open-policy-agent/opa/rego"
	"github.com/open-policy-agent/opa/storage/inmem"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TestPolicyEvaluation tests actual Rego policy evaluation using OPA
func TestPolicyEvaluation(t *testing.T) {
	t.Run("Private Mode Policy", func(t *testing.T) {
		gen := New()
		policy, err := gen.Generate(ModePrivate, &Context{
			OwnerSubject: "user_owner_123",
		})
		require.NoError(t, err)
		require.NotNil(t, policy)

		// Test cases for private mode
		testCases := []struct {
			name     string
			input    map[string]interface{}
			expected bool
		}{
			{
				name: "owner allowed",
				input: map[string]interface{}{
					"token": map[string]interface{}{
						"sub": "user_owner_123",
						"iss": "https://auth.example.com",
					},
				},
				expected: true,
			},
			{
				name: "non-owner denied",
				input: map[string]interface{}{
					"token": map[string]interface{}{
						"sub": "user_other_456",
						"iss": "https://auth.example.com",
					},
				},
				expected: false,
			},
			{
				name: "empty subject denied",
				input: map[string]interface{}{
					"token": map[string]interface{}{
						"sub": "",
						"iss": "https://auth.example.com",
					},
				},
				expected: false,
			},
		}

		for _, tc := range testCases {
			t.Run(tc.name, func(t *testing.T) {
				result := evaluatePolicy(t, policy.Source, policy.Data, tc.input)
				assert.Equal(t, tc.expected, result, "Policy evaluation mismatch for %s", tc.name)
			})
		}
	})

	t.Run("Organization Mode Policy", func(t *testing.T) {
		gen := New()
		policy, err := gen.Generate(ModeOrg, &Context{
			OrgID:      "org_abc123",
			OrgMembers: []string{"user_alice", "user_bob", "user_charlie"},
		})
		require.NoError(t, err)
		require.NotNil(t, policy)

		testCases := []struct {
			name     string
			input    map[string]interface{}
			expected bool
		}{
			{
				name: "org member (user) allowed",
				input: map[string]interface{}{
					"token": map[string]interface{}{
						"sub": "user_alice",
						"iss": "https://auth.example.com",
						"claims": map[string]interface{}{
							// User tokens don't have org_id
							"email": "alice@example.com",
						},
					},
				},
				expected: true,
			},
			{
				name: "non-member user denied",
				input: map[string]interface{}{
					"token": map[string]interface{}{
						"sub": "user_eve",
						"iss": "https://auth.example.com",
						"claims": map[string]interface{}{
							"email": "eve@example.com",
						},
					},
				},
				expected: false,
			},
			{
				name: "machine from same org allowed",
				input: map[string]interface{}{
					"token": map[string]interface{}{
						"sub": "machine_deploy_bot",
						"iss": "https://auth.example.com",
						"claims": map[string]interface{}{
							"org_id": "org_abc123",
							"purpose": "deployment",
						},
					},
				},
				expected: true,
			},
			{
				name: "machine from different org denied",
				input: map[string]interface{}{
					"token": map[string]interface{}{
						"sub": "machine_evil_bot",
						"iss": "https://auth.example.com",
						"claims": map[string]interface{}{
							"org_id": "org_different",
							"purpose": "deployment",
						},
					},
				},
				expected: false,
			},
			{
				name: "machine without org_id denied",
				input: map[string]interface{}{
					"token": map[string]interface{}{
						"sub": "machine_broken",
						"iss": "https://auth.example.com",
						"claims": map[string]interface{}{
							"purpose": "deployment",
						},
					},
				},
				expected: false,
			},
		}

		for _, tc := range testCases {
			t.Run(tc.name, func(t *testing.T) {
				result := evaluatePolicy(t, policy.Source, policy.Data, tc.input)
				assert.Equal(t, tc.expected, result, "Policy evaluation mismatch for %s", tc.name)
			})
		}
	})
}

// TestComplexPolicyScenarios tests more complex authorization scenarios
func TestComplexPolicyScenarios(t *testing.T) {
	t.Run("Org with empty members list", func(t *testing.T) {
		gen := New()
		policy, err := gen.Generate(ModeOrg, &Context{
			OrgID:      "org_empty",
			OrgMembers: []string{}, // Empty member list
		})
		require.NoError(t, err)

		// Only machines should be allowed
		testCases := []struct {
			name     string
			input    map[string]interface{}
			expected bool
		}{
			{
				name: "user denied when no members",
				input: map[string]interface{}{
					"token": map[string]interface{}{
						"sub": "user_anyone",
						"claims": map[string]interface{}{
							"email": "anyone@example.com",
						},
					},
				},
				expected: false,
			},
			{
				name: "machine from org still allowed",
				input: map[string]interface{}{
					"token": map[string]interface{}{
						"sub": "machine_x",
						"claims": map[string]interface{}{
							"org_id": "org_empty",
						},
					},
				},
				expected: true,
			},
		}

		for _, tc := range testCases {
			t.Run(tc.name, func(t *testing.T) {
				result := evaluatePolicy(t, policy.Source, policy.Data, tc.input)
				assert.Equal(t, tc.expected, result)
			})
		}
	})

	t.Run("Large org member list", func(t *testing.T) {
		// Generate a large member list
		members := make([]string, 1000)
		for i := 0; i < 1000; i++ {
			members[i] = fmt.Sprintf("user_%d", i)
		}

		gen := New()
		policy, err := gen.Generate(ModeOrg, &Context{
			OrgID:      "org_large",
			OrgMembers: members,
		})
		require.NoError(t, err)

		// Test edge cases
		testCases := []struct {
			name     string
			input    map[string]interface{}
			expected bool
		}{
			{
				name: "first member allowed",
				input: map[string]interface{}{
					"token": map[string]interface{}{
						"sub":    "user_0",
						"claims": map[string]interface{}{},
					},
				},
				expected: true,
			},
			{
				name: "last member allowed",
				input: map[string]interface{}{
					"token": map[string]interface{}{
						"sub":    "user_999",
						"claims": map[string]interface{}{},
					},
				},
				expected: true,
			},
			{
				name: "middle member allowed",
				input: map[string]interface{}{
					"token": map[string]interface{}{
						"sub":    "user_500",
						"claims": map[string]interface{}{},
					},
				},
				expected: true,
			},
			{
				name: "non-existent member denied",
				input: map[string]interface{}{
					"token": map[string]interface{}{
						"sub":    "user_1000",
						"claims": map[string]interface{}{},
					},
				},
				expected: false,
			},
		}

		for _, tc := range testCases {
			t.Run(tc.name, func(t *testing.T) {
				result := evaluatePolicy(t, policy.Source, policy.Data, tc.input)
				assert.Equal(t, tc.expected, result)
			})
		}
	})
}

// TestCustomPolicyEvaluation tests user-provided custom policies
func TestCustomPolicyEvaluation(t *testing.T) {
	testCases := []struct {
		name       string
		policy     string
		policyData map[string]interface{}
		input      map[string]interface{}
		expected   bool
	}{
		{
			name: "role-based access",
			policy: `package mcp.authorization
default allow = false

allow {
	input.token.claims.role == "admin"
}

allow {
	input.token.claims.role == "user"
	input.mcp.method == "tools/list"
}`,
			input: map[string]interface{}{
				"token": map[string]interface{}{
					"claims": map[string]interface{}{
						"role": "admin",
					},
				},
			},
			expected: true,
		},
		{
			name: "tool-specific authorization",
			policy: `package mcp.authorization
default allow = false

allow {
	input.mcp.tool == data.safe_tools[_]
}

allow {
	input.mcp.tool == data.admin_tools[_]
	input.token.claims.admin == true
}`,
			policyData: map[string]interface{}{
				"safe_tools":  []string{"read", "list"},
				"admin_tools": []string{"delete", "write"},
			},
			input: map[string]interface{}{
				"token": map[string]interface{}{
					"claims": map[string]interface{}{
						"admin": false,
					},
				},
				"mcp": map[string]interface{}{
					"tool": "read",
				},
			},
			expected: true,
		},
		{
			name: "complex claim validation",
			policy: `package mcp.authorization
default allow = false

allow {
	input.token.claims.department == "engineering"
	input.token.claims.level >= 3
	input.token.claims.permissions[_] == "write"
}`,
			input: map[string]interface{}{
				"token": map[string]interface{}{
					"claims": map[string]interface{}{
						"department":  "engineering",
						"level":       4,
						"permissions": []string{"read", "write", "execute"},
					},
				},
			},
			expected: true,
		},
	}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			result := evaluatePolicy(t, tc.policy, tc.policyData, tc.input)
			assert.Equal(t, tc.expected, result)
		})
	}
}

// Helper function to evaluate a Rego policy
func evaluatePolicy(t *testing.T, policySource string, data map[string]interface{}, input map[string]interface{}) bool {
	ctx := context.Background()

	// Build options for Rego
	options := []func(*rego.Rego){
		rego.Query("data.mcp.authorization.allow"),
		rego.Module("authorization.rego", policySource),
		rego.Input(input),
	}

	// Add data if provided
	if data != nil {
		// Create an in-memory store with the data
		store := inmem.NewFromObject(data)
		options = append(options, rego.Store(store))
	}

	// Create and evaluate the query
	r := rego.New(options...)
	rs, err := r.Eval(ctx)
	require.NoError(t, err, "Policy evaluation failed")

	// Check if we got a result
	if len(rs) == 0 || len(rs[0].Expressions) == 0 {
		return false
	}

	// Extract the boolean result
	result, ok := rs[0].Expressions[0].Value.(bool)
	if !ok {
		t.Fatalf("Expected boolean result, got %T", rs[0].Expressions[0].Value)
	}

	return result
}