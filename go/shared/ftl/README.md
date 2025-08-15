# FTL Shared Package

This package provides shared types, schemas, and synthesis capabilities for FTL (Faster Tools) applications. It ensures consistency between the CLI and platform backend.

## Overview

The `ftl` package is designed to be imported by both:
- **FTL CLI**: For local operations, building, and deploying
- **Platform Backend**: For processing deployments and synthesizing Spin manifests

## Key Components

### 1. Types (`types.go`)
Canonical schema definitions for FTL applications:
- `Application`: The core FTL application structure
- `Component`: Individual component definitions
- `AuthConfig`: Authentication configuration
- `ComponentSource`: Local or registry-based sources

### 2. Synthesis (`synthesis.go`)
CUE-based transformation from FTL to Spin manifests:
- `Synthesizer`: Transforms FTL apps to Spin manifests
- `SynthesizeToSpin()`: Creates Spin manifest struct
- `SynthesizeToTOML()`: Creates Spin TOML string

### 3. Deployment (`deployment.go`)
Deployment request/response contracts:
- `DeploymentRequest`: CLI → Platform contract
- `ProcessDeploymentRequest()`: Platform-side processing
- `PrepareDeployment()`: CLI-side preparation

### 4. CUE Patterns (`patterns.cue`)
The embedded CUE transformation logic that ensures consistent synthesis.

## Usage

### CLI Usage

```go
import "github.com/fastertools/ftl-cli/go/shared/ftl"

// Load FTL application from YAML
var app ftl.Application
err := yaml.Unmarshal(configData, &app)

// Prepare deployment
req, err := ftl.PrepareDeployment(&app, ftl.DeploymentOptions{
    Environment: "production",
    Variables: map[string]string{
        "API_KEY": "secret",
    },
})

// Send to platform API
response := apiClient.CreateDeployment(req)
```

### Backend Usage (Lambda/Platform)

```go
import "github.com/fastertools/ftl-cli/go/shared/ftl"

func handleDeployment(request DeploymentRequest) error {
    // Process the deployment request
    manifest, err := ftl.ProcessDeploymentRequest(&request)
    if err != nil {
        return err
    }
    
    // Generate Spin TOML
    synth := ftl.NewSynthesizer()
    toml, err := synth.SynthesizeToTOML(request.Application)
    
    // Deploy with Spin
    return deployToSpin(toml)
}
```

### Direct Synthesis

```go
// Create an FTL application
app := &ftl.Application{
    Name:    "my-app",
    Version: "1.0.0",
    Components: []ftl.Component{
        {
            ID: "api",
            Source: &ftl.RegistrySource{
                Registry: "ghcr.io",
                Package:  "myorg:api",
                Version:  "1.0.0",
            },
        },
    },
    Access: ftl.AccessPrivate,
    Auth: ftl.AuthConfig{
        Provider: ftl.AuthProviderWorkOS,
        OrgID:    "org_123",
    },
}

// Synthesize to Spin manifest
synth := ftl.NewSynthesizer()
manifest, err := synth.SynthesizeToSpin(app)
```

## Schema Validation

The package includes built-in validation:

```go
app := &ftl.Application{...}

// Set defaults (version, access mode, etc.)
app.SetDefaults()

// Validate the configuration
if err := app.Validate(); err != nil {
    log.Fatalf("Invalid app: %v", err)
}
```

## Component Sources

Components can have two types of sources:

### Local Source (for development)
```go
component := ftl.Component{
    ID:     "my-tool",
    Source: ftl.LocalSource("./dist/my-tool.wasm"),
    Build: &ftl.BuildConfig{
        Command: "cargo build --release",
    },
}
```

### Registry Source (for deployment)
```go
component := ftl.Component{
    ID: "my-tool",
    Source: &ftl.RegistrySource{
        Registry: "ecr.amazonaws.com/my-app",
        Package:  "app-id:my-tool",
        Version:  "1.0.0",
    },
}
```

## Ensuring Consistency

To maintain consistency between CLI and platform:

1. **Always use this package** for FTL types and synthesis
2. **Version together**: When updating schemas, update both CLI and backend
3. **Test synthesis**: Use the same test cases for both sides
4. **Embed CUE patterns**: The patterns.cue file is embedded in the binary

## Testing

```go
func TestSynthesis(t *testing.T) {
    app := &ftl.Application{
        Name: "test-app",
        Components: []ftl.Component{...},
    }
    
    synth := ftl.NewSynthesizer()
    manifest, err := synth.SynthesizeToSpin(app)
    
    assert.NoError(t, err)
    assert.Equal(t, "test-app", manifest.Application.Name)
    assert.Contains(t, manifest.Component, "ftl-mcp-gateway")
}
```

## Migration Path

For existing code:
1. Replace local type definitions with `ftl.Application`, `ftl.Component`, etc.
2. Use `ftl.NewSynthesizer()` instead of local synthesis
3. Use `ftl.DeploymentRequest` for API contracts
4. Validate with `app.Validate()` before processing

## Future Enhancements

- [ ] Direct CUE extraction (without intermediate JSON)
- [ ] Streaming synthesis for large applications
- [ ] Component dependency resolution
- [ ] Multi-environment configuration overlays
- [ ] Schema versioning and migration