# WorkOS Authentication Example

This example demonstrates a **private** FTL application secured with WorkOS authentication.

## Configuration

The key configuration for WorkOS authentication:

```yaml
access: private  # Requires authentication

auth:
  provider: workos
  org_id: "org_01HZQR3GNFQ6N1X8Q3ZKJP8QWS"  # Your WorkOS org ID
```

## What This Creates

When synthesized, this configuration:
1. Adds the `mcp-authorizer` component for JWT validation
2. Routes all requests through the authorizer first
3. Configures JWT validation with WorkOS issuer
4. Creates private routes between components

The resulting architecture:
```
Internet → MCP Authorizer → MCP Gateway → Your Tools
              ↓
         JWT Validation
```

## Available Formats

This example shows all three configuration formats:
- `ftl.yaml` - YAML configuration
- `ftl.json` - JSON configuration  
- `main.go` - Programmatic Go configuration

All produce identical output!

## Setting Up WorkOS

1. Sign up at [WorkOS](https://workos.com)
2. Create an organization
3. Get your organization ID from the dashboard
4. Configure SSO or other auth methods

## Usage

```bash
# Using YAML
ftl synth ftl.yaml -o spin.toml

# Using JSON
ftl synth ftl.json -o spin.toml

# Using Go
ftl synth main.go -o spin.toml
# or
go run main.go > spin.toml

# Run the application
spin up
```

## Testing with Authentication

```bash
# Get a JWT token from WorkOS (using your auth flow)
TOKEN="your-jwt-token-here"

# Access tools with authentication
curl -X POST http://127.0.0.1:3000/mcp \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"jsonrpc":"2.0","method":"tools/list","params":{},"id":1}'

# Without auth - will be rejected
curl -X POST http://127.0.0.1:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"tools/list","params":{},"id":1}'
# Returns: 401 Unauthorized
```

## Environment Variables

The synthesized manifest will include these auth-related variables:
- `mcp_jwt_issuer`: Set to WorkOS API endpoint
- `mcp_jwt_audience`: Defaults to your app name
- Component-specific variables you define

## Security Benefits

✅ **Enterprise-grade authentication** via WorkOS  
✅ **JWT validation** on every request  
✅ **SSO support** (SAML, OIDC)  
✅ **Audit trails** through WorkOS dashboard  
✅ **MFA support** when configured in WorkOS