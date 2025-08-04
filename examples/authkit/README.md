# AuthKit Example

FTL MCP server with WorkOS AuthKit authentication.

## Overview

This example demonstrates how to secure an MCP server using WorkOS AuthKit for JWT authentication. The MCP authorizer component provides out-of-the-box support for AuthKit with automatic JWKS discovery.

## Getting Started

### 1. Set up AuthKit

First, ensure you have a WorkOS AuthKit domain. You'll need the domain URL (e.g., `https://your-tenant.authkit.app`).

### 2. Configure Authentication

Update the `ftl.toml` file to enable AuthKit authentication:

```toml
[auth]
enabled = true

[auth.authkit]
issuer = "https://your-tenant.authkit.app"
audience = "mcp-api"  # optional
required_scopes = "mcp:read,mcp:write"  # optional
```

Alternatively, you can override settings with environment variables:

```bash
export SPIN_VARIABLE_MCP_JWT_ISSUER="https://your-tenant.authkit.app"
export SPIN_VARIABLE_MCP_JWT_AUDIENCE="mcp-api"
export SPIN_VARIABLE_MCP_JWT_REQUIRED_SCOPES="mcp:read,mcp:write"
```

Note: The old `SPIN_VARIABLE_AUTHKIT_DOMAIN` variable is no longer used. Use `SPIN_VARIABLE_MCP_JWT_ISSUER` instead.

### 3. Start the Server

```bash
ftl up
```

The server will be available at http://localhost:3000/mcp

## Authentication

### Without Authentication (401 Response)

```bash
curl -i http://localhost:3000/mcp
```

Response:
```
HTTP/1.1 401 Unauthorized
WWW-Authenticate: Bearer error="unauthorized", error_description="Missing authorization header", resource_metadata="http://localhost:3000/.well-known/oauth-protected-resource"
```

### With JWT Token

```bash
curl -H "Authorization: Bearer YOUR_JWT_TOKEN" \
     -H "Content-Type: application/json" \
     -d '{"jsonrpc":"2.0","method":"tools/list","id":1}' \
     http://localhost:3000/mcp
```

### OAuth Discovery Endpoints

The following endpoints are available without authentication:

```bash
# Protected resource metadata
curl http://localhost:3000/.well-known/oauth-protected-resource

# Authorization server metadata
curl http://localhost:3000/.well-known/oauth-authorization-server

# OpenID configuration
curl http://localhost:3000/.well-known/openid-configuration
```

## Adding Tools

### Custom Tools

```bash
ftl add my-tool --language rust
```

The tool will be automatically included in the generated spin.toml.

### Pre-built Tools

```bash
ftl tools add calculator
```

## Configuration Options

### Required Configuration

- AuthKit issuer in `ftl.toml` or `SPIN_VARIABLE_MCP_JWT_ISSUER` environment variable

### Optional Configuration

- `audience`: Expected audience for tokens
- `required_scopes`: Comma-separated list of required scopes (e.g., `mcp:read,mcp:write`)

### Advanced Configuration

For non-AuthKit JWT providers, configure OIDC in `ftl.toml`:

```toml
[auth]
enabled = true

[auth.oidc]
issuer = "https://auth.example.com"
audience = "your-api-audience"  # optional
jwks_uri = "https://auth.example.com/.well-known/jwks.json"
required_scopes = "read,write"  # optional
```

## Development Mode

For local development without authentication, you can use static tokens:

1. Configure static tokens in `ftl.toml`:
```toml
[auth]
enabled = true

[auth.static_token]
tokens = "dev-token:dev-client:dev-user:read,write"
required_scopes = "read"  # optional
```

2. Start the server:
```bash
ftl up
```

3. Use the static token:
```bash
curl -H "Authorization: Bearer dev-token" \
     -H "Content-Type: application/json" \
     -d '{"jsonrpc":"2.0","method":"tools/list","id":1}' \
     http://localhost:3000/mcp
```

## Deployment

Deploy your authenticated MCP server:

```bash
ftl deploy
```

Make sure to configure the appropriate authentication settings in your `ftl.toml` or through environment variables.

## How It Works

1. **MCP Authorizer** receives incoming requests at `/mcp`
2. Validates JWT tokens against WorkOS AuthKit (JWKS auto-discovered)
3. Checks required scopes if configured
4. Forwards authenticated requests to internal MCP gateway
5. **MCP Gateway** routes requests to appropriate tool components

## Security Notes

- All AuthKit domains automatically use HTTPS
- JWKS endpoints are cached for 5 minutes to reduce API calls
- Token expiration is always enforced
- Required scopes are validated on every request

For more information, visit:
- [FTL Documentation](https://docs.fastertools.com)
- [WorkOS AuthKit](https://workos.com/docs/authkit)