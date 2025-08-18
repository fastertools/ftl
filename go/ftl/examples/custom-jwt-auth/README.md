# Custom JWT Authentication Example

This example demonstrates a **private** FTL application secured with a custom JWT provider (Auth0, Okta, Keycloak, or your own).

## Configuration

```yaml
access: private  # Requires authentication

auth:
  provider: custom
  jwt_issuer: "https://auth.example.com"    # Your JWT issuer
  jwt_audience: "enterprise-mcp-tools"      # Expected audience
```

## What This Creates

The synthesized manifest includes:
1. `mcp-authorizer` component for JWT validation
2. Configuration for your custom JWT issuer and audience
3. Private routing between all components
4. Automatic JWT validation on every request

## Common JWT Providers

### Auth0
```yaml
auth:
  provider: custom
  jwt_issuer: "https://YOUR_DOMAIN.auth0.com/"
  jwt_audience: "YOUR_API_IDENTIFIER"
```

### Okta
```yaml
auth:
  provider: custom
  jwt_issuer: "https://YOUR_DOMAIN.okta.com/oauth2/default"
  jwt_audience: "api://default"
```

### Keycloak
```yaml
auth:
  provider: custom
  jwt_issuer: "https://YOUR_DOMAIN/auth/realms/YOUR_REALM"
  jwt_audience: "YOUR_CLIENT_ID"
```

### AWS Cognito
```yaml
auth:
  provider: custom
  jwt_issuer: "https://cognito-idp.REGION.amazonaws.com/USER_POOL_ID"
  jwt_audience: "YOUR_APP_CLIENT_ID"
```

## JWT Requirements

Your JWT tokens must:
1. Be signed (RS256 or HS256)
2. Include standard claims:
   - `iss` (issuer) - must match `jwt_issuer`
   - `aud` (audience) - must match `jwt_audience`
   - `exp` (expiration) - must be in the future
3. Be passed in the `Authorization: Bearer TOKEN` header

## Testing

### 1. Get a JWT Token

Example using a test token (for development only):
```bash
# Generate a test JWT (requires jwt-cli or similar)
TOKEN=$(jwt encode \
  --secret "your-secret" \
  --iss "https://auth.example.com" \
  --aud "enterprise-mcp-tools" \
  --exp "+1h" \
  '{"sub":"user123","role":"admin"}')
```

### 2. Test with Authentication

```bash
# With valid JWT
curl -X POST http://127.0.0.1:3000/mcp \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"jsonrpc":"2.0","method":"tools/list","params":{},"id":1}'

# Without JWT - will fail
curl -X POST http://127.0.0.1:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"tools/list","params":{},"id":1}'
# Returns: 401 Unauthorized
```

## Integration Example

```javascript
// Frontend integration example
const response = await fetch('http://your-app.com/mcp', {
  method: 'POST',
  headers: {
    'Content-Type': 'application/json',
    'Authorization': `Bearer ${getAuthToken()}` // Your auth token
  },
  body: JSON.stringify({
    jsonrpc: '2.0',
    method: 'tools/call',
    params: {
      name: 'compliance-scanner__scan',
      arguments: { target: 'production' }
    },
    id: 1
  })
});
```

## Security Considerations

✅ **Token Validation**: Every request is validated  
✅ **Expiration Checks**: Expired tokens are rejected  
✅ **Audience Verification**: Prevents token reuse from other apps  
✅ **Issuer Verification**: Ensures tokens come from trusted source  
⚠️ **HTTPS Required**: Always use HTTPS in production  
⚠️ **Token Rotation**: Implement short-lived tokens with refresh