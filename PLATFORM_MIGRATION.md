# Platform Backend Migration Guide - v0.8.0

## Breaking Changes in Authorization System

The FTL platform package (`pkg/platform`) has undergone a complete redesign of its authorization system, moving from ad-hoc claim validation to **policy-based authorization using Rego**.

### What Changed

#### 1. Removed Fields from ProcessRequest
The following fields have been **removed** from `ProcessRequest.AllowedSubjects`:
- ❌ No longer used for passing allowed subjects directly to CUE

#### 2. New Policy Generation
The platform now generates Rego policies based on access mode:

```go
// OLD: Subjects were passed directly to CUE
req.AllowedSubjects = []string{"user_123", "user_456"}

// NEW: Subjects are used to generate Rego policies
// The platform handles this automatically - you just provide the subjects
req.AllowedSubjects = []string{"user_123", "user_456"}
```

### Required Changes for Platform Backend

#### 1. Continue Providing AllowedSubjects
**No change needed here** - continue computing and providing `AllowedSubjects` as before:

```go
req := ProcessRequest{
    ConfigData: appConfig,
    Format: "yaml",
    AllowedSubjects: computedOrgMembers, // Still needed!
    DeploymentContext: &DeploymentContext{
        ActorType: "user", // or "machine"
        OrgID: "org_abc123",
    },
}
```

#### 2. The Platform Now Handles Policy Generation

For each access mode, the platform automatically generates appropriate Rego policies:

**Private Mode**: 
- Provide: Single owner subject in `AllowedSubjects[0]`
- Platform generates: Policy allowing only that owner

**Org Mode**:
- Provide: Org member subjects in `AllowedSubjects`
- Provide: `DeploymentContext.OrgID`
- Platform generates: Policy allowing org members (users) and machines from the org

**Custom Mode**:
- Provide: Nothing special needed
- User provides their own policy in their config

### How Authorization Works Now

Instead of simple claim matching, the mcp-authorizer evaluates Rego policies:

```rego
# Example generated policy for org mode
package mcp.authorization

default allow = false

# Allow org members (user tokens without org_id claim)
allow {
    not input.token.claims.org_id
    input.token.sub == data.members[_]
}

# Allow machines from the same org
allow {
    input.token.claims.org_id
    input.token.claims.org_id == data.org_id
}
```

### Migration Checklist

- [ ] **No code changes needed** if you're only using `Process()`
- [ ] Continue providing `AllowedSubjects` for private/org modes
- [ ] Continue providing `DeploymentContext` for org mode
- [ ] Remove any references to `RequiredClaims`, `AllowedRoles` (if any)
- [ ] Test with both user and machine tokens for org mode

### Testing the Migration

1. **Private Mode Test**:
```go
// Deploy a private app
req := ProcessRequest{
    ConfigData: privateAppYAML,
    Format: "yaml",
    AllowedSubjects: []string{"user_owner_123"},
}
result, _ := processor.Process(req)
// Verify only user_owner_123 can access
```

2. **Org Mode Test**:
```go
// Deploy an org app with user actor
req := ProcessRequest{
    ConfigData: orgAppYAML,
    Format: "yaml",
    AllowedSubjects: []string{"user_alice", "user_bob"},
    DeploymentContext: &DeploymentContext{
        ActorType: "user",
        OrgID: "org_123",
    },
}
result, _ := processor.Process(req)
// Verify:
// - user_alice and user_bob can access
// - machines with org_id="org_123" can access
// - other users/machines cannot access
```

### Benefits of the New System

1. **Cleaner Separation**: Authorization logic is in Rego policies, not Go code
2. **More Flexible**: Custom policies can implement complex authorization rules
3. **Better Machine Support**: Properly handles WorkOS JWT schema differences
4. **Auditable**: Policies are explicit and can be reviewed/tested independently

### Questions or Issues?

Contact the FTL team if you encounter any issues during migration. The new system is designed to be backward-compatible with your existing `AllowedSubjects` computation - you shouldn't need to change how you determine who should have access, only the internals of how that access is enforced have changed.

## Version Compatibility

- **Minimum mcp-authorizer version**: 0.0.15-alpha.0
- **Platform package version**: v0.8.0
- **Breaking change**: Yes, but transparent if using `Process()` correctly