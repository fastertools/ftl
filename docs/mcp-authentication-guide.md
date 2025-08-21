# MCP Authentication Guide

This guide explains how to configure and use authentication for MCP endpoints with FTL.

## Overview

MCP (Model Context Protocol) endpoints can be protected by JWT authentication. FTL provides flexible authentication options ranging from public access to enterprise OAuth providers. This guide covers both configuration and usage.

## Configuring Authentication

Authentication is configured in your `ftl.toml` file using the `access_control` field in the `[project]` section.

### Access Control Modes

#### Public Access (No Authentication)
```toml
[project]
name = "my-tools"
access_control = "public"  # Anyone can access without authentication
```

#### Private Access (User-Level)
```toml
[project]
name = "my-tools"
access_control = "private"  # Only you can access
# FTL automatically configures authentication
```

#### Organization Access
```toml
[project]
name = "team-tools"
access_control = "org"  # All organization members can access
# FTL automatically configures authentication
```

#### Custom OAuth Provider
```toml
[project]
name = "enterprise-tools"
access_control = "custom"

[oauth]
issuer = "https://auth.example.com"
audience = "https://api.myapp.com"  # Required for security
jwks_uri = "https://auth.example.com/.well-known/jwks.json"
required_scopes = "read,write"
```

#### Restricting to Specific Users
```toml
[project]
name = "team-tools"
access_control = "custom"

[oauth]
issuer = "https://auth.example.com"
audience = "https://api.myapp.com"
jwks_uri = "https://auth.example.com/.well-known/jwks.json"
# Only allow specific users to access
allowed_subjects = ["alice@company.com", "bob@company.com", "carol@company.com"]
```

### Security Requirements

When using authentication (`private`, `org`, or `custom` modes):

1. **Audience is Required**: The `audience` field is mandatory to prevent confused deputy attacks
2. **HTTPS Only**: All OAuth URLs must use HTTPS
3. **Token Validation**: Tokens are validated for:
   - Signature verification (using JWKS or public key)
   - Issuer matching
   - Audience matching
   - Expiration time
   - Required scopes (if configured)

### Supported OAuth Providers

FTL works with any OAuth 2.0 / OpenID Connect provider. Common examples:

#### Auth0
```toml
[oauth]
issuer = "https://your-tenant.auth0.com/"
audience = "https://api.yourapp.com"
jwks_uri = "https://your-tenant.auth0.com/.well-known/jwks.json"
```

#### Okta
```toml
[oauth]
issuer = "https://your-org.okta.com/oauth2/default"
audience = "api://yourapp"
jwks_uri = "https://your-org.okta.com/oauth2/default/v1/keys"
```

#### Azure AD
```toml
[oauth]
issuer = "https://login.microsoftonline.com/{tenant-id}/v2.0"
audience = "api://{client-id}"
jwks_uri = "https://login.microsoftonline.com/{tenant-id}/discovery/v2.0/keys"
```

#### WorkOS AuthKit (Auto-Configuration)
```toml
[oauth]
issuer = "https://your-org.authkit.app"  # JWKS auto-discovered
audience = "your-api-identifier"
# JWKS URI is automatically set to https://your-org.authkit.app/oauth2/jwks
```

## Interactive Authentication

For regular CLI usage, simply login once:

```bash
ftl login
```

This will:
1. Open your browser for authentication
2. Store refresh tokens securely in your system keyring
3. Automatically refresh access tokens as needed

## Automated Access (CI/CD, Scripts, Agents)

For automation scenarios where interactive login isn't possible, you can use the `ftl auth token` command to get a valid access token:

### Basic Usage

```bash
# Get current access token (auto-refreshes if needed)
TOKEN=$(ftl eng auth token)

# Use it with curl
curl -X POST https://your-app.ftl.tools/mcp \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "tools/list",
    "params": {},
    "id": 1
  }'
```

### Shell Script Example

```bash
#!/bin/bash
# mcp-tools.sh - List available MCP tools

# Get authentication token
TOKEN=$(ftl eng auth token)
if [ $? -ne 0 ]; then
  echo "Error: Not authenticated. Please run 'ftl login' first."
  exit 1
fi

# Call MCP endpoint
response=$(curl -s -X POST https://your-app.ftl.tools/mcp \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "tools/list",
    "params": {},
    "id": 1
  }')

# Pretty print the response
echo "$response" | jq .
```

### CI/CD Example (GitHub Actions)

```yaml
name: MCP Integration Test

on: [push]

jobs:
  test-mcp:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install FTL CLI
        run: |
          curl -fsSL https://ftl.tools/install | sh
          
      - name: Restore FTL credentials
        env:
          FTL_CREDENTIALS: ${{ secrets.FTL_CREDENTIALS }}
        run: |
          # Store credentials in keyring format
          echo "$FTL_CREDENTIALS" | ftl auth restore
          
      - name: Get auth token
        id: auth
        run: |
          TOKEN=$(ftl eng auth token)
          echo "token=$TOKEN" >> $GITHUB_OUTPUT
          
      - name: Call MCP endpoint
        run: |
          curl -X POST https://your-app.ftl.tools/mcp \
            -H "Authorization: Bearer ${{ steps.auth.outputs.token }}" \
            -H "Content-Type: application/json" \
            -d '{
              "jsonrpc": "2.0",
              "method": "tools/list",
              "params": {},
              "id": 1
            }'
```

### Python Example

