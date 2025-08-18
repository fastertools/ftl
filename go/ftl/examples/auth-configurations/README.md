# FTL Authentication Configurations

## Understanding `access: private`

`access: private` means authentication is **required** - it doesn't specify which provider. You then configure the auth provider separately.

## The 4 Authentication Options

### 1. Platform-Managed WorkOS (Simplest for FTL users)

**ftl.yaml:**
```yaml
application:
  name: my-app
  version: "1.0.0"

# Just set private - FTL handles the rest
access: private

components:
  - id: my-tool
    source: "./my-tool.wasm"
```

**What this creates in the mcp-authorizer:**
- `mcp_jwt_issuer`: `https://divine-lion-50-staging.authkit.app/`
- `mcp_jwt_audience`: `client_01JZM53FW3WYV08AFC4QWQ3BNB` (FTL's WorkOS client ID - **REQUIRED**)
- JWKS URI: Auto-derived from issuer
- Users authenticate through FTL's WorkOS setup

### 2. Organization-Level Access

**ftl.yaml:**
```yaml
application:
  name: my-app
  version: "1.0.0"

access: org  # All org members can access

components:
  - id: my-tool
    source: "./my-tool.wasm"
```

**What this creates in the mcp-authorizer:**
- `mcp_jwt_issuer`: `https://divine-lion-50-staging.authkit.app/`
- `mcp_jwt_audience`: `client_01JZM53FW3WYV08AFC4QWQ3BNB`
- JWKS URI: Auto-derived from issuer
- Platform validates org membership through JWT claims
- All organization members can access

### 3. Custom JWT Provider (Auth0, Okta, Keycloak, etc.)

**ftl.yaml:**
```yaml
application:
  name: my-app
  version: "1.0.0"

access: custom

auth:
  jwt_issuer: "https://your-auth.auth0.com/"  # Your auth provider
  jwt_audience: "your-api-identifier"         # REQUIRED - your API identifier

components:
  - id: my-tool
    source: "./my-tool.wasm"
```

**What this creates in the mcp-authorizer:**
- `mcp_jwt_issuer`: Your specified issuer URL
- `mcp_jwt_audience`: Your specified audience (**REQUIRED** - defaults to app name if omitted in config)
- JWKS URI: Must be derived from issuer or specified
- Users authenticate through YOUR auth system

### 4. Public Access (No Authentication)

**ftl.yaml:**
```yaml
application:
  name: my-app
  version: "1.0.0"

# No authentication required
access: public

components:
  - id: my-tool
    source: "./my-tool.wasm"
```

**What this creates:**
- NO authentication required
- NO mcp-authorizer component
- Direct public access to tools
- Use only for demos or truly public tools

## Important: How the MCP Authorizer Works

The mcp-authorizer component validates JWT tokens by checking:
1. **Signature** - Using JWKS from the issuer or a static public key
2. **Issuer** (`iss` claim) - Must match `mcp_jwt_issuer` 
3. **Audience** (`aud` claim) - Must match `mcp_jwt_audience` (**REQUIRED**)
4. **Expiration** (`exp` claim) - Token must not be expired
5. **Scopes** (optional) - If `mcp_jwt_required_scopes` is set

The authorizer does NOT:
- Check organization IDs (that's in the token claims, not config)
- Manage users or sessions
- Issue tokens (it only validates them)

## Quick Decision Guide

Choose based on your needs:

| Scenario | Configuration | Why |
|----------|--------------|-----|
| **Quick start with FTL** | `access: private` (only) | FTL handles auth for you |
| **Team/Organization** | `access: org` | All org members can access |
| **Existing auth system** | `access: custom` + your issuer/audience | Integrate with Auth0, Okta, etc. |
| **Demo/Development** | `access: public` | No auth needed |

## Authentication Flow

### For Private Access:
```
User Request → MCP Authorizer → (Validates JWT) → MCP Gateway → Your Tools
                     ↓
              [Rejects if invalid]
```

### For Public Access:
```
User Request → MCP Gateway → Your Tools
            (No auth check)
```

## Examples

### Minimal Secure App (Platform-Managed)
```yaml
application:
  name: secure-tools
  version: "1.0.0"
access: private  # That's it! FTL handles the auth
components:
  - id: admin-tool
    source: "./admin.wasm"
```

### Organization App
```yaml
application:
  name: team-tools
  version: "1.0.0"
access: org  # All org members can access
components:
  - id: hr-tool
    source: "./hr.wasm"
```

### Custom Auth Integration
```yaml
application:
  name: integrated-tools
  version: "1.0.0"
access: custom
auth:
  jwt_issuer: "https://mycompany.us.auth0.com/"
  jwt_audience: "https://api.mycompany.com"
components:
  - id: api-tool
    source: "./api.wasm"
```

## Testing Authentication

### With Platform-Managed or WorkOS:
```bash
# Get token from WorkOS (via your SSO flow)
TOKEN="your-jwt-token"

curl -X POST http://localhost:3000/mcp \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"tools/list","id":1}'
```

### With Custom Provider:
```bash
# Get token from your provider (Auth0, Okta, etc.)
TOKEN="your-jwt-token"

curl -X POST http://localhost:3000/mcp \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"tools/list","id":1}'
```

### With Public Access:
```bash
# No token needed!
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"tools/list","id":1}'
```