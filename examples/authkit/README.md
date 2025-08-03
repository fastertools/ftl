# AuthKit Example

FTL MCP server with WorkOS AuthKit authentication.

## Overview

This example demonstrates how to secure an MCP server using WorkOS AuthKit for JWT authentication. The MCP authorizer component provides out-of-the-box support for AuthKit with automatic JWKS discovery.

## Getting Started

### 1. Set up AuthKit

First, ensure you have a WorkOS AuthKit domain. You'll need the domain URL (e.g., `https://your-tenant.authkit.app`).

### 2. Configure Authentication

Set the AuthKit domain using an environment variable:

```bash
export SPIN_VARIABLE_AUTHKIT_DOMAIN="https://your-tenant.authkit.app"
```

Optionally, require specific scopes for API access:

```bash
export SPIN_VARIABLE_MCP_REQUIRED_SCOPES="mcp:read,mcp:write"
```

### 3. Start the Server

```bash
spin up
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

Then add to `tool_components` in spin.toml:
```toml
tool_components = { default = "example-tool-component,my-tool" }
```

### Pre-built Tools

```bash
ftl tools add calculator
```

## Configuration Options

### Required Configuration

- `SPIN_VARIABLE_AUTHKIT_DOMAIN`: Your WorkOS AuthKit domain (e.g., `https://your-tenant.authkit.app`)

### Optional Configuration

- `SPIN_VARIABLE_MCP_REQUIRED_SCOPES`: Comma-separated list of required scopes (e.g., `read,write,admin`)

### Advanced Configuration

For non-AuthKit JWT providers, you can manually configure the JWKS endpoint by modifying the spin.toml variables directly or using environment variables:

```bash
export SPIN_VARIABLE_MCP_JWT_ISSUER="https://auth.example.com"
export SPIN_VARIABLE_MCP_JWT_JWKS_URI="https://auth.example.com/.well-known/jwks.json"
export SPIN_VARIABLE_MCP_JWT_AUDIENCE="your-api-audience"
```

## Development Mode

For local development without authentication, you can use static tokens:

1. Create a `.env.local` file:
```bash
MCP_PROVIDER_TYPE=static
MCP_STATIC_TOKENS="dev-token:dev-client:dev-user:read,write"
```

2. Start with local config:
```bash
spin up --env-file .env.local
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

Make sure to set the `SPIN_VARIABLE_AUTHKIT_DOMAIN` environment variable in your deployment environment.

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