// Package policy provides Rego policy generation for FTL authorization.
package policy

import (
	"encoding/json"
	"fmt"
)

// Mode represents the authorization mode
type Mode string

const (
	ModePublic  Mode = "public"
	ModePrivate Mode = "private"
	ModeOrg     Mode = "org"
	ModeCustom  Mode = "custom"
)

// Generator creates Rego policies for different authorization modes
type Generator struct{}

// New creates a new policy generator
func New() *Generator {
	return &Generator{}
}

// Generate creates a policy and data for the given mode and context
func (g *Generator) Generate(mode Mode, ctx *Context) (*Policy, error) {
	switch mode {
	case ModePublic:
		// Public mode doesn't need auth, but for consistency we return an allow-all policy
		return nil, nil
	case ModePrivate:
		return g.generatePrivate(ctx)
	case ModeOrg:
		return g.generateOrg(ctx)
	case ModeCustom:
		// Custom mode uses user-provided policy
		return nil, nil
	default:
		return nil, fmt.Errorf("unknown authorization mode: %s", mode)
	}
}

// Context provides information needed for policy generation
type Context struct {
	// For private mode: the owner's subject
	OwnerSubject string

	// For org mode: list of org member subjects
	OrgMembers []string

	// For org mode: the organization ID
	OrgID string

	// Actor type for deployment context
	ActorType string // "user" or "machine"
}

// Policy represents a Rego policy with its data
type Policy struct {
	// The Rego policy source code
	Source string

	// The policy data (will be JSON encoded)
	Data map[string]interface{}
}

// generatePrivate creates a policy for private mode
func (g *Generator) generatePrivate(ctx *Context) (*Policy, error) {
	if ctx.OwnerSubject == "" {
		return nil, fmt.Errorf("owner subject required for private mode")
	}

	policy := `package mcp.authorization

# Private mode: Only the owner can access
default allow = false

allow if {
	input.token.sub == data.owner
}
`

	return &Policy{
		Source: policy,
		Data: map[string]interface{}{
			"owner": ctx.OwnerSubject,
		},
	}, nil
}

// generateOrg creates a policy for organization mode
func (g *Generator) generateOrg(ctx *Context) (*Policy, error) {
	if ctx.OrgID == "" {
		return nil, fmt.Errorf("organization ID required for org mode")
	}

	// The policy handles both user and machine tokens elegantly
	policy := `package mcp.authorization

# Organization mode: Members and machines can access
default allow = false

# Allow org members (user tokens without org_id claim)
allow if {
	not input.token.claims.org_id
	input.token.sub == data.members[_]
}

# Allow machines from the same org (machine tokens with org_id claim)
allow if {
	input.token.claims.org_id
	input.token.claims.org_id == data.org_id
}
`

	data := map[string]interface{}{
		"org_id": ctx.OrgID,
	}

	// Add members if provided
	if len(ctx.OrgMembers) > 0 {
		data["members"] = ctx.OrgMembers
	} else {
		// Empty list if no members provided
		data["members"] = []string{}
	}

	return &Policy{
		Source: policy,
		Data:   data,
	}, nil
}

// ToJSON converts policy data to JSON string
func (p *Policy) ToJSON() (string, error) {
	if p == nil || p.Data == nil {
		return "", nil
	}

	bytes, err := json.Marshal(p.Data)
	if err != nil {
		return "", fmt.Errorf("failed to marshal policy data: %w", err)
	}

	return string(bytes), nil
}
