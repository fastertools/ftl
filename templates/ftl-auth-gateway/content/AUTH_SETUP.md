# MCP Authorizer Setup

The MCP Authorizer has been added to your project to provide JWT authentication for your MCP endpoints.

## 1. Manual Configuration Required

Update your `spin.toml` file:

1. Find the existing MCP component trigger configuration:
   ```toml
   [[trigger.http]]
   route = "/mcp"
   component = "your-mcp-component"
   ```

2. Update it to use a private route:
   ```toml
   [[trigger.http]]
   route = { private = true }
   component = "ftl-mcp-gateway"
   ```

3. The authorizer component has already been added with the correct routes:
   - `/mcp` - Main MCP endpoint (with JWT authentication)
   - `/.well-known/oauth-protected-resource` - OAuth discovery
   - `/.well-known/oauth-authorization-server` - OAuth discovery

## 2. Authentication Configuration

The MCP Authorizer requires JWT configuration. Set the following variables in your `spin.toml` or via environment variables:

### Required Configuration

```toml
[variables]
# Access control mode (set by platform)
mcp_access_control = { default = "private" }  # "public", "private", "org", or "custom"

# App ownership (set by platform at deploy time)
mcp_user_id = { default = "" }  # User who created the app
mcp_org_id = { default = "" }   # Organization ID (may be empty)

# JWT configuration (required for non-public modes)
mcp_jwt_issuer = { required = true }

# One of these is required:
mcp_jwt_jwks_uri = { default = "" }    # JWKS endpoint URL
mcp_jwt_public_key = { default = "" }  # OR static RSA public key (PEM format)
```

### Optional Configuration

```toml
[variables]
# JWT validation
mcp_jwt_audience = { default = "" }        # Expected audience claim
mcp_jwt_algorithm = { default = "RS256" }  # Signing algorithm
mcp_jwt_required_scopes = { default = "" } # Space-separated required scopes

# OAuth endpoints (for discovery)
mcp_oauth_authorize_endpoint = { default = "" }
mcp_oauth_token_endpoint = { default = "" }
mcp_oauth_userinfo_endpoint = { default = "" }
```

## 3. Provider Examples

### WorkOS AuthKit

```bash
# AuthKit auto-detects JWKS endpoint
export SPIN_VARIABLE_MCP_JWT_ISSUER="https://your-tenant.authkit.app"
export SPIN_VARIABLE_MCP_JWT_AUDIENCE="your-api-identifier"  # optional
spin up
```

### Auth0

```bash
export SPIN_VARIABLE_MCP_JWT_ISSUER="https://your-domain.auth0.com"
export SPIN_VARIABLE_MCP_JWT_JWKS_URI="https://your-domain.auth0.com/.well-known/jwks.json"
export SPIN_VARIABLE_MCP_JWT_AUDIENCE="your-api-identifier"
spin up
```

### Static Public Key (Development)

```bash
export SPIN_VARIABLE_MCP_JWT_ISSUER="https://example.com"
export SPIN_VARIABLE_MCP_JWT_PUBLIC_KEY="-----BEGIN PUBLIC KEY-----
MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA...
-----END PUBLIC KEY-----"
spin up
```

## 4. Access Control Modes

**Local Development:**
- No `[oauth]` section in ftl.toml = Public access (no authentication)
- With `[oauth]` section = Custom OAuth authentication

**Deployment to FTL Engine:**
Use the `--access-control` flag when deploying:
- `ftl eng deploy --access-control public` - No authentication required
- `ftl eng deploy --access-control private` - Only you can access
- `ftl eng deploy --access-control org` - You and your organization can access
- `ftl eng deploy --access-control custom` - Use custom OAuth (requires `[oauth]` in ftl.toml)

## 5. Testing

### Get a JWT Token

```bash
# Using FTL CLI (for FTL-managed auth)
ftl auth token

# Or use your OAuth provider's token endpoint
```

### Make an Authenticated Request

```bash
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -d '{"jsonrpc":"2.0","method":"tools/list","id":1}'
```

### Check OAuth Discovery

```bash
# OAuth Protected Resource metadata
curl http://localhost:3000/.well-known/oauth-protected-resource

# OAuth Authorization Server metadata
curl http://localhost:3000/.well-known/oauth-authorization-server
```

## 6. Troubleshooting

### Common Errors

- **401 Unauthorized**: Check that your JWT token is valid and not expired
- **"Either mcp_jwt_jwks_uri or mcp_jwt_public_key must be provided"**: You must configure one key source (for non-public modes)
- **"Access denied: organization mismatch"**: The token's org_id doesn't match the app's org_id (in org mode)
- **"Access denied: organization membership required"**: Token lacks org_id claim (in org mode)
- **"Access denied: only {user} can access this app"**: Wrong user trying to access private app

### Debug Mode

To see detailed error messages, check the Spin logs:

```bash
spin up --log-level debug
```