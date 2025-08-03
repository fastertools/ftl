# FTL MCP Authorizer

A lightweight JWT authentication gateway for Model Context Protocol (MCP) servers, designed specifically for Fermyon Spin and WebAssembly deployment.

## Overview

The FTL MCP Authorizer provides OAuth 2.0 bearer token authentication for MCP endpoints. It validates JWT tokens against configured OIDC providers and injects user context into MCP requests. Built for serverless WebAssembly environments, it uses Spin's Key-Value store for JWKS caching.

```
MCP Client → FTL MCP Authorizer → FTL MCP Gateway → Tool Components
             (Bearer Token Auth)    (Internal)        (WASM)
```

## Key Features

- **JWT/OIDC Authentication**: Validates tokens from any OIDC-compliant provider
- **Multi-Provider Support**: Configure multiple providers (AuthKit, Auth0, Okta, etc.)
- **OAuth 2.0 Discovery**: Standard-compliant metadata endpoints
- **JWKS Caching**: 5-minute cache in KV store reduces provider API calls
- **Serverless Native**: Zero in-memory state, built for Fermyon Spin
- **MCP Integration**: Automatic user context injection into MCP initialize requests

## Configuration

Configure using Spin variables (environment variables):

### Core Settings

```toml
[component.mcp-authorizer.variables]
# Enable/disable authentication (default: false)
auth_enabled = "true"

# Internal MCP gateway URL
auth_gateway_url = "http://ftl-mcp-gateway.spin.internal/mcp-internal"

# Trace ID header for request correlation
auth_trace_header = "X-Trace-Id"
```

### AuthKit Provider

```toml
auth_provider_type = "authkit"
auth_provider_issuer = "https://your-domain.authkit.app"
auth_provider_audience = "your-api-audience"  # Optional
```

### Generic OIDC Provider

```toml
auth_provider_type = "oidc"
auth_provider_name = "auth0"
auth_provider_issuer = "https://your-domain.auth0.com"
auth_provider_jwks_uri = "https://your-domain.auth0.com/.well-known/jwks.json"
auth_provider_audience = "your-api-audience"  # Optional
auth_provider_authorize_endpoint = "https://your-domain.auth0.com/authorize"
auth_provider_token_endpoint = "https://your-domain.auth0.com/oauth/token"
auth_provider_userinfo_endpoint = "https://your-domain.auth0.com/userinfo"  # Optional
auth_provider_allowed_domains = "*.auth0.com"  # Comma-separated list
```

## Complete Example

```toml
spin_manifest_version = 2

[application]
name = "mcp-with-auth"
version = "0.1.0"

# MCP Authorizer
[[trigger.http]]
route = "/mcp"
component = "mcp-authorizer"

[component.mcp-authorizer]
source = "target/wasm32-wasip1/release/ftl_mcp_authorizer.wasm"
allowed_outbound_hosts = ["http://*.spin.internal", "https://*"]
key_value_stores = ["default"]

[component.mcp-authorizer.variables]
auth_enabled = "true"
auth_gateway_url = "http://ftl-mcp-gateway.spin.internal/mcp-internal"
auth_provider_type = "authkit"
auth_provider_issuer = "https://your-tenant.authkit.app"
auth_provider_audience = "mcp-api"

# MCP Gateway - internal endpoint
[[trigger.http]]
route = "/mcp-internal"
component = "ftl-mcp-gateway"

[component.ftl-mcp-gateway]
source = { registry = "ghcr.io", package = "fastertools:ftl-mcp-gateway", version = "0.0.3" }
allowed_outbound_hosts = ["http://*.spin.internal"]
```

## Authentication Flow

1. **Client Request**: MCP client sends request with `Authorization: Bearer <token>`
2. **Token Extraction**: Authorizer extracts bearer token from header
3. **JWKS Verification**: 
   - Fetches JWKS from provider (with 5-minute caching)
   - Validates JWT signature using public key
   - Checks issuer, audience, and expiration
4. **Context Injection**: Injects user info into MCP initialize requests:
   ```json
   {
     "_authContext": {
       "authenticated_user": "user123",
       "email": "user@example.com",
       "provider": "authkit"
     }
   }
   ```
5. **Request Forwarding**: Forwards authenticated request to MCP gateway

## OAuth Discovery Endpoints

The authorizer implements standard OAuth 2.0 discovery:

- `GET /.well-known/oauth-protected-resource` - Resource server metadata
- `GET /.well-known/oauth-authorization-server` - Authorization server metadata

These endpoints enable MCP clients to discover authentication requirements automatically.

## Development

### Prerequisites
- Rust with `wasm32-wasip1` target
- Spin CLI v2.0+

### Building
```bash
# Add WASM target
rustup target add wasm32-wasip1

# Build
cargo build --target wasm32-wasip1 --release

# Run tests
cargo test
```

### Testing Authentication

```bash
# Start the server
spin up

# Test unauthenticated (returns 401)
curl -i http://localhost:3000/mcp

# Response includes OAuth discovery:
# WWW-Authenticate: Bearer error="unauthorized", 
#   error_description="Missing authorization header",
#   resource_metadata="http://localhost:3000/.well-known/oauth-protected-resource"

# Test with JWT token
curl -H "Authorization: Bearer YOUR_JWT_TOKEN" \
     -H "Content-Type: application/json" \
     -d '{"jsonrpc":"2.0","method":"tools/list","id":1}' \
     http://localhost:3000/mcp
```

## Error Responses

The authorizer returns standard OAuth 2.0 error responses:

| Status | Error | Description |
|--------|-------|-------------|
| 401 | `unauthorized` | Missing authorization header |
| 401 | `invalid_token` | Token validation failed |
| 500 | `server_error` | Internal server error |

## Security

- **HTTPS Only**: Provider URLs must use HTTPS
- **No Secrets**: All verification uses public keys from JWKS
- **Automatic Expiration**: Cached JWKS expires after 5 minutes
- **Standard Compliance**: Follows OAuth 2.0 and OpenID Connect specifications

## Architecture

The authorizer follows MCP and OAuth standards:

```
src/
├── lib.rs              # Main request handler
├── token_verifier.rs   # JWT verification logic
├── providers.rs        # Provider abstractions
├── metadata.rs         # OAuth discovery endpoints
├── proxy.rs           # MCP gateway forwarding
├── kv.rs              # KV store for JWKS caching
├── config.rs          # Configuration management
└── logging.rs         # Structured logging
```

## Testing

The MCP authorizer includes a comprehensive test suite with parity to FastMCP's JWT provider tests. See [tests/README.md](tests/README.md) for detailed testing documentation.

### Quick Test
```bash
# Run integration tests
pip install -r tests/requirements.txt
python tests/run_integration_tests.py
```

## Troubleshooting

### "No authentication providers configured"
- Ensure `auth_provider_type` is set to "authkit" or "oidc"
- Check all required provider variables are set

### "Failed to fetch JWKS"
- Verify `allowed_outbound_hosts` includes provider domains
- Check JWKS URI is accessible and returns valid JSON

### "Invalid audience"
- Set `auth_provider_audience` to expected value
- Or omit for no audience validation

## License

Apache-2.0