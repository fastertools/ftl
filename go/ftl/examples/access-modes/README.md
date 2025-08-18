# FTL Access Modes

FTL provides four distinct access control modes to match different security and sharing requirements.

## The 4 Access Modes

### 1. `public` - No Authentication
**Use when:** Building demos, public tools, or development environments

```yaml
access: public
```

- ✅ No authentication required
- ✅ Anyone can access
- ❌ No mcp-authorizer component
- ⚠️ Use only for non-sensitive tools

### 2. `private` - User-Only Access
**Use when:** Building personal tools that only you should access

```yaml
access: private
```

- ✅ FTL platform handles authentication
- ✅ Uses FTL's WorkOS setup
- ✅ Only the authenticated user can access
- 🔒 JWT validation with FTL's client ID
- 📝 No additional config needed

**What gets configured:**
- `mcp_jwt_issuer`: `https://divine-lion-50-staging.authkit.app/`
- `mcp_jwt_audience`: `client_01JZM53FW3WYV08AFC4QWQ3BNB`

### 3. `org` - Organization-Level Access
**Use when:** Building team tools that all org members should access

```yaml
access: org
```

- ✅ FTL platform handles authentication
- ✅ Platform validates org membership
- ✅ All organization members can access
- 🏢 Shared within your organization
- 🔜 Future: M2M tokens scoped to org
- 📝 No additional config needed

**What gets configured:**
- `mcp_jwt_issuer`: `https://divine-lion-50-staging.authkit.app/`
- `mcp_jwt_audience`: `client_01JZM53FW3WYV08AFC4QWQ3BNB`
- Platform layer handles org validation

### 4. `custom` - Bring Your Own Auth
**Use when:** Integrating with existing auth systems

```yaml
access: custom
auth:
  jwt_issuer: "https://your-auth-provider.com"
  jwt_audience: "your-api-identifier"
```

- ✅ Full control over authentication
- ✅ Works with any JWT provider
- ✅ Integrate with existing systems
- 🔧 You configure everything
- 📝 Must provide auth config

## Quick Decision Tree

```
Do you need authentication?
├─ No → use `public`
└─ Yes → Who should access?
    ├─ Just me → use `private`
    ├─ My organization → use `org`
    └─ Complex/existing auth → use `custom`
```

## Examples

### Public Demo App
```yaml
application:
  name: demo-tools
  version: "1.0.0"
access: public  # Anyone can use
components:
  - id: calculator
    source: "./calc.wasm"
```

### Personal Tools
```yaml
application:
  name: my-tools
  version: "1.0.0"
access: private  # Only you can use
components:
  - id: personal-assistant
    source: "./assistant.wasm"
```

### Team Tools
```yaml
application:
  name: team-tools
  version: "1.0.0"
access: org  # Your whole org can use
components:
  - id: shared-dashboard
    source: "./dashboard.wasm"
```

### Enterprise Integration
```yaml
application:
  name: enterprise-app
  version: "1.0.0"
access: custom  # You control auth
auth:
  jwt_issuer: "https://auth.company.com"
  jwt_audience: "api.company.com"
components:
  - id: enterprise-api
    source: "./api.wasm"
```

## Testing Each Mode

### Public (no auth needed)
```bash
curl http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"tools/list","id":1}'
```

### Private/Org (FTL auth)
```bash
# Get token from FTL platform
TOKEN="your-ftl-jwt-token"

curl http://localhost:3000/mcp \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"tools/list","id":1}'
```

### Custom (your auth)
```bash
# Get token from your provider
TOKEN="your-custom-jwt-token"

curl http://localhost:3000/mcp \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"tools/list","id":1}'
```

## Security Comparison

| Mode | Auth Required | Provider | Who Can Access | Use Case |
|------|--------------|----------|----------------|----------|
| `public` | No | None | Anyone | Demos, public tools |
| `private` | Yes | FTL/WorkOS | Just you | Personal tools |
| `org` | Yes | FTL/WorkOS | Your org | Team collaboration |
| `custom` | Yes | Your choice | You decide | Enterprise integration |

## Migration Guide

### From Old Format to New

**Old format (deprecated):**
```yaml
access: private
auth:
  provider: workos
  org_id: "org_123"
```

**New equivalent:**
```yaml
access: org  # For org-level access
# or
access: private  # For user-only access
```

**Old format (custom provider):**
```yaml
access: private
auth:
  provider: custom
  jwt_issuer: "https://auth.example.com"
  jwt_audience: "api.example.com"
```

**New equivalent:**
```yaml
access: custom
auth:
  jwt_issuer: "https://auth.example.com"
  jwt_audience: "api.example.com"
```

## Future Enhancements

- **M2M Tokens for Org Mode**: Machine-to-machine tokens scoped to organizations
- **Fine-grained Permissions**: Tool-level access control within orgs
- **Multi-org Support**: Tools shared across multiple organizations