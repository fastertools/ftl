# MCP Authorizer

A high-performance JWT authorizer for Model Context Protocol (MCP) servers, built for Fermyon Spin and WebAssembly deployment.

## Overview

The MCP Authorizer provides OAuth 2.0 bearer token authentication for MCP endpoints. It validates JWT tokens using JWKS or static public keys, enforces scope-based authorization, and forwards authenticated requests to internal MCP gateways.

## Features

- **JWT Authentication**: Validates tokens using JWKS endpoints or static public keys
- **Policy-Based Authorization**: Enforce required scopes for API access
- **WorkOS AuthKit**: Out-of-the-box support with automatic JWKS discovery
- **OAuth 2.0 Discovery**: Standard-compliant metadata endpoints
- **JWKS Caching**: 5-minute cache reduces provider API calls
- **Optional Issuer Validation**: Support for tokens without issuer claims

## Configuration

Configure via Spin variables (environment variables with `MCP_` prefix):

### Core Settings

```toml
# MCP gateway URL to forward authenticated requests
mcp_gateway_url = "http://mcp-gateway.spin.internal"  # default

# Header name for request tracing
mcp_trace_header = "x-trace-id"  # default

# Provider type: "jwt"
mcp_provider_type = "jwt"  # default
```

### JWT Provider Configuration

```toml
# Issuer URL (optional - empty string disables issuer validation)
mcp_jwt_issuer = "https://your-tenant.authkit.app"

# JWKS URI for key discovery (auto-derived for AuthKit domains)
mcp_jwt_jwks_uri = "https://your-tenant.authkit.app/oauth2/jwks"

# OR static public key in PEM format (choose one: jwks_uri OR public_key)
mcp_jwt_public_key = """
-----BEGIN RSA PUBLIC KEY-----
MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA...
-----END RSA PUBLIC KEY-----
"""

# Expected audience (optional - omit to skip validation)
mcp_jwt_audience = "your-api-audience"

# Signing algorithm (optional - defaults to RS256)
mcp_jwt_algorithm = "RS256"

# Required scopes (comma-separated, optional)
mcp_jwt_required_scopes = "read,write"

# OAuth endpoints for discovery (optional)
mcp_oauth_authorize_endpoint = "https://your-tenant.authkit.app/oauth2/authorize"
mcp_oauth_token_endpoint = "https://your-tenant.authkit.app/oauth2/token"
mcp_oauth_userinfo_endpoint = "https://your-tenant.authkit.app/oauth2/userinfo"
```

## Configuration Examples

### WorkOS AuthKit

```toml
[component.mcp-authorizer.variables]
mcp_provider_type = "jwt"
mcp_jwt_issuer = "https://your-tenant.authkit.app"
# JWKS URI auto-derived as: https://your-tenant.authkit.app/oauth2/jwks
```

### Auth0

```toml
[component.mcp-authorizer.variables]
mcp_provider_type = "jwt"
mcp_jwt_issuer = "https://your-domain.auth0.com/"
mcp_jwt_jwks_uri = "https://your-domain.auth0.com/.well-known/jwks.json"
mcp_jwt_audience = "your-api-identifier"
```

## Authentication Flow

1. **Token Extraction**: Bearer token from `Authorization` header
2. **Token Validation**:
   - For JWT: Verify signature using JWKS or public key
   - Check issuer (if configured)
   - Check audience (if configured)
   - Check expiration
   - Validate required scopes
3. **Request Forwarding**: Add auth context headers and forward to gateway
   - `x-auth-client-id`: Client identifier
   - `x-auth-user-id`: User identifier (subject)
   - `x-auth-issuer`: Token issuer
   - `x-auth-scopes`: Space-separated scopes

## OAuth 2.0 Discovery Endpoints

The authorizer implements standard OAuth 2.0 discovery:

- `GET /.well-known/oauth-protected-resource` - RFC 9728 protected resource metadata
- `GET /.well-known/oauth-authorization-server` - Authorization server metadata
- `GET /.well-known/openid-configuration` - OpenID Connect configuration

These endpoints require no authentication and enable automatic client configuration.

## Complete spin.toml Example

```toml
spin_manifest_version = 2

[application]
name = "mcp-with-auth"
version = "0.1.0"

[[trigger.http]]
route = "/..."
component = "mcp-authorizer"

[component.mcp-authorizer]
source = "mcp_authorizer.wasm"
allowed_outbound_hosts = ["http://*.spin.internal", "https://*"]
key_value_stores = ["default"]

[component.mcp-authorizer.variables]
mcp_gateway_url = "http://mcp-gateway.spin.internal"
mcp_provider_type = "jwt"
mcp_jwt_issuer = "https://your-tenant.authkit.app"
mcp_jwt_required_scopes = "mcp:read,mcp:write"
```

## Testing

```bash
# Test without authentication (returns 401)
curl -i http://localhost:3000/mcp

# Response includes WWW-Authenticate header:
# WWW-Authenticate: Bearer error="unauthorized", 
#   error_description="Missing authorization header",
#   resource_metadata="https://localhost:3000/.well-known/oauth-protected-resource"

# Test with JWT token
curl -H "Authorization: Bearer YOUR_JWT_TOKEN" \
     -H "Content-Type: application/json" \
     -d '{"jsonrpc":"2.0","method":"tools/list","id":1}' \
     http://localhost:3000/mcp

# Check discovery endpoints
curl http://localhost:3000/.well-known/oauth-protected-resource
curl http://localhost:3000/.well-known/oauth-authorization-server
```

## Error Responses

Standard OAuth 2.0 error responses:

| Status | Error | Description |
|--------|-------|-------------|
| 401 | `unauthorized` | Missing authorization header |
| 401 | `invalid_token` | Token validation failed (expired, invalid signature, etc.) |
| 500 | `server_error` | Configuration or internal error |

## Security Considerations

- **HTTPS Required**: All issuer and JWKS URLs must use HTTPS
- **No Secrets**: Only public keys are used for verification
- **Scope Enforcement**: Required scopes are validated on every request
- **Token Expiration**: Expired tokens are automatically rejected
- **JWKS Caching**: 5-minute TTL prevents frequent key fetches

## Building

```bash
# Add WASM target
rustup target add wasm32-wasip1

# Build the component
spin build

# Run tests
spin test
```

## Troubleshooting

### "Either mcp_jwt_jwks_uri or mcp_jwt_public_key must be provided"
- For JWKS: Set `mcp_jwt_jwks_uri` to your provider's JWKS endpoint
- For static key: Set `mcp_jwt_public_key` with RSA public key in PEM format
- For AuthKit: Just set `mcp_jwt_issuer` - JWKS URI is auto-derived

### "Cannot specify both mcp_jwt_jwks_uri and mcp_jwt_public_key"
- Choose either JWKS (dynamic) or public key (static), not both
- Clear the unused variable or set it to empty string

### "Token missing required scopes"
- Ensure token includes all scopes listed in `mcp_jwt_required_scopes`
- Token scopes can be in `scope` or `scp` claim
- Required scopes use comma separation, token scopes use space separation