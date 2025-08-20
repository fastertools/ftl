# M2M Authentication Architecture

## Overview
This document describes the Machine-to-Machine (M2M) authentication implementation for FTL CLI and platform deployments.

## CLI M2M Authentication

### Environment Variables
The FTL CLI supports M2M authentication through environment variables:
- `FTL_CLIENT_ID`: OAuth2 client ID for the machine account
- `FTL_CLIENT_SECRET`: OAuth2 client secret
- `FTL_M2M_TOKEN`: Pre-generated M2M token (alternative to client credentials)

### Commands
```bash
# Login with client credentials from environment
ftl auth login --machine

# Login with pre-generated token
ftl auth login --machine --token="<token>"

# Deploy with automatic M2M detection
ftl deploy  # Automatically uses M2M if env vars are set
```

### Implementation Details
- M2M credentials are detected automatically in `ftl deploy` command
- Machine tokens are always associated with exactly one organization
- Actor type is sent as `X-FTL-Actor-Type: machine` header in API requests
- Organization ID is passed via `X-FTL-Org-ID` header during deployment

## Platform M2M Authorization

### CUE Pattern Updates
The `patterns.cue` file now supports automatic injection of required claims for M2M deployments:

```cue
// For org-scoped apps deployed by machines
if input.access == "org" {
    if platform.deployment_context.actor_type == "machine" {
        if platform.deployment_context.org_id != _|_ {
            // Automatically inject org_id claim requirement
            mcp_auth_required_claims: "{\"org_id\": \"" + platform.deployment_context.org_id + "\"}"
        }
    }
}
```

### Security Model
1. **Machine actors** deploying to org-scoped apps automatically get `org_id` claim validation
2. This ensures M2M tokens can only access applications within their designated organization
3. The `mcp-authorizer` component validates JWT claims according to the injected requirements

### Deployment Context
The platform receives deployment context through HTTP headers:
- `X-FTL-Actor-Type`: "user" or "machine"
- `X-FTL-Org-ID`: Organization ID for org-scoped deployments
- `X-FTL-User-ID`: User ID (empty for machines)

These headers are used to populate `platform.deployment_context` in the CUE evaluation:
```cue
deployment_context: {
    actor_type: "machine"
    org_id: "org_01HQ3GXFP7"
}
```

## JWT Token Structure

### Machine Token Claims
M2M tokens from WorkOS include:
```json
{
    "iss": "https://divine-lion-50-staging.authkit.app",
    "aud": "client_01JZM53FW3WYV08AFC4QWQ3BNB",
    "org_id": "org_01HQ3GXFP7",
    "sub": "client_xxx",
    "iat": 1234567890,
    "exp": 1234567890
}
```

### Validation
The `mcp-authorizer` component validates:
1. Standard JWT claims (issuer, audience, expiration)
2. Required claims based on `mcp_auth_required_claims` configuration
3. For M2M to org-scoped apps: `org_id` must match the deployment org

## CI/CD Integration

### GitHub Actions Example
```yaml
env:
  FTL_CLIENT_ID: ${{ secrets.FTL_CLIENT_ID }}
  FTL_CLIENT_SECRET: ${{ secrets.FTL_CLIENT_SECRET }}

steps:
  - name: Deploy to FTL
    run: ftl deploy --yes  # Auto-detects M2M credentials
```

### Benefits
1. **Automatic Detection**: No need to explicitly specify M2M mode
2. **Secure**: Credentials stored as secrets, never exposed in logs
3. **Org Isolation**: M2M tokens can only deploy to their assigned organization
4. **Audit Trail**: Platform tracks deployments by machine vs user actors

## Testing
To test M2M authentication:
1. Set up a machine account in WorkOS
2. Export credentials as environment variables
3. Run `ftl auth login --machine` to verify authentication
4. Deploy an org-scoped app with `ftl deploy`
5. Verify the synthesized Spin manifest includes `mcp_auth_required_claims` with org_id