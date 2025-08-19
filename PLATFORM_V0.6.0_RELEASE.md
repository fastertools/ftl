# 🚀 FTL CLI v0.6.0-alpha.1 - New Platform Package Ready

**To:** FTL-AWS Platform Team  
**From:** FTL CLI Team  
**Date:** November 2024  
**Subject:** New pkg/platform API - Clean, Explicit, Production-Ready

## Executive Summary

We've released **v0.6.0-alpha.1** with a brand new `pkg/platform` package that provides a clean, explicit API for platform integrations. This replaces the internal/ftl usage and gives you full control over deployment processing.

## Quick Migration (5 minutes)

### 1. Update your go.mod
```go
require github.com/fastertools/ftl-cli v0.6.0-alpha.1
```

### 2. Change your imports
```go
// OLD
import "github.com/fastertools/ftl-cli/internal/ftl"

// NEW
import (
    "github.com/fastertools/ftl-cli/pkg/platform"
    "github.com/fastertools/ftl-cli/pkg/types"
)
```

### 3. Update your Lambda handler
```go
func HandleDeployment(ctx context.Context, req APIGatewayRequest) error {
    // Create client with explicit configuration
    config := platform.Config{
        InjectGateway:     true,
        InjectAuthorizer:  true,
        GatewayVersion:    "0.0.13-alpha.0",
        AuthorizerVersion: "0.0.15-alpha.0",
        
        // Production security settings
        RequireRegistryComponents: true,
        AllowedRegistries: []string{
            "ghcr.io",
            "123456789012.dkr.ecr.us-west-2.amazonaws.com",
        },
        
        MaxComponents:      50,
        DefaultEnvironment: "production",
    }
    
    client := platform.NewClient(config)
    
    // Parse request into platform types
    deployReq := &platform.DeploymentRequest{
        Application: parseToApplication(req.Body),
        Environment: "production",
        Variables: req.Variables,
    }
    
    // Process deployment (handles all platform component injection)
    result, err := client.ProcessDeployment(deployReq)
    if err != nil {
        return fmt.Errorf("deployment processing failed: %w", err)
    }
    
    // Deploy to Fermyon Cloud
    err = deployToFermyon(result.SpinTOML)
    if err != nil {
        return fmt.Errorf("fermyon deployment failed: %w", err)
    }
    
    return nil
}
```

## What's New in v0.6.0

### 1. Explicit Platform Client
```go
// Everything is configured explicitly - no hidden behavior
config := platform.Config{
    InjectGateway:     true,  // Control gateway injection
    InjectAuthorizer:  true,  // Control auth injection
    GatewayVersion:    "0.0.13-alpha.0",
    AuthorizerVersion: "0.0.15-alpha.0",
}

client := platform.NewClient(config)
```

### 2. Clean Deployment Processing
```go
result, err := client.ProcessDeployment(request)
// result contains:
// - result.Manifest: Processed manifest
// - result.SpinTOML: Ready-to-deploy TOML
// - result.Metadata: Deployment metadata
```

### 3. Built-in Security Validation
```go
config.RequireRegistryComponents = true  // Reject local sources
config.AllowedRegistries = []string{     // Whitelist registries
    "ghcr.io",
    "your-ecr.amazonaws.com",
}
config.MaxComponents = 50  // Prevent abuse
```

### 4. Component Source Detection
```go
// Use the types package to parse sources
localPath, registrySource := types.ParseComponentSource(comp.Source)
if localPath != "" {
    // It's a local source - reject in production
    return errors.New("local sources not allowed")
}
if registrySource != nil {
    // It's from a registry
    log.Printf("Component from %s/%s:%s", 
        registrySource.Registry,
        registrySource.Package,
        registrySource.Version)
}
```

## Platform Components (Automatic Injection)

### mcp-gateway (Always Injected)
- Source: `ghcr.io/fastertools:mcp-gateway:0.0.13-alpha.0`
- Route: `/*`
- Handles all routing and request processing

### mcp-authorizer (Injected for Non-Public Apps)
- Source: `ghcr.io/fastertools:mcp-authorizer:0.0.15-alpha.0`
- Configured based on access mode and auth settings
- Handles JWT validation

**Note:** Component package format uses colon separator (`fastertools:mcp-gateway`) for Spin compatibility.

## Complete Integration Example

