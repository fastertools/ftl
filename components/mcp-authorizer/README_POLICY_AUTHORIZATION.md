# Policy-Based Authorization with Regorous

The MCP Authorizer uses [Regorous](https://github.com/microsoft/regorus), a fast and lightweight Rego interpreter, for flexible policy-based authorization. This replaces manual claim validation with industry-standard Open Policy Agent (OPA) Rego policies.

## Configuration

Policy-based authorization requires just two Spin variables:

```toml
[component.mcp-authorizer.variables]
# Rego policy (inline string) - Required
mcp_policy = '''
package mcp.authorization
import rego.v1

default allow := false

# Your authorization rules here
allow if {
    input.token.sub in data.allowed_users
}
'''

# Policy data (JSON string) - Optional
mcp_policy_data = '''
{
    "allowed_users": ["user1", "user2"],
    "dangerous_tools": ["delete_database", "reset_system"]
}
'''
```

The system automatically provides complete context to your policy. The policy decides what to validate.

## Policy Input Structure

Your policy always receives this input structure:

```json
{
    "token": {
        "sub": "user123",
        "iss": "https://issuer.com",
        "claims": {
            "email": "user@example.com",
            "roles": ["admin", "user"]
        },
        "scopes": ["read", "write"]
    },
    "request": {
        "method": "POST",
        "path": "/mcp/x/data-processor",
        "component": "data-processor",  // null if not scoped to component
        "headers": {
            "content-type": "application/json"
        }
    },
    "mcp": {  // Present for POST requests with JSON bodies
        "method": "tools/call",
        "tool": "process_data",
        "arguments": {}
    }
}
```

**Note:** The `mcp` field is only present when:
- The request method is POST
- The Content-Type indicates JSON
- The body can be successfully parsed as a JSON-RPC request

Your policy can check for the presence of `input.mcp` to determine if it's an MCP tool request.

## Example Policies

### Basic Component Authorization

```rego
package mcp.authorization
import rego.v1

default allow := false

# Allow users to access their assigned components
allow if {
    input.request.component
    input.request.component in data.user_components[input.token.sub]
}

# Allow admins to access everything
allow if {
    "admin" in input.token.claims.roles
}
```

### Tool-Level Authorization

```rego
package mcp.authorization
import rego.v1

default allow := false

# Component access check
component_allowed if {
    input.request.component in data.user_components[input.token.sub]
}

# Allow tool discovery
allow if {
    component_allowed
    input.mcp.method == "tools/list"
}

# Check tool permissions
allow if {
    component_allowed
    input.mcp.method == "tools/call"
    
    # Get required role for this tool
    tool_config := data.tool_permissions[input.request.component][input.mcp.tool]
    tool_config.required_role in input.token.claims.roles
}

# Protect dangerous tools
deny if {
    input.mcp.tool in data.dangerous_tools
    not "super_admin" in input.token.claims.roles
}

# Final allow (no deny rules triggered)
allow if {
    component_allowed
    not deny
}
```

### Scope-Based Authorization

```rego
package mcp.authorization
import rego.v1

default allow := false

# Check OAuth scopes for component access
allow if {
    required_scope := sprintf("mcp:%s:read", [input.request.component])
    required_scope in input.token.scopes
}

# Write operations need write scope
allow if {
    input.mcp.method == "tools/call"
    input.mcp.tool in data.write_tools
    
    required_scope := sprintf("mcp:%s:write", [input.request.component])
    required_scope in input.token.scopes
}
```

## Migration from Previous Authorization

### Previous Configuration
```toml
mcp_auth_allowed_subjects = "user1,user2,user3"
mcp_auth_required_claims = '{"department": "engineering", "level": 3}'
```

### Equivalent Rego Policy
```rego
package mcp.authorization
import rego.v1

default allow := false

allow if {
    # Check allowed subjects
    input.token.sub in ["user1", "user2", "user3"]
    
    # Check required claims
    input.token.claims.department == "engineering"
    input.token.claims.level == 3
}
```

## Testing Policies

Test your policies locally using the Regorous CLI:

```bash
# Install regorus CLI
cargo install --git https://github.com/microsoft/regorus

# Test policy evaluation
regorus eval -d policy_data.json -i input.json 'data.mcp.authorization.allow' policy.rego
```

## Performance Considerations

- Policies are compiled once and cached
- Component-only mode is faster (no body parsing)
- Keep policies simple and focused
- Use data files for large allow/deny lists

## Security Best Practices

1. **Default Deny**: Always start with `default allow := false`
2. **Explicit Deny Rules**: Use deny rules for dangerous operations
3. **Validate All Inputs**: Don't trust token claims blindly
4. **Audit Policies**: Log policy decisions for security auditing
5. **Test Thoroughly**: Test both allow and deny scenarios

## Troubleshooting

If authorization fails unexpectedly:

1. Check the policy syntax is valid Rego
2. Verify the input structure matches expectations
3. Test the policy with sample inputs using regorus CLI
4. Enable debug logging to see the actual input being evaluated
5. Ensure policy data JSON is valid if provided

## Additional Resources

- [Rego Language Documentation](https://www.openpolicyagent.org/docs/latest/policy-language/)
- [Regorous GitHub Repository](https://github.com/microsoft/regorus)
- [OPA Playground](https://play.openpolicyagent.org/) for testing policies