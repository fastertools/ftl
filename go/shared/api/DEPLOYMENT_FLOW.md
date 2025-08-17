# FTL Deployment Flow

## Overview

The deployment flow uses `spinc.yaml` as the single source of truth for application configuration. This same configuration drives both local development (via spin-compose) and platform deployments.

## CLI Deployment Flow

1. **Read spinc.yaml** - Load the FTL configuration
2. **Build components** - Run `spin build` if needed
3. **Get ECR credentials** - Call `/v1/registry/tokens` with app ID
4. **Docker login** - Authenticate Docker with ECR credentials
5. **Push components** - Use `spin deps push` for each component
6. **Submit deployment** - Send spinc.yaml + component refs to platform

## New Deployment Endpoint

Instead of the current multi-step process, we propose a simplified endpoint:

### Current Flow (Complex)
```
POST /v1/apps                    # Create app
POST /v1/registry/tokens         # Get ECR token  
PUT /v1/apps/{id}/components     # Update components
POST /v1/apps/{id}/deployments   # Deploy with component list
```

### Proposed Flow (Simple)
```
POST /v1/deployments
{
  "config": { /* spinc.yaml as JSON */ },
  "components": [
    {
      "id": "weather-tool",
      "registry_uri": "123456789.dkr.ecr.us-west-2.amazonaws.com/ftl/app-123/weather-tool:v1.0.0",
      "digest": "sha256:abc123..."
    }
  ],
  "environment": "production"
}
```

## Platform Backend Flow

When the platform receives a deployment request:

1. **Validate spinc.yaml** - Ensure configuration is valid
2. **Run spin-compose** - Synthesize final spin.toml with:
   - User components (from registry)
   - Platform MCP gateway component
   - Platform MCP authorizer component
   - Platform monitoring/telemetry components
3. **Deploy to Fermyon** - Submit synthesized spin.toml to Fermyon Cloud
4. **Return status** - Provide deployment ID and app URL

## Benefits of This Approach

1. **Single Source of Truth** - spinc.yaml defines everything
2. **Platform Reuses Tools** - Backend uses same spin-compose as CLI
3. **Clean Separation** - User components vs platform components
4. **Simpler API** - One endpoint instead of multiple
5. **Consistency** - Local dev and production use same config format

## Component Registry Management

Components are pushed individually before deployment:

```bash
# CLI pushes each component
spin deps push weather-tool.wasm --registry $ECR_REGISTRY
spin deps push calculator.wasm --registry $ECR_REGISTRY
```

The platform tracks these in ECR and references them when building the final app.

## MCP Platform Features

The platform automatically adds:

1. **MCP Gateway** - Routes MCP requests to appropriate tools
2. **MCP Authorizer** - Handles authentication/authorization
3. **Monitoring** - Telemetry and logging components
4. **Rate Limiting** - Request throttling
5. **CORS Handling** - For browser-based clients

## Migration Path

To migrate from current API to new schema:

1. **Phase 1** - Add new `/v1/deployments` endpoint alongside existing
2. **Phase 2** - Update CLI to use new endpoint
3. **Phase 3** - Deprecate old endpoints
4. **Phase 4** - Remove old endpoints

## Example Platform Processing

Given this spinc.yaml:
```yaml
application:
  name: my-app
  
components:
  - id: user-tool
    source: registry.example.com/user-tool:v1
    
mcp:
  gateway:
    enabled: true
  authorizer:
    enabled: true
    access_control: private
```

Platform generates this spin.toml:
```toml
[application]
name = "my-app"

# User component
[[component]]
id = "user-tool"
source = "registry.example.com/user-tool:v1"

# Platform-injected MCP gateway
[[component]]
id = "mcp-gateway"
source = "platform.registry.com/mcp-gateway:latest"
environment = { DOWNSTREAM_URL = "http://user-tool" }

# Platform-injected authorizer
[[component]]
id = "mcp-authorizer"  
source = "platform.registry.com/mcp-authorizer:latest"
environment = { 
  AUTH_ENABLED = "true",
  JWT_ISSUER = "https://auth.ftl.com"
}

# HTTP routing
[[trigger.http]]
route = "/*"
component = "mcp-gateway"
```

## Security Considerations

1. **Component Isolation** - Each component runs in its own WASM sandbox
2. **Registry Auth** - ECR tokens are temporary and scoped
3. **Runtime Auth** - MCP authorizer validates all requests
4. **Network Policy** - Components can only access allowed hosts
5. **Secrets Management** - Platform injects secrets at runtime

## Next Steps

1. Backend team implements `/v1/deployments` endpoint
2. Backend team integrates spin-compose for synthesis
3. CLI team updates deploy command to use new flow
4. Both teams coordinate on testing and rollout