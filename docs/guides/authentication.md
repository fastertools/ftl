# Handling Authentication with OAuth 2.0

**Problem**: Your MCP server needs to authenticate and authorize users before allowing access to tools.

**Solution**: Configure the FTL Authorizer component with OAuth 2.0 JWT token validation to secure your MCP endpoints.

## Overview

FTL provides built-in authentication through the `mcp-authorizer` component, which supports:

- **JWT Token Validation**: Standard OAuth 2.0 Bearer tokens
- **Multiple Providers**: WorkOS AuthKit, Auth0, custom OIDC providers
- **Scope-Based Authorization**: Fine-grained access control
- **High Performance**: Efficient JWT validation with JWKS caching

## Step 1: Enable Authentication

Configure your project for private access in `ftl.toml`:

```toml
[project]
name = "my-secure-project"
access_control = "private"  # Enables authentication

[mcp]
gateway = "ghcr.io/fastertools/mcp-gateway:latest"
authorizer = "ghcr.io/fastertools/mcp-authorizer:latest"
validate_arguments = true
```

This automatically:
- Adds the authorizer component to your application
- Routes all requests through authentication first
- Forwards auth context to your tools

## Step 2: Choose Your Authentication Provider

### Option A: WorkOS AuthKit (Recommended)

WorkOS provides enterprise-grade authentication with minimal configuration:

```toml
[project]
access_control = "private"

[workos]
client_id = "client_01H1234567890ABCDEF"
client_secret = "${WORKOS_CLIENT_SECRET}"  # Use environment variable
```

