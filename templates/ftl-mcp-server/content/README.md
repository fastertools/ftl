# {{project-name}}

{{project-description}}

## Getting Started

This FTL MCP server is ready to host MCP tools. Add tools using:

```bash
# Add a TypeScript tool
ftl add -t ftl-mcp-ts

# Add a Rust tool
ftl add -t ftl-mcp-rust
```

## Running the Server

```bash
ftl up
```

The MCP endpoint will be available at `http://localhost:3000/mcp`

## Testing

List available tools:
```bash
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"tools/list","id":1}'
```

## Authentication

This MCP server includes authentication support via FTL Auth Gateway, which is **disabled by default**. 

To enable authentication, edit the `[auth]` section in `ftl.toml`:

Example configuration with WorkOS AuthKit:
```toml
[auth]
enabled = true
provider = "authkit"
issuer = "https://your-tenant.authkit.app"
audience = "mcp-api"  # optional
```

Example configuration with Auth0:
```toml
[auth]
enabled = true
provider = "oidc"
issuer = "https://your-domain.auth0.com"
audience = "your-api-identifier"  # optional

[auth.oidc]
provider_name = "auth0"
jwks_uri = "https://your-domain.auth0.com/.well-known/jwks.json"
authorize_endpoint = "https://your-domain.auth0.com/authorize"
token_endpoint = "https://your-domain.auth0.com/oauth/token"
userinfo_endpoint = "https://your-domain.auth0.com/userinfo"  # optional
allowed_domains = "*.auth0.com"  # optional
```

When authentication is enabled, the auth gateway will:
- Handle OAuth 2.0 flows at `/.well-known/oauth-protected-resource` and `/.well-known/oauth-authorization-server`
- Validate tokens on requests to `/mcp`
- Forward authenticated requests to the internal MCP gateway