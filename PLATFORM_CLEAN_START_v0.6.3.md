# Platform Team - Clean Start with v0.6.3-alpha.1

## 🎯 Recommended Version

```bash
go get github.com/fastertools/ftl-cli@v0.6.3-alpha.1
```

## What's in v0.6.3-alpha.1

This release provides you with a **fully consistent and clean codebase**:

### ✅ All Issues Fixed
- **Module download** - No Unicode characters in filenames
- **Synthesis errors** - Platform components inject correctly
- **Naming consistency** - All 'ftl-' prefixes removed

### 📦 Standardized Component Names

| Component | ID | Package Reference |
|-----------|-------|------------------|
| Gateway | `mcp-gateway` | `ghcr.io/fastertools:mcp-gateway:0.0.13-alpha.0` |
| Authorizer | `mcp-authorizer` | `ghcr.io/fastertools:mcp-authorizer:0.0.15-alpha.0` |

### 🔧 Platform API (pkg/platform)

```go
import "github.com/fastertools/ftl-cli/pkg/platform"

config := platform.DefaultConfig()
client := platform.NewClient(config)

result, err := client.ProcessDeployment(request)
// result.SpinTOML is ready to deploy
```

## Why Use v0.6.3 Over Previous Versions

| Version | Issue | Status |
|---------|-------|--------|
| v0.6.0-alpha.1 | Unicode in filenames - module won't download | ❌ Broken |
| v0.6.1-alpha.1 | Synthesis errors with mcp-authorizer | ❌ Fixed download, broken synthesis |
| v0.6.2-alpha.1 | Inconsistent naming (mix of ftl-mcp-* and mcp-*) | ⚠️ Works but messy |
| **v0.6.3-alpha.1** | **All issues fixed, consistent naming** | **✅ Recommended** |

## Quick Integration Test

```go
package main

import (
    "testing"
    "github.com/fastertools/ftl-cli/pkg/platform"
)

func TestPlatformIntegration(t *testing.T) {
    config := platform.DefaultConfig()
    client := platform.NewClient(config)
    
    req := &platform.DeploymentRequest{
        Application: &platform.Application{
            Name:    "test-app",
            Version: "1.0.0",
            Access:  "private", // Tests mcp-authorizer injection
            Components: []platform.Component{{
                ID: "api",
                Source: map[string]interface{}{
                    "registry": "ghcr.io",
                    "package":  "test/api",
                    "version":  "1.0.0",
                },
            }},
        },
    }
    
    result, err := client.ProcessDeployment(req)
    if err != nil {
        t.Fatalf("Failed: %v", err)
    }
    
    // Should have 3 components: mcp-gateway, mcp-authorizer, api
    if result.Metadata.ComponentCount != 3 {
        t.Errorf("Expected 3 components, got %d", result.Metadata.ComponentCount)
    }
    
    t.Logf("✅ Platform integration working with v0.6.3-alpha.1")
}
```

## Component Publishing Note

The `mcp-gateway` and `mcp-authorizer` components are published as WASM binaries to:
- **Registry**: ghcr.io (GitHub Container Registry)
- **Namespace**: fastertools
- **Format**: OCI artifacts containing WASM modules

They are **not** published to crates.io like the SDK packages.

## Summary

**v0.6.3-alpha.1** is the clean, consistent starting point for your platform integration:
- All naming standardized on `mcp-gateway` and `mcp-authorizer`
- All known issues fixed
- pkg/platform API stable and tested
- Ready for production use

This is the version to build on!