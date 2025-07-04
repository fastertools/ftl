# {{project-name}}

{{project-description}}

## Quick Start

1. Build the handler component:
   ```bash
   cd handler
   cargo component build --release
   cd ..
   ```

2. Run the MCP server:
   ```bash
   spin up
   ```

Your MCP server is now running at http://localhost:3000/mcp

## Testing

Test your server with curl:

```bash
# List tools
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc": "2.0", "method": "tools/list", "id": 1}'

# Call a tool
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "tools/call",
    "params": {
      "name": "example_tool",
      "arguments": "{\"message\": \"Hello, MCP!\"}"
    },
    "id": 2
  }'
```

## Development

The MCP handler is in the `handler/` directory. To add new functionality:

1. **Add Tools**: Edit `handler/src/lib.rs` and update the `list_tools()` and `call_tool()` functions
2. **Add Resources**: Implement `list_resources()` and `read_resource()` 
3. **Add Prompts**: Implement `list_prompts()` and `get_prompt()`

After making changes, rebuild the handler:

```bash
cd handler
cargo component build --release
cd ..
```

## Deployment

Deploy to Spin Cloud:

```bash
spin cloud deploy
```

## MCP Client Configuration

### Claude Desktop

Add to your Claude Desktop configuration:

```json
{
  "mcpServers": {
    "{{project-name | kebab_case}}": {
      "url": "http://127.0.0.1:3000/mcp",
      "transport": "http"
    }
  }
}
```

### Cursor

Add to your Cursor settings:

```json
{
  "mcp": {
    "servers": {
      "{{project-name | kebab_case}}": {
        "url": "http://127.0.0.1:3000/mcp",
        "transport": "http"
      }
    }
  }
}
```

## Architecture

This MCP server uses:
- A pre-built gateway component ({{gateway-package}}) that handles the MCP protocol
- Your custom handler component that implements the business logic

The gateway is published to {{registry}} and handles all HTTP and JSON-RPC protocol details, while your handler focuses on implementing tools, resources, and prompts.