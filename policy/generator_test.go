package policy

import (
	"encoding/json"
	"strings"
	"testing"
)

func TestGeneratePrivatePolicy(t *testing.T) {
	gen := New()

	tests := []struct {
		name      string
		ctx       *Context
		wantError bool
		validate  func(t *testing.T, p *Policy)
	}{
		{
			name: "valid private policy",
			ctx: &Context{
				OwnerSubject: "user_123",
			},
			validate: func(t *testing.T, p *Policy) {
				// Check policy structure
				if !strings.Contains(p.Source, "package mcp.authorization") {
					t.Error("Policy should declare mcp.authorization package")
				}
				if !strings.Contains(p.Source, "default allow = false") {
					t.Error("Policy should default to deny")
				}
				if !strings.Contains(p.Source, "input.token.sub == data.owner") {
					t.Error("Policy should check owner")
				}

				// Check data structure
				if p.Data["owner"] != "user_123" {
					t.Errorf("Expected owner to be user_123, got %v", p.Data["owner"])
				}
			},
		},
		{
			name:      "missing owner subject",
			ctx:       &Context{},
			wantError: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			policy, err := gen.Generate(ModePrivate, tt.ctx)

			if tt.wantError {
				if err == nil {
					t.Error("Expected error but got none")
				}
				return
			}

			if err != nil {
				t.Fatalf("Unexpected error: %v", err)
			}

			if policy == nil {
				t.Fatal("Expected policy but got nil")
			}

			if tt.validate != nil {
				tt.validate(t, policy)
			}
		})
	}
}

func TestGenerateOrgPolicy(t *testing.T) {
	gen := New()

	tests := []struct {
		name      string
		ctx       *Context
		wantError bool
		validate  func(t *testing.T, p *Policy)
	}{
		{
			name: "org with members",
			ctx: &Context{
				OrgID:      "org_abc123",
				OrgMembers: []string{"user_1", "user_2", "user_3"},
				ActorType:  "user",
			},
			validate: func(t *testing.T, p *Policy) {
				// Check policy handles both user and machine cases
				if !strings.Contains(p.Source, "not input.token.claims.org_id") {
					t.Error("Policy should handle user tokens (no org_id)")
				}
				if !strings.Contains(p.Source, "input.token.claims.org_id == data.org_id") {
					t.Error("Policy should handle machine tokens (with org_id)")
				}

				// Check data
				if p.Data["org_id"] != "org_abc123" {
					t.Errorf("Expected org_id to be org_abc123, got %v", p.Data["org_id"])
				}

				members, ok := p.Data["members"].([]string)
				if !ok {
					t.Fatal("Expected members to be []string")
				}
				if len(members) != 3 {
					t.Errorf("Expected 3 members, got %d", len(members))
				}
			},
		},
		{
			name: "org without members (machine deployment)",
			ctx: &Context{
				OrgID:      "org_xyz789",
				OrgMembers: nil,
				ActorType:  "machine",
			},
			validate: func(t *testing.T, p *Policy) {
				// Should still have empty members array
				members, ok := p.Data["members"].([]string)
				if !ok {
					t.Fatal("Expected members to be []string")
				}
				if len(members) != 0 {
					t.Errorf("Expected empty members array, got %d members", len(members))
				}
			},
		},
		{
			name:      "missing org ID",
			ctx:       &Context{OrgMembers: []string{"user_1"}},
			wantError: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			policy, err := gen.Generate(ModeOrg, tt.ctx)

			if tt.wantError {
				if err == nil {
					t.Error("Expected error but got none")
				}
				return
			}

			if err != nil {
				t.Fatalf("Unexpected error: %v", err)
			}

			if policy == nil {
				t.Fatal("Expected policy but got nil")
			}

			if tt.validate != nil {
				tt.validate(t, policy)
			}
		})
	}
}

