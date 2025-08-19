# FTL Platform Team Migration Guide: v0.3.2 → v1.0

## Critical Breaking Changes for Cleaner Architecture

### 1. Module Structure - COMPLETE REDESIGN

**OLD (v0.3.2):**
```go
import "github.com/fastertools/ftl-cli/go/shared/ftl"
```

**NEW (v1.0):**
```go
import (
    "github.com/fastertools/ftl-cli/pkg/platform"  // Platform integration API
    "github.com/fastertools/ftl-cli/pkg/types"     // Shared types
)
```

> **Action Required**: We should move platform-specific APIs to `pkg/platform` for v1.0

### 2. Component Source Structure - BREAKING CHANGE

**OLD (v0.3.2):**
```go
// Complex interface-based approach
if comp.Source.IsLocal() { 
    // handle local
}
```

**NEW (Current):**
```go
// Simple, explicit approach
localPath, registrySource := types.ParseComponentSource(comp.Source)
if localPath != "" {
    return fmt.Errorf("local sources not allowed in production")
}
// registrySource has Registry, Package, Version fields
```

### 3. Authentication Flow - COMPLETELY DIFFERENT

**What You're Doing (Correct):**
- Using `spin registry login` for ECR auth ✅
- This is the right approach

**What We Removed:**
- `UpdateWkgAuthForECR()` - DELETED
- All wkg-related auth code - DELETED
- Direct wkg integration - DELETED

**New WASM Push Format:**
- We now create proper WASM OCI artifacts following CNCF spec
- Compatible with wkg but doesn't require it
- Media types: `application/vnd.wasm.config.v0+json` and `application/wasm`

### 4. Recommended Platform Architecture Changes

#### Move to Explicit Contract

**Current (Hidden Behavior):**
```go
// Magic injection of platform components
manifest, err := ftl.ProcessDeploymentRequest(&req)
// Somehow mcp-gateway and mcp-authorizer appear
```

**Proposed v1.0 (Explicit):**
```go
import "github.com/fastertools/ftl-cli/pkg/platform"

// Explicit platform configuration
config := platform.Config{
    InjectGateway:     true,  // Always true for you
    InjectAuthorizer:  req.Application.Access != "public",
    GatewayVersion:    "0.0.13-alpha.0",
    AuthorizerVersion: "0.0.13-alpha.0",
}

// Clear transformation
manifest, err := platform.ProcessDeployment(req, config)
```

### 5. Type System Cleanup

**Remove These Types (Redundant):**
- `LocalSource` interface
- `RegistrySource` interface  
- Complex source type system

**Use These Instead:**
```go
// Simple map for component source
type Component struct {
    ID     string                 `json:"id"`
    Source map[string]interface{} `json:"source"`
    // source["registry"], source["package"], source["version"]
    // OR just source = "./local/path" for local
}
```

### 6. Platform Integration Best Practices

#### Your Lambda Should:

```go
package lambda

import (
    "github.com/fastertools/ftl-cli/pkg/types"
    "github.com/fastertools/ftl-cli/internal/ftl" // Will move to pkg/platform
)

func HandleDeployment(req DeploymentRequest) error {
    // 1. Validate all components are from registry
    for _, comp := range req.Application.Components {
        localPath, _ := types.ParseComponentSource(comp.Source)
        if localPath != "" {
            return fmt.Errorf("component %s: local sources not allowed", comp.ID)
        }
    }
    
    // 2. Process with platform rules
    manifest, err := ftl.ProcessDeploymentRequest(&req)
    if err != nil {
        return err
    }
    
    // 3. Generate Spin TOML
    synth := ftl.NewSynthesizer()
    toml, err := synth.SynthesizeToTOML(manifest.Application)
    if err != nil {
        return err
    }
    
    // 4. Deploy to Fermyon
    return deployToFermyon(toml)
}
```

### 7. What We Should Clean Up Together

#### For v1.0, let's:

1. **Create `pkg/platform` package** with clean API:
   ```go
   package platform
   
   type Client struct {
       GatewayVersion    string
       AuthorizerVersion string
   }
   
   func (c *Client) ProcessDeployment(req DeploymentRequest) (*SpinManifest, error)
   func (c *Client) GenerateTOML(manifest *SpinManifest) (string, error)
   ```

2. **Remove internal package exposure**:
   - Move platform APIs to `pkg/platform`
   - Keep `internal/` truly internal
   - Clean type system in `pkg/types`

3. **Explicit platform component configuration**:
   - No hidden injection
   - Configurable versions
   - Clear security rules

### 8. Immediate Action Items

#### For Platform Team:

1. **Update imports** to use current structure
2. **Test with v0.5.0** using the patterns above
3. **Provide feedback** on the proposed v1.0 API

#### For FTL CLI Team:

1. **Create `pkg/platform`** package (I can do this now)
2. **Move platform APIs** from `internal/ftl`
3. **Document security model** for component injection

### 9. Breaking Changes We Should Make Now

Since we're pre-release, let's fix these architectural issues:

1. ❌ **Remove** complex interface-based source types
2. ❌ **Remove** backward compatibility shims
3. ❌ **Remove** CUE template remnants
4. ✅ **Add** explicit platform configuration
5. ✅ **Add** versioned component injection
6. ✅ **Add** clear security boundaries

### 10. Timeline

**November 2024:**
- Platform team tests with current codebase
- Identify any missing functionality

**December 2024:**
- Create `pkg/platform` with clean API
- Platform team migrates to new API
- Remove old code paths

**January 2025:**
- v1.0-rc1 with final API
- No more breaking changes after this

**February 2025:**
- v1.0 release

### Questions to Resolve

1. **Component Versions**: Should platform components (mcp-gateway/authorizer) use:
   - Fixed versions you control?
   - Configurable versions?
   - Latest with override option?

2. **Security Model**: Should we make auth injection:
   - Automatic based on access mode?
   - Explicit configuration required?
   - Policy-based with rules engine?

3. **Registry Format**: Are you okay with the new WASM OCI format?
   - Compatible with wkg but doesn't require it
   - Follows CNCF spec
   - Works with standard OCI tooling

### Contact

Let's discuss the v1.0 API design directly. Since we're greenfield, we should optimize for:
- Clean, obvious code
- Explicit behavior (no magic)
- Strong security defaults
- Easy debugging

No compatibility baggage needed!

---

*This is a living document. Let's iterate quickly while we're still pre-release.*