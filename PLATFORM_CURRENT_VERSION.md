# Current Version for Platform Integration

## Version to Use: v0.5.0-alpha.1

The platform team should use the following in their `go.mod`:

```go
require (
    github.com/fastertools/ftl-cli v0.5.0-alpha.1
)
```

## What's in v0.5.0-alpha.1

### ✅ Working Features
- `ProcessDeploymentRequest()` - Located in `internal/ftl/deployment.go`
- `NewSynthesizer().SynthesizeToTOML()` - Located in `internal/ftl/synthesis.go`
- Component source parsing via `types.ParseComponentSource()`
- WASM OCI artifact creation (CNCF spec compliant)
- Proper ECR push/pull support

### 🔧 Fixed Issues
- ✅ "archive/tar: invalid tar header" error when pushing to ECR
- ✅ Invalid reference format with multiple colons
- ✅ WASM OCI artifacts now compatible with wkg

### 📦 Import Paths

```go
import (
    "github.com/fastertools/ftl-cli/internal/ftl"  // Platform functions
    "github.com/fastertools/ftl-cli/pkg/types"     // Type definitions
)
```

### 🚨 Breaking Changes from v0.3.2

1. **Import path changed**: 
   - Old: `github.com/fastertools/ftl-cli/go/shared/ftl`
   - New: `github.com/fastertools/ftl-cli/internal/ftl`

2. **Component source checking**:
   ```go
   // Old way (doesn't work anymore)
   if comp.Source.IsLocal() { }
   
   // New way
   localPath, registrySource := types.ParseComponentSource(comp.Source)
   if localPath != "" {
       // It's local
   }
   ```

3. **Removed functions**:
   - `UpdateWkgAuthForECR()` - No longer exists or needed

## Testing the Integration

```bash
# Get the specific version
go get github.com/fastertools/ftl-cli@v0.5.0-alpha.1

# Update your imports and test
go test ./...
```

## Sample Integration Code

```go
package lambda

import (
    "fmt"
    "github.com/fastertools/ftl-cli/internal/ftl"
    "github.com/fastertools/ftl-cli/pkg/types"
)

func HandleDeployment(req *ftl.DeploymentRequest) error {
    // Validate components are from registry
    for _, comp := range req.Application.Components {
        localPath, _ := types.ParseComponentSource(comp.Source)
        if localPath != "" {
            return fmt.Errorf("local sources not allowed")
        }
    }
    
    // Process deployment
    manifest, err := ftl.ProcessDeploymentRequest(req)
    if err != nil {
        return err
    }
    
    // Generate TOML
    synth := ftl.NewSynthesizer()
    toml, err := synth.SynthesizeToTOML(manifest.Application)
    if err != nil {
        return err
    }
    
    // Deploy to Fermyon
    return deployToFermyon(toml)
}
```

## Known Working Configuration

- ✅ Platform component injection (mcp-gateway, mcp-authorizer)
- ✅ Access modes: public, private, custom
- ✅ WASM OCI artifacts push/pull with ECR
- ✅ Spin manifest generation

## Support

If you encounter issues with v0.5.0-alpha.1:
1. Check this document first
2. Review PLATFORM_MIGRATION_GUIDE.md
3. Open an issue with the `platform-integration` label

---
*Last updated: November 2024*
*Commit: a593fee*