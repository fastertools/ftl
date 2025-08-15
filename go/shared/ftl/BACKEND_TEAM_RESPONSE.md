# Response to Backend Team: v0.2.0 Released with All Critical Fixes

**To:** Platform Backend Team  
**From:** CLI/Library Team  
**Date:** August 15, 2025  
**Subject:** v0.2.0 Released - All Critical Issues Resolved

## Executive Summary

We've released v0.2.0 of the shared FTL package with all critical issues resolved. You can upgrade immediately:

```bash
go get -u github.com/fastertools/ftl-cli/go/shared/ftl@v0.2.0
```

## Issues Resolved ✅

### 1. ✅ Access Control Mode Support
**Fixed:** All access modes now work correctly
```go
// These all work now:
app.Access = ftl.AccessPublic   ✅
app.Access = ftl.AccessPrivate  ✅  
app.Access = ftl.AccessOrg      ✅ (was broken)
app.Access = ftl.AccessCustom   ✅ (was broken)
```

### 2. ✅ SpinManifestVersion 
**Fixed:** Now correctly set to 2
```go
manifest, _ := ftl.ProcessDeploymentRequest(req)
fmt.Println(manifest.SpinManifestVersion) // Now prints: 2
```

### 3. ✅ Enhanced Validation
**Fixed:** Validation now catches synthesis issues
```go
app := &ftl.Application{
    Name:   "test-app",
    Access: ftl.AccessOrg,
    // Missing OrgID
}
app.Validate() // Now returns error: "org ID is required for org access mode"
```

### 4. ✅ Application-Level Variables
**Added:** Variables field to Application struct
```go
type Application struct {
    // ... other fields ...
    Variables map[string]string `json:"variables,omitempty"`
}
```

### 5. ✅ Component Source Helpers
**Added:** Convenience methods as requested
```go
// New helper functions:
reg, ok := ftl.AsRegistry(comp.Source)  // Type-safe cast to *RegistrySource
path, ok := ftl.AsLocal(comp.Source)    // Type-safe cast to string
```

### 6. ✅ Better Error Messages
**Improved:** CUE errors are now human-readable
```go
// Before: "failed to fill input app: _transform.input.access: 2 errors in empty disjunction"
// Now: "invalid access mode 'invalid': must be one of 'public', 'private', 'org', or 'custom'"
```

## Test Your Integration

Here's a complete test to verify everything works:

```go
package main

import (
    "testing"
    "github.com/fastertools/ftl-cli/go/shared/ftl"
)

func TestV02Features(t *testing.T) {
    // Test org access mode
    orgApp := &ftl.Application{
        Name:    "org-test",
        Version: "1.0.0",
        Access:  ftl.AccessOrg,
        Auth: ftl.AuthConfig{
            Provider: ftl.AuthProviderWorkOS,
            OrgID:    "org_123",
        },
        Variables: map[string]string{
            "API_KEY": "secret",
        },
    }
    
    req := ftl.DeploymentRequest{
        Application: orgApp,
    }
    
    manifest, err := ftl.ProcessDeploymentRequest(&req)
    if err != nil {
        t.Fatalf("Failed to process org mode: %v", err)
    }
    
    // Verify fixes
    if manifest.SpinManifestVersion != 2 {
        t.Errorf("SpinManifestVersion = %d, want 2", manifest.SpinManifestVersion)
    }
    
    // Test custom access mode
    customApp := &ftl.Application{
        Name:    "custom-test",
        Access:  ftl.AccessCustom,
        Auth: ftl.AuthConfig{
            Provider:    ftl.AuthProviderCustom,
            JWTIssuer:   "https://auth.example.com",
            JWTAudience: "api.example.com",
        },
    }
    
    req.Application = customApp
    _, err = ftl.ProcessDeploymentRequest(&req)
    if err != nil {
        t.Fatalf("Failed to process custom mode: %v", err)
    }
    
    t.Log("✅ All v0.2.0 features working!")
}
```

## Your Failing Test Cases - Now Fixed

Both test cases from Appendix A now pass:

```go
// ✅ Test case 1: Org access - NOW WORKS
{
    Application: &ftl.Application{
        Name:   "test-app",
        Access: ftl.AccessOrg,
        Auth:   ftl.AuthConfig{Provider: "workos", OrgID: "org-123"},
    },
}

// ✅ Test case 2: Custom JWT - NOW WORKS  
{
    Application: &ftl.Application{
        Name:   "test-app",
        Access: ftl.AccessCustom,
        Auth:   ftl.AuthConfig{
            Provider:    "custom",
            JWTIssuer:   "https://auth.example.com",
            JWTAudience: "api.example.com",
        },
    },
}
```

## Migration Guide from v0.1.0 to v0.2.0

No breaking changes! Simply upgrade:

```bash
# Update your go.mod
go get -u github.com/fastertools/ftl-cli/go/shared/ftl@v0.2.0
go mod tidy

# Run your tests - they should all pass
go test ./...
```

## What's Next (v0.3.0 Roadmap)

Based on your feedback, we're planning:

1. **Configurable platform component versions** (instead of hardcoded)
2. **Extension mechanism** for custom middleware components  
3. **Performance optimizations** for 50+ component applications
4. **Migration tooling** from spinc.yaml format
5. **Deployment environment** field utilization

## Immediate Actions for Your Team

1. **Upgrade to v0.2.0** - All critical issues are fixed
2. **Remove workarounds** - No longer needed
3. **Enable org/custom modes** - They work now
4. **Run your test suite** - Coverage should improve

## Questions & Support

We're available for immediate support:
- **Slack:** #ftl-platform (I'm monitoring actively)
- **GitHub Issues:** File any new issues
- **Direct:** Happy to pair on any integration challenges

## Thank You!

Your detailed feedback was invaluable. The specificity of your test cases and error messages made it possible to fix everything quickly. This is exactly the kind of partnership that makes great software.

Please let us know if v0.2.0 resolves all your issues. We're standing by to help with any integration questions.

Best regards,  
The CLI/Library Team

---

**P.S.** Your test coverage numbers are impressive! With v0.2.0, you should be able to remove those pre-validation workarounds and potentially improve coverage even further.