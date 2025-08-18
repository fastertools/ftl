# FTL Examples

This directory contains comprehensive examples demonstrating FTL's configuration capabilities, access modes, and authentication options.

## Quick Start Examples

### 📂 Access Control & Authentication
- **[public-access/](./public-access)** - Open access without authentication
- **[workos-auth/](./workos-auth)** - Enterprise SSO with WorkOS
- **[custom-jwt-auth/](./custom-jwt-auth)** - Custom JWT providers (Auth0, Okta, etc.)
- **[local-development/](./local-development)** - Development setup with build configs

### 📂 Configuration Formats
- **[yaml-format/](./yaml-format)** - YAML configuration
- **[json-format/](./json-format)** - JSON configuration  
- **[go-format/](./go-format)** - Programmatic Go configuration
- **[cue-format/](./cue-format)** - Native CUE configuration

## Access Modes Overview

FTL supports two primary access modes that control how your MCP tools are exposed:

### 🔓 Public Access
```yaml
access: public  # Anyone can access - no authentication
```
Use for: demos, public tools, local development

### 🔒 Private Access
```yaml
access: private  # Requires authentication
auth:
  provider: workos  # or "custom"
  # ... auth config
```
Use for: production, sensitive tools, enterprise deployments

## Authentication Providers

When using `access: private`, you must configure authentication:

### WorkOS (Enterprise SSO)
```yaml
auth:
  provider: workos
  org_id: "org_YOUR_ID"
```
- Single Sign-On (SAML, OIDC)
- Enterprise directory sync
- Audit logs
- MFA support

### Custom JWT
```yaml
auth:
  provider: custom
  jwt_issuer: "https://your-auth.com"
  jwt_audience: "your-app"
```
Works with: Auth0, Okta, Keycloak, AWS Cognito, custom providers

## Configuration Formats

### 1. YAML Format (`yaml-format/`)
Simple, declarative configuration in YAML.

```bash
cd yaml-format
ftl synth ftl.yaml -o spin.toml  # Generate spin.toml
# or
ftl build                         # Build with automatic synthesis
spin up                           # Run the application
```

### 2. Go Format (`go-format/`)
Programmatic configuration using the FTL SDK in Go.

```bash
cd go-format
ftl synth main.go -o spin.toml   # Generate spin.toml
# or
go run main.go > spin.toml        # Run directly
spin up                           # Run the application
```

### 3. JSON Format (`json-format/`)
Standard JSON configuration for programmatic generation or tool integration.

```bash
cd json-format
ftl synth ftl.json -o spin.toml  # Generate spin.toml
# or
ftl build --config ftl.json       # Build with automatic synthesis
spin up                           # Run the application
```

## Key Point: Identical Output

All three formats produce **identical** `spin.toml` files. This demonstrates FTL's powerful synthesis engine that:

1. **Abstracts complexity** - Users write simple configs
2. **Adds intelligence** - Automatically includes MCP gateway, routing, and wiring
3. **Maintains consistency** - Same output regardless of input format

## Testing the Applications

Once running with `spin up`, test the MCP endpoint:

```bash
# List available tools
curl -X POST http://127.0.0.1:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"tools/list","params":{},"id":1}'

# Call a tool
echo '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"geo__example_tool","arguments":{"message":"Hello"}},"id":2}' | \
  curl -X POST http://127.0.0.1:3000/mcp \
    -H "Content-Type: application/json" \
    -d @-
```

## Complete Example: From Zero to Deployed

Here's a complete workflow showing access configuration:

### Step 1: Create Configuration

**Public Access** (`public-app.yaml`):
```yaml
application:
  name: my-tools
  version: "1.0.0"
access: public  # ← Key setting
components:
  - id: calculator
    source:
      registry: ghcr.io
      package: "tools:calc"
      version: "1.0.0"
```

**Private with Auth** (`private-app.yaml`):
```yaml
application:
  name: secure-tools
  version: "1.0.0"
access: private  # ← Requires auth
auth:
  provider: workos
  org_id: "org_123"  # ← Your WorkOS org
components:
  - id: admin-tool
    source:
      registry: ghcr.io
      package: "tools:admin"
      version: "1.0.0"
```

### Step 2: Synthesize to Spin Manifest

```bash
# Public app
ftl synth public-app.yaml -o spin.toml

# Private app with auth
ftl synth private-app.yaml -o spin.toml
```

### Step 3: What Gets Generated

**Public** generates:
- Direct routing to MCP gateway
- No authentication components
- Open access on port 3000

**Private** generates:
- MCP authorizer component
- JWT validation configuration  
- Protected internal routes
- Auth middleware chain

### Step 4: Deploy and Test

```bash
# Deploy locally
spin up

# Deploy to Fermyon Cloud
spin deploy

# Test public endpoint (no auth)
curl http://127.0.0.1:3000/mcp ...

# Test private endpoint (needs JWT)
curl -H "Authorization: Bearer $TOKEN" http://127.0.0.1:3000/mcp ...
```

## Architecture: How Access Modes Work

### Public Mode Architecture
```
Internet → MCP Gateway → Your Tools
         (No auth layer)
```

### Private Mode Architecture  
```
Internet → MCP Authorizer → MCP Gateway → Your Tools
         (JWT validation)
```

The synthesis process automatically:
1. Detects your access mode
2. Adds required auth components
3. Configures proper routing
4. Sets up security policies

## Understanding the Synthesis

FTL uses a pure CUE-based transformation pipeline:

```
User Config → CUE Patterns → Spin Manifest
```

The magic happens in the CUE transformations (`patterns.cue`) that:
- Automatically inject required components (MCP gateway, authorizer)
- Configure routing based on access mode
- Set up component communication
- Apply security defaults
- Validate configuration correctness

This is **platform engineering as code** - encoding best practices, security, and architectural decisions into the synthesis process.

## Best Practices

1. **Start with public** for local development
2. **Use private + WorkOS** for enterprise deployments
3. **Use private + custom JWT** for existing auth systems
4. **Mix local and registry** components during development
5. **Test auth locally** before deploying to production

## Need Help?

- Check individual example READMEs for detailed instructions
- Review `patterns.cue` to understand transformations
- Use `ftl synth --help` for command options
- File issues at the FTL repository