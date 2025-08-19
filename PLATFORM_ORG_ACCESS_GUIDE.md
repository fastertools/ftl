# Platform Team Guide: Org Access Mode

## Overview

Starting with v0.6.4-alpha.1, the platform API supports injecting allowed user subjects for org access mode. This enables you to restrict access to specific users within an organization.

## How It Works

1. **You compute** the allowed users from your org membership system
2. **You filter** by roles if needed (admin, developer, etc.)
3. **You pass** the user IDs to FTL via `AllowedSubjects`
4. **FTL injects** this list into the mcp-authorizer component
5. **The authorizer** restricts access to only these users

## Implementation

```go
func HandleOrgDeployment(orgID string, allowedRoles []string) error {
    // 1. Query org membership from your system (e.g., WorkOS)
    members := queryOrgMembers(orgID)
    
    // 2. Filter by roles if specified
    var allowedSubjects []string
    for _, member := range members {
        if allowedRoles == nil || userHasRole(member, allowedRoles) {
            // Use the 'sub' claim value from their JWT
            allowedSubjects = append(allowedSubjects, member.UserID)
        }
    }
    
    // 3. Pass to FTL platform API
    request := &platform.DeploymentRequest{
        Application: &platform.Application{
            Name:    "internal-tool",
            Version: "1.0.0",
            Access:  "org",
            Auth: &platform.Auth{
                OrgID:     orgID,
                JWTIssuer: "https://api.workos.com",
            },
            Components: []platform.Component{
                // ... your components
            },
        },
        AllowedSubjects: allowedSubjects,  // ← This is the key part
        AllowedRoles:    allowedRoles,     // For your reference/logging
    }
    
    // 4. Process deployment
    config := platform.DefaultConfig()
    client := platform.NewClient(config)
    result, err := client.ProcessDeployment(request)
    
    // The mcp-authorizer will be configured with:
    // mcp_auth_allowed_subjects = "user_01234,user_56789,user_abcde"
    
    return deployToFermyon(result.SpinTOML)
}
```

## What Gets Generated

For an org access deployment with allowed subjects, the Spin TOML will include:

```toml
[component.mcp-authorizer.variables]
mcp_auth_allowed_subjects = 'user_01234,user_56789,user_abcde'
mcp_auth_provider = 'workos'
mcp_jwt_issuer = 'https://api.workos.com'
mcp_org_id = 'org_123456'
```

## Key Points

1. **User IDs** should match the JWT 'sub' claim exactly
2. **Empty list** means no users can access (be careful!)
3. **Nil/omitted** means no subject-based restriction (org membership only)
4. **Role filtering** happens on your side before passing to FTL

## Example: WorkOS Integration

```go
func computeAllowedSubjects(orgID string, roles []string) ([]string, error) {
    // Query WorkOS for org members
    members, err := workosClient.ListOrganizationMembers(ctx, orgID)
    if err != nil {
        return nil, err
    }
    
    var subjects []string
    for _, member := range members {
        // Check role membership if roles specified
        if len(roles) > 0 {
            hasRole := false
            for _, role := range roles {
                if member.HasRole(role) {
                    hasRole = true
                    break
                }
            }
            if !hasRole {
                continue
            }
        }
        
        // Add user's subject ID (from WorkOS user profile)
        subjects = append(subjects, member.UserID)
    }
    
    return subjects, nil
}
```

## Testing

```go
func TestOrgAccessWithAllowedSubjects(t *testing.T) {
    request := &platform.DeploymentRequest{
        Application: &platform.Application{
            Name:   "test-app",
            Access: "org",
            Auth: &platform.Auth{
                OrgID:     "org_test",
                JWTIssuer: "https://api.workos.com",
            },
            Components: []platform.Component{
                {
                    ID: "api",
                    Source: map[string]interface{}{
                        "registry": "ghcr.io",
                        "package":  "test/api",
                        "version":  "1.0.0",
                    },
                },
            },
        },
        AllowedSubjects: []string{"user_alice", "user_bob"},
    }
    
    client := platform.NewClient(platform.DefaultConfig())
    result, err := client.ProcessDeployment(request)
    assert.NoError(t, err)
    
    // Verify the variable is set
    assert.Contains(t, result.SpinTOML, "mcp_auth_allowed_subjects = 'user_alice,user_bob'")
}
```

## Migration

If you're already using org access mode without allowed subjects:
1. No changes required - it continues to work
2. To add subject filtering, just add the `AllowedSubjects` field
3. The authorizer will enforce the additional restriction

## Summary

- **Simple pass-through**: You compute subjects, we inject them
- **Flexible**: Filter by roles, groups, or any criteria you need
- **Secure**: Only listed users can access the application
- **Compatible**: Works with existing org access deployments