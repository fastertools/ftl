# MCP Authorizer Configuration Schema

## Core Settings

- `mcp_gateway_url` (string, default: "http://mcp-gateway.spin.internal") - MCP gateway URL to forward authenticated requests
- `mcp_trace_header` (string, default: "x-trace-id") - Header name for request tracing (case-insensitive)
- `mcp_provider_type` (string, default: "jwt") - Authentication provider type: "jwt"

## JWT Provider Settings (when mcp_provider_type = "jwt")

### Required (one of the following)
- `mcp_jwt_jwks_uri` (string) - JWKS endpoint URL for dynamic key discovery
- `mcp_jwt_public_key` (string) - Static RSA public key in PEM format

Note: For AuthKit domains (.authkit.app, .workos.com), JWKS URI is automatically derived from the issuer.

### Optional
- `mcp_jwt_issuer` (string, default: "") - Expected token issuer. Empty string disables issuer validation.
- `mcp_jwt_audience` (string, default: "") - Expected audience. Empty string disables audience validation.
- `mcp_jwt_algorithm` (string, default: "") - Signing algorithm (e.g., RS256, ES256). Empty uses default validation.
- `mcp_jwt_required_scopes` (string, default: "") - Comma-separated list of required scopes

## OAuth Discovery Settings (optional, JWT provider only)

- `mcp_oauth_authorize_endpoint` (string, default: "") - OAuth authorization endpoint
- `mcp_oauth_token_endpoint` (string, default: "") - OAuth token endpoint  
- `mcp_oauth_userinfo_endpoint` (string, default: "") - OAuth userinfo endpoint

## Design Principles

1. **Provider-based configuration** - JWT authentication provider
2. **Automatic JWKS discovery** - AuthKit domains get JWKS URI auto-derived
3. **Optional validation** - Issuer and audience validation can be disabled
4. **Scope-based authorization** - Enforce required scopes on all requests

## Example Configurations

### WorkOS AuthKit (Recommended)
```toml
mcp_provider_type = "jwt"
mcp_jwt_issuer = "https://your-tenant.authkit.app"
# JWKS URI auto-derived as: https://your-tenant.authkit.app/oauth2/jwks
mcp_jwt_required_scopes = "mcp:read,mcp:write"
```

### Auth0
```toml
mcp_provider_type = "jwt"
mcp_jwt_issuer = "https://your-domain.auth0.com/"
mcp_jwt_jwks_uri = "https://your-domain.auth0.com/.well-known/jwks.json"
mcp_jwt_audience = "your-api-identifier"
```

### Static Public Key
```toml
mcp_provider_type = "jwt"
mcp_jwt_issuer = "https://auth.example.com"
mcp_jwt_public_key = """
-----BEGIN RSA PUBLIC KEY-----
MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA...
-----END RSA PUBLIC KEY-----
"""
```

### No Issuer Validation (Legacy Support)
```toml
mcp_provider_type = "jwt"
mcp_jwt_issuer = ""  # Empty string disables issuer validation
mcp_jwt_jwks_uri = "https://auth.example.com/.well-known/jwks.json"
```

## Security Notes

- All issuer and JWKS URLs must use HTTPS (enforced)
- Required scopes are validated using subset checking
- Token expiration is always enforced
- JWKS responses are cached for 5 minutes