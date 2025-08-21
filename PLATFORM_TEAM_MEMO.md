# Platform Backend Team: v0.8.0 Integration Guide

## Executive Summary

The v0.8.0 release introduces Rego-based policy authorization. **The good news: Process() handles everything automatically for you!** No changes required to your existing integration.

## What's Changed Under the Hood

The platform processor (`pkg/platform`) now automatically generates Rego policies based on the access mode and your existing inputs. The legacy fields (`allowed_subjects`, `required_claims`, `allowed_roles`) have been removed from the CUE schema, but **your API calls remain unchanged**.

## How It Works Now

### Your Existing Code (No Changes Needed!)

```go
// Your current code continues to work exactly as before:
resp, err := platformClient.Process(&platform.ProcessRequest{
    ConfigData: appYAML,
    Format: "yaml",
    AllowedSubjects: []string{"user_123", "user_456"},  // For private/org modes
    DeploymentContext: &platform.DeploymentContext{
        ActorType: "user",
        OrgID: "org_abc123",
        ForwardClaims: []string{"email", "name"},
    },
})
```

### What Process() Does Automatically

Based on the access mode in the FTL config, Process() now:

1. **Private Mode** (`access: private`):
   - Takes the first `AllowedSubjects` entry as the owner
   - Generates a Rego policy that only allows that specific user
   - No action required from you

2. **Org Mode** (`access: org`):
   - Uses your `AllowedSubjects` list as org members
   - Uses the `OrgID` from DeploymentContext
   - Generates a Rego policy that allows:
     - All org members (users without org_id in token)
     - Machines with matching org_id
   - No action required from you

3. **Custom Mode** (`access: custom`):
   - User provides their own Rego policy in the FTL config
   - Platform doesn't inject any policy
   - No action required from you

4. **Public Mode** (`access: public`):
   - No authentication required
   - No policy generated
   - No action required from you

## Migration Checklist

✅ **Nothing to migrate!** The Process() function maintains full backwards compatibility:

- [x] Continue passing `AllowedSubjects` for private/org modes
- [x] Continue passing `DeploymentContext` with OrgID and ActorType
- [x] Process() automatically generates the appropriate Rego policies
- [x] The synthesizer handles everything else

## Key Benefits You Get Automatically

1. **Cleaner Authorization**: All auth logic now uses consistent Rego policies
2. **Better WorkOS Support**: Automatic handling of user vs machine token differences
3. **More Flexibility**: Users can write custom policies for complex scenarios
4. **Future-Proof**: Ready for advanced policy features coming in future releases

## Testing Your Integration

Run your existing tests - they should all pass without modification:

```bash
# Your existing test should continue to work
go test ./your-platform-integration/...
```

## For Custom Mode Users

If your users want to write custom Rego policies, they add them to their FTL config:

```yaml
access: custom
auth:
  jwt_issuer: "https://example.com"
  jwt_audience: "api"
  policy: |
    package mcp.authorization
    default allow = false
    
    # Custom logic here
    allow {
      input.token.roles[_] == "admin"
    }
```

The platform doesn't need to do anything special - just pass the config to Process() as usual.

## Questions or Issues?

The Process() API is unchanged, so your integration should work without modifications. If you encounter any issues:

1. Check that you're passing the latest component versions
2. Ensure `AllowedSubjects` is populated for private/org modes
3. Verify `DeploymentContext.OrgID` is set for org mode

## Summary

**No action required!** The v0.8.0 release is fully backwards compatible at the API level. Process() handles all the Rego policy generation automatically based on your existing inputs. Your platform integration continues to work exactly as before, but now with cleaner, more powerful authorization under the hood.