```python
#!/usr/bin/env python3
import subprocess
import json
import requests

def get_ftl_token():
    """Get FTL authentication token"""
    try:
        result = subprocess.run(
            ['ftl', 'auth', 'token'],
            capture_output=True,
            text=True,
            check=True
        )
        return result.stdout.strip()
    except subprocess.CalledProcessError:
        raise Exception("Not authenticated. Please run 'ftl login' first.")

def call_mcp(method, params=None):
    """Call an MCP endpoint"""
    token = get_ftl_token()
    
    payload = {
        "jsonrpc": "2.0",
        "method": method,
        "id": 1
    }
    if params:
        payload["params"] = params
    
    response = requests.post(
        "https://your-app.ftl.tools/mcp",
        headers={
            "Authorization": f"Bearer {token}",
            "Content-Type": "application/json"
        },
        json=payload
    )
    response.raise_for_status()
    return response.json()

# Example usage
if __name__ == "__main__":
    result = call_mcp("tools/list")
    print(json.dumps(result, indent=2))
```

### Node.js Example

```javascript
#!/usr/bin/env node
const { execSync } = require('child_process');
const https = require('https');

// Get FTL token
function getFtlToken() {
  try {
    const token = execSync('ftl auth token', { encoding: 'utf8' }).trim();
    return token;
  } catch (error) {
    throw new Error('Not authenticated. Please run "ftl login" first.');
  }
}

// Call MCP endpoint
async function callMCP(method, params = null) {
  const token = getFtlToken();
  
  const payload = JSON.stringify({
    jsonrpc: '2.0',
    method: method,
    params: params,
    id: 1
  });

  const options = {
    hostname: 'your-app.ftl.tools',
    path: '/mcp',
    method: 'POST',
    headers: {
      'Authorization': `Bearer ${token}`,
      'Content-Type': 'application/json',
      'Content-Length': payload.length
    }
  };

  return new Promise((resolve, reject) => {
    const req = https.request(options, (res) => {
      let data = '';
      res.on('data', (chunk) => data += chunk);
      res.on('end', () => resolve(JSON.parse(data)));
    });
    req.on('error', reject);
    req.write(payload);
    req.end();
  });
}

// Example usage
callMCP('tools/list').then(console.log).catch(console.error);
```

## How It Works

1. **Initial Login**: `ftl login` uses OAuth device flow to authenticate
2. **Token Storage**: Refresh tokens are stored securely in your system keyring
3. **Token Refresh**: `ftl auth token` automatically refreshes expired tokens using the stored refresh token
4. **MCP Access**: The access token is a standard JWT that works with all MCP endpoints

## Security Best Practices

1. **Never commit tokens**: Use environment variables or secrets management for CI/CD
2. **Rotate regularly**: Periodically run `ftl logout` and `ftl login` to get new refresh tokens
3. **Limit scope**: Use the minimum required permissions for your automation
4. **Monitor usage**: Check `ftl auth status` to see token expiration

## Troubleshooting

### Token Expired
If you see "Token expired" errors:
```bash
# Token will auto-refresh if refresh token is valid
ftl auth token

# If refresh token is also expired
ftl login
```

### Not Authenticated
If you see "Not authenticated" errors:
```bash
# Check authentication status
ftl auth status

# Login if needed
ftl login
```

### CI/CD Issues
For CI/CD environments:
1. Run `ftl login` locally
2. Extract credentials: `ftl auth export`
3. Store as secret in CI/CD platform
4. Restore in CI: `echo "$SECRET" | ftl auth restore`

## Machine-to-Machine (M2M) Authentication

For service-to-service authentication where no user interaction is available, you can use M2M tokens:

### Setup M2M Credentials

First, create a M2M application in WorkOS and then store the credentials:

```bash
# Interactive setup (credentials stored in keyring)
ftl eng auth token --m2m-setup

# You'll be prompted for:
# - Client ID
# - Client Secret  
# - AuthKit Domain (defaults to staging)
```

### Get M2M Token

Once configured, you can get M2M tokens:

```bash
# Use stored M2M credentials
TOKEN=$(ftl eng auth token --m2m)

# Or provide credentials directly (not recommended for production)
TOKEN=$(ftl eng auth token \
  --m2m-client-id "your_client_id" \
  --m2m-client-secret "your_client_secret")
```

### M2M Token Caching

M2M tokens are automatically cached with a 1-hour expiry. The CLI will:
- Return cached tokens if still valid
- Automatically fetch new tokens when expired
- Store the token securely in your keyring

### CI/CD with M2M

For CI/CD environments using M2M authentication:

```yaml
name: Deploy with M2M Auth

on: [push]

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install FTL CLI
        run: |
          curl -fsSL https://ftl.tools/install | sh
          
      - name: Get M2M token
        env:
          M2M_CLIENT_ID: ${{ secrets.M2M_CLIENT_ID }}
          M2M_CLIENT_SECRET: ${{ secrets.M2M_CLIENT_SECRET }}
        run: |
          TOKEN=$(ftl eng auth token \
            --m2m-client-id "$M2M_CLIENT_ID" \
            --m2m-client-secret "$M2M_CLIENT_SECRET")
          echo "FTL_TOKEN=$TOKEN" >> $GITHUB_ENV
          
      - name: Deploy to FTL
        run: |
          # Use the M2M token for deployment
          ftl eng deploy --token "$FTL_TOKEN"
```

## Comparison with Other Methods

| Method | Use Case | Pros | Cons |
|--------|----------|------|------|
| `ftl eng auth token` | CLI automation, scripts | Simple, auto-refresh, secure | Requires FTL CLI, user login |
| `ftl eng auth token --m2m` | Service-to-service | No user needed, cached tokens | Requires M2M app setup |
| Direct M2M | CI/CD, serverless | No FTL CLI needed | Must manage token refresh |
| User JWT | Interactive apps | Full user context | Requires user interaction |

## See Also

- [FTL CLI Documentation](https://ftl.tools/docs/cli)
- [MCP Protocol Specification](https://modelcontextprotocol.io)
- [WorkOS Documentation](https://workos.com/docs)