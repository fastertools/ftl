# Backend Team - Synthesis Error Fixed in v0.6.2-alpha.1

## ✅ The Fix is Live

```bash
go get github.com/fastertools/ftl-cli@v0.6.2-alpha.1
```

## What Was Wrong

The synthesis error you encountered:
```
_transform.output.component."mcp-authorizer".source: 2 errors in empty disjunction
```

Was caused by a mismatch between the platform API and internal CUE patterns:
- Platform API was injecting component ID: `mcp-gateway`  
- CUE patterns expected: `ftl-mcp-gateway`

## What We Fixed

1. **Standardized on `mcp-gateway`** (removed the `ftl-` prefix)
2. Updated all internal references to match
3. Platform components now synthesize correctly

## Your Test Should Now Pass

```go
// This should now work for private access applications
result, err := client.ProcessDeployment(&platform.DeploymentRequest{
    Application: &platform.Application{
        Name:    "test-app",
        Version: "1.0.0",
        Access:  "private",  // ✅ Will inject mcp-authorizer correctly
        Components: []platform.Component{
            {
                ID: "api",
                Source: map[string]interface{}{
                    "registry": "ghcr.io",
                    "package":  "myorg/api",
                    "version":  "1.0.0",
                },
            },
        },
    },
})
```

## Verification

Run your test again:
```bash
go test -v -run TestHandleRequest/successful_deployment_with_registry_components
```

Should now see:
- ✅ Deployment processing succeeds
- ✅ Both `mcp-gateway` and `mcp-authorizer` are injected
- ✅ Synthesis completes without errors
- ✅ Status code 202 returned

## Platform Components Reference

The platform now correctly injects:

### mcp-gateway (always)
- Component ID: `mcp-gateway`
- Source: `ghcr.io/fastertools:mcp-gateway:0.0.13-alpha.0`

### mcp-authorizer (for non-public apps)
- Component ID: `mcp-authorizer`
- Source: `ghcr.io/fastertools:mcp-authorizer:0.0.15-alpha.0`

## Technical Details

The CUE synthesis patterns now expect:
```cue
component: {
    "mcp-gateway": { ... }      // ✅ Correct
    "mcp-authorizer": { ... }    // ✅ Correct
}
```

Not:
```cue
component: {
    "ftl-mcp-gateway": { ... }   // ❌ Old format
}
```

## Summary

- **v0.6.2-alpha.1** fixes the synthesis error
- Component IDs are now consistent
- Your tests should pass
- Private/org/custom access modes will correctly inject the authorizer

Let us know if you encounter any other issues!