```go
package lambda

import (
    "context"
    "encoding/json"
    "fmt"
    
    "github.com/fastertools/ftl-cli/pkg/platform"
    "github.com/fastertools/ftl-cli/pkg/types"
)

type LambdaHandler struct {
    client *platform.Client
}

func NewHandler() *LambdaHandler {
    config := platform.Config{
        // Platform components
        InjectGateway:     true,
        InjectAuthorizer:  true,
        GatewayVersion:    "0.0.13-alpha.0",
        AuthorizerVersion: "0.0.15-alpha.0",
        
        // Security
        RequireRegistryComponents: true,
        AllowedRegistries: []string{
            "ghcr.io",
            getECRRegistry(), // Your ECR registry
        },
        
        // Limits
        MaxComponents:      50,
        DefaultEnvironment: "production",
    }
    
    return &LambdaHandler{
        client: platform.NewClient(config),
    }
}

func (h *LambdaHandler) HandleCreateDeployment(ctx context.Context, req APIGatewayRequest) (APIGatewayResponse, error) {
    // Parse request body
    var appConfig platform.Application
    if err := json.Unmarshal([]byte(req.Body), &appConfig); err != nil {
        return errorResponse(400, "invalid request body"), nil
    }
    
    // Validate components before processing
    if err := h.client.ValidateComponents(appConfig.Components); err != nil {
        return errorResponse(400, fmt.Sprintf("validation failed: %v", err)), nil
    }
    
    // Create deployment request
    deployReq := &platform.DeploymentRequest{
        Application: &appConfig,
        Environment: getEnvironment(req),
        Variables:   getVariables(req),
    }
    
    // Add custom auth if needed
    if appConfig.Access == "custom" {
        deployReq.CustomAuth = &platform.CustomAuthConfig{
            Issuer:   getCustomIssuer(req),
            Audience: []string{getCustomAudience(req)},
        }
    }
    
    // Process deployment
    result, err := h.client.ProcessDeployment(deployReq)
    if err != nil {
        // Check for specific errors
        if strings.Contains(err.Error(), "local sources not allowed") {
            return errorResponse(400, "Only registry components allowed in production"), nil
        }
        if strings.Contains(err.Error(), "registry") && strings.Contains(err.Error(), "not in allowed list") {
            return errorResponse(400, "Component from unauthorized registry"), nil
        }
        if strings.Contains(err.Error(), "too many components") {
            return errorResponse(400, "Application exceeds component limit"), nil
        }
        
        return errorResponse(500, fmt.Sprintf("processing failed: %v", err)), nil
    }
    
    // Log deployment metadata
    log.Printf("Deployment processed: %d components, gateway=%v, auth=%v, access=%s",
        result.Metadata.ComponentCount,
        result.Metadata.InjectedGateway,
        result.Metadata.InjectedAuthorizer,
        result.Metadata.AccessMode)
    
    // Deploy to Fermyon Cloud
    deploymentID, err := deployToFermyon(result.SpinTOML)
    if err != nil {
        return errorResponse(500, "Fermyon deployment failed"), nil
    }
    
    // Return success response
    response := map[string]interface{}{
        "deployment_id": deploymentID,
        "components":    result.Metadata.ComponentCount,
        "environment":   result.Metadata.Environment,
        "access_mode":   result.Metadata.AccessMode,
    }
    
    return successResponse(200, response), nil
}

// Helper to check component sources
func (h *LambdaHandler) validateComponentSources(components []platform.Component) error {
    for _, comp := range components {
        localPath, registrySource := types.ParseComponentSource(comp.Source)
        
        if localPath != "" {
            return fmt.Errorf("component %s uses local source: %s", comp.ID, localPath)
        }
        
        if registrySource == nil {
            return fmt.Errorf("component %s has invalid source", comp.ID)
        }
        
        // Additional validation
        if registrySource.Registry == "" || registrySource.Package == "" || registrySource.Version == "" {
            return fmt.Errorf("component %s missing registry details", comp.ID)
        }
    }
    return nil
}
```

## Testing Your Integration

```go
func TestPlatformIntegration(t *testing.T) {
    handler := NewHandler()
    
    req := APIGatewayRequest{
        Body: `{
            "name": "test-app",
            "version": "1.0.0",
            "access": "private",
            "auth": {
                "jwt_issuer": "https://auth.example.com",
                "jwt_audience": "api.example.com"
            },
            "components": [{
                "id": "api",
                "source": {
                    "registry": "ghcr.io",
                    "package": "myorg/api",
                    "version": "1.0.0"
                }
            }]
        }`,
    }
    
    resp, err := handler.HandleCreateDeployment(context.Background(), req)
    assert.NoError(t, err)
    assert.Equal(t, 200, resp.StatusCode)
}
```

## Migration Checklist

- [ ] Update to v0.6.0-alpha.1 in go.mod
- [ ] Change imports from internal/ftl to pkg/platform
- [ ] Create platform.Config with your settings
- [ ] Update component source validation to use types.ParseComponentSource
- [ ] Test with your existing test suite
- [ ] Deploy to staging environment
- [ ] Verify platform components are injected correctly

## Key Differences from v0.5.0

1. **New Package**: `pkg/platform` instead of `internal/ftl`
2. **Explicit Client**: Create client with configuration
3. **Clean Types**: Platform-specific types separate from internal
4. **Better Validation**: Built-in component validation methods
5. **Clear Metadata**: Deployment result includes detailed metadata

## API Reference

### Config Options
```go
type Config struct {
    InjectGateway     bool     // Inject mcp-gateway
    InjectAuthorizer  bool     // Inject mcp-authorizer for non-public
    GatewayVersion    string   // Version of gateway to inject
    AuthorizerVersion string   // Version of authorizer to inject
    
    GatewayRegistry    string   // Registry for gateway (default: ghcr.io)
    AuthorizerRegistry string   // Registry for authorizer (default: ghcr.io)
    
    RequireRegistryComponents bool     // Reject local sources
    AllowedRegistries         []string // Whitelist of registries
    
    DefaultEnvironment string // Default if not specified
    MaxComponents      int    // Maximum components (0 = unlimited)
}
```

### Deployment Result
```go
type DeploymentResult struct {
    Manifest *Manifest           // Processed manifest
    SpinTOML string              // Ready-to-deploy TOML
    Metadata DeploymentMetadata  // Processing metadata
}

type DeploymentMetadata struct {
    ProcessedAt        time.Time
    ComponentCount     int
    InjectedGateway    bool
    InjectedAuthorizer bool
    AccessMode         string
    Environment        string
}
```

## Support

- **Documentation**: See `pkg/platform/README.md`
- **Tests**: See `pkg/platform/client_test.go` for examples
- **Issues**: Tag with `platform-integration`

## Next Steps

1. **Today**: Update to v0.6.0-alpha.1 and test
2. **This Week**: Deploy to staging
3. **Next Week**: Production rollout
4. **Future**: v1.0 with stable API (no more breaking changes)

---

**The new pkg/platform package is production-ready and provides the clean, explicit API you need for reliable platform integrations.**