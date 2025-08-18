# Answers for Backend Team

## 1. spin-compose Installation

**A: spin-compose is a separate Go binary we're building**

- It's NOT part of spin CLI - it's our own tool in this monorepo
- Location: `/go/spin-compose` in the ftl-cli repo
- We'll provide pre-built binaries for Lambda (Linux x86_64)
- No runtime dependencies - single static binary
- Download from our releases or build from source:
  ```bash
  cd go/spin-compose
  go build -o spin-compose
  ```

## 2. Component URI Format

**A: Use full OCI registry URIs (without oci:// prefix)**

```yaml
components:
  - id: weather-tool
    source: 123456789.dkr.ecr.us-east-1.amazonaws.com/ftl/app-123/weather-tool:v1
```

- spin-compose will handle these as OCI references
- No need for oci:// prefix
- DO NOT pull to local paths - Spin can fetch from registries directly

## 3. Platform Component Injection

**A: spin-compose will support overlays/layers**

We're designing spin-compose to work with multiple config sources:

```bash
# Merge user config with platform overlay
spin-compose \
  --config user-spinc.yaml \
  --overlay platform-components.yaml \
  --output spin.toml
```

Platform overlay example:
```yaml
# platform-components.yaml
components:
  - id: mcp-gateway
    source: platform.registry.com/mcp-gateway:latest
    environment:
      DOWNSTREAM_URL: "{{ route_to('user-tool') }}"
      
  - id: mcp-authorizer
    source: platform.registry.com/mcp-authorizer:latest
    environment:
      AUTH_ENABLED: "{{ mcp.authorizer.enabled }}"
      ACCESS_CONTROL: "{{ mcp.authorizer.access_control }}"

triggers:
  - type: http
    route: "/*"
    component: mcp-gateway
```

Alternatively, spin-compose can accept platform config via:
- Environment variables: `SPINC_PLATFORM_COMPONENTS`
- Config directory: `--platform-dir /path/to/platform/configs`
- Direct injection: `--add-component mcp-gateway:platform.registry.com/mcp-gateway:latest`

## 4. Access Control Configuration

**A: Yes, it goes in the mcp.authorizer section**

```yaml
mcp:
  authorizer:
    enabled: true
    access_control: private  # ✓ Correct location
    jwt_issuer: "https://auth.ftl.com"  # For custom auth
    org_id: "org-123"  # For org-level access
```

The platform reads these values and configures the authorizer component accordingly.

## 5. Validation

**A: spin-compose will validate, but pre-validate in Lambda**

```go
// In your Lambda handler
config, err := config.LoadSpincYAML(configData)
if err != nil {
    return BadRequest("Invalid config")
}

// Validate before processing
if err := config.Validate(); err != nil {
    return BadRequest(err.Error())
}

// Then run spin-compose (which also validates)
```

We're providing:
- Go validation via the schema package: `/go/shared/config/schema.go`
- JSON Schema: `/go/shared/api/ftl-deployment-schema.yaml`
- spin-compose built-in validation

## 6. Environment Variables

**A: Template syntax with multiple resolution stages**

spinc.yaml supports templates:
```yaml
components:
  - id: my-tool
    environment:
      API_KEY: "{{ secrets.weather_api_key }}"
      LOG_LEVEL: "{{ variables.log_level }}"
      ENDPOINT: "{{ env.API_ENDPOINT }}"
```

Resolution order:
1. Platform secrets (injected by backend)
2. Config variables (from spinc.yaml variables section)
3. Environment variables (runtime)
4. Defaults

Override at deployment:
```bash
spin-compose \
  --set variables.log_level=debug \
  --set-env API_ENDPOINT=https://prod.api.com
```

## Your Understanding - CONFIRMED ✅

You've got it exactly right:
1. ✅ Components are pre-pushed to ECR
2. ✅ spinc.yaml references components by registry URIs
3. ✅ Platform runs spin-compose to merge configs
4. ✅ Final spin.toml includes both user and platform components
5. ✅ Deploy to Fermyon with synthesized config

## Implementation Recommendations

### YES, Start Deleting Transpiler Code! 🎉

The transpiler approach was a stopgap. With spin-compose, you can:
1. Delete all transpiler code
2. Implement the cleaner `/v1/deployments` endpoint
3. Use spin-compose as a library or subprocess

### Lambda Integration Pattern

```python
# Python example for Lambda
import subprocess
import json
import yaml

def handle_deployment(event, context):
    # 1. Parse request
    spinc_config = event['config']
    components = event['components']
    
    # 2. Write configs to temp files
    with open('/tmp/user.yaml', 'w') as f:
        yaml.dump(spinc_config, f)
    
    with open('/tmp/platform.yaml', 'w') as f:
        yaml.dump(platform_overlay, f)
    
    # 3. Run spin-compose
    result = subprocess.run([
        '/opt/bin/spin-compose',
        '--config', '/tmp/user.yaml',
        '--overlay', '/tmp/platform.yaml',
        '--output', '/tmp/spin.toml',
        '--format', 'toml'
    ], capture_output=True)
    
    # 4. Read generated spin.toml
    with open('/tmp/spin.toml', 'r') as f:
        spin_toml = f.read()
    
    # 5. Deploy to Fermyon
    deploy_to_fermyon(spin_toml)
```

### Platform Component Registry

Consider maintaining platform components in a separate ECR repo:
```
platform.ecr.region.amazonaws.com/
  ftl-platform/
    mcp-gateway:v1.0.0
    mcp-authorizer:v1.0.0
    telemetry-collector:v1.0.0
```

## Next Steps

1. **Backend**: Start removing transpiler code
2. **CLI Team**: We'll ensure spin-compose is available as a binary
3. **Both**: Coordinate on testing the new flow

## spin-compose API (for backend integration)

### As a CLI tool:
```bash
spin-compose --config user.yaml --overlay platform.yaml --output spin.toml
```

### As a Go library:
```go
import "github.com/fastertools/ftl-cli/go/spin-compose/lib"

composer := spincompose.New()
spinToml, err := composer.Compose(
    spincompose.WithConfig(userConfig),
    spincompose.WithOverlay(platformConfig),
    spincompose.WithVariables(vars),
)
```

## Sample Timeline

- **Week 1**: Delete transpiler, implement basic /v1/deployments
- **Week 2**: Integrate spin-compose, test component assembly
- **Week 3**: Add platform component injection
- **Week 4**: Full E2E testing with real MCP apps

This architecture is absolutely the right direction - treating the platform as a component orchestrator rather than a config transformer!