# MCP Authorizer Configuration Schema

## Core Settings

- `mcp_gateway_url` (string, required) - MCP gateway URL to forward requests to
- `mcp_trace_header` (string, default: "x-trace-id") - Header name for request tracing

## JWT Provider Settings

- `mcp_jwt_issuer` (string, required) - JWT token issuer URL (must be HTTPS)
- `mcp_jwt_audience` (string, optional) - Expected audience for JWT validation
- `mcp_jwt_jwks_uri` (string, optional*) - JWKS endpoint for key discovery
- `mcp_jwt_public_key` (string, optional*) - Static RSA public key in PEM format

*One of `mcp_jwt_jwks_uri` or `mcp_jwt_public_key` is required

## OAuth Discovery Settings (optional)

- `mcp_oauth_authorize_endpoint` (string, optional) - OAuth authorization endpoint
- `mcp_oauth_token_endpoint` (string, optional) - OAuth token endpoint
- `mcp_oauth_userinfo_endpoint` (string, optional) - OAuth userinfo endpoint

## Design Principles

1. **Prefix all variables with `mcp_`** to avoid conflicts
2. **Flat structure** - no complex provider types, just direct configuration
3. **Clear naming** - `jwt_` prefix for JWT-specific, `oauth_` for OAuth endpoints
4. **Minimal required fields** - only issuer and one key source required
5. **Secure by default** - authentication always required, HTTPS enforced

## Example Configurations

### With JWKS Discovery
```toml
mcp_jwt_issuer = "https://auth.example.com"
mcp_jwt_jwks_uri = "https://auth.example.com/.well-known/jwks.json"
mcp_jwt_audience = "my-api"
```

### With Static Public Key
```toml
mcp_jwt_issuer = "https://auth.example.com"
mcp_jwt_public_key = """
-----BEGIN PUBLIC KEY-----
MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA...
-----END PUBLIC KEY-----
"""
```

### AuthKit (Automatic JWKS Discovery)
```toml
mcp_jwt_issuer = "https://tenant.authkit.app"
# JWKS URI will be derived as https://tenant.authkit.app/.well-known/jwks.json
```