# MCP Gateway Example

This example demonstrates how to use the FTL MCP Gateway to aggregate multiple tools behind a single MCP endpoint.

## Architecture

```
┌─────────────────┐
│   MCP Client    │
└────────┬────────┘
         │ MCP Protocol
┌────────▼────────┐
│     Gateway     │ (/gateway/mcp)
│   MCP Server    │
└────────┬────────┘
         │ HTTP (via Spin)
    ┌────┴────┬────────┬────────┐
    │         │        │        │
┌───▼──┐ ┌───▼──┐ ┌───▼──┐ 
│Weather│ │ Calc │ │Trans │
│ Tool │ │ Tool │ │ Tool │
└──────┘ └──────┘ └──────┘
```

## Benefits

1. **Single Endpoint**: Clients connect to one MCP endpoint instead of managing multiple connections
2. **Unified Discovery**: `tools/list` returns all tools from all components
3. **Transparent Routing**: The gateway automatically routes `tools/call` to the right component
4. **Dynamic Configuration**: Easy to add/remove tools by updating the gateway config

## Configuration

The gateway is configured in `src/lib.rs`:

```rust
GatewayConfig {
    tools: vec![
        ToolEndpoint {
            name: "weather",
            route: "/weather",
            description: Some("Get weather information".to_string()),
        },
        // ... more tools
    ],
    server_info: ServerInfo {
        name: "mcp-gateway".to_string(),
        version: "1.0.0".to_string(),
    },
    base_url: "http://self".to_string(), // Uses Spin's local routing
}
```

## Usage

1. Build and run the application:
   ```bash
   spin build
   spin up
   ```

2. The gateway MCP endpoint is available at: `http://localhost:3000/gateway/mcp`

3. Test with an MCP client:
   ```bash
   # List all tools
   curl -X POST http://localhost:3000/gateway/mcp \
     -H "Content-Type: application/json" \
     -d '{"jsonrpc":"2.0","method":"tools/list","params":{},"id":1}'

   # Call a specific tool
   curl -X POST http://localhost:3000/gateway/mcp \
     -H "Content-Type: application/json" \
     -d '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"weather","arguments":{"location":"London"}},"id":2}'
   ```

## Deployment Considerations

- The gateway needs `allowed_outbound_hosts = ["http://self"]` to communicate with tools
- Tools can be developed and deployed independently
- The gateway can cache tool information for performance
- Consider adding authentication/rate limiting at the gateway level