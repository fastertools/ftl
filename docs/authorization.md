# FTL Authorization

FTL uses [Rego](https://www.openpolicyagent.org/docs/latest/policy-language/) policies for authorization, providing flexible, declarative access control for MCP tools.

## Architecture

Every FTL application uses a Rego policy for authorization:
- **Public mode**: No authentication (no authorizer component)
- **Private mode**: Platform-managed policy restricting access to owner
- **Organization mode**: Platform-managed policy for org-wide access
- **Custom mode**: User-defined policy for advanced scenarios

## Authorization Modes

### Public Mode
No authentication or authorization. Requests pass directly to tools.

```yaml
name: public-app
access: public
```

### Private Mode
Restricts access to the application owner only.

```yaml
name: personal-app
access: private
```

Platform automatically injects:
```rego
package mcp.authorization

default allow = false

allow {
    input.token.sub == data.owner
}
```

### Organization Mode
Enables access for organization members and machines.

```yaml
name: team-app
access: org
```

Platform automatically injects:
```rego
package mcp.authorization

default allow = false

# Allow organization members (users)
allow {
    not input.token.claims.org_id
    input.token.sub == data.members[_]
}

# Allow organization machines
allow {
    input.token.claims.org_id
    input.token.claims.org_id == data.org_id
}
```

### Custom Mode
Full control over authentication and authorization.

```yaml
name: custom-app
access: custom
auth:
  jwt_issuer: "https://auth.example.com"
  jwt_audience: "my-api"
  jwt_jwks_uri: "https://auth.example.com/.well-known/jwks.json"
  policy: |
    package mcp.authorization
    
    default allow = false
    
    # Custom authorization logic
    allow {
      input.token.claims.role == "admin"
    }
    
    allow {
      input.token.claims.role == "user"
      input.mcp.method == "tools/list"
    }
  policy_data: |
    {
      "admin_tools": ["dangerous_tool", "admin_tool"],
      "user_tools": ["safe_tool", "read_tool"]
    }
```

## Policy Input Structure

Policies receive a standardized input:

```json
{
  "token": {
    "sub": "user_123",
    "iss": "https://issuer.example.com",
    "claims": {
      // All JWT claims
    },
    "scopes": ["scope1", "scope2"]
  },
  "request": {
    "method": "POST",
    "path": "/mcp/x/component-name",
    "component": "component-name",
    "headers": {
      // Request headers
    }
  },
  "mcp": {  // Present for MCP protocol requests
    "method": "tools/call",
    "tool": "tool_name",
    "arguments": {}
  }
}
```

## Common Policy Patterns

### Role-Based Access
```rego
allow {
  input.token.claims.role == "admin"
}

allow {
  input.token.claims.role == "user"
  input.mcp.tool == data.user_tools[_]
}
```

### Component-Scoped Access
```rego
allow {
  input.request.component == "public-api"
  input.token.scopes[_] == "read"
}

allow {
  input.request.component == "admin-api"
  input.token.claims.admin == true
}
```

### Tool-Specific Authorization
```rego
# Define tool permissions
dangerous_tools := ["delete_all", "drop_table"]
safe_tools := ["list", "read", "search"]

allow {
  input.mcp.method == "tools/call"
  input.mcp.tool == safe_tools[_]
}

allow {
  input.mcp.method == "tools/call"
  input.mcp.tool == dangerous_tools[_]
  input.token.claims.role == "admin"
}
```

### Time-Based Access
```rego
import future.keywords.if

allow if {
  current_time := time.now_ns() / 1000000000
  hour := time.clock(current_time)[0]
  hour >= 9
  hour < 17
  input.token.sub == data.owner
}
```

## Policy Data

Policy data can be provided as JSON and accessed via the `data` namespace:

```yaml
auth:
  policy_data: |
    {
      "allowed_users": ["user_1", "user_2"],
      "admin_emails": ["admin@example.com"],
      "rate_limits": {
        "default": 100,
        "premium": 1000
      }
    }
```

Access in policy:
```rego
allow {
  input.token.sub == data.allowed_users[_]
}

allow {
  input.token.claims.email == data.admin_emails[_]
}
```

## Token Types

FTL handles different JWT token types:

### User Tokens
Standard user authentication tokens without organization claims:
```json
{
  "sub": "user_abc123",
  "email": "user@example.com",
  "name": "John Doe"
}
```

### Machine Tokens
Service/machine authentication with organization scope:
```json
{
  "sub": "machine_xyz789",
  "org_id": "org_123",
  "purpose": "ci-deployment"
}
```

## Best Practices

1. **Default Deny**: Always use `default allow = false`
2. **Minimal Exposure**: Grant minimum necessary permissions
3. **Clear Rules**: Write readable, well-commented policies
4. **Test Policies**: Validate with various token scenarios
5. **Audit Regularly**: Review policies for security gaps

## Testing Policies

Test your policies using the [OPA Playground](https://play.openpolicyagent.org/) or locally:

1. Create sample input matching the structure above
2. Add your policy
3. Include any required data
4. Evaluate the `data.mcp.authorization.allow` rule

## Security Considerations

- JWT signatures are validated before policy evaluation
- Policies cannot modify the request or response
- Failed authorization returns 403 Forbidden
- Policy errors default to deny
- All authenticated modes require HTTPS issuers