func TestPolicyToJSON(t *testing.T) {
	tests := []struct {
		name     string
		policy   *Policy
		wantJSON string
	}{
		{
			name: "simple data",
			policy: &Policy{
				Data: map[string]interface{}{
					"owner": "user_123",
				},
			},
			wantJSON: `{"owner":"user_123"}`,
		},
		{
			name: "complex data",
			policy: &Policy{
				Data: map[string]interface{}{
					"org_id":  "org_abc",
					"members": []string{"user_1", "user_2"},
				},
			},
			wantJSON: `{"members":["user_1","user_2"],"org_id":"org_abc"}`,
		},
		{
			name:     "nil policy",
			policy:   nil,
			wantJSON: "",
		},
		{
			name:     "nil data",
			policy:   &Policy{Data: nil},
			wantJSON: "",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			jsonStr, err := tt.policy.ToJSON()
			if err != nil {
				t.Fatalf("Unexpected error: %v", err)
			}

			if tt.wantJSON == "" {
				if jsonStr != "" {
					t.Errorf("Expected empty JSON, got: %s", jsonStr)
				}
				return
			}

			// Parse both to compare structure (ignore key order)
			var got, want interface{}
			if err := json.Unmarshal([]byte(jsonStr), &got); err != nil {
				t.Fatalf("Failed to parse generated JSON: %v", err)
			}
			if err := json.Unmarshal([]byte(tt.wantJSON), &want); err != nil {
				t.Fatalf("Failed to parse expected JSON: %v", err)
			}

			// Re-marshal both to normalize
			gotBytes, _ := json.Marshal(got)
			wantBytes, _ := json.Marshal(want)

			if string(gotBytes) != string(wantBytes) {
				t.Errorf("JSON mismatch:\ngot:  %s\nwant: %s", gotBytes, wantBytes)
			}
		})
	}
}

func TestGenerateForAllModes(t *testing.T) {
	gen := New()

	t.Run("public mode returns nil", func(t *testing.T) {
		policy, err := gen.Generate(ModePublic, nil)
		if err != nil {
			t.Errorf("Public mode should not error: %v", err)
		}
		if policy != nil {
			t.Error("Public mode should return nil policy")
		}
	})

	t.Run("custom mode returns nil", func(t *testing.T) {
		policy, err := gen.Generate(ModeCustom, nil)
		if err != nil {
			t.Errorf("Custom mode should not error: %v", err)
		}
		if policy != nil {
			t.Error("Custom mode should return nil policy")
		}
	})

	t.Run("unknown mode returns error", func(t *testing.T) {
		_, err := gen.Generate("unknown", nil)
		if err == nil {
			t.Error("Unknown mode should return error")
		}
	})
}

// TestPolicySemantics validates that generated policies have correct Rego semantics
func TestPolicySemantics(t *testing.T) {
	gen := New()

	t.Run("private policy structure", func(t *testing.T) {
		ctx := &Context{OwnerSubject: "user_test"}
		policy, err := gen.Generate(ModePrivate, ctx)
		if err != nil {
			t.Fatalf("Failed to generate policy: %v", err)
		}

		// Validate it's valid Rego syntax (basic checks)
		lines := strings.Split(policy.Source, "\n")

		foundPackage := false
		foundDefault := false
		foundAllow := false

		for _, line := range lines {
			line = strings.TrimSpace(line)
			if strings.HasPrefix(line, "package ") {
				foundPackage = true
				if !strings.Contains(line, "mcp.authorization") {
					t.Error("Package should be mcp.authorization")
				}
			}
			if strings.HasPrefix(line, "default allow") {
				foundDefault = true
				if !strings.Contains(line, "false") {
					t.Error("Default should be false (deny by default)")
				}
			}
			if strings.HasPrefix(line, "allow if {") || line == "allow if {" {
				foundAllow = true
			}
		}

		if !foundPackage {
			t.Error("Policy missing package declaration")
		}
		if !foundDefault {
			t.Error("Policy missing default deny")
		}
		if !foundAllow {
			t.Error("Policy missing allow rule")
		}
	})

	t.Run("org policy handles different token types", func(t *testing.T) {
		ctx := &Context{
			OrgID:      "org_test",
			OrgMembers: []string{"user_1"},
		}
		policy, err := gen.Generate(ModeOrg, ctx)
		if err != nil {
			t.Fatalf("Failed to generate policy: %v", err)
		}

		// Check for user token handling (no org_id claim)
		if !strings.Contains(policy.Source, "not input.token.claims.org_id") {
			t.Error("Policy should check for absence of org_id (user tokens)")
		}

		// Check for machine token handling (has org_id claim)
		if !strings.Contains(policy.Source, "input.token.claims.org_id == data.org_id") {
			t.Error("Policy should validate org_id for machine tokens")
		}

		// Check both allow blocks exist
		allowCount := strings.Count(policy.Source, "allow if {")
		if allowCount < 2 {
			t.Errorf("Expected at least 2 allow blocks, found %d", allowCount)
		}
	})
}
