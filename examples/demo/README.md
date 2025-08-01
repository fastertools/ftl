# ftl-mcp-demo

FTL MCP server for hosting MCP tools

## Getting Started

This FTL MCP server is ready to host MCP tools. Add tools using:

```bash
# Add a TypeScript tool
spin add -t ftl-mcp-ts

# Add a Rust tool
spin add -t ftl-mcp-rust

# Add a Python tool
spin add -t ftl-mcp-python

# For Go tools, see the included examples
```

## Running the Server

### Without Authentication (Default)

```bash
spin build --up
```

The MCP endpoint will be available at `http://localhost:3000/mcp`

### With Authentication

Authentication is now controlled via the `auth_config` variable. By default, authentication is disabled. To enable it, override the configuration:

```bash
# Example with AuthKit
export SPIN_VARIABLE_AUTH_CONFIG='{
  "mcp_gateway_url": "http://ftl-mcp-gateway.spin.internal/mcp-internal",
  "trace_id_header": "X-Trace-Id",
  "enabled": true,
  "providers": [{
    "type": "authkit",
    "issuer": "https://your-tenant.authkit.app"
  }]
}'

spin build --up
```

For more authentication examples and provider configurations, see `.env.example`.

The authenticated MCP endpoint will be available at `http://localhost:3000/mcp` and requires a valid JWT token from the configured provider.

## Using Application Variables

This demo includes examples of using Spin application variables. Variables are defined in the `[variables]` section of `ftl.toml` and can be:

1. **Required variables** - Must be provided at runtime
2. **Variables with defaults** - Have default values but can be overridden

### Defining Variables in ftl.toml

```toml
[variables]
# Variables with default values
api_token = { default = "demo-token-12345" }  # Demo default, override for production
api_url = { default = "https://api.example.com" }
api_version = { default = "v1" }
environment = { default = "development" }
```

To require a variable at runtime (e.g., for production), use:
```toml
api_token = { required = true }
```

### Using Variables in Tools

Tools access variables through their component configuration:

```toml
[tools.variables-demo]
variables = { 
    api_token = "{{ api_token }}", 
    api_url = "{{ api_url }}", 
    api_version = "{{ api_version }}", 
    environment = "{{ environment }}" 
}
```

### Setting Variables at Runtime

Variables can be set using environment variables with the `SPIN_VARIABLE_` prefix:

```bash
# Run with default values (demo mode)
spin build --up

# Or override variables for production
export SPIN_VARIABLE_API_TOKEN="your-production-token"
export SPIN_VARIABLE_API_URL="https://api.production.com"
export SPIN_VARIABLE_ENVIRONMENT="production"
spin build --up

# Deploy with variables
ftl eng deploy --variable api_token=your-production-token --variable environment=production
```

### Variables Demo Tool

The `variables-demo` tool demonstrates three ways to use variables:

1. **config_info** - Shows all configured variables
2. **make_api_call** - Demonstrates using variables for API configuration
3. **environment_check** - Shows environment-specific behavior

Try it out:

```bash
# Build and run with default demo values
spin build --up

# Test the tools
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "tools/call",
    "params": {
      "name": "config_info",
      "arguments": {}
    },
    "id": 1
  }'

# Or run with custom values
export SPIN_VARIABLE_API_TOKEN="my-secret-token"
export SPIN_VARIABLE_ENVIRONMENT="staging"
spin build --up
```

## Testing

### Without Authentication

List available tools:
```bash
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"tools/list","id":1}'
```

### With Authentication

First, check the authentication requirements:
```bash
# This will return 401 with authentication details
curl -i http://localhost:3000/mcp

# Discover OAuth configuration
curl http://localhost:3000/.well-known/oauth-protected-resource
```

Then make authenticated requests with a JWT token:
```bash
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -d '{"jsonrpc":"2.0","method":"tools/list","id":1}'
```
