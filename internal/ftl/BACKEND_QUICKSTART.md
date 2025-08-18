# Backend Team Quick Start

## Immediate Usage (No Waiting Required!)

You can start using the shared FTL package **right now** without waiting for formal publishing.

### Option 1: Direct GitHub Import (Simplest)

```bash
# In your backend project
go get github.com/fastertools/ftl-cli/go/shared/ftl@main

# This pulls directly from our main branch
```

Your `go.mod` will show something like:
```
require github.com/fastertools/ftl-cli/go/shared/ftl v0.0.0-20240115-abcdef123456
```

### Option 2: Pin to Specific Commit (Most Stable)

```bash
# Use our current stable commit
go get github.com/fastertools/ftl-cli/go/shared/ftl@6bb943e
```

### Option 3: Local Development (For Testing)

```bash
# Clone our repo
git clone https://github.com/fastertools/ftl-cli.git /tmp/ftl-cli

# In your go.mod, add:
replace github.com/fastertools/ftl-cli/go/shared/ftl => /tmp/ftl-cli/go/shared/ftl

# Then import normally
go mod tidy
```

## Complete Working Example

### 1. Create a test file `deployment_handler.go`:

```go
package main

import (
    "encoding/json"
    "fmt"
    "log"
    
    "github.com/fastertools/ftl-cli/go/shared/ftl"
)

func ProcessDeployment(jsonRequest string) error {
    // Parse deployment request
    var req ftl.DeploymentRequest
    if err := json.Unmarshal([]byte(jsonRequest), &req); err != nil {
        return fmt.Errorf("parse error: %w", err)
    }
    
    // Validate application
    if err := req.Application.Validate(); err != nil {
        return fmt.Errorf("invalid app: %w", err)
    }
    
    // Synthesize Spin manifest
    manifest, err := ftl.ProcessDeploymentRequest(&req)
    if err != nil {
        return fmt.Errorf("synthesis failed: %w", err)
    }
    
    // Convert to TOML
    synth := ftl.NewSynthesizer()
    toml, err := synth.SynthesizeToTOML(req.Application)
    if err != nil {
        return fmt.Errorf("TOML generation failed: %w", err)
    }
    
    log.Printf("Generated Spin manifest:\n%s", toml)
    return nil
}
```

### 2. Test with sample data:

```go
func main() {
    sampleRequest := `{
        "application": {
            "name": "test-app",
            "version": "1.0.0",
            "components": [
                {
                    "id": "weather-api",
                    "source": {
                        "registry": "123.dkr.ecr.us-west-2.amazonaws.com/app-123",
                        "package": "app-123:weather-api",
                        "version": "1.0.0"
                    }
                }
            ],
            "access": "private",
            "auth": {
                "provider": "workos",
                "org_id": "org_123"
            }
        },
        "variables": {
            "API_KEY": "secret123"
        }
    }`
    
    if err := ProcessDeployment(sampleRequest); err != nil {
        log.Fatal(err)
    }
}
```

### 3. Run it:

```bash
go mod init my-backend
go get github.com/fastertools/ftl-cli/go/shared/ftl@main
go run deployment_handler.go
```

## Expected Output

You should see a generated Spin manifest like:

```toml
spin_manifest_version = 2

[application]
name = "test-app"
version = "1.0.0"

[component.weather-api]
source = { registry = "123.dkr.ecr.us-west-2.amazonaws.com/app-123", package = "app-123:weather-api", version = "1.0.0" }

[component.ftl-mcp-gateway]
source = { registry = "ghcr.io", package = "fastertools:mcp-gateway", version = "0.0.13-alpha.0" }
allowed_outbound_hosts = ["http://*.spin.internal"]
[component.ftl-mcp-gateway.variables]
component_names = "weather-api"

[component.mcp-authorizer]
source = { registry = "ghcr.io", package = "fastertools:mcp-authorizer", version = "0.0.15-alpha.0" }
allowed_outbound_hosts = ["http://*.spin.internal", "https://*.authkit.app", "https://*.workos.com"]
[component.mcp-authorizer.variables]
mcp_gateway_url = "http://ftl-mcp-gateway.spin.internal"
mcp_jwt_issuer = "https://api.workos.com"
mcp_jwt_audience = "test-app"

[[trigger.http]]
route = "/..."
component = "mcp-authorizer"

[[trigger.http]]
route = { private = true }
component = "ftl-mcp-gateway"

[[trigger.http]]
route = { private = true }
component = "weather-api"
```

## Key Types You'll Use

```go
// Main request from CLI
type ftl.DeploymentRequest struct {
    Application   *Application
    Variables     map[string]string
    Environment   string
    AccessControl *AccessMode
    CustomAuth    *CustomAuthConfig
    AllowedRoles  []string
}

// Process it
manifest, err := ftl.ProcessDeploymentRequest(&request)

// Or synthesize directly
synth := ftl.NewSynthesizer()
toml, err := synth.SynthesizeToTOML(app)
```

## Common Patterns

### Validate Components Are From Registry

```go
for _, comp := range req.Application.Components {
    if comp.Source.IsLocal() {
        return fmt.Errorf("component %s must be pushed to registry first", comp.ID)
    }
}
```

### Extract Registry Info

```go
if !comp.Source.IsLocal() {
    reg := comp.Source.GetRegistry()
    fmt.Printf("Registry: %s\n", reg.Registry)
    fmt.Printf("Package: %s\n", reg.Package)
    fmt.Printf("Version: %s\n", reg.Version)
}
```

### Apply Variable Overrides

```go
// Variables from request override defaults
manifest, _ := ftl.ProcessDeploymentRequest(&req)
// Variables are automatically applied
```

## Troubleshooting

### Import Issues?

```bash
# Clear module cache
go clean -modcache

# Re-download
go get -u github.com/fastertools/ftl-cli/go/shared/ftl@main
```

### Can't Find Module?

Make sure the repository is public or configure:
```bash
export GOPRIVATE=github.com/fastertools/*
```

### Version Mismatch?

Check what version you have:
```bash
go list -m github.com/fastertools/ftl-cli/go/shared/ftl
```

## Support

- **Slack**: #ftl-platform
- **Issues**: https://github.com/fastertools/ftl-cli/issues
- **Direct**: Contact the CLI team

## Next Steps

1. Import the module ✅
2. Create deployment handler ✅  
3. Test with sample data ✅
4. Integrate with your Lambda
5. Deploy!

---

**You can start implementing immediately!** No need to wait for formal releases or publishing.