**Setup WorkOS**:
1. Sign up at [workos.com](https://workos.com)
2. Create a new application
3. Configure redirect URIs for your domain
4. Copy client ID and secret

### Option B: Custom OIDC Provider

For Auth0, Okta, or other OAuth 2.0 providers:

```toml
[project]
access_control = "private"

[oidc]
issuer = "https://your-domain.auth0.com"
audience = "https://api.your-domain.com"
# Optional: Override JWKS endpoint
jwks_uri = "https://your-domain.auth0.com/.well-known/jwks.json"
```

### Option C: Static JWT Validation

For development or simple deployments with a fixed public key:

```toml
[project]
access_control = "private"

[jwt]
issuer = "your-service"
audience = "your-api"
public_key = """-----BEGIN PUBLIC KEY-----
MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA...
-----END PUBLIC KEY-----"""
```

## Step 3: Configure Environment Variables

Set up your environment variables:

```bash
# For production
export WORKOS_CLIENT_SECRET="wkos_sk_live_..."
export OIDC_CLIENT_SECRET="your-auth0-secret"

# For development
export WORKOS_CLIENT_SECRET="wkos_sk_test_..."
```

Or use a `.env` file:

```bash
# .env
WORKOS_CLIENT_SECRET=wkos_sk_test_01H1234567890ABCDEF
OIDC_CLIENT_SECRET=your-development-secret
```

## Step 4: Build and Test Authentication

Rebuild your project to apply authentication:

```bash
ftl build
ftl up
```

Now all requests require authentication:

### Test Without Token (Should Fail)
```bash
curl -X POST http://localhost:3000/tools/list
# Returns: 401 Unauthorized
```

### Test With Valid Token
```bash
# First get a token from your auth provider
TOKEN="eyJ0eXAiOiJKV1QiLCJhbGci..."

curl -X POST http://localhost:3000/tools/list \
  -H "Authorization: Bearer $TOKEN"
# Returns: Tool list
```

## Step 5: Access Auth Context in Tools

The authorizer automatically adds authentication context headers that your tools can access:

### Rust Tools

```rust
use ftl_sdk::prelude::*;

#[tool]
pub fn get_user_info() -> ToolResponse {
    // Access auth context from request headers
    let user_id = std::env::var("HTTP_X_AUTH_USER_ID")
        .unwrap_or_else(|_| "anonymous".to_string());
    let client_id = std::env::var("HTTP_X_AUTH_CLIENT_ID")
        .unwrap_or_else(|_| "unknown".to_string());
    let scopes = std::env::var("HTTP_X_AUTH_SCOPES")
        .unwrap_or_else(|_| "".to_string());
    
    let info = format!(
        "User: {}, Client: {}, Scopes: {}", 
        user_id, client_id, scopes
    );
    
    ToolResponse::ok(&info)
}

#[tool]
pub fn admin_only_tool() -> ToolResponse {
    let scopes = std::env::var("HTTP_X_AUTH_SCOPES")
        .unwrap_or_else(|_| "".to_string());
    
    if !scopes.contains("admin") {
        return ToolResponse::error("Admin scope required");
    }
    
    // Admin logic here
    ToolResponse::ok("Admin operation completed")
}
```

### Python Tools

```python
import os
from ftl_sdk import tool, ToolResponse

@tool
def get_user_info() -> ToolResponse:
    """Get current user information from auth context."""
    user_id = os.environ.get('HTTP_X_AUTH_USER_ID', 'anonymous')
    client_id = os.environ.get('HTTP_X_AUTH_CLIENT_ID', 'unknown')
    scopes = os.environ.get('HTTP_X_AUTH_SCOPES', '')
    
    info = {
        'user_id': user_id,
        'client_id': client_id,
        'scopes': scopes.split(' ') if scopes else []
    }
    
    return ToolResponse.ok(str(info))

@tool
def user_specific_data(data_type: str) -> ToolResponse:
    """Fetch user-specific data."""
    user_id = os.environ.get('HTTP_X_AUTH_USER_ID')
    if not user_id:
        return ToolResponse.error("User authentication required")
    
    # Fetch data specific to this user
    user_data = fetch_user_data(user_id, data_type)
    return ToolResponse.ok(user_data)

def fetch_user_data(user_id: str, data_type: str) -> str:
    # Your user-specific logic here
    return f"Data for user {user_id}: {data_type}"
```

### Go Tools

```go
import (
    "context"
    "os"
    "strings"
)

func (t *ToolImpl) GetUserInfo(ctx context.Context) cm.Result[string, string] {
    userID := getEnvOrDefault("HTTP_X_AUTH_USER_ID", "anonymous")
    clientID := getEnvOrDefault("HTTP_X_AUTH_CLIENT_ID", "unknown")
    scopes := getEnvOrDefault("HTTP_X_AUTH_SCOPES", "")
    
    info := fmt.Sprintf("User: %s, Client: %s, Scopes: %s", userID, clientID, scopes)
    return cm.OK[string](info)
}

func (t *ToolImpl) RequireScope(ctx context.Context, requiredScope string) cm.Result[string, string] {
    scopes := getEnvOrDefault("HTTP_X_AUTH_SCOPES", "")
    scopeList := strings.Split(scopes, " ")
    
    for _, scope := range scopeList {
        if scope == requiredScope {
            return cm.OK[string]("Access granted")
        }
    }
    
    return cm.Err[string]("Required scope not found: " + requiredScope)
}

func getEnvOrDefault(key, defaultValue string) string {
    if value := os.Getenv(key); value != "" {
        return value
    }
    return defaultValue
}
```

## Advanced Authentication Patterns

### Scope-Based Tool Access

Configure different tools for different user scopes:

```toml
[tools.public-tool]
path = "components/public-tool"
# No scope restrictions

[tools.admin-tool]
path = "components/admin-tool"  
required_scopes = ["admin"]

[tools.manager-tool]
path = "components/manager-tool"
required_scopes = ["manager", "admin"]  # Either scope works
```

### Custom Token Validation

For advanced use cases, implement custom validation logic:

```rust
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,    // Subject (user ID)
    exp: usize,     // Expiration
    iat: usize,     // Issued at
    custom_field: String,
}

#[tool]
pub fn validate_custom_token(token: String) -> ToolResponse {
    let public_key = include_str!("../keys/public.pem");
    let key = DecodingKey::from_rsa_pem(public_key.as_bytes()).unwrap();
    
    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_audience(&["my-api"]);
    validation.set_issuer(&["my-issuer"]);
    
    match decode::<Claims>(&token, &key, &validation) {
        Ok(token_data) => {
            let user_info = format!(
                "User: {}, Custom: {}", 
                token_data.claims.sub,
                token_data.claims.custom_field
            );
            ToolResponse::ok(&user_info)
        }
        Err(e) => ToolResponse::error(&format!("Invalid token: {}", e)),
    }
}
```

### Multi-Tenant Authentication

Support multiple organizations or tenants:

```python
@tool
def get_tenant_data() -> ToolResponse:
    """Get data specific to the user's tenant."""
    # Extract tenant from token claims or user context
    user_id = os.environ.get('HTTP_X_AUTH_USER_ID')
    issuer = os.environ.get('HTTP_X_AUTH_ISSUER')
    
    # Determine tenant from issuer or user ID
    tenant = extract_tenant_from_user(user_id, issuer)
    
    # Fetch tenant-specific data
    data = get_data_for_tenant(tenant)
    return ToolResponse.ok(data)

def extract_tenant_from_user(user_id: str, issuer: str) -> str:
    # Your tenant extraction logic
    if 'company-a' in issuer:
        return 'tenant_a'
    elif 'company-b' in issuer:
        return 'tenant_b'
    return 'default'
```

## Client Integration Examples

### Claude Desktop

Configure Claude Desktop to use your authenticated FTL server:

```json
{
  "mcpServers": {
    "my-secure-ftl": {
      "command": "curl",
      "args": [
        "-X", "POST",
        "-H", "Authorization: Bearer YOUR_JWT_TOKEN",
        "-H", "Content-Type: application/json",
        "http://localhost:3000/tools/list"
      ]
    }
  }
}
```

### Web Application

JavaScript client with token management:

```javascript
class AuthenticatedMCPClient {
    constructor(baseUrl, tokenProvider) {
        this.baseUrl = baseUrl;
        this.tokenProvider = tokenProvider;
    }
    
    async callTool(toolName, arguments) {
        const token = await this.tokenProvider.getToken();
        
        const response = await fetch(`${this.baseUrl}/tools/call`, {
            method: 'POST',
            headers: {
                'Authorization': `Bearer ${token}`,
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({
                jsonrpc: '2.0',
                id: Date.now(),
                method: 'tools/call',
                params: {
                    name: toolName,
                    arguments: arguments
                }
            })
        });
        
        if (response.status === 401) {
            // Token expired, refresh and retry
            await this.tokenProvider.refreshToken();
            return this.callTool(toolName, arguments);
        }
        
        return response.json();
    }
}

// Usage
const tokenProvider = new WorkOSTokenProvider(clientId);
const client = new AuthenticatedMCPClient('http://localhost:3000', tokenProvider);

const result = await client.callTool('my-tool/process_data', {
    input: 'Hello, world!'
});
```

## Troubleshooting

### Common Issues

**401 Unauthorized with valid token**:
- Check token expiration (`exp` claim)
- Verify issuer matches configuration (`iss` claim)
- Ensure audience is correct (`aud` claim)
- Check JWKS endpoint accessibility

**Token validation errors**:
```bash
# Debug JWT token
echo "eyJ0eXAiOiJKV1QiLCJhbGci..." | base64 -d
# Check token structure and claims
```

**JWKS fetching errors**:
- Verify JWKS endpoint is accessible
- Check network connectivity from server
- Ensure JWKS endpoint returns valid JSON

**Environment variable issues**:
```bash
# Check if variables are set
env | grep AUTH
env | grep WORKOS
env | grep OIDC
```

### Debugging Authentication

Enable detailed auth logging:

```toml
[mcp.authorizer.config]
log_level = "debug"
log_token_claims = true  # Only in development!
```

**Warning**: Never log tokens or sensitive data in production.

### Testing Different Providers

Use online JWT debuggers for development:
1. Go to [jwt.io](https://jwt.io)
2. Paste your JWT token
3. Verify claims and signature
4. Test with your public key

## Security Best Practices

### Token Security
- Use HTTPS in production
- Set appropriate token expiration times
- Implement token refresh mechanisms
- Store secrets in environment variables, not code

### Scope Management
- Use principle of least privilege
- Implement fine-grained scopes
- Regularly audit scope assignments
- Document scope requirements

### Key Management
- Rotate JWT signing keys regularly
- Use strong cryptographic algorithms (RS256 or ES256)
- Store private keys securely
- Never commit keys to version control

## Production Deployment

### Environment Configuration
```bash
# Production environment variables
export FTL_ENV=production
export WORKOS_CLIENT_SECRET=wkos_sk_live_...
export OIDC_ISSUER=https://auth.yourcompany.com
export OIDC_AUDIENCE=https://api.yourcompany.com
```

### SSL/TLS Setup
```toml
[deployment]
ssl_cert = "/path/to/cert.pem"
ssl_key = "/path/to/private.key"
force_https = true
```

### Health Checks
```bash
# Check auth endpoint
curl -f http://localhost:3000/_health/auth

# Verify JWKS endpoint
curl -f http://localhost:3000/.well-known/jwks.json
```

## Next Steps

- **HTTP Requests**: Learn to make authenticated API calls in [Making HTTP Requests](./http-requests.md)
- **Testing**: Write tests for authenticated tools in [Testing Your Tools](./testing.md)
- **Advanced Security**: Explore enterprise patterns in [Examples](../../examples/)
- **Deployment**: Deploy to production in [Contributing](../contributing/development-setup.md)

Authentication with FTL provides enterprise-grade security while maintaining the simplicity of the development experience. The JWT-based approach ensures your tools can access user context while keeping the implementation straightforward.