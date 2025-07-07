# Gateway Architecture

The FTL gateway is a component that aggregates multiple tools into a single MCP endpoint. It's automatically generated when building toolkits and provides a unified interface for AI agents to discover and interact with all tools in a toolkit.

## Overview

When you build a toolkit using `ftl toolkit build`, FTL automatically:

1. Bundles all specified tools together
2. Generates a gateway component
3. Configures routing between the gateway and individual tools

## Architecture

The gateway operates as a reverse proxy at the MCP protocol level:

```
AI Agent <--> Gateway MCP Endpoint <--> Tool 1 MCP Endpoint
                                   <--> Tool 2 MCP Endpoint
                                   <--> Tool 3 MCP Endpoint
```

### Request Flow

1. **Tool Discovery**: When an agent calls `tools/list` on the gateway, it:
   - Queries each tool's MCP endpoint for available tools
   - Aggregates the responses
   - Returns a unified tool list

2. **Tool Invocation**: When an agent calls `tools/call` on the gateway, it:
   - Identifies the target tool from the request
   - Routes the request to the appropriate tool endpoint
   - Returns the tool's response to the agent

3. **Protocol Compliance**: The gateway maintains full MCP protocol compatibility:
   - Handles `initialize` requests
   - Manages protocol version negotiation
   - Preserves request IDs for proper JSON-RPC communication

## Implementation

The gateway is implemented using the wasmcp SDK's gateway functionality:

```rust
use wasmcp::gateway::*;

fn create_gateway_config() -> GatewayConfig {
    GatewayConfig {
        tools: vec![
            ToolEndpoint {
                name: "tool1".to_string(),
                route: "/tool1".to_string(),
                description: None,
            },
            ToolEndpoint {
                name: "tool2".to_string(),
                route: "/tool2".to_string(),
                description: None,
            },
        ],
        server_info: ServerInfo {
            name: "my-toolkit-gateway".to_string(),
            version: "1.0.0".to_string(),
        },
        base_url: "".to_string(), // Uses Spin's internal service discovery
    }
}

// The wasmcp SDK provides the gateway macro
create_gateway!(create_gateway_config());
```

## Service Discovery

In the Spin WebAssembly runtime, the gateway uses internal service chaining to communicate with tool components. Each tool is accessible via:

```
http://{component-id}.spin.internal/mcp
```

This allows the gateway to make HTTP requests to other components within the same application without external network access.

## Benefits

1. **Single Endpoint**: AI agents only need to know about one endpoint to access all tools
2. **Dynamic Discovery**: Tools can be added or removed without changing the agent configuration
3. **Protocol Compatibility**: Each tool maintains its own MCP implementation
4. **Performance**: Minimal overhead as the gateway only routes requests
5. **Flexibility**: Individual tools remain accessible via their direct endpoints

## Usage

### Local Development

```bash
ftl toolkit serve my-toolkit
```

Access the gateway at: `http://localhost:3000/mcp`

### Production Deployment

```bash
ftl toolkit deploy my-toolkit
```

The deployed URL will include the gateway endpoint at `/mcp`.

## Limitations

- All tools in a toolkit must be compatible with the same MCP protocol version
- The gateway adds minimal latency for request routing
- Tool names must be unique within a toolkit