# Public Access Example

This example demonstrates a **public** FTL application where tools are accessible without authentication.

## Configuration

```yaml
access: public  # Anyone can access the tools
```

## What This Creates

When synthesized, this configuration:
1. Creates a public `/...` route to the MCP gateway
2. Does NOT include the MCP authorizer component
3. Allows unauthenticated access to all tools

## Usage

```bash
# Generate the Spin manifest
ftl synth ftl.yaml -o spin.toml

# Or build directly
ftl build

# Run the application
spin up
```

## Testing

Access the tools without any authentication:

```bash
# List tools - no auth required
curl -X POST http://127.0.0.1:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"tools/list","params":{},"id":1}'
```

## Security Considerations

⚠️ **Warning**: Public access means anyone can use your tools. Only use this for:
- Public demos
- Open tools with no sensitive data
- Local development environments

For production use with sensitive tools, use `private` access mode with authentication.