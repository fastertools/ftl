# FTL Platform Integration Package

The `pkg/platform` package provides the official API for integrating FTL with cloud platforms that deploy applications to WebAssembly runtimes like Fermyon Cloud.

## Installation

```go
import "github.com/fastertools/ftl/pkg/platform"
```

## Quick Start

```go
config := platform.DefaultConfig()
client := platform.NewClient(config)

request := &platform.DeploymentRequest{
    Application: &platform.Application{
        Name:    "my-app",
        Version: "1.0.0",
        Components: []platform.Component{
            {
                ID: "api",
                Source: map[string]interface{}{
                    "registry": "ghcr.io",
                    "package":  "myorg/api",
                    "version":  "v1.0.0",
                },
            },
        },
    },
}

result, err := client.ProcessDeployment(request)
if err != nil {
    return err
}

// Deploy to Fermyon using result.SpinTOML
```

## Configuration

### Default Configuration

```go
config := platform.DefaultConfig()
```

This provides:
- Gateway injection enabled
- Authorizer injection for non-public apps
- Registry-only components required
- 50 component limit

### Custom Configuration

```go
config := platform.Config{
    InjectGateway:     true,
    InjectAuthorizer:  true,
    GatewayVersion:    "0.0.13-alpha.0",
    AuthorizerVersion: "0.0.15-alpha.0",
    
    RequireRegistryComponents: true,
    AllowedRegistries: []string{
        "ghcr.io",
        "123456789012.dkr.ecr.us-west-2.amazonaws.com",
    },
    
    MaxComponents:      100,
    DefaultEnvironment: "production",
}
```

## Platform Components

The platform automatically injects security components:

### mcp-gateway
- Always injected when `InjectGateway` is true
- Handles routing and request processing
- Source: `ghcr.io/fastertools:mcp-gateway`

### mcp-authorizer
- Injected for non-public applications when `InjectAuthorizer` is true
- Handles JWT authentication
- Source: `ghcr.io/fastertools:mcp-authorizer`

## Access Modes

- `public`: No authentication required
- `private`: Organization authentication required
- `org`: Organization with specific roles
- `custom`: Custom JWT authentication

## Component Sources

Components can be from registries or local paths:

```go
// Registry source
Source: map[string]interface{}{
    "registry": "ghcr.io",
    "package":  "myorg/component",
    "version":  "1.0.0",
}

// Local source (may be rejected in production)
Source: "./build/component.wasm"
```

## Validation

Pre-validate components before processing:

```go
err := client.ValidateComponents(components)
if err != nil {
    // Handle validation error
}
```

## AWS Lambda Integration Example

```go
package lambda

import (
    "context"
    "github.com/fastertools/ftl/pkg/platform"
)

func HandleDeployment(ctx context.Context, req APIGatewayRequest) (APIGatewayResponse, error) {
    config := platform.DefaultConfig()
    config.RequireRegistryComponents = true
    config.AllowedRegistries = []string{
        "ghcr.io",
        getECRRegistry(),
    }
    
    client := platform.NewClient(config)
    
    deployReq := &platform.DeploymentRequest{
        Application: parseApplication(req.Body),
        Environment: "production",
    }
    
    result, err := client.ProcessDeployment(deployReq)
    if err != nil {
        return errorResponse(err), nil
    }
    
    // Deploy to Fermyon
    err = deployToFermyon(result.SpinTOML)
    if err != nil {
        return errorResponse(err), nil
    }
    
    return successResponse(result.Metadata), nil
}
```

## Security Features

1. **Component Source Validation**: Reject local sources in production
2. **Registry Whitelist**: Only allow components from approved registries
3. **Component Limits**: Prevent resource abuse
4. **Automatic Auth Injection**: Add authentication for non-public apps

## Error Handling

The package returns detailed errors for common issues:

- Invalid component sources
- Registry not in whitelist
- Too many components
- Missing required fields
- Synthesis failures

## Testing

```go
func TestPlatformIntegration(t *testing.T) {
    config := platform.DefaultConfig()
    client := platform.NewClient(config)
    
    request := &platform.DeploymentRequest{
        Application: &platform.Application{
            Name:    "test-app",
            Version: "1.0.0",
            Components: []platform.Component{
                {
                    ID: "test",
                    Source: map[string]interface{}{
                        "registry": "ghcr.io",
                        "package":  "test/component",
                        "version":  "1.0.0",
                    },
                },
            },
        },
    }
    
    result, err := client.ProcessDeployment(request)
    assert.NoError(t, err)
    assert.NotNil(t, result.Manifest)
    assert.NotEmpty(t, result.SpinTOML)
}
```